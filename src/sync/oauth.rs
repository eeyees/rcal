use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuth2Config {
    pub fn for_google(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri: "http://127.0.0.1:8088/oauth/callback".to_string(),
            scopes: vec![
                "https://www.googleapis.com/auth/calendar".to_string(),
                "https://www.googleapis.com/auth/calendar.events".to_string(),
            ],
        }
    }

    pub fn for_microsoft(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
            token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            redirect_uri: "http://127.0.0.1:8088/oauth/callback".to_string(),
            scopes: vec![
                "https://graph.microsoft.com/Calendars.ReadWrite".to_string(),
                "offline_access".to_string(),
            ],
        }
    }

    pub fn build_auth_url(&self, state: &str) -> String {
        let scopes_joined = self.scopes.join(" ");
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&access_type=offline&prompt=consent",
            self.auth_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes_joined),
            urlencoding::encode(state),
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse> {
        let client = Client::new();
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        let res = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .with_context(|| "Failed to request OAuth2 token")?;

        if !res.status().is_success() {
            let error_body = res.text().await.unwrap_or_default();
            return Err(anyhow!("OAuth2 token exchange error: {}", error_body));
        }

        let token: OAuthTokenResponse = res.json().await
            .with_context(|| "Failed to parse OAuth2 token JSON response")?;
        Ok(token)
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse> {
        let client = Client::new();
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let res = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .with_context(|| "Failed to refresh OAuth2 token")?;

        if !res.status().is_success() {
            let error_body = res.text().await.unwrap_or_default();
            return Err(anyhow!("OAuth2 token refresh error: {}", error_body));
        }

        let token: OAuthTokenResponse = res.json().await
            .with_context(|| "Failed to parse refresh token response")?;
        Ok(token)
    }
}

mod urlencoding {
    pub fn encode(data: &str) -> String {
        let mut encoded = String::new();
        for b in data.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(b as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", b));
                }
            }
        }
        encoded
    }
}
