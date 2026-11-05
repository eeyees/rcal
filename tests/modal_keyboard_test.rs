#[cfg(test)]
mod tests {
    use chrono::Local;
    use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::app::{ActiveModal, App};
    use rcal::events::handle_crossterm_event;
    use rcal::model::{CalDavAccount, Event, ProviderType, WeekStartDay};
    use rcal::storage::Database;
    use rcal::ui;

    fn make_key(code: KeyCode) -> CrosstermEvent {
        CrosstermEvent::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn test_event_form_tab_navigation_and_cycling() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        app.open_new_event_modal(None);
        assert!(matches!(app.modal, ActiveModal::EventForm(_)));

        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.active_field, 0); // Initial field: Title
        }

        // Tab through all fields: 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 (Save) -> 9 (Cancel) -> 0 (Title)
        for expected in 1..=9 {
            handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
            if let ActiveModal::EventForm(ref form) = app.modal {
                assert_eq!(form.active_field, expected, "Expected field {}", expected);
            }
        }
        // Wrapping from 9 to 0
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.active_field, 0);
        }

        // Shift-Tab (BackTab) backwards: 0 -> 9 -> 8 -> 7 -> ... -> 0
        handle_crossterm_event(&mut app, make_key(KeyCode::BackTab)).unwrap();
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.active_field, 9);
        }
        handle_crossterm_event(&mut app, make_key(KeyCode::Up)).unwrap();
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.active_field, 8);
        }
    }

    #[test]
    fn test_event_form_interaction_and_save() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();
        let _initial_events_count = app.db.get_all_events().unwrap().len();

        app.open_new_event_modal(None);

        // 1. Title (field 0)
        for c in "Team Sync".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.title, "Team Sync");
        }

        // 2. Calendar (field 1)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        // Press Space on Calendar
        handle_crossterm_event(&mut app, make_key(KeyCode::Char(' '))).unwrap();
        // Or Left / Right
        handle_crossterm_event(&mut app, make_key(KeyCode::Right)).unwrap();

        // 3. Date (field 2)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        // Clear and set new date
        for _ in 0..10 {
            handle_crossterm_event(&mut app, make_key(KeyCode::Backspace)).unwrap();
        }
        for c in "2026-11-20".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // 4. Start Time (field 3)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        for _ in 0..5 {
            handle_crossterm_event(&mut app, make_key(KeyCode::Backspace)).unwrap();
        }
        for c in "14:00".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // 5. End Time (field 4)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        for _ in 0..5 {
            handle_crossterm_event(&mut app, make_key(KeyCode::Backspace)).unwrap();
        }
        for c in "15:30".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // 6. All Day (field 5)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        // Toggle with Space
        handle_crossterm_event(&mut app, make_key(KeyCode::Char(' '))).unwrap();
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert!(form.all_day);
        }
        handle_crossterm_event(&mut app, make_key(KeyCode::Char(' '))).unwrap();
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert!(!form.all_day);
        }

        // 7. Location (field 6)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        for c in "Room 402".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // 8. Description (field 7)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        for c in "Discuss roadmap".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // 9. Tab to Save button (field 8) and press Enter
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.active_field, 8);
        }
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();

        // Modal should be closed now
        assert_eq!(app.modal, ActiveModal::None);

        // Verify event in database
        let events = app.db.get_all_events().unwrap();
        let team_sync = events
            .iter()
            .find(|e| e.title == "Team Sync")
            .expect("Team Sync event must exist");
        assert_eq!(team_sync.location.as_deref(), Some("Room 402"));
        assert_eq!(team_sync.description.as_deref(), Some("Discuss roadmap"));
    }

    #[test]
    fn test_event_form_cancel_button_and_esc() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        // Cancel via button (field 9)
        app.open_new_event_modal(None);
        // Tab to 9: Cancel
        for _ in 0..9 {
            handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        }
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.active_field, 9);
        }
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
        let event_titles: Vec<String> = app
            .db
            .get_all_events()
            .unwrap()
            .into_iter()
            .map(|e| e.title)
            .collect();
        assert!(!event_titles.contains(&"Cancelled Event".to_string()));

        // Cancel via Esc
        app.open_new_event_modal(None);
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
    }

    #[test]
    fn test_account_modal_add_mode_navigation_and_connect() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        app.open_accounts_modal();
        assert!(matches!(app.modal, ActiveModal::Accounts(_)));

        // Press 'a' to enter Add mode
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('a'))).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert!(state.is_adding);
            assert_eq!(state.active_field, 0); // Presets row
        }

        // On preset row (field 0), press Right to cycle presets
        handle_crossterm_event(&mut app, make_key(KeyCode::Right)).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.provider_idx, 1); // Apple
            assert_eq!(state.name, "Apple iCloud");
        }
        handle_crossterm_event(&mut app, make_key(KeyCode::Char(' '))).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.provider_idx, 2); // Google
        }

        // Tab to 1: Name
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.active_field, 1);
        }

        // Tab to 2: URL
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.active_field, 2);
        }

        // Tab to 3: Username
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        for c in "my_user".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // Tab to 4: Password
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        for c in "secret123".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // Tab to 5: [ 🌐 1-Click Login ] button
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.active_field, 5);
        }

        // Tab to 6: [ Manual Save ] button for OAuth
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.active_field, 6);
        }

        // Press Enter on [ Manual Save ]
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();

        // Modal should close and account be added
        assert_eq!(app.modal, ActiveModal::None);
        let accounts = app.db.get_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].username, "my_user");
    }

    #[test]
    fn test_account_modal_cancel_and_esc() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        app.open_accounts_modal();
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('n'))).unwrap();

        // Tab to 6: [ Cancel ] button
        for _ in 0..6 {
            handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        }
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.active_field, 6);
        }
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();

        // Should return to account list mode (not closed, but is_adding = false)
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert!(!state.is_adding);
        }

        // Esc closes modal
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
    }

    #[test]
    fn test_account_modal_list_navigation_and_deletion() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let acc1 = CalDavAccount::new(
            "Acc 1".into(),
            ProviderType::Naver,
            "url1".into(),
            "u1".into(),
            "p1".into(),
        );
        let acc2 = CalDavAccount::new(
            "Acc 2".into(),
            ProviderType::Apple,
            "url2".into(),
            "u2".into(),
            "p2".into(),
        );
        app.db.save_account(&acc1).unwrap();
        app.db.save_account(&acc2).unwrap();

        app.open_accounts_modal();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.selected_account_idx, 0);
        }

        // Down / j to select Acc 2
        handle_crossterm_event(&mut app, make_key(KeyCode::Down)).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.selected_account_idx, 1);
        }

        // Up / k to select Acc 1
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('k'))).unwrap();
        if let ActiveModal::Accounts(ref state) = app.modal {
            assert_eq!(state.selected_account_idx, 0);
        }

        // Delete Acc 1 with 'd' -> prompts for confirmation
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::ConfirmDelete(_)));
        // Confirm deletion with 'y'
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('y'))).unwrap();
        let remaining = app.db.get_accounts().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].name, "Acc 2");

        // Esc closes modal
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('q'))).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
    }

    #[test]
    fn test_calendar_modal_navigation_and_create() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        app.open_calendar_modal();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));

        // In list mode, press 'n' to enter create mode
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('n'))).unwrap();
        if let ActiveModal::CalendarManager(ref state) = app.modal {
            assert!(state.is_creating);
            assert_eq!(state.active_field, 0); // Name
        }

        // Type calendar name
        for c in "Project X".chars() {
            handle_crossterm_event(&mut app, make_key(KeyCode::Char(c))).unwrap();
        }

        // Tab to 1: Color selector
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::CalendarManager(ref state) = app.modal {
            assert_eq!(state.active_field, 1);
        }
        // Change color with Right / Left
        handle_crossterm_event(&mut app, make_key(KeyCode::Right)).unwrap();
        if let ActiveModal::CalendarManager(ref state) = app.modal {
            assert_eq!(state.color_idx, 1);
        }

        // Tab to 2: [ Save ]
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        if let ActiveModal::CalendarManager(ref state) = app.modal {
            assert_eq!(state.active_field, 2);
        }

        // Press Enter on Save
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);

        // Verify calendar created
        let cals = app.db.get_calendars().unwrap();
        assert!(cals.iter().any(|c| c.name == "Project X"));

        // Open calendar modal again and select the custom calendar
        app.open_calendar_modal();
        if let ActiveModal::CalendarManager(ref mut s) = app.modal {
            if let Some(idx) = app.calendars.iter().position(|c| c.name == "Project X") {
                s.selected_calendar_idx = idx;
            }
        }
        // Delete with 'd' -> prompts for confirmation
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::ConfirmDelete(_)));
        // Confirm deletion with 'y'
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('y'))).unwrap();
        let after_delete = app.db.get_calendars().unwrap();
        assert!(!after_delete.iter().any(|c| c.name == "Project X"));
    }

    #[test]
    fn test_settings_modal_keyboard_controls() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        app.open_settings_modal();
        assert_eq!(app.modal, ActiveModal::Settings);
        assert_eq!(app.settings_section, rcal::app::SettingsSection::WeekStart);

        // s: Sunday, m: Monday, t: Saturday, 1: Tab 1 & Sunday
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('m'))).unwrap();
        assert_eq!(app.week_start_day, WeekStartDay::Monday);

        handle_crossterm_event(&mut app, make_key(KeyCode::Char('t'))).unwrap();
        assert_eq!(app.week_start_day, WeekStartDay::Saturday);

        handle_crossterm_event(&mut app, make_key(KeyCode::Char('s'))).unwrap();
        assert_eq!(app.week_start_day, WeekStartDay::Sunday);

        handle_crossterm_event(&mut app, make_key(KeyCode::Char('1'))).unwrap();
        assert_eq!(app.week_start_day, WeekStartDay::Sunday);

        // Right/Left/Up/Down arrows change week start day in WeekStart section
        handle_crossterm_event(&mut app, make_key(KeyCode::Right)).unwrap();
        assert_eq!(app.week_start_day, WeekStartDay::Monday);

        // Tab cycles across major tabs in big steps:
        // WeekStart(1) -> Holidays(2) -> Notifications(3) -> Theme(4) -> Language(5) -> WeekStart(1)
        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::Holidays);

        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(
            app.settings_section,
            rcal::app::SettingsSection::Notifications
        );

        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::Theme);

        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::Language);

        handle_crossterm_event(&mut app, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::WeekStart);

        // BackTab (Shift+Tab) cycles backwards
        handle_crossterm_event(&mut app, make_key(KeyCode::BackTab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::Language);

        handle_crossterm_event(&mut app, make_key(KeyCode::BackTab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::Theme);

        handle_crossterm_event(&mut app, make_key(KeyCode::BackTab)).unwrap();
        assert_eq!(
            app.settings_section,
            rcal::app::SettingsSection::Notifications
        );

        handle_crossterm_event(&mut app, make_key(KeyCode::BackTab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::Holidays);

        handle_crossterm_event(&mut app, make_key(KeyCode::BackTab)).unwrap();
        assert_eq!(app.settings_section, rcal::app::SettingsSection::WeekStart);

        // Language modal with 'l'
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('l'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::Language(_)));

        // Enter closes modal and selects language
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
    }

    #[test]
    fn test_event_detail_shortcuts() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let cal_id = app.calendars[0].id.clone();
        let now = Local::now();
        let ev = Event::new(
            cal_id,
            "Special Event".into(),
            now,
            now + chrono::Duration::hours(1),
            false,
        );
        app.db.save_event(&ev).unwrap();
        app.reload_events().unwrap();

        let ev_id = ev.id.clone();
        app.open_event_detail_modal(ev_id.clone());
        assert_eq!(app.modal, ActiveModal::EventDetail(ev_id.clone()));

        // Press 'e' to edit
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('e'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::EventForm(_)));

        // Close and reopen detail
        app.close_modal();
        app.open_event_detail_modal(ev_id.clone());

        // Press 'd' to delete -> prompts for confirmation
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::ConfirmDelete(_)));

        // Confirm deletion with 'y'
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('y'))).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
        assert!(app.db.get_event_by_id(&ev_id).unwrap().is_none());
    }

    #[test]
    fn test_help_modal_shortcuts() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        app.open_help_modal();
        assert_eq!(app.modal, ActiveModal::Help);

        // Enter closes
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);

        // 'q' closes
        app.open_help_modal();
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('q'))).unwrap();
        assert_eq!(app.modal, ActiveModal::None);

        // Esc closes
        app.open_help_modal();
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
    }

    #[test]
    fn test_cursor_positioning_on_modals() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        // 1. EventForm modal cursor
        app.open_new_event_modal(None);
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();
        let pos1 = terminal.get_cursor_position().unwrap();
        assert!(
            pos1.x > 0 && pos1.y > 0,
            "Cursor must be set inside EventForm modal"
        );

        // 2. Account modal (Add mode) cursor
        app.open_accounts_modal();
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('a'))).unwrap();
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();
        let pos2 = terminal.get_cursor_position().unwrap();
        assert!(
            pos2.x > 0 && pos2.y > 0,
            "Cursor must be set inside AccountModal"
        );

        // 3. Calendar modal (Create mode) cursor
        app.open_calendar_modal();
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('n'))).unwrap();
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();
        let pos3 = terminal.get_cursor_position().unwrap();
        assert!(
            pos3.x > 0 && pos3.y > 0,
            "Cursor must be set inside CalendarModal"
        );

        // 4. Settings modal cursor
        app.open_settings_modal();
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();
        let pos4 = terminal.get_cursor_position().unwrap();
        assert!(
            pos4.x > 0 && pos4.y > 0,
            "Cursor must be set inside SettingsModal"
        );

        // 5. Help modal cursor
        app.open_help_modal();
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();
        let pos5 = terminal.get_cursor_position().unwrap();
        assert!(
            pos5.x > 0 && pos5.y > 0,
            "Cursor must be set inside HelpModal"
        );
    }

    #[test]
    fn test_month_view_enter_opens_new_event_modal() {
        use rcal::app::ViewMode;
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        assert_eq!(app.current_view, ViewMode::Month);
        assert!(matches!(app.modal, ActiveModal::None));

        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();

        assert!(matches!(app.modal, ActiveModal::EventForm(_)));
        assert_eq!(app.current_view, ViewMode::Month);
    }
}
