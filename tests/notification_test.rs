use chrono::{Duration as ChronoDuration, Local, NaiveDate, TimeZone};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rcal::app::{ActiveModal, App};
use rcal::events::handle_crossterm_event;
use rcal::i18n::Language;
use rcal::model::{Calendar, Event, ProviderType, SyncStatus};
use rcal::mouse::HitTarget;
use rcal::notification::{
    check_and_notify_upcoming, format_event_notification, NotificationConfig,
};
use rcal::storage::Database;

fn make_key(code: KeyCode) -> crossterm::event::Event {
    crossterm::event::Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn test_notification_config_defaults_and_db_persistence() -> anyhow::Result<()> {
    let db = Database::in_memory()?;

    // 1. Default config
    let cfg = NotificationConfig::load_from_db(&db);
    assert!(cfg.enabled);
    assert_eq!(cfg.lead_minutes, 10);
    assert!(cfg.sound);

    // 2. Modify and save
    let updated = NotificationConfig {
        enabled: false,
        lead_minutes: 15,
        sound: false,
    };
    updated.save_to_db(&db)?;

    // 3. Load again and verify
    let loaded = NotificationConfig::load_from_db(&db);
    assert!(!loaded.enabled);
    assert_eq!(loaded.lead_minutes, 15);
    assert!(!loaded.sound);

    Ok(())
}

#[test]
fn test_notification_formatting_and_localization() {
    let now = Local::now();
    let start = now + ChronoDuration::minutes(10);
    let end = start + ChronoDuration::hours(1);

    let event = Event {
        id: "evt_test".to_string(),
        calendar_id: "cal_work".to_string(),
        title: "Sprint Planning".to_string(),
        description: Some("Discussion on sprint backlog and deliverables".to_string()),
        location: Some("Meeting Room A".to_string()),
        start_time: start,
        end_time: end,
        all_day: false,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };

    // Korean formatting
    let (summary_ko, body_ko) =
        format_event_notification(&event, "회사 캘린더", 10, 10, Language::Ko);
    assert!(summary_ko.contains("Sprint Planning"));
    assert!(summary_ko.contains("10분 전"));
    assert!(body_ko.contains("Meeting Room A"));
    assert!(body_ko.contains("회사 캘린더"));
    assert!(body_ko.contains("Discussion on sprint backlog"));

    // English formatting
    let (summary_en, body_en) = format_event_notification(&event, "Work", 10, 10, Language::En);
    assert!(summary_en.contains("Upcoming"));
    assert!(summary_en.contains("in 10m"));
    assert!(body_en.contains("Work"));

    // Starting now (0 minutes)
    let (summary_now, _) = format_event_notification(&event, "Work", 0, 0, Language::Ko);
    assert!(summary_now.contains("시작"));

    // All-day event formatting
    let all_day_ev = Event {
        id: "evt_allday".to_string(),
        calendar_id: "cal_personal".to_string(),
        title: "Foundation Day Holiday".to_string(),
        description: None,
        location: None,
        start_time: start,
        end_time: end,
        all_day: true,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };

    let (summary_allday, _) = format_event_notification(&all_day_ev, "개인", 10, 0, Language::Ko);
    assert!(summary_allday.contains("오늘 종일 일정"));
}

#[test]
fn test_upcoming_event_detection_and_deduplication() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let now = Local.with_ymd_and_hms(2026, 11, 15, 14, 0, 0).unwrap();

    let cal_id = "cal_personal".to_string();

    // Event 1: Starts in 10 minutes (14:10) -> Should trigger for lead_minutes = 10
    let ev1 = Event {
        id: "evt_10m".to_string(),
        calendar_id: cal_id.clone(),
        title: "Team Sync".to_string(),
        description: Some("Daily check-in".to_string()),
        location: Some("Room 101".to_string()),
        start_time: now + ChronoDuration::minutes(10),
        end_time: now + ChronoDuration::minutes(40),
        all_day: false,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };
    db.save_event(&ev1)?;

    // Event 2: Starts in 40 minutes (14:40) -> Should NOT trigger for lead_minutes = 10
    let ev2 = Event {
        id: "evt_40m".to_string(),
        calendar_id: cal_id.clone(),
        title: "Future Event".to_string(),
        description: None,
        location: None,
        start_time: now + ChronoDuration::minutes(40),
        end_time: now + ChronoDuration::minutes(70),
        all_day: false,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };
    db.save_event(&ev2)?;

    // Event 3: Started 2 hours ago (12:00) -> Should NOT trigger
    let ev3 = Event {
        id: "evt_past".to_string(),
        calendar_id: cal_id.clone(),
        title: "Past Lunch".to_string(),
        description: None,
        location: None,
        start_time: now - ChronoDuration::hours(2),
        end_time: now - ChronoDuration::hours(1),
        all_day: false,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };
    db.save_event(&ev3)?;

    let config = NotificationConfig {
        enabled: true,
        lead_minutes: 10,
        sound: false,
    };

    // First check: should find and notify Event 1
    let notified1 = check_and_notify_upcoming(&db, &config, now, Language::Ko)?;
    assert_eq!(notified1.len(), 1);
    assert_eq!(notified1[0].0.id, "evt_10m");
    assert_eq!(notified1[0].0.title, "Team Sync");

    // Verify marked in DB
    assert!(db.has_notified("evt_10m", "lead_10m")?);
    assert!(!db.has_notified("evt_40m", "lead_10m")?);

    // Second check at the same time: deduplication should prevent re-notifying Event 1
    let notified2 = check_and_notify_upcoming(&db, &config, now, Language::Ko)?;
    assert!(notified2.is_empty(), "Event 1 must not be notified twice");

    Ok(())
}

#[test]
fn test_disabled_calendar_events_are_not_notified() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    // Use a date without seeded events
    let now = Local.with_ymd_and_hms(2026, 11, 15, 10, 0, 0).unwrap();

    // Create a disabled calendar
    let dis_cal = Calendar {
        id: "cal_disabled".to_string(),
        account_id: None,
        name: "Disabled Calendar".to_string(),
        color_code: "#FF0000".to_string(),
        provider: ProviderType::Local,
        enabled: false,
        is_default: false,
        remote_url: None,
        ctag: None,
    };
    db.save_calendar(&dis_cal)?;

    let ev = Event {
        id: "evt_dis".to_string(),
        calendar_id: "cal_disabled".to_string(),
        title: "Secret Meeting".to_string(),
        description: None,
        location: None,
        start_time: now + ChronoDuration::minutes(5),
        end_time: now + ChronoDuration::minutes(35),
        all_day: false,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };
    db.save_event(&ev)?;

    let config = NotificationConfig {
        enabled: true,
        lead_minutes: 10,
        sound: false,
    };

    let notified = check_and_notify_upcoming(&db, &config, now, Language::En)?;
    assert!(
        notified.is_empty(),
        "Events in disabled calendars should not trigger notifications"
    );

    Ok(())
}

#[test]
fn test_event_reschedule_clears_notification_history() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    // Use a date without seeded sample events
    let now = Local.with_ymd_and_hms(2026, 11, 15, 14, 0, 0).unwrap();
    let cal_id = "cal_personal".to_string();

    let mut ev = Event {
        id: "evt_resched".to_string(),
        calendar_id: cal_id,
        title: "Dentist Appointment".to_string(),
        description: None,
        location: None,
        start_time: now + ChronoDuration::minutes(10),
        end_time: now + ChronoDuration::minutes(40),
        all_day: false,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };
    db.save_event(&ev)?;

    let config = NotificationConfig {
        enabled: true,
        lead_minutes: 10,
        sound: false,
    };

    // First check notifies the event
    let notified1 = check_and_notify_upcoming(&db, &config, now, Language::Ko)?;
    assert_eq!(notified1.len(), 1);
    assert!(db.has_notified("evt_resched", "lead_10m")?);

    // Now reschedule event to 1 hour later (15:10)
    ev.start_time = now + ChronoDuration::hours(1) + ChronoDuration::minutes(10);
    ev.end_time = ev.start_time + ChronoDuration::minutes(30);
    db.save_event(&ev)?;

    // Modifying event should clear old notification records
    assert!(!db.has_notified("evt_resched", "lead_10m")?);

    // At 15:00, when checking again, it should trigger for the new time!
    let later = now + ChronoDuration::hours(1);
    let notified2 = check_and_notify_upcoming(&db, &config, later, Language::Ko)?;
    assert_eq!(notified2.len(), 1);
    assert_eq!(notified2[0].0.id, "evt_resched");

    Ok(())
}

#[test]
fn test_all_day_event_notification_timing() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let cal_id = "cal_personal".to_string();

    // All day event on 2026-11-15
    let day = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
    let ev = Event {
        id: "evt_allday_today".to_string(),
        calendar_id: cal_id,
        title: "National Holiday".to_string(),
        description: None,
        location: None,
        start_time: Local
            .from_local_datetime(&day.and_hms_opt(0, 0, 0).unwrap())
            .unwrap(),
        end_time: Local
            .from_local_datetime(&day.and_hms_opt(23, 59, 59).unwrap())
            .unwrap(),
        all_day: true,
        rrule: None,
        sync_status: SyncStatus::Synced,
        remote_id: None,
        etag: None,
        updated_at: Local::now(),
    };
    db.save_event(&ev)?;

    let config = NotificationConfig {
        enabled: true,
        lead_minutes: 10,
        sound: false,
    };

    // At 05:00 AM: should NOT notify yet to prevent waking user up
    let early_morning = Local.with_ymd_and_hms(2026, 11, 15, 5, 0, 0).unwrap();
    let notif_early = check_and_notify_upcoming(&db, &config, early_morning, Language::Ko)?;
    assert!(notif_early.is_empty());

    // At 09:00 AM: SHOULD notify
    let morning = Local.with_ymd_and_hms(2026, 11, 15, 9, 0, 0).unwrap();
    let notif_morning = check_and_notify_upcoming(&db, &config, morning, Language::Ko)?;
    assert_eq!(notif_morning.len(), 1);
    assert_eq!(notif_morning[0].0.id, "evt_allday_today");

    // Second check during day: should NOT notify again
    let afternoon = Local.with_ymd_and_hms(2026, 11, 15, 14, 0, 0).unwrap();
    let notif_afternoon = check_and_notify_upcoming(&db, &config, afternoon, Language::Ko)?;
    assert!(notif_afternoon.is_empty());

    Ok(())
}

#[test]
fn test_app_notification_toggles_and_mouse_interaction() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    // Initial default
    assert!(app.notification_config.enabled);
    assert_eq!(app.notification_config.lead_minutes, 10);
    assert!(app.notification_config.sound);

    // Toggle notification
    app.toggle_notifications();
    assert!(!app.notification_config.enabled);
    assert_eq!(
        app.db.get_setting("notification_enabled")?,
        Some("false".to_string())
    );

    // Toggle back
    app.toggle_notifications();
    assert!(app.notification_config.enabled);
    assert_eq!(
        app.db.get_setting("notification_enabled")?,
        Some("true".to_string())
    );

    // Set lead time to 30 min
    app.set_notification_lead_minutes(30);
    assert_eq!(app.notification_config.lead_minutes, 30);
    assert_eq!(
        app.db.get_setting("notification_lead_minutes")?,
        Some("30".to_string())
    );

    // Toggle sound
    app.toggle_notification_sound();
    assert!(!app.notification_config.sound);
    assert_eq!(
        app.db.get_setting("notification_sound")?,
        Some("false".to_string())
    );

    // Mouse clicks in Settings modal
    app.open_settings_modal();
    assert!(matches!(app.modal, ActiveModal::Settings));

    // Click toggle
    app.hit_map.register(
        ratatui::layout::Rect::new(10, 10, 15, 2),
        HitTarget::ModalSettingNotificationToggle,
    );
    app.handle_mouse_click(12, 11)?;
    assert!(!app.notification_config.enabled);

    // Click lead time 5m
    app.hit_map.register(
        ratatui::layout::Rect::new(26, 10, 10, 2),
        HitTarget::ModalSettingNotificationLead(5),
    );
    app.handle_mouse_click(28, 11)?;
    assert_eq!(app.notification_config.lead_minutes, 5);

    // Click sound toggle
    app.hit_map.register(
        ratatui::layout::Rect::new(40, 10, 15, 2),
        HitTarget::ModalSettingNotificationSound,
    );
    app.handle_mouse_click(42, 11)?;
    assert!(app.notification_config.sound);

    // Keyboard shortcuts in Settings modal: 'n' toggles notifications, 'o' toggles sound
    handle_crossterm_event(&mut app, make_key(KeyCode::Char('n')))?;
    assert!(app.notification_config.enabled);

    handle_crossterm_event(&mut app, make_key(KeyCode::Char('o')))?;
    assert!(!app.notification_config.sound);

    Ok(())
}

#[test]
fn test_cleanup_old_notifications() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let now = Local::now();

    db.mark_notified("evt_old", "lead_10m")?;
    assert!(db.has_notified("evt_old", "lead_10m")?);

    // Clean up notifications older than tomorrow (should delete)
    let cleaned = db.cleanup_old_notifications(now + ChronoDuration::days(1))?;
    assert!(cleaned >= 1);
    assert!(!db.has_notified("evt_old", "lead_10m")?);

    Ok(())
}
