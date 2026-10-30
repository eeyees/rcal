#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::app::{ActiveModal, App};
    use rcal::model::{CalDavAccount, ProviderType};
    use rcal::mouse::HitTarget;
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

    #[test]
    fn test_account_modal_delete_click() {
        let db = Database::in_memory().unwrap();
        let acc_google = CalDavAccount::new(
            "Google Work".to_string(),
            ProviderType::Google,
            "https://apidata.googleusercontent.com/".to_string(),
            "work@gmail.com".to_string(),
            "oauth_token_123".to_string(),
        );
        db.save_account(&acc_google).unwrap();

        let mut app = App::new(db).unwrap();
        app.open_accounts_modal();
        assert!(matches!(app.modal, ActiveModal::Accounts(_)));

        // Render using TestBackend to populate hit_map with account modal widgets
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            rcal::ui::render(f, &mut app);
        }).unwrap();

        // Find the registered ModalAccountDelete hit target in hit_map
        let delete_target = app.hit_map.targets().iter().find(|(_, target)| {
            matches!(target, HitTarget::ModalAccountDelete(id) if id == &acc_google.id)
        });
        assert!(delete_target.is_some(), "Delete button for Google Work should be registered in hit_map");

        let (del_rect, _) = delete_target.unwrap();
        let click_x = del_rect.x + del_rect.width / 2;
        let click_y = del_rect.y;

        // Perform mouse click on the Delete button
        app.handle_mouse_click(click_x, click_y).unwrap();

        // Verify account was successfully deleted
        let remaining_accounts = app.db.get_accounts().unwrap();
        assert_eq!(remaining_accounts.len(), 0);
    }
}
