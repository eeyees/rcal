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

        let remaining = app.db.get_calendars().unwrap();
        assert!(remaining.iter().all(|c| c.id != "cal_hobby"));
    }
}
