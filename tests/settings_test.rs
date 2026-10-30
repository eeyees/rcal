use chrono::NaiveDate;
use rcal::app::{ActiveModal, App};
use rcal::model::WeekStartDay;
use rcal::mouse::HitTarget;
use rcal::storage::Database;

#[test]
fn test_week_start_day_offsets_and_dates() {
    // 2026-10-31 is a Saturday
    let sat_date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
    assert_eq!(WeekStartDay::Sunday.days_offset(sat_date), 6);
    assert_eq!(WeekStartDay::Monday.days_offset(sat_date), 5);
    assert_eq!(WeekStartDay::Saturday.days_offset(sat_date), 0);

    assert_eq!(WeekStartDay::Sunday.week_start_date(sat_date), NaiveDate::from_ymd_opt(2026, 10, 25).unwrap());
    assert_eq!(WeekStartDay::Monday.week_start_date(sat_date), NaiveDate::from_ymd_opt(2026, 10, 26).unwrap());
    assert_eq!(WeekStartDay::Saturday.week_start_date(sat_date), NaiveDate::from_ymd_opt(2026, 10, 31).unwrap());

    // 2026-10-25 is a Sunday
    let sun_date = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
    assert_eq!(WeekStartDay::Sunday.days_offset(sun_date), 0);
    assert_eq!(WeekStartDay::Monday.days_offset(sun_date), 6);
    assert_eq!(WeekStartDay::Saturday.days_offset(sun_date), 1);

    assert_eq!(WeekStartDay::Sunday.week_start_date(sun_date), NaiveDate::from_ymd_opt(2026, 10, 25).unwrap());
    assert_eq!(WeekStartDay::Monday.week_start_date(sun_date), NaiveDate::from_ymd_opt(2026, 10, 19).unwrap());
    assert_eq!(WeekStartDay::Saturday.week_start_date(sun_date), NaiveDate::from_ymd_opt(2026, 10, 24).unwrap());
}

#[test]
fn test_db_settings_persistence() -> anyhow::Result<()> {
    let db = Database::in_memory()?;

    assert_eq!(db.get_setting("week_start_day")?, None);

    db.set_setting("week_start_day", "monday")?;
    assert_eq!(db.get_setting("week_start_day")?, Some("monday".to_string()));

    db.set_setting("week_start_day", "saturday")?;
    assert_eq!(db.get_setting("week_start_day")?, Some("saturday".to_string()));

    Ok(())
}

#[test]
fn test_app_settings_and_mouse_click() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_rcal_{}.db", uuid::Uuid::new_v4()));

    let db = Database::new(Some(&db_path))?;
    let mut app = App::new(db)?;

    // Default should be Sunday
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);

    // Open settings modal
    app.open_settings_modal();
    assert!(matches!(app.modal, ActiveModal::Settings));

    // Simulate mouse click on Monday button
    app.hit_map.register(
        ratatui::layout::Rect::new(10, 10, 15, 2),
        HitTarget::ModalSettingWeekStart(WeekStartDay::Monday),
    );
    app.handle_mouse_click(12, 11)?;
    assert_eq!(app.week_start_day, WeekStartDay::Monday);

    // Check DB persistence
    assert_eq!(app.db.get_setting("week_start_day")?, Some("monday".to_string()));

    // Simulate mouse click on Saturday button
    app.hit_map.register(
        ratatui::layout::Rect::new(30, 10, 15, 2),
        HitTarget::ModalSettingWeekStart(WeekStartDay::Saturday),
    );
    app.handle_mouse_click(32, 11)?;
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);
    assert_eq!(app.db.get_setting("week_start_day")?, Some("saturday".to_string()));

    // Create a new App instance pointing to the same DB file - it should automatically load Saturday
    let db2 = Database::new(Some(&db_path))?;
    let app2 = App::new(db2)?;
    assert_eq!(app2.week_start_day, WeekStartDay::Saturday);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}
