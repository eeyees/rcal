use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rcal::app::{ActiveModal, App};
use rcal::events::handle_crossterm_event;
use rcal::mouse::HitTarget;
use rcal::storage::Database;
use rcal::ui;
use rcal::ui::theme::{Theme, ThemeMode};

fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> crossterm::event::Event {
    crossterm::event::Event::Key(KeyEvent::new(code, modifiers))
}

#[test]
fn test_theme_mode_all_and_cycling() {
    let all = ThemeMode::all();
    assert_eq!(all.len(), 7);

    // Verify round-trip strings
    for mode in all {
        let s = mode.as_str();
        assert_eq!(ThemeMode::from_str(s), *mode);
    }

    // Next cycling
    let mut current = ThemeMode::ModernDark;
    current = current.next();
    assert_eq!(current, ThemeMode::Nord);
    current = current.next();
    assert_eq!(current, ThemeMode::Dracula);
    current = current.next();
    assert_eq!(current, ThemeMode::Forest);
    current = current.next();
    assert_eq!(current, ThemeMode::Sunset);
    current = current.next();
    assert_eq!(current, ThemeMode::Monokai);
    current = current.next();
    assert_eq!(current, ThemeMode::Solarized);
    current = current.next();
    assert_eq!(current, ThemeMode::ModernDark);

    // Prev cycling
    assert_eq!(ThemeMode::ModernDark.prev(), ThemeMode::Solarized);
    assert_eq!(ThemeMode::Nord.prev(), ThemeMode::ModernDark);
}

#[test]
fn test_theme_palettes_distinction() {
    let modern = Theme::from_mode(ThemeMode::ModernDark);
    let nord = Theme::from_mode(ThemeMode::Nord);
    let dracula = Theme::from_mode(ThemeMode::Dracula);
    let forest = Theme::from_mode(ThemeMode::Forest);
    let sunset = Theme::from_mode(ThemeMode::Sunset);
    let monokai = Theme::from_mode(ThemeMode::Monokai);
    let solarized = Theme::from_mode(ThemeMode::Solarized);

    // Verify that primary accent colors are unique across all themes
    let primary_colors = vec![
        modern.primary,
        nord.primary,
        dracula.primary,
        forest.primary,
        sunset.primary,
        monokai.primary,
        solarized.primary,
    ];
    let unique_primary_count = primary_colors.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_primary_count, 7, "Each theme must have a distinct primary accent color");
}

#[test]
fn test_theme_db_persistence_and_startup() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_rcal_theme_{}.db", uuid::Uuid::new_v4()));
    let db = Database::new(Some(&db_path))?;

    let mut app = App::new(db)?;
    assert_eq!(app.theme.mode, ThemeMode::ModernDark);

    // Change to Dracula
    app.set_theme(ThemeMode::Dracula);
    assert_eq!(app.theme.mode, ThemeMode::Dracula);
    assert_eq!(app.db.get_setting("theme")?, Some("dracula".to_string()));

    // Create new App from same db file -> should restore Dracula
    let db2 = Database::new(Some(&db_path))?;
    let app2 = App::new(db2)?;
    assert_eq!(app2.theme.mode, ThemeMode::Dracula);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[test]
fn test_normal_mode_theme_shortcuts() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;
    assert_eq!(app.theme.mode, ThemeMode::ModernDark);

    // Shift + T in normal mode cycles theme
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('T'), KeyModifiers::SHIFT))?;
    assert_eq!(app.theme.mode, ThemeMode::Nord);

    // Press Shift + T again -> Dracula
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('T'), KeyModifiers::SHIFT))?;
    assert_eq!(app.theme.mode, ThemeMode::Dracula);

    Ok(())
}

#[test]
fn test_settings_modal_theme_interaction() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    app.open_settings_modal();
    assert!(matches!(app.modal, ActiveModal::Settings));

    // Shortcut 'c' inside settings modal cycles theme
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('c'), KeyModifiers::NONE))?;
    assert_eq!(app.theme.mode, ThemeMode::Nord);

    // Shortcut 'T' inside settings modal cycles theme
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('T'), KeyModifiers::SHIFT))?;
    assert_eq!(app.theme.mode, ThemeMode::Dracula);

    // Mouse click on Forest theme button
    app.hit_map.register(
        ratatui::layout::Rect::new(20, 10, 15, 2),
        HitTarget::ModalSettingTheme(ThemeMode::Forest),
    );
    app.handle_mouse_click(22, 11)?;
    assert_eq!(app.theme.mode, ThemeMode::Forest);
    assert_eq!(app.db.get_setting("theme")?, Some("forest".to_string()));

    // Mouse click on Sunset theme button
    app.hit_map.register(
        ratatui::layout::Rect::new(40, 10, 15, 2),
        HitTarget::ModalSettingTheme(ThemeMode::Sunset),
    );
    app.handle_mouse_click(42, 11)?;
    assert_eq!(app.theme.mode, ThemeMode::Sunset);
    assert_eq!(app.db.get_setting("theme")?, Some("sunset".to_string()));

    Ok(())
}

#[test]
fn test_terminal_rendering_all_themes() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;

    for mode in ThemeMode::all() {
        app.set_theme(*mode);
        terminal.draw(|f| {
            ui::render(f, &mut app);
        })?;
    }

    Ok(())
}
