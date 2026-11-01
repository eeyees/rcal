#[cfg(test)]
mod tests {
    use rcal::model::{AuthType, CalDavAccount, ProviderType};
    use rcal::storage::Database;
    use rcal::sync::OAuth2Config;

    #[test]
    fn test_samsung_calendar_account() {
        let db = Database::in_memory().unwrap();

        let acc_samsung = CalDavAccount::new(
            "Galaxy S25 Calendar".to_string(),
            ProviderType::Samsung,
            "https://caldav.samsung.com/".to_string(),
            "samsung_user".to_string(),
            "samsung_pw".to_string(),
        );

        db.save_account(&acc_samsung).unwrap();
        let accounts = db.get_accounts().unwrap();

        let found = accounts.iter().find(|a| a.provider == ProviderType::Samsung);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Galaxy S25 Calendar");
    }

    #[test]
    fn test_google_and_ms_oauth_accounts() {
        let db = Database::in_memory().unwrap();

        let acc_google = CalDavAccount::new_oauth(
            "Work Google Calendar".to_string(),
            ProviderType::Google,
            "client-id-123.apps.googleusercontent.com".to_string(),
            "client-secret-xyz".to_string(),
            Some("ya29.access-token-mock".to_string()),
            Some("1//refresh-token-mock".to_string()),
        );

        let acc_ms = CalDavAccount::new_oauth(
            "Office 365 Outlook".to_string(),
            ProviderType::Microsoft,
            "ms-app-id-456".to_string(),
            "ms-secret-789".to_string(),
            Some("EwB...access-token-mock".to_string()),
            Some("M.R3...refresh-token-mock".to_string()),
        );

        db.save_account(&acc_google).unwrap();
        db.save_account(&acc_ms).unwrap();

        let accounts = db.get_accounts().unwrap();
        assert_eq!(accounts.len(), 2);

        let google = accounts.iter().find(|a| a.provider == ProviderType::Google).unwrap();
        assert_eq!(google.auth_type, AuthType::OAuth2);
        assert_eq!(google.access_token.as_deref(), Some("ya29.access-token-mock"));

        let ms = accounts.iter().find(|a| a.provider == ProviderType::Microsoft).unwrap();
        assert_eq!(ms.auth_type, AuthType::OAuth2);
        assert_eq!(ms.refresh_token.as_deref(), Some("M.R3...refresh-token-mock"));
    }

    #[test]
    fn test_oauth_url_builder() {
        let google_config = OAuth2Config::for_google("my-google-client-id", "my-secret");
        let auth_url = google_config.build_auth_url("state-1234");
        assert!(auth_url.contains("accounts.google.com"));
        assert!(auth_url.contains("client_id=my-google-client-id"));
        assert!(auth_url.contains("state=state-1234"));

        let ms_config = OAuth2Config::for_microsoft("my-ms-client-id", "my-secret");
        let ms_auth_url = ms_config.build_auth_url("ms-state");
        assert!(ms_auth_url.contains("login.microsoftonline.com"));
        assert!(ms_auth_url.contains("client_id=my-ms-client-id"));
        assert!(ms_auth_url.contains("Calendars.ReadWrite"));
        assert!(ms_auth_url.contains("User.Read"));
    }

    #[test]
    fn test_ms_credentials_resolution() {
        let (cid, csec) = OAuth2Config::resolve_microsoft_credentials(Some("user-custom-id"), Some("user-custom-sec"));
        assert_eq!(cid, "user-custom-id");
        assert_eq!(csec, "user-custom-sec");

        // When user input is None or empty, fallback to default public client ID
        let (def_cid, def_csec) = OAuth2Config::resolve_microsoft_credentials(None, None);
        assert!(!def_cid.is_empty());
        assert_eq!(def_cid, "d3590ed6-52b3-4102-aeff-aad2292ab01c");
        assert_eq!(def_csec, "");
    }

    #[test]
    fn test_query_param_parsing() {
        use rcal::sync::oauth::parse_query_param;

        let req = "GET /oauth/callback?code=M.R3_mock_12345&state=xyz_state HTTP/1.1";
        assert_eq!(parse_query_param(req, "code").as_deref(), Some("M.R3_mock_12345"));
        assert_eq!(parse_query_param(req, "state").as_deref(), Some("xyz_state"));
        assert_eq!(parse_query_param(req, "error"), None);

        let err_req = "GET /oauth/callback?error=access_denied&error_description=User+cancelled HTTP/1.1";
        assert_eq!(parse_query_param(err_req, "error").as_deref(), Some("access_denied"));
    }

    use std::sync::atomic::{AtomicU16, Ordering};
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(19300);

    fn get_free_port() -> u16 {
        PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    #[tokio::test]
    async fn test_oauth_loopback_listener_success() {
        use rcal::sync::oauth::listen_for_oauth_code;

        let test_port = get_free_port();
        let test_state = "test-csrf-token-abc";

        let server_handle = tokio::spawn(async move {
            listen_for_oauth_code(test_port, test_state, 5, "Microsoft 365").await
        });

        // Give the listener server 50ms to bind
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Simulate browser callback from Microsoft login
        let callback_url = format!(
            "http://127.0.0.1:{}/oauth/callback?code=mock-ms-code-999&state={}",
            test_port, test_state
        );
        let client = reqwest::Client::new();
        let resp = client.get(&callback_url).send().await.expect("Request failed");
        assert!(resp.status().is_success());
        let html_body = resp.text().await.expect("Failed to read text");
        assert!(html_body.contains("Microsoft 365 Connected"));

        let received_code = server_handle.await.expect("Task failed").expect("Loopback error");
        assert_eq!(received_code, "mock-ms-code-999");
    }

    #[tokio::test]
    async fn test_oauth_loopback_csrf_mismatch_prevention() {
        use rcal::sync::oauth::listen_for_oauth_code;

        let test_port = get_free_port();
        let expected_state = "real-state-123";

        let server_handle = tokio::spawn(async move {
            listen_for_oauth_code(test_port, expected_state, 5, "Microsoft 365").await
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Simulate attacking request with spoofed state
        let callback_url = format!(
            "http://127.0.0.1:{}/oauth/callback?code=evil-code&state=fake-state",
            test_port
        );
        let client = reqwest::Client::new();
        let _ = client.get(&callback_url).send().await;

        let res = server_handle.await.expect("Task failed");
        assert!(res.is_err(), "Expected CSRF error due to state mismatch");
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("state mismatch"), "Error should mention state mismatch");
    }
}
