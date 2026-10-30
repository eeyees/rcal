#[cfg(test)]
mod tests {
    use rcal::model::{CalDavAccount, ProviderType};
    use rcal::storage::Database;

    #[test]
    fn test_caldav_account_crud() {
        let db = Database::in_memory().unwrap();

        let acc_naver = CalDavAccount::new(
            "My Naver Calendar".to_string(),
            ProviderType::Naver,
            "https://caldav.calendar.naver.com/".to_string(),
            "naver_user".to_string(),
            "secret_token".to_string(),
        );

        let acc_apple = CalDavAccount::new(
            "Work iCloud".to_string(),
            ProviderType::Apple,
            "https://caldav.icloud.com/".to_string(),
            "apple_user@icloud.com".to_string(),
            "app_password".to_string(),
        );

        // Save accounts
        db.save_account(&acc_naver).unwrap();
        db.save_account(&acc_apple).unwrap();

        // Retrieve
        let accounts = db.get_accounts().unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.iter().any(|a| a.name == "My Naver Calendar" && a.provider == ProviderType::Naver));
        assert!(accounts.iter().any(|a| a.name == "Work iCloud" && a.provider == ProviderType::Apple));

        // Delete Naver account
        db.delete_account(&acc_naver.id).unwrap();
        let updated = db.get_accounts().unwrap();
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].name, "Work iCloud");
    }
}
