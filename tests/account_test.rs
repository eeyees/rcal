#[cfg(test)]
mod tests {
    use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::app::{ActiveModal, App};
    use rcal::events::handle_crossterm_event;
    use rcal::i18n::Language;
    use rcal::model::{CalDavAccount, ProviderType};
    use rcal::mouse::HitTarget;
    use rcal::storage::Database;

    fn make_key(code: KeyCode) -> CrosstermEvent {
        CrosstermEvent::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn make_shift_key(code: KeyCode) -> CrosstermEvent {
        CrosstermEvent::Key(KeyEvent::new(code, KeyModifiers::SHIFT))
    }

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

    #[test]
    fn test_account_modal_add_shortcuts() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        app.open_accounts_modal();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // Test keys that must enter Add mode:
        // 1. '+' key
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('+'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 2. Shift+'=' (plus key representation on some terminals)
        handle_crossterm_event(&mut app, make_shift_key(KeyCode::Char('='))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 3. 'a' key
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('a'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 4. 'A' key (uppercase / Shift+A)
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('A'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 5. 'n' key
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('n'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 6. 'N' key
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('N'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 7. 'Enter' key
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 8. Korean 'ㅁ' key (mapped to 'a')
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('ㅁ'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 9. Korean 'ㅜ' key (mapped to 'n')
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('ㅜ'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));

        // 10. Korean 'ㅊ' key (mapped to 'c')
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('ㅊ'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if state.is_adding && state.active_field == 0));
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::Accounts(ref state) if !state.is_adding));
    }

    #[test]
    fn test_account_modal_render_add_button_and_hints() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();
        app.open_accounts_modal();

        // 1. English rendering
        app.lang = Language::En;
        let mut term_en = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term_en.draw(|f| {
            rcal::ui::render(f, &mut app);
        }).unwrap();
        let buffer_str: String = term_en.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        assert!(buffer_str.contains("Add Account (a/+)"), "English Add Account button with shortcut should be rendered");
        assert!(buffer_str.contains("[a/+]"), "Shortcut hint [a/+] should be rendered");
        assert!(buffer_str.contains("[d]"), "Shortcut hint [d] should be rendered");
        assert!(buffer_str.contains("[r]"), "Shortcut hint [r] should be rendered");
        assert!(buffer_str.contains("[Esc/q]"), "Shortcut hint [Esc/q] should be rendered");

        // 2. Korean rendering
        app.lang = Language::Ko;
        let mut term_ko = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term_ko.draw(|f| {
            rcal::ui::render(f, &mut app);
        }).unwrap();
        let buffer_str_ko: String = term_ko.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        let clean_ko: String = buffer_str_ko.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(clean_ko.contains("계정추가(a/+)"), "Korean Add Account button with shortcut should be rendered");
        assert!(clean_ko.contains("[a/+]계정추가"), "Shortcut hint [a/+] should be rendered");
        assert!(clean_ko.contains("[d]삭제"), "Shortcut hint [d] should be rendered");
        assert!(clean_ko.contains("[r]동기화"), "Shortcut hint [r] should be rendered");
        assert!(clean_ko.contains("[Esc/q]닫기"), "Shortcut hint [Esc/q] should be rendered");

        // 3. Japanese rendering
        app.lang = Language::Ja;
        let mut term_ja = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term_ja.draw(|f| {
            rcal::ui::render(f, &mut app);
        }).unwrap();
        let buffer_str_ja: String = term_ja.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        let clean_ja: String = buffer_str_ja.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(clean_ja.contains("アカウント追加(a/+)"), "Japanese Add Account button with shortcut should be rendered");
        assert!(clean_ja.contains("[a/+]アカウント追加"), "Shortcut hint [a/+] should be rendered");
        assert!(clean_ja.contains("[d]削除"), "Shortcut hint [d] should be rendered");
        assert!(clean_ja.contains("[r]同期"), "Shortcut hint [r] should be rendered");
        assert!(clean_ja.contains("[Esc/q]閉じる"), "Shortcut hint [Esc/q] should be rendered");
    }
}
