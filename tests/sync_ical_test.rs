#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};
    use rcal::model::{Event, SyncStatus};
    use rcal::sync::ical::{event_to_ics, ics_to_events};

    #[test]
    fn test_event_to_ics_and_back() {
        let today = Local::now().date_naive();
        let start = Local
            .from_local_datetime(&today.and_hms_opt(15, 30, 0).unwrap())
            .unwrap();
        let end = Local
            .from_local_datetime(&today.and_hms_opt(17, 0, 0).unwrap())
            .unwrap();

        let mut event = Event::new(
            "cal_work".to_string(),
            "Naver Team Sync".to_string(),
            start,
            end,
            false,
        );
        event.location = Some("Green Factory 8F".to_string());
        event.description = Some("Discussing CalDAV endpoint integration".to_string());
        event.remote_id = Some("uuid-1234-5678".to_string());

        let ics = event_to_ics(&event);
        assert!(ics.contains("BEGIN:VCALENDAR"));
        assert!(ics.contains("BEGIN:VEVENT"));
        assert!(ics.contains("SUMMARY:Naver Team Sync"));
        assert!(ics.contains("LOCATION:Green Factory 8F"));
        assert!(ics.contains("UID:uuid-1234-5678"));

        // Parse back
        let parsed_events =
            ics_to_events(&ics, "cal_work", "/remote/cal.ics", Some("\"etag-999\""));
        assert_eq!(parsed_events.len(), 1);

        let parsed = &parsed_events[0];
        assert_eq!(parsed.title, "Naver Team Sync");
        assert_eq!(parsed.location.as_deref(), Some("Green Factory 8F"));
        assert_eq!(
            parsed.description.as_deref(),
            Some("Discussing CalDAV endpoint integration")
        );
        assert_eq!(parsed.remote_id.as_deref(), Some("uuid-1234-5678"));
        assert_eq!(parsed.etag.as_deref(), Some("etag-999"));
        assert_eq!(parsed.sync_status, SyncStatus::Synced);
    }

    #[test]
    fn test_all_day_event_ics() {
        let today = Local::now().date_naive();
        let start = Local
            .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let end = Local
            .from_local_datetime(&today.and_hms_opt(23, 59, 59).unwrap())
            .unwrap();

        let mut event = Event::new(
            "cal_holidays".to_string(),
            "National Holiday".to_string(),
            start,
            end,
            true,
        );
        event.remote_id = Some("holiday-2026-10-30".to_string());

        let ics = event_to_ics(&event);
        assert!(ics.contains("VALUE=DATE:"));

        let parsed = ics_to_events(&ics, "cal_holidays", "/holiday.ics", None);
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].all_day);
        assert_eq!(parsed[0].title, "National Holiday");
    }
}
