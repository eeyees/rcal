#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use ratatui::layout::Rect;
    use rcal::app::{ActiveModal, App, ViewMode};
    use rcal::mouse::{HitMap, HitTarget};
    use rcal::storage::Database;

    #[test]
    fn test_hit_map_registration_and_lookup() {
        let mut hit_map = HitMap::new();

        // Register tab buttons
        hit_map.register(Rect::new(0, 0, 10, 1), HitTarget::TabMonth);
        hit_map.register(Rect::new(12, 0, 10, 1), HitTarget::TabWeek);
        hit_map.register(Rect::new(24, 0, 10, 1), HitTarget::TabDay);

        // Click on TabMonth
        assert_eq!(hit_map.check(5, 0), Some(HitTarget::TabMonth));
        // Click on TabWeek
        assert_eq!(hit_map.check(15, 0), Some(HitTarget::TabWeek));
        // Click on empty space
        assert_eq!(hit_map.check(11, 0), None);

        // Topmost takes priority (e.g. modal on top of cell)
        hit_map.register(Rect::new(0, 0, 30, 20), HitTarget::ModalClose);
        assert_eq!(hit_map.check(5, 0), Some(HitTarget::ModalClose));
    }

    #[test]
    fn test_mouse_click_view_switching() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        assert_eq!(app.current_view, ViewMode::Month);

        // Register targets into app.hit_map as if rendered
        app.hit_map
            .register(Rect::new(10, 0, 10, 1), HitTarget::TabWeek);
        app.hit_map
            .register(Rect::new(20, 0, 10, 1), HitTarget::TabDay);
        app.hit_map
            .register(Rect::new(30, 0, 10, 1), HitTarget::TabAgenda);

        // Click TabWeek
        app.handle_mouse_click(15, 0).unwrap();
        assert_eq!(app.current_view, ViewMode::Week);

        // Click TabDay
        app.handle_mouse_click(25, 0).unwrap();
        assert_eq!(app.current_view, ViewMode::Day);

        // Click TabAgenda
        app.handle_mouse_click(35, 0).unwrap();
        assert_eq!(app.current_view, ViewMode::Agenda);
    }

    #[test]
    fn test_mouse_click_date_selection() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let target_date = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        app.hit_map
            .register(Rect::new(5, 5, 10, 4), HitTarget::MonthDay(target_date));

        // Click on date cell
        app.handle_mouse_click(7, 6).unwrap();
        assert_eq!(app.selected_date, target_date);

        // Double click on already selected date opens Day view
        app.handle_mouse_click(7, 6).unwrap();
        assert_eq!(app.current_view, ViewMode::Day);
    }

    #[test]
    fn test_mouse_click_create_event_from_slot() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let target_date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
        app.hit_map.register(
            Rect::new(10, 8, 12, 2),
            HitTarget::WeekSlot(target_date, 14),
        );

        // Click on week time slot at 14:00
        app.handle_mouse_click(12, 9).unwrap();
        assert_eq!(app.selected_date, target_date);
        assert_eq!(app.selected_hour, 14);

        if let ActiveModal::EventForm(ref form) = app.modal {
            assert_eq!(form.start_time_str, "14:00");
            assert_eq!(form.end_time_str, "15:00");
            assert!(form.is_new);
        } else {
            panic!("Expected EventForm modal to open");
        }
    }
}
