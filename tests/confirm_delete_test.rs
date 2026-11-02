#[cfg(test)]
mod tests {
    use chrono::Local;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::app::{ActiveModal, App, DeleteTarget};
    use rcal::events::handle_crossterm_event;
    use rcal::model::{Calendar, Event, ProviderType};
    use rcal::mouse::HitTarget;
    use rcal::storage::Database;

    fn make_key(code: KeyCode) -> crossterm::event::Event {
        crossterm::event::Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn test_confirm_delete_event_keyboard_cancel_and_confirm() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        // Create an event
        let cal_id = app.calendars[0].id.clone();
        let now = Local::now();
        let ev = Event::new(
            cal_id,
            "회의".to_string(),
            now,
            now + chrono::Duration::hours(1),
            false,
        );
        let ev_id = ev.id.clone();
        app.db.save_event(&ev).unwrap();
        app.reload_events().unwrap();

        // 1. Open EventDetail modal
        app.open_event_detail_modal(ev_id.clone());
        assert_eq!(app.modal, ActiveModal::EventDetail(ev_id.clone()));

        // 2. Press 'd' -> ConfirmDelete modal should open (NOT immediately deleted)
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(
            app.modal,
            ActiveModal::ConfirmDelete(DeleteTarget::Event { ref id, ref title })
            if id == &ev_id && title == "회의"
        ));
        // Verify event still exists in DB
        assert!(app.db.get_event_by_id(&ev_id).unwrap().is_some());

        // 3. Press 'n' (Cancel) -> should cancel deletion and return to EventDetail
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('n'))).unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail(ev_id.clone()));
        assert!(app.db.get_event_by_id(&ev_id).unwrap().is_some());

        // 4. Press 'd' again and then 'y' (Confirm) -> should delete event
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(app.modal, ActiveModal::ConfirmDelete(_)));
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('y'))).unwrap();

        assert_eq!(app.modal, ActiveModal::None);
        assert!(app.db.get_event_by_id(&ev_id).unwrap().is_none());
    }

    #[test]
    fn test_confirm_delete_calendar_cancel_with_esc() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let custom_cal = Calendar {
            id: "cal_test".to_string(),
            account_id: None,
            name: "테스트 캘린더".to_string(),
            color_code: "#4285F4".to_string(),
            provider: ProviderType::Local,
            enabled: true,
            is_default: false,
            remote_url: None,
            ctag: None,
        };
        app.db.save_calendar(&custom_cal).unwrap();
        app.reload_calendars().unwrap();

        // Open calendar modal
        app.open_calendar_modal();
        if let ActiveModal::CalendarManager(ref mut state) = app.modal {
            let idx = app
                .calendars
                .iter()
                .position(|c| c.id == "cal_test")
                .unwrap();
            state.selected_calendar_idx = idx;
        }

        // Press 'd' -> ConfirmDelete modal opens
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(
            app.modal,
            ActiveModal::ConfirmDelete(DeleteTarget::Calendar { ref id, ref name })
            if id == "cal_test" && name == "테스트 캘린더"
        ));

        // Press Esc -> Cancel deletion and return to CalendarManager
        handle_crossterm_event(&mut app, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));
        assert!(app
            .db
            .get_calendars()
            .unwrap()
            .iter()
            .any(|c| c.id == "cal_test"));

        // Press 'd' again and confirm with Enter
        if let ActiveModal::CalendarManager(ref mut state) = app.modal {
            let idx = app
                .calendars
                .iter()
                .position(|c| c.id == "cal_test")
                .unwrap();
            state.selected_calendar_idx = idx;
        }
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();

        // Calendar should now be deleted
        assert!(!app
            .db
            .get_calendars()
            .unwrap()
            .iter()
            .any(|c| c.id == "cal_test"));
    }

    #[test]
    fn test_confirm_delete_modal_mouse_interaction() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let cal_id = app.calendars[0].id.clone();
        let now = Local::now();
        let ev = Event::new(
            cal_id,
            "치과 예약".to_string(),
            now,
            now + chrono::Duration::hours(1),
            false,
        );
        let ev_id = ev.id.clone();
        app.db.save_event(&ev).unwrap();
        app.reload_events().unwrap();

        app.open_event_detail_modal(ev_id.clone());

        // 1. Mouse click Delete button on EventDetail modal
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| rcal::ui::render(f, &mut app)).unwrap();

        let del_target = app
            .hit_map
            .targets()
            .iter()
            .find(|(_, t)| matches!(t, HitTarget::ModalDelete))
            .map(|(r, _)| *r)
            .unwrap();

        app.handle_mouse_click(del_target.x + 2, del_target.y)
            .unwrap();
        assert!(matches!(app.modal, ActiveModal::ConfirmDelete(_)));

        // 2. Render ConfirmDelete modal and click Cancel button
        terminal.draw(|f| rcal::ui::render(f, &mut app)).unwrap();

        let no_target = app
            .hit_map
            .targets()
            .iter()
            .find(|(_, t)| matches!(t, HitTarget::ModalConfirmNo))
            .map(|(r, _)| *r)
            .unwrap();

        app.handle_mouse_click(no_target.x + 2, no_target.y)
            .unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail(ev_id.clone()));
        assert!(app.db.get_event_by_id(&ev_id).unwrap().is_some());

        // 3. Re-render EventDetail modal, click Delete again, render ConfirmDelete, and click Confirm button
        terminal.draw(|f| rcal::ui::render(f, &mut app)).unwrap();
        let del_target_again = app
            .hit_map
            .targets()
            .iter()
            .find(|(_, t)| matches!(t, HitTarget::ModalDelete))
            .map(|(r, _)| *r)
            .unwrap();

        app.handle_mouse_click(del_target_again.x + 2, del_target_again.y)
            .unwrap();
        assert!(matches!(app.modal, ActiveModal::ConfirmDelete(_)));

        terminal.draw(|f| rcal::ui::render(f, &mut app)).unwrap();

        let yes_target = app
            .hit_map
            .targets()
            .iter()
            .find(|(_, t)| matches!(t, HitTarget::ModalConfirmYes))
            .map(|(r, _)| *r)
            .unwrap();

        app.handle_mouse_click(yes_target.x + 2, yes_target.y)
            .unwrap();
        assert_eq!(app.modal, ActiveModal::None);
        assert!(app.db.get_event_by_id(&ev_id).unwrap().is_none());
    }
}
