#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};
    use rcal::model::Event;
    use rcal::storage::Database;

    #[test]
    fn test_in_memory_database_init() {
        let db = Database::in_memory().expect("Failed to create in-memory db");
        let calendars = db.get_calendars().expect("Failed to get calendars");
        
        // Should have 3 default calendars: Personal, Work, Holidays
        assert_eq!(calendars.len(), 3);
        assert!(calendars.iter().any(|c| c.name == "Personal"));
        assert!(calendars.iter().any(|c| c.name == "Work"));
        assert!(calendars.iter().any(|c| c.name == "Holidays"));

        let events = db.get_all_events().expect("Failed to get events");
        assert!(!events.is_empty(), "Should have seeded default events");
    }

    #[test]
    fn test_event_crud() {
        let db = Database::in_memory().expect("Failed to create in-memory db");
        let today = Local::now().date_naive();
        let start = Local.from_local_datetime(&today.and_hms_opt(14, 0, 0).unwrap()).unwrap();
        let end = Local.from_local_datetime(&today.and_hms_opt(15, 0, 0).unwrap()).unwrap();

        let mut event = Event::new("cal_work".to_string(), "Test Meeting".to_string(), start, end, false);
        event.location = Some("Conference Room B".to_string());
        let event_id = event.id.clone();

        // Save
        db.save_event(&event).expect("Failed to save event");

        // Retrieve
        let retrieved = db.get_event_by_id(&event_id).expect("Query error").expect("Event not found");
        assert_eq!(retrieved.title, "Test Meeting");
        assert_eq!(retrieved.location, Some("Conference Room B".to_string()));

        // Update
        event.title = "Updated Meeting".to_string();
        db.save_event(&event).expect("Failed to update event");
        let updated = db.get_event_by_id(&event_id).unwrap().unwrap();
        assert_eq!(updated.title, "Updated Meeting");

        // Delete
        db.delete_event(&event_id).expect("Failed to delete event");
        let deleted = db.get_event_by_id(&event_id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_toggle_calendar() {
        let db = Database::in_memory().expect("Failed to create in-memory db");
        let calendars = db.get_calendars().unwrap();
        let personal = calendars.iter().find(|c| c.name == "Personal").unwrap();
        assert!(personal.enabled);

        let new_state = db.toggle_calendar(&personal.id).unwrap();
        assert!(!new_state);

        let updated_calendars = db.get_calendars().unwrap();
        let updated_personal = updated_calendars.iter().find(|c| c.id == personal.id).unwrap();
        assert!(!updated_personal.enabled);
    }
}
