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

    // 4. Tab wraps directly back to Section 1: WeekStart (1 -> 2 -> 3 -> 4 -> 1)
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // 5. BackTab (Shift+Tab) goes backwards: WeekStart -> Language -> Theme -> Notifications -> WeekStart
    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::Language);

    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::Theme);

    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    send_key(&mut app, KeyCode::BackTab);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // 6. Test interaction within each section
    // In WeekStart: Right arrow changes week start day
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);
    send_key(&mut app, KeyCode::Right);
    assert_eq!(app.week_start_day, WeekStartDay::Monday);

    // Down arrow also cycles week start day (does not leave WeekStart tab!)
    send_key(&mut app, KeyCode::Down);
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Up arrow cycles backward within WeekStart
    send_key(&mut app, KeyCode::Up);
    assert_eq!(app.week_start_day, WeekStartDay::Monday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Left arrow cycles backward
    send_key(&mut app, KeyCode::Left);
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Tab moves to Notifications section
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    // In Notifications: Space toggles notification
    let initial_notif = app.notification_config.enabled;
    send_key(&mut app, KeyCode::Char(' '));
    assert_eq!(app.notification_config.enabled, !initial_notif);

    // Right arrow cycles lead minutes
    let old_lead = app.notification_config.lead_minutes;
    send_key(&mut app, KeyCode::Right);
    assert_ne!(app.notification_config.lead_minutes, old_lead);

    // Down arrow stays within Notifications section (does not leave to Theme!)
    send_key(&mut app, KeyCode::Down);
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    // Tab moves to Theme section
    send_key(&mut app, KeyCode::Tab);
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

    // Tab wraps to WeekStart
    send_key(&mut app, KeyCode::Tab);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Esc closes modal
    send_key(&mut app, KeyCode::Esc);
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

#[test]
fn test_month_view_weekend_coloring_with_week_start_modes() -> anyhow::Result<()> {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::ui;

    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    // November 2026: Nov 1 is Sunday, Nov 2 is Monday, Nov 7 is Saturday, Nov 8 is Sunday
    // Select Wednesday Nov 4, 2026 so it's not today and not selected on Nov 2 / Nov 7 / Nov 8
    app.selected_date = NaiveDate::from_ymd_opt(2026, 11, 4).unwrap();
    app.current_view = rcal::app::ViewMode::Month;

    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend)?;

    // 1. Monday as First Day of Week
    app.set_week_start_day(WeekStartDay::Monday);
    terminal.draw(|f| ui::render(f, &mut app))?;

    let text_normal = app.theme.text_normal;
    let accent = app.theme.accent;

    // Helper to find cell date position and verify character color
    let find_date_fg = |app: &App, terminal: &Terminal<TestBackend>, target_date: NaiveDate| -> ratatui::style::Color {
        let hit = app.hit_map.targets().iter().find(|(_, t)| *t == HitTarget::MonthDay(target_date));
        let rect = hit.expect("Target date should exist in month grid").0;
        let cell = &terminal.backend().buffer()[(rect.x + 2, rect.y + 1)];
        cell.fg
    };

    // Nov 2, 2026 is Monday: should be normal text, NOT weekend accent
    let mon_fg = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 2).unwrap());
    assert_eq!(mon_fg, text_normal, "Monday in Monday-start mode must NOT be colored with weekend accent");

    // Nov 7, 2026 is Saturday: should be weekend accent
    let sat_fg = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 7).unwrap());
    assert_eq!(sat_fg, accent, "Saturday in Monday-start mode must be colored with weekend accent");

    // Nov 8, 2026 is Sunday: should be weekend accent
    let sun_fg = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 8).unwrap());
    assert_eq!(sun_fg, accent, "Sunday in Monday-start mode must be colored with weekend accent");

    // 2. Sunday as First Day of Week
    app.set_week_start_day(WeekStartDay::Sunday);
    terminal.draw(|f| ui::render(f, &mut app))?;

    let mon_fg_sun = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 2).unwrap());
    assert_eq!(mon_fg_sun, text_normal, "Monday in Sunday-start mode must be normal text");

    let sat_fg_sun = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 7).unwrap());
    assert_eq!(sat_fg_sun, accent, "Saturday in Sunday-start mode must be weekend accent");

    let sun_fg_sun = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 8).unwrap());
    assert_eq!(sun_fg_sun, accent, "Sunday in Sunday-start mode must be weekend accent");

    // 3. Saturday as First Day of Week
    app.set_week_start_day(WeekStartDay::Saturday);
    terminal.draw(|f| ui::render(f, &mut app))?;

    let sat_fg_sat = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 7).unwrap());
    assert_eq!(sat_fg_sat, accent, "Saturday in Saturday-start mode must be weekend accent");

    let sun_fg_sat = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 8).unwrap());
    assert_eq!(sun_fg_sat, accent, "Sunday in Saturday-start mode must be weekend accent");

    let mon_fg_sat = find_date_fg(&app, &terminal, NaiveDate::from_ymd_opt(2026, 11, 9).unwrap());
    assert_eq!(mon_fg_sat, text_normal, "Monday in Saturday-start mode must be normal text");

    Ok(())
}

#[test]
fn test_settings_tab_bar_mouse_switching_and_state_preservation() -> anyhow::Result<()> {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::app::SettingsSection;
    use rcal::i18n::Language;
    use rcal::ui;
    use rcal::ui::theme::ThemeMode;

    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    let backend = TestBackend::new(100, 32);
    let mut terminal = Terminal::new(backend)?;

    app.open_settings_modal();
    assert_eq!(app.modal, ActiveModal::Settings);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Render to populate hit map
    terminal.draw(|f| ui::render(f, &mut app))?;

    // 1. Change WeekStart setting to Monday
    app.set_week_start_day(WeekStartDay::Monday);
    assert_eq!(app.week_start_day, WeekStartDay::Monday);

    // 2. Click Tab 2 (Notifications) via hit target
    let notif_tab_target = app.hit_map.targets().iter().find_map(|(r, t)| {
        if *t == HitTarget::ModalSettingTab(SettingsSection::Notifications) {
            Some((*r, t.clone()))
        } else {
            None
        }
    }).expect("Notifications tab should be registered in hit map");
    app.handle_mouse_click(notif_tab_target.0.x + 2, notif_tab_target.0.y)?;
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    // Render Notifications tab
    terminal.draw(|f| ui::render(f, &mut app))?;

    // Modify notification state
    let orig_enabled = app.notification_config.enabled;
    app.toggle_notifications();
    assert_eq!(app.notification_config.enabled, !orig_enabled);
    app.set_notification_lead_minutes(15);
    assert_eq!(app.notification_config.lead_minutes, 15);

    // 3. Click Tab 3 (Theme) via hit target
    let theme_tab_target = app.hit_map.targets().iter().find_map(|(r, t)| {
        if *t == HitTarget::ModalSettingTab(SettingsSection::Theme) {
            Some((*r, t.clone()))
        } else {
            None
        }
    }).expect("Theme tab should be registered in hit map");
    app.handle_mouse_click(theme_tab_target.0.x + 2, theme_tab_target.0.y)?;
    assert_eq!(app.settings_section, SettingsSection::Theme);

    // Render Theme tab and change theme
    terminal.draw(|f| ui::render(f, &mut app))?;
    app.set_theme(ThemeMode::Nord);
    assert_eq!(app.theme.mode, ThemeMode::Nord);

    // 4. Click Tab 4 (Language) via hit target
    let lang_tab_target = app.hit_map.targets().iter().find_map(|(r, t)| {
        if *t == HitTarget::ModalSettingTab(SettingsSection::Language) {
            Some((*r, t.clone()))
        } else {
            None
        }
    }).expect("Language tab should be registered in hit map");
    app.handle_mouse_click(lang_tab_target.0.x + 2, lang_tab_target.0.y)?;
    assert_eq!(app.settings_section, SettingsSection::Language);

    // Render Language tab and change language to Japanese
    terminal.draw(|f| ui::render(f, &mut app))?;
    app.set_language(Language::Ja);
    assert_eq!(app.lang, Language::Ja);

    // 5. Navigate back to Tab 1 (WeekStart) and verify ALL states are perfectly preserved
    let week_tab_target = app.hit_map.targets().iter().find_map(|(r, t)| {
        if *t == HitTarget::ModalSettingTab(SettingsSection::WeekStart) {
            Some((*r, t.clone()))
        } else {
            None
        }
    }).expect("WeekStart tab should be registered in hit map");
    app.handle_mouse_click(week_tab_target.0.x + 2, week_tab_target.0.y)?;
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Verify all states across tabs
    assert_eq!(app.week_start_day, WeekStartDay::Monday, "WeekStartDay must be preserved");
    assert_eq!(app.notification_config.enabled, !orig_enabled, "Notification enabled state must be preserved");
    assert_eq!(app.notification_config.lead_minutes, 15, "Lead minutes must be preserved");
    assert_eq!(app.theme.mode, ThemeMode::Nord, "Theme must be preserved");
    assert_eq!(app.lang, Language::Ja, "Language must be preserved");

    Ok(())
}

#[test]
fn test_settings_bracket_and_number_keys_navigation() -> anyhow::Result<()> {
    use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
    use rcal::app::SettingsSection;
    use rcal::events::handle_crossterm_event;

    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    app.open_settings_modal();
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    let send_key = |app: &mut App, code: KeyCode| {
        let key_ev = KeyEvent::new(code, KeyModifiers::NONE);
        handle_crossterm_event(app, CrosstermEvent::Key(key_ev)).unwrap();
    };

    // ']' advances tab
    send_key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    send_key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.settings_section, SettingsSection::Theme);

    // '[' goes to previous tab
    send_key(&mut app, KeyCode::Char('['));
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    // '4' directly jumps to Language tab
    send_key(&mut app, KeyCode::Char('4'));
    assert_eq!(app.settings_section, SettingsSection::Language);

    // ']' from Language tab wraps directly to WeekStart (cyclic)
    send_key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // '[' from WeekStart wraps to Language (cyclic)
    send_key(&mut app, KeyCode::Char('['));
    assert_eq!(app.settings_section, SettingsSection::Language);

    // Number keys directly jump from any tab
    send_key(&mut app, KeyCode::Char('2'));
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    send_key(&mut app, KeyCode::Char('3'));
    assert_eq!(app.settings_section, SettingsSection::Theme);

    send_key(&mut app, KeyCode::Char('1'));
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // In WeekStart: 's' selects Sunday, 'm' selects Monday, 't' selects Saturday
    send_key(&mut app, KeyCode::Char('m'));
    assert_eq!(app.week_start_day, WeekStartDay::Monday);

    send_key(&mut app, KeyCode::Char('t'));
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);

    send_key(&mut app, KeyCode::Char('s'));
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);

    // '1' while inside WeekStart keeps focus in WeekStart and selects Sunday
    send_key(&mut app, KeyCode::Char('m'));
    assert_eq!(app.week_start_day, WeekStartDay::Monday);
    send_key(&mut app, KeyCode::Char('1'));
    assert_eq!(app.settings_section, SettingsSection::WeekStart);
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);

    // '2' from WeekStart goes to Notifications
    send_key(&mut app, KeyCode::Char('2'));
    assert_eq!(app.settings_section, SettingsSection::Notifications);

    // '3' from Notifications goes to Theme
    send_key(&mut app, KeyCode::Char('3'));
    assert_eq!(app.settings_section, SettingsSection::Theme);

    // '4' from Theme goes to Language
    send_key(&mut app, KeyCode::Char('4'));
    assert_eq!(app.settings_section, SettingsSection::Language);

    Ok(())
}

#[test]
fn test_settings_week_start_modal_controls_and_rendering() -> anyhow::Result<()> {
    use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::app::SettingsSection;
    use rcal::events::handle_crossterm_event;
    use rcal::ui;

    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    app.open_settings_modal();
    assert_eq!(app.settings_section, SettingsSection::WeekStart);
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);

    let send_key = |app: &mut App, code: KeyCode| {
        let key_ev = KeyEvent::new(code, KeyModifiers::NONE);
        handle_crossterm_event(app, CrosstermEvent::Key(key_ev)).unwrap();
    };

    // Up and Down keys cycle week start day and DO NOT leave WeekStart tab
    send_key(&mut app, KeyCode::Down);
    assert_eq!(app.week_start_day, WeekStartDay::Monday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    send_key(&mut app, KeyCode::Down);
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    send_key(&mut app, KeyCode::Down);
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    send_key(&mut app, KeyCode::Up);
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    send_key(&mut app, KeyCode::Up);
    assert_eq!(app.week_start_day, WeekStartDay::Monday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Test vim navigation keys: j (down/next), k (up/prev), and arrow keys: Right (next), Left (prev)
    send_key(&mut app, KeyCode::Char('j'));
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    send_key(&mut app, KeyCode::Char('k'));
    assert_eq!(app.week_start_day, WeekStartDay::Monday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    send_key(&mut app, KeyCode::Right);
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    send_key(&mut app, KeyCode::Left);
    assert_eq!(app.week_start_day, WeekStartDay::Monday);
    assert_eq!(app.settings_section, SettingsSection::WeekStart);

    // Verify UI rendering contains button labels with [s], [m], [t]
    let backend = TestBackend::new(100, 32);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &mut app))?;

    let buffer = terminal.backend().buffer();
    let mut rendered_text = String::new();
    for y in 0..32 {
        for x in 0..100 {
            rendered_text.push_str(buffer[(x, y)].symbol());
        }
        rendered_text.push('\n');
    }

    assert!(rendered_text.contains("[s]"), "Rendered UI must contain [s] shortcut");
    assert!(rendered_text.contains("[m]"), "Rendered UI must contain [m] shortcut");
    assert!(rendered_text.contains("[t]"), "Rendered UI must contain [t] shortcut");
    assert!(!rendered_text.contains("[1/s]"), "Rendered UI must not contain old [1/s]");
    assert!(!rendered_text.contains("[2/m]"), "Rendered UI must not contain old [2/m]");
    assert!(!rendered_text.contains("[3/t]"), "Rendered UI must not contain old [3/t]");

    Ok(())
}
