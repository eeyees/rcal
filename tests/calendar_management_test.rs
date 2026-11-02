#[cfg(test)]
mod tests {
    use rcal::model::{Calendar, ProviderType};
    use rcal::storage::Database;

    #[test]
    fn test_create_and_delete_custom_calendar() {
        let db = Database::in_memory().unwrap();

        let initial_count = db.get_calendars().unwrap().len();

        // Create custom calendar (e.g. "스터디", "사이드 프로젝트")
        let custom_cal = Calendar {
            id: "cal_study_group".to_string(),
            account_id: None,
            name: "스터디 그룹".to_string(),
            color_code: "#9C27B0".to_string(), // Purple
            provider: ProviderType::Local,
            enabled: true,
            is_default: false,
            remote_url: None,
            ctag: None,
        };

        db.save_calendar(&custom_cal).unwrap();

        let calendars = db.get_calendars().unwrap();
        assert_eq!(calendars.len(), initial_count + 1);

        let created = calendars.iter().find(|c| c.id == "cal_study_group");
        assert!(created.is_some());
        assert_eq!(created.unwrap().name, "스터디 그룹");
        assert_eq!(created.unwrap().color_code, "#9C27B0");

        // Delete custom calendar
        db.delete_calendar("cal_study_group").unwrap();
        let remaining = db.get_calendars().unwrap();
        assert_eq!(remaining.len(), initial_count);
        assert!(remaining.iter().all(|c| c.id != "cal_study_group"));
    }

    #[test]
    fn test_calendar_modal_delete_click() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use rcal::app::{ActiveModal, App};
        use rcal::mouse::HitTarget;

        let db = Database::in_memory().unwrap();
        let custom_cal = Calendar {
            id: "cal_hobby".to_string(),
            account_id: None,
            name: "취미".to_string(),
            color_code: "#00ACC1".to_string(),
            provider: ProviderType::Local,
            enabled: true,
            is_default: false,
            remote_url: None,
            ctag: None,
        };
        db.save_calendar(&custom_cal).unwrap();

        let mut app = App::new(db).unwrap();
        app.open_calendar_modal();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                rcal::ui::render(f, &mut app);
            })
            .unwrap();

        let delete_target = app.hit_map.targets().iter().find(
            |(_, target)| matches!(target, HitTarget::ModalCalDelete(id) if id == &custom_cal.id),
        );
        assert!(
            delete_target.is_some(),
            "Delete button for custom calendar should be registered in hit_map"
        );

        let (del_rect, _) = delete_target.unwrap();
        let click_x = del_rect.x + del_rect.width / 2;
        let click_y = del_rect.y;

        app.handle_mouse_click(click_x, click_y).unwrap();
        assert!(matches!(app.modal, ActiveModal::ConfirmDelete(_)));

        app.confirm_delete().unwrap();

        let remaining = app.db.get_calendars().unwrap();
        assert!(remaining.iter().all(|c| c.id != "cal_hobby"));
    }

    #[test]
    fn test_calendar_modal_open_shortcuts() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use rcal::app::{ActiveModal, App};
        use rcal::events::handle_crossterm_event;

        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let make_key = |c: char, m: KeyModifiers| {
            crossterm::event::Event::Key(KeyEvent::new(KeyCode::Char(c), m))
        };

        // 1. Shortcut 'v' opens calendar modal
        handle_crossterm_event(&mut app, make_key('v', KeyModifiers::NONE)).unwrap();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));
        app.close_modal();

        // 2. Shortcut 'V' opens calendar modal
        handle_crossterm_event(&mut app, make_key('V', KeyModifiers::SHIFT)).unwrap();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));
        app.close_modal();

        // 3. Shortcut 'C' (Shift+C) opens calendar modal
        handle_crossterm_event(&mut app, make_key('C', KeyModifiers::SHIFT)).unwrap();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));
        app.close_modal();

        // 4. Korean 'ㅍ' (maps to 'v') opens calendar modal
        handle_crossterm_event(&mut app, make_key('ㅍ', KeyModifiers::NONE)).unwrap();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));
        app.close_modal();
    }

    #[test]
    fn test_calendar_modal_toggle_space_and_save() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use rcal::app::{ActiveModal, App};
        use rcal::events::handle_crossterm_event;

        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let make_key =
            |code: KeyCode| crossterm::event::Event::Key(KeyEvent::new(code, KeyModifiers::NONE));

        // Initially default calendar is enabled = true
        let default_cal = &app.calendars[0];
        let default_id = default_cal.id.clone();
        assert!(default_cal.enabled);

        // Open calendar modal via 'v'
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('v'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));

        // Press Space to toggle default calendar
        handle_crossterm_event(&mut app, make_key(KeyCode::Char(' '))).unwrap();
        if let ActiveModal::CalendarManager(ref state) = app.modal {
            assert_eq!(state.is_calendar_enabled(&default_id, true), false);
        }

        // Press 's' to save and close
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('s'))).unwrap();
        assert_eq!(app.modal, ActiveModal::None);

        // Verify that default calendar is now disabled in DB and app state
        let updated = app.db.get_calendars().unwrap();
        let found = updated.iter().find(|c| c.id == default_id).unwrap();
        assert_eq!(found.enabled, false);

        // Re-open modal, toggle back with Enter to save
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('v'))).unwrap();
        handle_crossterm_event(&mut app, make_key(KeyCode::Char(' '))).unwrap();
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();

        let updated2 = app.db.get_calendars().unwrap();
        let found2 = updated2.iter().find(|c| c.id == default_id).unwrap();
        assert_eq!(found2.enabled, true);
    }

    #[test]
    fn test_calendar_modal_number_key_toggle_and_cancel_esc() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use rcal::app::{ActiveModal, App};
        use rcal::events::handle_crossterm_event;

        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let make_key =
            |code: KeyCode| crossterm::event::Event::Key(KeyEvent::new(code, KeyModifiers::NONE));

        let default_id = app.calendars[0].id.clone();
        assert!(app.calendars[0].enabled);

        // Open calendar modal
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('v'))).unwrap();

        // Press '1' to toggle 1st calendar
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('1'))).unwrap();
        if let ActiveModal::CalendarManager(ref state) = app.modal {
            assert_eq!(state.is_calendar_enabled(&default_id, true), false);
        }

        // Press Esc to cancel without saving
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);

        // In DB, calendar must still be enabled = true (not changed)
        let cals = app.db.get_calendars().unwrap();
        assert_eq!(cals[0].enabled, true);
    }

    #[test]
    fn test_calendar_modal_mouse_interaction() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use rcal::app::{ActiveModal, App};
        use rcal::mouse::HitTarget;

        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();
        let cal_id = app.calendars[0].id.clone();

        app.open_calendar_modal();
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                rcal::ui::render(f, &mut app);
            })
            .unwrap();

        // 1. Find ModalCalToggle target for default calendar
        let toggle_target =
            app.hit_map.targets().iter().find(
                |(_, target)| matches!(target, HitTarget::ModalCalToggle(id) if id == &cal_id),
            );
        assert!(
            toggle_target.is_some(),
            "ModalCalToggle target should exist"
        );

        let (toggle_rect, _) = toggle_target.unwrap();
        app.handle_mouse_click(toggle_rect.x + 2, toggle_rect.y)
            .unwrap();

        if let ActiveModal::CalendarManager(ref state) = app.modal {
            assert_eq!(state.is_calendar_enabled(&cal_id, true), false);
        }

        // 2. Find ModalCalSaveVisibility target and click
        let save_target = app
            .hit_map
            .targets()
            .iter()
            .find(|(_, target)| matches!(target, HitTarget::ModalCalSaveVisibility));
        assert!(
            save_target.is_some(),
            "ModalCalSaveVisibility target should exist"
        );

        let (save_rect, _) = save_target.unwrap();
        app.handle_mouse_click(save_rect.x + 2, save_rect.y)
            .unwrap();
        assert_eq!(app.modal, ActiveModal::None);

        // Verify DB saved
        let cals = app.db.get_calendars().unwrap();
        assert_eq!(cals[0].enabled, false);
    }
}
