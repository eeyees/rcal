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

#[test]
fn test_settings_modal_tab_section_navigation() -> anyhow::Result<()> {
    use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
    use rcal::app::SettingsSection;
    use rcal::events::handle_crossterm_event;

    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    app.open_settings_modal();
    assert_eq!(app.modal, ActiveModal::Settings);
    // Initially focused on WeekStart
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    let send_key = |app: &mut App, code: KeyCode| {
        let key_ev = KeyEvent::new(code, KeyModifiers::NONE);
        handle_crossterm_event(app, CrosstermEvent::Key(key_ev)).unwrap();
    };

    // 1. Tab moves to Section 2: Notifications
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    // 2. Tab moves to Section 3: Theme
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::Theme);

    // 3. Tab moves to Section 4: Language
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::Language);

    // 4. Tab moves to Section 5: Close
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::Close);

    // 5. Tab wraps back to Section 1: WeekStart
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // 6. BackTab (Shift+Tab) goes backwards: WeekStart -> Close -> Language -> Theme -> Notifications -> WeekStart
    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::Close);

    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::Language);

    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::Theme);

    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // 7. Test interaction within each section
    // In WeekStart: Right arrow changes week start day
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);
    send_key(&mut app, KeyCode::Right);
    assert_eq!(app.week_start_day, WeekStartDay::Monday);

    // Down arrow moves to Notifications section
    send_key(&mut app, KeyCode::Down);
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    // In Notifications: Space toggles notification
    let initial_notif = app.notification_config.enabled;
    send_key(&mut app, KeyCode::Char(' '));
    assert_eq!(app.notification_config.enabled, !initial_notif);

    // Right arrow cycles lead minutes
    let old_lead = app.notification_config.lead_minutes;
    send_key(&mut app, KeyCode::Right);
    assert_ne!(app.notification_config.lead_minutes, old_lead);

    // Down arrow moves to Theme section
    send_key(&mut app, KeyCode::Down);
    assert_eq!(app.settings_section, SettingsSection::Theme);

    // In Theme: Right arrow cycles theme
    let old_theme = app.theme.mode;
    send_key(&mut app, KeyCode::Right);
    assert_ne!(app.theme.mode, old_theme);

    // Tab to Language
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::Language);

    // In Language: Right arrow navigates grid, Enter selects
    let old_lang = app.lang;
    send_key(&mut app, KeyCode::Right);
    send_key(&mut app, KeyCode::Enter);
    assert_ne!(app.lang, old_lang);

    // Tab to Close button and Enter closes modal
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::Close);
    send_key(&mut app, KeyCode::Enter);
    assert_eq!(app.modal, ActiveModal::None);

    Ok(())
}

#[test]
fn test_settings_modal_cursor_per_section() -> anyhow::Result<()> {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::app::SettingsSection;
    use rcal::ui;

    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    let backend = TestBackend::new(100, 32);
    let mut terminal = Terminal::new(backend)?;

    app.open_settings_modal();

    // Section 1: WeekStart cursor
    app.settings_section = SettingsSection::WeekStart;
    terminal.draw(|f| ui::render(f, &mut app))?;
    let pos_week = terminal.get_cursor_position()?;
    assert!(pos_week.x > 0 && pos_week.y > 0);

    // Section 2: Notifications cursor
    app.settings_section = SettingsSection::Notifications;
    terminal.draw(|f| ui::render(f, &mut app))?;
    let pos_notif = terminal.get_cursor_position()?;
    assert!(pos_notif.x > 0 && pos_notif.y > 0);
    assert_ne!(pos_notif.y, pos_week.y);

    // Section 3: Theme cursor
    app.settings_section = SettingsSection::Theme;
    terminal.draw(|f| ui::render(f, &mut app))?;
    let pos_theme = terminal.get_cursor_position()?;
    assert!(pos_theme.x > 0 && pos_theme.y > 0);
    assert_ne!(pos_theme.y, pos_notif.y);

    // Section 4: Language cursor
    app.settings_section = SettingsSection::Language;
    terminal.draw(|f| ui::render(f, &mut app))?;
    let pos_lang = terminal.get_cursor_position()?;
    assert!(pos_lang.x > 0 && pos_lang.y > 0);
    assert_ne!(pos_lang.y, pos_theme.y);

    // Section 5: Close button cursor
    app.settings_section = SettingsSection::Close;
    terminal.draw(|f| ui::render(f, &mut app))?;
    let pos_close = terminal.get_cursor_position()?;
    assert!(pos_close.x > 0 && pos_close.y > 0);
    assert_ne!(pos_close.y, pos_lang.y);

    Ok(())
}
