#[cfg(test)]
mod tests {
    use chrono::{Local, NaiveDate, TimeZone};
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use rcal::app::{ActiveModal, App, ViewMode};
    use rcal::model::{Event, SyncStatus};
    use rcal::mouse::HitTarget;
    use rcal::storage::Database;
    use rcal::ui;

    #[test]
    fn test_day_view_displays_all_day_events() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let today = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        app.selected_date = today;
        app.current_view = ViewMode::Day;

        // Create an all-day event
        let all_day_ev = Event {
            id: "ev_all_day_1".to_string(),
            calendar_id: "cal_work".to_string(),
            title: "전사 창립기념일 (종일)".to_string(),
            description: Some("Company Anniversary".to_string()),
            location: Some("본사 대강당".to_string()),
            start_time: Local
                .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
                .unwrap(),
            end_time: Local
                .from_local_datetime(&today.and_hms_opt(23, 59, 59).unwrap())
                .unwrap(),
            all_day: true,
            rrule: None,
            sync_status: SyncStatus::Synced,
            remote_id: None,
            etag: None,
            updated_at: Local::now(),
        };

        app.db.save_event(&all_day_ev).unwrap();
        app.reload_events().unwrap();

        // Render Day View into TestBackend
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();

        // Verify that the all-day event was registered in hit_map
        let found_event_hit = app.hit_map.targets().iter().any(|(_, target)| {
            if let HitTarget::EventItem(ref id) = target {
                id == "ev_all_day_1"
            } else {
                false
            }
        });
        assert!(
            found_event_hit,
            "All-day event must have a hit target registered in Day View"
        );

        // Verify that all-day slot is registered in hit_map
        let found_all_day_slot = app.hit_map.targets().iter().any(|(_, target)| {
            if let HitTarget::DayAllDaySlot(d) = target {
                *d == today
            } else {
                false
            }
        });
        assert!(
            found_all_day_slot,
            "DayAllDaySlot must be registered in Day View"
        );
    }

    #[test]
    fn test_click_all_day_slot_opens_modal_with_all_day_true() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        app.selected_date = date;
        app.current_view = ViewMode::Day;

        // Register DayAllDaySlot target and simulate mouse click
        app.hit_map
            .register(Rect::new(10, 3, 40, 2), HitTarget::DayAllDaySlot(date));
        app.handle_mouse_click(15, 4).unwrap();

        // Verify EventForm modal is opened with all_day = true
        match app.modal {
            ActiveModal::EventForm(ref form) => {
                assert!(
                    form.all_day,
                    "New event modal must have all_day = true when triggered from all-day slot"
                );
                assert_eq!(form.date_str, "2026-11-02");
            }
            _ => panic!("Expected EventForm modal to be opened"),
        }
    }

    #[test]
    fn test_week_view_displays_all_day_events() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let test_date = NaiveDate::from_ymd_opt(2026, 11, 4).unwrap(); // Wednesday
        app.selected_date = test_date;
        app.current_view = ViewMode::Week;

        let all_day_ev = Event {
            id: "ev_week_all_day".to_string(),
            calendar_id: "cal_personal".to_string(),
            title: "가족 여행 (종일)".to_string(),
            description: None,
            location: None,
            start_time: Local
                .from_local_datetime(&test_date.and_hms_opt(0, 0, 0).unwrap())
                .unwrap(),
            end_time: Local
                .from_local_datetime(&test_date.and_hms_opt(23, 59, 59).unwrap())
                .unwrap(),
            all_day: true,
            rrule: None,
            sync_status: SyncStatus::Synced,
            remote_id: None,
            etag: None,
            updated_at: Local::now(),
        };

        app.db.save_event(&all_day_ev).unwrap();
        app.reload_events().unwrap();

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();

        let found_event_hit = app.hit_map.targets().iter().any(|(_, target)| {
            if let HitTarget::EventItem(ref id) = target {
                id == "ev_week_all_day"
            } else {
                false
            }
        });
        assert!(
            found_event_hit,
            "All-day event must have a hit target registered in Week View"
        );
    }

    #[test]
    fn test_day_view_keyboard_selection_and_enter_to_details() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use rcal::app::ActiveModal;
        use rcal::events::handle_crossterm_event;

        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let test_date = NaiveDate::from_ymd_opt(2028, 12, 25).unwrap();
        app.selected_date = test_date;
        app.current_view = ViewMode::Day;

        let all_day_ev = Event {
            id: "ev_day_1".to_string(),
            calendar_id: "cal_personal".to_string(),
            title: "워크샵".to_string(),
            description: None,
            location: None,
            start_time: Local
                .from_local_datetime(&test_date.and_hms_opt(0, 0, 0).unwrap())
                .unwrap(),
            end_time: Local
                .from_local_datetime(&test_date.and_hms_opt(23, 59, 59).unwrap())
                .unwrap(),
            all_day: true,
            rrule: None,
            sync_status: SyncStatus::Synced,
            remote_id: None,
            etag: None,
            updated_at: Local::now(),
        };

        let timed_ev = Event {
            id: "ev_day_2".to_string(),
            calendar_id: "cal_personal".to_string(),
            title: "회의".to_string(),
            description: None,
            location: None,
            start_time: Local
                .from_local_datetime(&test_date.and_hms_opt(14, 0, 0).unwrap())
                .unwrap(),
            end_time: Local
                .from_local_datetime(&test_date.and_hms_opt(15, 0, 0).unwrap())
                .unwrap(),
            all_day: false,
            rrule: None,
            sync_status: SyncStatus::Synced,
            remote_id: None,
            etag: None,
            updated_at: Local::now(),
        };

        app.db.save_event(&all_day_ev).unwrap();
        app.db.save_event(&timed_ev).unwrap();
        app.reload_events().unwrap();

        assert_eq!(app.day_selected_event_idx, None);

        // 1. Press Tab: Select first event (all-day event)
        let tab_key = crossterm::event::Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        handle_crossterm_event(&mut app, tab_key.clone()).unwrap();
        assert_eq!(app.day_selected_event_idx, Some(0));

        // 2. Press Tab again: Select second event (timed event)
        handle_crossterm_event(&mut app, tab_key).unwrap();
        assert_eq!(app.day_selected_event_idx, Some(1));

        // 3. Press Enter: Opens EventDetail for ev_day_2
        let enter_key =
            crossterm::event::Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        handle_crossterm_event(&mut app, enter_key.clone()).unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail("ev_day_2".to_string()));

        // Close modal
        app.close_modal();
        assert_eq!(app.modal, ActiveModal::None);

        // 4. Press BackTab: cycles back to ev_day_1 (index 0)
        let backtab_key =
            crossterm::event::Event::Key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
        handle_crossterm_event(&mut app, backtab_key).unwrap();
        assert_eq!(app.day_selected_event_idx, Some(0));

        // 5. Press Enter: Opens EventDetail for ev_day_1
        handle_crossterm_event(&mut app, enter_key.clone()).unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail("ev_day_1".to_string()));

        app.close_modal();

        // 6. Test hour-based Enter fallback when day_selected_event_idx is None:
        app.day_selected_event_idx = None;
        app.selected_hour = 14; // Matches ev_day_2 (14:00)
        handle_crossterm_event(&mut app, enter_key).unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail("ev_day_2".to_string()));
    }
}
