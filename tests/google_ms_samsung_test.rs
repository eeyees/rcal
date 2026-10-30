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
    }
}
