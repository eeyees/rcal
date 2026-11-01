use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use ratatui::Terminal;
use rcal::app::{ActiveModal, App};
use rcal::events::handle_crossterm_event;
use rcal::mouse::HitTarget;
use rcal::storage::Database;
use rcal::ui;
use rcal::ui::theme::{Theme, ThemeCategory, ThemeMode};
use rcal::ui::theme_config::{parse_color, ThemeColorOverrides, ThemeConfigFile, VisibilityOptions};

fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> crossterm::event::Event {
    crossterm::event::Event::Key(KeyEvent::new(code, modifiers))
}

#[test]
fn test_theme_mode_all_and_cycling() {
    let all = ThemeMode::all();
    assert_eq!(all.len(), 37, "Must support at least 30 themes (currently 37)");

    // Verify round-trip strings for all 37 themes
    for mode in all {
        let s = mode.as_str();
        assert_eq!(ThemeMode::from_str(s), *mode);
    }

    // Verify cycling
    let first = all[0];
    let second = first.next();
    assert_eq!(second, all[1]);
    assert_eq!(first.prev(), all[all.len() - 1]);
}

#[test]
fn test_theme_categories_and_light_modes() {
    let all = ThemeMode::all();
    let mut dark_count = 0;
    let mut nature_count = 0;
    let mut vibrant_count = 0;
    let mut light_count = 0;

    for mode in all {
        match mode.category() {
            ThemeCategory::Dark => {
                dark_count += 1;
                assert!(!mode.is_light());
            }
            ThemeCategory::Nature => {
                nature_count += 1;
                assert!(!mode.is_light());
            }
            ThemeCategory::Vibrant => {
                vibrant_count += 1;
                assert!(!mode.is_light());
            }
            ThemeCategory::Light => {
                light_count += 1;
                assert!(mode.is_light());
                // Light themes must have explicit light background to avoid transparent black terminal clash
                let theme = Theme::from_mode(*mode);
                assert_ne!(theme.bg, Color::Reset, "Light themes must have non-Reset background");
            }
        }
    }

    assert_eq!(dark_count, 10);
    assert_eq!(nature_count, 9);
    assert_eq!(vibrant_count, 8);
    assert_eq!(light_count, 10);
}

#[test]
fn test_theme_palettes_distinction() {
    let modern = Theme::from_mode(ThemeMode::ModernDark);
    let nord = Theme::from_mode(ThemeMode::Nord);
    let dracula = Theme::from_mode(ThemeMode::Dracula);
    let tokyo = Theme::from_mode(ThemeMode::TokyoNight);
    let cat_mocha = Theme::from_mode(ThemeMode::CatppuccinMocha);
    let gruvbox = Theme::from_mode(ThemeMode::GruvboxDark);
    let gh_dark = Theme::from_mode(ThemeMode::GitHubDark);
    let gh_light = Theme::from_mode(ThemeMode::GitHubLight);

    assert_ne!(modern.panel_bg, nord.panel_bg);
    assert_ne!(dracula.primary, tokyo.primary);
    assert_ne!(cat_mocha.primary, gruvbox.primary);
    assert_ne!(gh_dark.text_normal, gh_light.text_normal);
}

#[test]
fn test_theme_config_parsing_and_overrides() {
    assert_eq!(parse_color("#FF00AA"), Some(Color::Rgb(255, 0, 170)));
    assert_eq!(parse_color("#fff"), Some(Color::Rgb(255, 255, 255)));
    assert_eq!(parse_color("rgb(10, 20, 30)"), Some(Color::Rgb(10, 20, 30)));
    assert_eq!(parse_color("reset"), Some(Color::Reset));
    assert_eq!(parse_color("invalid"), None);

    // Test overrides applied to Theme
    let mut config = ThemeConfigFile::default();
    let mut overrides = ThemeColorOverrides::default();
    overrides.border = Some("#123456".to_string());
    overrides.text_normal = Some("#ABCDEF".to_string());
    config.themes.insert("tokyo_night".to_string(), overrides);

    let mut theme = Theme::from_mode(ThemeMode::TokyoNight);
    config.apply_to(&mut theme);
    assert_eq!(theme.border, Color::Rgb(0x12, 0x34, 0x56));
    assert_eq!(theme.text_normal, Color::Rgb(0xAB, 0xCD, 0xEF));

    // Test high contrast mode
    let mut hc_config = ThemeConfigFile::default();
    hc_config.options = VisibilityOptions {
        force_opaque_bg: true,
        high_contrast: true,
        boost_muted_text: true,
    };

    let mut dark_theme = Theme::from_mode(ThemeMode::ModernDark);
    hc_config.apply_to(&mut dark_theme);
    assert_ne!(dark_theme.bg, Color::Reset, "force_opaque_bg should make background non-Reset");
    assert_eq!(dark_theme.text_normal, Color::Rgb(255, 255, 255));
}

#[test]
fn test_theme_db_persistence_and_startup() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_rcal_theme_{}.db", uuid::Uuid::new_v4()));
    let db = Database::new(Some(&db_path))?;

    let mut app = App::new(db)?;
    assert_eq!(app.theme.mode, ThemeMode::ModernDark);

    // Change to Tokyo Night
    app.set_theme(ThemeMode::TokyoNight);
    assert_eq!(app.theme.mode, ThemeMode::TokyoNight);
    assert_eq!(app.db.get_setting("theme")?, Some("tokyo_night".to_string()));

    // Create new App from same db file -> should restore Tokyo Night
    let db2 = Database::new(Some(&db_path))?;
    let app2 = App::new(db2)?;
    assert_eq!(app2.theme.mode, ThemeMode::TokyoNight);

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

    // Shortcut 'C' inside settings modal cycles theme backwards
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('C'), KeyModifiers::SHIFT))?;
    assert_eq!(app.theme.mode, ThemeMode::ModernDark);

    // Mouse click on category tab
    app.handle_mouse_click(0, 0)?; // click somewhere harmless
    app.set_theme_category(ThemeCategory::Light);
    assert_eq!(app.theme_category, ThemeCategory::Light);
    assert!(app.theme.mode.is_light());

    // Mouse click on Tokyo Night Light
    app.hit_map.register(
        ratatui::layout::Rect::new(20, 10, 16, 2),
        HitTarget::ModalSettingTheme(ThemeMode::TokyoNightLight),
    );
    app.handle_mouse_click(22, 11)?;
    assert_eq!(app.theme.mode, ThemeMode::TokyoNightLight);
    assert_eq!(app.db.get_setting("theme")?, Some("tokyo_night_light".to_string()));

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
