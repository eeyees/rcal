use chrono::{Duration, Local};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use rcal::app::{ActiveModal, App, ViewMode};
use rcal::events::handle_crossterm_event;
use rcal::model::WeekStartDay;
use rcal::storage::Database;

fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> CrosstermEvent {
    CrosstermEvent::Key(KeyEvent::new(code, modifiers))
}

#[test]
fn test_hangul_vim_navigation() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;
    let start_date = app.selected_date;

    // 'ㅗ' is 'h' (Left) -> 1 day back
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅗ'), KeyModifiers::NONE))?;
    assert_eq!(app.selected_date, start_date - Duration::days(1));

    // 'ㅣ' is 'l' (Right) -> 1 day forward
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅣ'), KeyModifiers::NONE))?;
    assert_eq!(app.selected_date, start_date);

    // 'ㅓ' is 'j' (Down) -> 7 days forward in Month view
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅓ'), KeyModifiers::NONE))?;
    assert_eq!(app.selected_date, start_date + Duration::days(7));

    // 'ㅏ' is 'k' (Up) -> 7 days back in Month view
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅏ'), KeyModifiers::NONE))?;
    assert_eq!(app.selected_date, start_date);

    Ok(())
}

#[test]
fn test_hangul_view_switching() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;
    assert_eq!(app.current_view, ViewMode::Month);

    // 'ㅈ' is 'w' -> Week view
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅈ'), KeyModifiers::NONE))?;
    assert_eq!(app.current_view, ViewMode::Week);

    // 'ㅇ' is 'd' -> Day view
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅇ'), KeyModifiers::NONE))?;
    assert_eq!(app.current_view, ViewMode::Day);

    // 'ㅁ' is 'a' -> Agenda view
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅁ'), KeyModifiers::NONE))?;
    assert_eq!(app.current_view, ViewMode::Agenda);

    // 'ㅡ' is 'm' -> Month view
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅡ'), KeyModifiers::NONE))?;
    assert_eq!(app.current_view, ViewMode::Month);

    Ok(())
}

#[test]
fn test_hangul_action_shortcuts() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    // 'ㅅ' is 't' -> Today
    app.selected_date = Local::now().date_naive() - Duration::days(10);
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅅ'), KeyModifiers::NONE))?;
    assert_eq!(app.selected_date, Local::now().date_naive());

    // 'ㅊ' is 'c' -> New event modal
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅊ'), KeyModifiers::NONE))?;
    assert!(matches!(app.modal, ActiveModal::EventForm(_)));
    app.close_modal();

    // 'ㅜ' is 'n' -> New event modal
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅜ'), KeyModifiers::NONE))?;
    assert!(matches!(app.modal, ActiveModal::EventForm(_)));
    app.close_modal();

    // 'ㅐ' is 'o' -> Settings modal
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅐ'), KeyModifiers::NONE))?;
    assert!(matches!(app.modal, ActiveModal::Settings));
    app.close_modal();

    // 'ㅂ' is 'q' -> Quit app
    assert!(app.running);
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅂ'), KeyModifiers::NONE))?;
    assert!(!app.running);

    Ok(())
}

#[test]
fn test_hangul_shift_shortcuts() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    // Shift + 'ㅣ' is 'L' -> Opens Language Selection modal
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅣ'), KeyModifiers::SHIFT))?;
    assert!(matches!(app.modal, ActiveModal::Language(_)));
    app.close_modal();

    // Shift + 'ㅊ' is 'C' -> Calendar Manager modal
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅊ'), KeyModifiers::SHIFT))?;
    assert!(matches!(app.modal, ActiveModal::CalendarManager(_)));
    app.close_modal();

    // Shift + 'ㅁ' is 'A' -> Accounts modal
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅁ'), KeyModifiers::SHIFT))?;
    assert!(matches!(app.modal, ActiveModal::Accounts(_)));
    app.close_modal();

    Ok(())
}

#[test]
fn test_hangul_in_settings_modal() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;
    app.open_settings_modal();

    // In settings modal, 'ㅡ' (m) selects Monday
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅡ'), KeyModifiers::NONE))?;
    assert_eq!(app.week_start_day, WeekStartDay::Monday);

    // 'ㅅ' (t) selects Saturday
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅅ'), KeyModifiers::NONE))?;
    assert_eq!(app.week_start_day, WeekStartDay::Saturday);

    // 'ㄴ' (s) selects Sunday
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㄴ'), KeyModifiers::NONE))?;
    assert_eq!(app.week_start_day, WeekStartDay::Sunday);

    // 'ㅂ' (q) closes modal
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('ㅂ'), KeyModifiers::NONE))?;
    assert!(matches!(app.modal, ActiveModal::None));

    Ok(())
}

#[test]
fn test_korean_text_typing_preservation_in_form() -> anyhow::Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;
    app.open_new_event_modal(None);

    // When typing in text field (active_field == 0, Title), Korean characters are preserved!
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('팀'), KeyModifiers::NONE))?;
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char(' '), KeyModifiers::NONE))?;
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('회'), KeyModifiers::NONE))?;
    handle_crossterm_event(&mut app, make_key_event(KeyCode::Char('의'), KeyModifiers::NONE))?;

    if let ActiveModal::EventForm(ref form) = app.modal {
        assert_eq!(form.title, "팀 회의");
    } else {
        panic!("Modal should be EventForm");
    }

    Ok(())
}
