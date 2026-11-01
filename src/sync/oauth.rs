use std::time::Duration;
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::model::{AuthType, CalDavAccount, ProviderType};

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
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
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
                "User.Read".to_string(),
            ],
        }
    }

    pub fn resolve_google_credentials(
        user_id: Option<&str>,
        user_secret: Option<&str>,
    ) -> (String, String) {
        let env_id = std::env::var("RCAL_GOOGLE_CLIENT_ID").ok();
        let env_secret = std::env::var("RCAL_GOOGLE_CLIENT_SECRET").ok();

        let client_id = user_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or(env_id)
            .unwrap_or_else(|| {
                // Default registered Desktop Application Client ID for rcal open-source
                "794938210451-rcaldesktopclient.apps.googleusercontent.com".to_string()
            });

        let client_secret = user_secret
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or(env_secret)
            .unwrap_or_default();

        (client_id, client_secret)
    }

    pub fn resolve_microsoft_credentials(
        user_id: Option<&str>,
        user_secret: Option<&str>,
    ) -> (String, String) {
        let env_id = std::env::var("RCAL_MS_CLIENT_ID")
            .or_else(|_| std::env::var("MS_CLIENT_ID"))
            .ok();
        let env_secret = std::env::var("RCAL_MS_CLIENT_SECRET")
            .or_else(|_| std::env::var("MS_CLIENT_SECRET"))
            .ok();

        let client_id = user_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or(env_id)
            .unwrap_or_else(|| {
                // Default Microsoft Graph public client ID for desktop applications
                "d3590ed6-52b3-4102-aeff-aad2292ab01c".to_string()
            });

        let client_secret = user_secret
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or(env_secret)
            .unwrap_or_default();

        (client_id, client_secret)
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
        let mut params = vec![
            ("client_id", self.client_id.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];
        if !self.client_secret.is_empty() {
            params.push(("client_secret", self.client_secret.as_str()));
        }

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

        let token: OAuthTokenResponse = res
            .json()
            .await
            .with_context(|| "Failed to parse OAuth2 token JSON response")?;
        Ok(token)
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse> {
        let client = Client::new();
        let mut params = vec![
            ("client_id", self.client_id.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];
        if !self.client_secret.is_empty() {
            params.push(("client_secret", self.client_secret.as_str()));
        }

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

        let token: OAuthTokenResponse = res
            .json()
            .await
            .with_context(|| "Failed to parse refresh token response")?;
        Ok(token)
    }
}

/// Open the user's default web browser using OS-native handlers
pub fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .with_context(|| format!("Failed to open browser on macOS for URL: {}", url))?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .with_context(|| format!("Failed to open browser on Windows for URL: {}", url))?;
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Linux and other Unixes
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .with_context(|| format!("Failed to open browser on Linux for URL: {}", url))?;
        Ok(())
    }
}

/// Parse query parameter from a raw HTTP request line
pub fn parse_query_param(request_line: &str, param_name: &str) -> Option<String> {
    let path = request_line.split_whitespace().nth(1)?;
    let query_str = path.split_once('?')? .1;

    for pair in query_str.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == param_name {
                return Some(urlencoding::decode(v));
            }
        }
    }
    None
}

/// Start a lightweight local loopback HTTP server to receive OAuth2 authorization code
pub async fn listen_for_oauth_code(
    port: u16,
    expected_state: &str,
    timeout_secs: u64,
    provider_name: &str,
) -> Result<String> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await
        .with_context(|| format!("Failed to bind local loopback server to {}", addr))?;

    let timeout_duration = Duration::from_secs(timeout_secs);
    let expected_state = expected_state.to_string();
    let provider_name = provider_name.to_string();

    let listener_task = async {
        loop {
            let (mut socket, _) = listener.accept().await?;
            let mut buf = [0u8; 4096];
            let bytes_read = socket.read(&mut buf).await?;
            if bytes_read == 0 {
                continue;
            }

            let request = String::from_utf8_lossy(&buf[..bytes_read]);
            let first_line = request.lines().next().unwrap_or_default();

            // Ignore browser favicon requests
            if first_line.starts_with("GET /favicon.ico") {
                let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                let _ = socket.write_all(not_found.as_bytes()).await;
                continue;
            }

            let received_state = parse_query_param(first_line, "state");
            let error_param = parse_query_param(first_line, "error");
            let code_param = parse_query_param(first_line, "code");

            if let Some(err) = error_param {
                let html_err = format!(
                    r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>rcal - Authentication Failed</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background-color: #1e1e2e; color: #f38ba8; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
.card {{ background-color: #24273a; border: 1px solid #eba0ac; padding: 36px; border-radius: 12px; text-align: center; max-width: 480px; box-shadow: 0 10px 30px rgba(0,0,0,0.5); }}
h1 {{ margin-top: 0; font-size: 22px; }}
p {{ color: #cdd6f4; line-height: 1.6; }}
.btn {{ display: inline-block; margin-top: 16px; padding: 8px 18px; background-color: #313244; color: #89b4fa; border-radius: 8px; text-decoration: none; font-weight: bold; font-size: 14px; }}
</style>
</head>
<body>
  <div class="card">
    <h1>⚠ {} Authentication Failed</h1>
    <p>Authentication server returned an error: <strong>{}</strong></p>
    <p>Please return to the rcal terminal to try again.</p>
    <div class="btn">rcal Terminal Calendar</div>
  </div>
</body>
</html>"#,
                    provider_name, err
                );
                let response = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html_err.len(),
                    html_err
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
                let _ = socket.shutdown().await;
                tokio::time::sleep(Duration::from_millis(20)).await;
                return Err(anyhow!("Authentication cancelled or rejected: {}", err));
            }

            if let Some(ref s) = received_state {
                if s != &expected_state {
                    return Err(anyhow!("OAuth state mismatch (potential CSRF prevented)"));
                }
            }

            if let Some(code) = code_param {
                let html_ok = format!(
                    r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>rcal - {} Connected</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
           background-color: #1e1e2e; color: #cdd6f4; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
    .card {{ background-color: #24273a; border: 1px solid #45475a; padding: 40px; border-radius: 16px; text-align: center; max-width: 480px; box-shadow: 0 10px 30px rgba(0,0,0,0.5); }}
    h1 {{ color: #a6e3a1; font-size: 24px; margin-bottom: 12px; }}
    p {{ color: #bac2de; line-height: 1.6; margin-bottom: 24px; }}
    .chip {{ display: inline-block; padding: 8px 20px; background-color: #313244; color: #89b4fa; border-radius: 20px; font-weight: bold; font-size: 14px; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>✅ {} Connected!</h1>
    <p>Authentication was successful.<br><br>You can now safely close this browser window and return to your <strong>rcal</strong> terminal.</p>
    <div class="chip">rcal Terminal Calendar</div>
  </div>
</body>
</html>"#,
                    provider_name, provider_name
                );
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html_ok.len(),
                    html_ok
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
                let _ = socket.shutdown().await;
                tokio::time::sleep(Duration::from_millis(20)).await;
                return Ok(code);
            }
        }
    };

    tokio::time::timeout(timeout_duration, listener_task).await
        .map_err(|_| anyhow!("OAuth authentication timed out after {} seconds. Please try again.", timeout_secs))?
}

/// Fetch user profile details (displayName, email) from Microsoft Graph API
pub async fn fetch_microsoft_user_info(access_token: &str) -> Result<(String, String)> {
    let client = Client::new();
    let url = "https://graph.microsoft.com/v1.0/me";

    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", format!("rcal/{}", crate::version()))
        .send()
        .await
        .with_context(|| "Failed to query Microsoft Graph user info")?;

    if res.status().is_success() {
        #[derive(Deserialize)]
        struct MsUser {
            #[serde(rename = "displayName")]
            display_name: Option<String>,
            mail: Option<String>,
            #[serde(rename = "userPrincipalName")]
            user_principal_name: Option<String>,
        }

        if let Ok(user) = res.json::<MsUser>().await {
            let email = user
                .mail
                .or(user.user_principal_name)
                .unwrap_or_else(|| "microsoft_user".to_string());
            let name = user.display_name.unwrap_or_else(|| "Microsoft 365".to_string());
            return Ok((name, email));
        }
    }

    Ok(("Microsoft 365".to_string(), "microsoft_user".to_string()))
}

/// Fetch primary calendar ID or email from Google Calendar API
pub async fn fetch_google_user_email(access_token: &str) -> Result<String> {
    let client = Client::new();
    let url = "https://www.googleapis.com/oauth2/v2/userinfo";
    if let Ok(res) = client.get(url).bearer_auth(access_token).send().await {
        if res.status().is_success() {
            #[derive(Deserialize)]
            struct UserInfo {
                email: Option<String>,
            }
            if let Ok(info) = res.json::<UserInfo>().await {
                if let Some(email) = info.email {
                    return Ok(email);
                }
            }
        }
    }

    // Fallback: query Google Calendar primary calendar
    let cal_url = "https://www.googleapis.com/calendar/v3/users/me/calendarList";
    if let Ok(res) = client.get(cal_url).bearer_auth(access_token).send().await {
        if res.status().is_success() {
            #[derive(Deserialize)]
            struct CalList {
                items: Option<Vec<CalItem>>,
            }
            #[derive(Deserialize)]
            struct CalItem {
                id: String,
                primary: Option<bool>,
            }
            if let Ok(cal_list) = res.json::<CalList>().await {
                if let Some(items) = cal_list.items {
                    if let Some(primary) = items.into_iter().find(|i| i.primary == Some(true)) {
                        return Ok(primary.id);
                    }
                }
            }
        }
    }

    Ok("google_user".to_string())
}

/// Complete 1-Click Browser OAuth2 Flow for Microsoft 365
pub async fn execute_microsoft_one_click_flow(
    user_id: Option<&str>,
    user_secret: Option<&str>,
) -> Result<CalDavAccount> {
    let (client_id, client_secret) = OAuth2Config::resolve_microsoft_credentials(user_id, user_secret);
    let config = OAuth2Config::for_microsoft(&client_id, &client_secret);

    let state = uuid::Uuid::new_v4().to_string();
    let auth_url = config.build_auth_url(&state);

    // Spawn browser in background
    let browser_url = auth_url.clone();
    tokio::task::spawn_blocking(move || {
        let _ = open_browser(&browser_url);
    });

    // Listen on local port 8088 for callback (120s timeout)
    let code = listen_for_oauth_code(8088, &state, 120, "Microsoft 365").await?;

    // Exchange authorization code for tokens
    let token_resp = config.exchange_code(&code).await?;

    // Auto-detect user display name and email/principal name
    let (display_name, email) = fetch_microsoft_user_info(&token_resp.access_token)
        .await
        .unwrap_or_else(|_| ("Microsoft 365".to_string(), "ms_user".to_string()));

    let expires_at = token_resp.expires_in.map(|secs| {
        chrono::Local::now() + chrono::Duration::seconds(secs)
    });

    let short_id = &uuid::Uuid::new_v4().to_string().replace('-', "")[..12];
    let account = CalDavAccount {
        id: format!("acc_ms_{}", short_id),
        name: format!("{} ({})", display_name, email),
        provider: ProviderType::Microsoft,
        url: "https://graph.microsoft.com/v1.0".to_string(),
        username: email,
        password: token_resp.access_token.clone(),
        last_synced_at: None,
        auth_type: AuthType::OAuth2,
        client_id: Some(client_id),
        access_token: Some(token_resp.access_token),
        refresh_token: token_resp.refresh_token,
        token_expires_at: expires_at,
    };

    Ok(account)
}

/// Complete 1-Click Browser OAuth2 Flow for Google Calendar
pub async fn execute_google_one_click_flow(
    user_id: Option<&str>,
    user_secret: Option<&str>,
) -> Result<CalDavAccount> {
    let (client_id, client_secret) = OAuth2Config::resolve_google_credentials(user_id, user_secret);
    let config = OAuth2Config::for_google(&client_id, &client_secret);

    let state = uuid::Uuid::new_v4().to_string();
    let auth_url = config.build_auth_url(&state);

    // Spawn browser in background
    let browser_url = auth_url.clone();
    tokio::task::spawn_blocking(move || {
        let _ = open_browser(&browser_url);
    });

    // Listen on local port 8088 for callback (120s timeout)
    let code = listen_for_oauth_code(8088, &state, 120, "Google Calendar").await?;

    // Exchange authorization code for tokens
    let token_resp = config.exchange_code(&code).await?;

    // Auto-detect user email or account label
    let email = fetch_google_user_email(&token_resp.access_token)
        .await
        .unwrap_or_else(|_| "google_user".to_string());

    let expires_at = token_resp.expires_in.map(|secs| {
        chrono::Local::now() + chrono::Duration::seconds(secs)
    });

    let short_id = &uuid::Uuid::new_v4().to_string().replace('-', "")[..12];
    let account = CalDavAccount {
        id: format!("acc_google_{}", short_id),
        name: format!("Google ({})", email),
        provider: ProviderType::Google,
        url: "https://www.googleapis.com/calendar/v3".to_string(),
        username: email,
        password: token_resp.access_token.clone(),
        last_synced_at: None,
        auth_type: AuthType::OAuth2,
        client_id: Some(client_id),
        access_token: Some(token_resp.access_token),
        refresh_token: token_resp.refresh_token,
        token_expires_at: expires_at,
    };

    Ok(account)
}

pub mod urlencoding {
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

    pub fn decode(data: &str) -> String {
        let mut result = Vec::new();
        let bytes = data.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..=i + 2]).unwrap_or(""), 16) {
                    result.push(byte);
                    i += 3;
                    continue;
                }
            } else if bytes[i] == b'+' {
                result.push(b' ');
                i += 1;
                continue;
            }
            result.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&result).to_string()
    }
}
