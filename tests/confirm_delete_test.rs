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

    #[test]
    fn test_agenda_view_delete_event_via_enter_detail_and_confirm() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();
        // 기본 시드 이벤트 정리
        for cal in &app.calendars {
            app.db.delete_events_by_calendar(&cal.id).unwrap();
        }
        app.reload_events().unwrap();

        let cal_id = app.calendars[0].id.clone();
        let now = Local::now();

        // 과거 일정 2개 (과거 일정이 DB에 있어도 화면의 오늘 일정이 정확히 선택되어 삭제되어야 함)
        let past_ev1 = Event::new(
            cal_id.clone(),
            "과거 일정 1".to_string(),
            now - chrono::Duration::days(10),
            now - chrono::Duration::days(10) + chrono::Duration::hours(1),
            false,
        );
        let past_ev2 = Event::new(
            cal_id.clone(),
            "과거 일정 2".to_string(),
            now - chrono::Duration::days(5),
            now - chrono::Duration::days(5) + chrono::Duration::hours(1),
            false,
        );
        // 오늘 일정 1개
        let today_ev = Event::new(
            cal_id.clone(),
            "오늘 삭제할 회의".to_string(),
            now,
            now + chrono::Duration::hours(1),
            false,
        );
        let today_id = today_ev.id.clone();
        let past_id1 = past_ev1.id.clone();

        app.db.save_event(&past_ev1).unwrap();
        app.db.save_event(&past_ev2).unwrap();
        app.db.save_event(&today_ev).unwrap();
        app.reload_events().unwrap();

        // 1. 목록뷰(Agenda)로 전환
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('4'))).unwrap();
        assert_eq!(app.current_view, rcal::app::ViewMode::Agenda);
        assert_eq!(app.agenda_selected_index, 0);

        // 목록뷰에 나타나는 첫 번째 이벤트가 오늘 일정인지 확인
        let agenda_list = app.agenda_events();
        assert_eq!(agenda_list.len(), 1);
        assert_eq!(agenda_list[0].1.id, today_id);

        // 2. Enter 키로 상세 모달 열기 -> 과거 일정이 아닌 화면에 선택된 '오늘 일정'의 모달이 열려야 함
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();
        assert_eq!(app.modal, ActiveModal::EventDetail(today_id.clone()));

        // 3. 'd'로 삭제 확인 모달 열기
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(
            app.modal,
            ActiveModal::ConfirmDelete(DeleteTarget::Event { ref id, .. }) if id == &today_id
        ));

        // 4. 'y'로 삭제 확인 -> 오늘 일정이 DB 및 캐시에서 제거되어야 함
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('y'))).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
        assert!(app.db.get_event_by_id(&today_id).unwrap().is_none());
        assert!(!app.events_cache.iter().any(|e| e.id == today_id));
        // 과거 일정은 삭제되지 않고 유지되어야 함
        assert!(app.db.get_event_by_id(&past_id1).unwrap().is_some());
    }

    #[test]
    fn test_agenda_view_direct_delete_shortcut_and_index_clamping() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();
        // 기본 시드 이벤트 정리
        for cal in &app.calendars {
            app.db.delete_events_by_calendar(&cal.id).unwrap();
        }
        app.reload_events().unwrap();

        let cal_id = app.calendars[0].id.clone();
        let now = Local::now();

        // 2개의 오늘/미래 일정 생성
        let ev1 = Event::new(
            cal_id.clone(),
            "일정 1".to_string(),
            now,
            now + chrono::Duration::hours(1),
            false,
        );
        let ev2 = Event::new(
            cal_id.clone(),
            "일정 2".to_string(),
            now + chrono::Duration::hours(2),
            now + chrono::Duration::hours(3),
            false,
        );
        let ev1_id = ev1.id.clone();
        let ev2_id = ev2.id.clone();

        app.db.save_event(&ev1).unwrap();
        app.db.save_event(&ev2).unwrap();
        app.reload_events().unwrap();

        // 목록뷰로 전환
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('4'))).unwrap();
        assert_eq!(app.current_view, rcal::app::ViewMode::Agenda);

        // 아래 방향키(j)로 2번째 일정(인덱스 1) 선택
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.agenda_selected_index, 1);

        // 목록뷰에서 바로 'd'를 눌러 삭제 모달 띄우기
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(
            app.modal,
            ActiveModal::ConfirmDelete(DeleteTarget::Event { ref id, .. }) if id == &ev2_id
        ));

        // Enter로 삭제 승인
        handle_crossterm_event(&mut app, make_key(KeyCode::Enter)).unwrap();
        assert_eq!(app.modal, ActiveModal::None);
        assert!(app.db.get_event_by_id(&ev2_id).unwrap().is_none());

        // 목록에는 1개만 남았으므로 인덱스가 0으로 클램핑되어야 함
        assert_eq!(app.agenda_events().len(), 1);
        assert_eq!(app.agenda_selected_index, 0);

        // 남은 일정 1에 대해 Delete 키로 삭제 요청
        handle_crossterm_event(&mut app, make_key(KeyCode::Delete)).unwrap();
        assert!(matches!(
            app.modal,
            ActiveModal::ConfirmDelete(DeleteTarget::Event { ref id, .. }) if id == &ev1_id
        ));
        handle_crossterm_event(&mut app, make_key(KeyCode::Char('y'))).unwrap();

        // 모두 삭제되어 빈 목록이 되고 인덱스는 0이어야 함
        assert_eq!(app.agenda_events().len(), 0);
        assert_eq!(app.agenda_selected_index, 0);
    }
}
