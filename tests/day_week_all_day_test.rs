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

    #[test]
    fn test_week_view_keyboard_selection_and_enter_to_details() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use rcal::app::ActiveModal;
        use rcal::events::handle_crossterm_event;

        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();
        // 기본 시드 이벤트 정리
        for cal in &app.calendars {
            app.db.delete_events_by_calendar(&cal.id).unwrap();
        }
        app.reload_events().unwrap();

        // 2028-11-06은 월요일
        let monday = NaiveDate::from_ymd_opt(2028, 11, 6).unwrap();
        let tuesday = NaiveDate::from_ymd_opt(2028, 11, 7).unwrap();
        let thursday = NaiveDate::from_ymd_opt(2028, 11, 9).unwrap();

        app.selected_date = monday;
        app.current_view = ViewMode::Week;

        let cal_id = app.calendars[0].id.clone();

        // 화요일 종일 일정
        let all_day_ev = Event {
            id: "ev_tue_allday".to_string(),
            calendar_id: cal_id.clone(),
            title: "화요 세미나".to_string(),
            description: None,
            location: None,
            start_time: Local
                .from_local_datetime(&tuesday.and_hms_opt(0, 0, 0).unwrap())
                .unwrap(),
            end_time: Local
                .from_local_datetime(&tuesday.and_hms_opt(23, 59, 59).unwrap())
                .unwrap(),
            all_day: true,
            rrule: None,
            sync_status: SyncStatus::Synced,
            remote_id: None,
            etag: None,
            updated_at: Local::now(),
        };

        // 목요일 14:00 시간 일정
        let timed_ev = Event {
            id: "ev_thu_timed".to_string(),
            calendar_id: cal_id,
            title: "목요 미팅".to_string(),
            description: None,
            location: None,
            start_time: Local
                .from_local_datetime(&thursday.and_hms_opt(14, 0, 0).unwrap())
                .unwrap(),
            end_time: Local
                .from_local_datetime(&thursday.and_hms_opt(15, 0, 0).unwrap())
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

        assert_eq!(app.week_selected_event_idx, None);

        let tab_key = crossterm::event::Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        let backtab_key =
            crossterm::event::Event::Key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
        let enter_key =
            crossterm::event::Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        // 1. Tab 누름: 첫 번째 일정 (화요일 종일 일정) 선택 및 selected_date 이동
        handle_crossterm_event(&mut app, tab_key.clone()).unwrap();
        assert_eq!(app.week_selected_event_idx, Some(0));
        assert_eq!(app.selected_date, tuesday);

        // UI 렌더링 확인 (크래시 없이 정상 렌더링)
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();

        // 2. Tab 다시 누름: 두 번째 일정 (목요일 시간 일정) 선택 및 selected_date 이동
        handle_crossterm_event(&mut app, tab_key.clone()).unwrap();
        assert_eq!(app.week_selected_event_idx, Some(1));
        assert_eq!(app.selected_date, thursday);

        // 3. Enter 누름: 목요일 일정의 상세 모달 열림
        handle_crossterm_event(&mut app, enter_key.clone()).unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail("ev_thu_timed".to_string()));
        app.close_modal();
        assert_eq!(app.modal, ActiveModal::None);

        // 4. Tab 누르면 처음(인덱스 0)으로 순환
        handle_crossterm_event(&mut app, tab_key).unwrap();
        assert_eq!(app.week_selected_event_idx, Some(0));
        assert_eq!(app.selected_date, tuesday);

        // 5. BackTab 누르면 마지막(인덱스 1)으로 역순환
        handle_crossterm_event(&mut app, backtab_key).unwrap();
        assert_eq!(app.week_selected_event_idx, Some(1));
        assert_eq!(app.selected_date, thursday);

        // 6. Enter 누름: 목요일 일정 상세 모달 열림
        handle_crossterm_event(&mut app, enter_key).unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail("ev_thu_timed".to_string()));
    }

    #[test]
    fn test_new_event_modal_default_times_and_duration() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        // 1. default_hour가 None일 때: 현재 시각 기준 시작, 30분 뒤 종료
        app.open_new_event_modal(None);
        if let ActiveModal::EventForm(ref form) = app.modal {
            let now = Local::now();
            let expected_start = now.format("%H:%M").to_string();
            let expected_end = (now + chrono::Duration::minutes(30)).format("%H:%M").to_string();

            assert_eq!(form.start_time_str, expected_start);
            assert_eq!(form.end_time_str, expected_end);
            assert!(!form.all_day);
            assert!(form.is_new);
        } else {
            panic!("Expected EventForm modal to be opened");
        }

        // 2. default_hour가 Some(hour)일 때 (타임라인 슬롯 클릭):
        app.open_new_event_modal(Some(16));
        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.start_time_str, "16:00");
            assert_eq!(form.end_time_str, "17:00");
        } else {
            panic!("Expected EventForm modal to be opened");
        }

        // 3. 자정을 넘기는 일정(23:45 ~ 00:15) 저장 시 익일로 자동 조정 확인
        let target_date = NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();
        let mut form = rcal::app::EventFormState::new_with_date(target_date, app.calendars[0].id.clone(), None);
        form.title = "심야 작업".to_string();
        form.start_time_str = "23:45".to_string();
        form.end_time_str = "00:15".to_string();

        app.save_event_from_form(form).unwrap();
        app.reload_events().unwrap();

        let saved = app.events_cache.iter().find(|e| e.title == "심야 작업").unwrap();
        assert!(saved.end_time > saved.start_time);
        assert_eq!(saved.start_time.format("%Y-%m-%d %H:%M").to_string(), "2026-12-01 23:45");
        assert_eq!(saved.end_time.format("%Y-%m-%d %H:%M").to_string(), "2026-12-02 00:15");
    }
}
