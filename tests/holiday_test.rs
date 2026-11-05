use anyhow::Result;
use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rcal::app::{ActiveModal, App, SettingsSection, ViewMode};
use rcal::events::handle_crossterm_event;
use rcal::holidays::{get_holidays, HolidayRegion};
use rcal::mouse::HitTarget;
use rcal::storage::Database;
use rcal::ui;

#[test]
fn test_holiday_region_definitions_and_labels() {
    assert_eq!(HolidayRegion::from_code("kr"), HolidayRegion::Kr);
    assert_eq!(HolidayRegion::from_code("korea"), HolidayRegion::Kr);
    assert_eq!(HolidayRegion::from_code("us"), HolidayRegion::Us);
    assert_eq!(HolidayRegion::from_code("usa"), HolidayRegion::Us);
    assert_eq!(HolidayRegion::from_code("jp"), HolidayRegion::Jp);
    assert_eq!(HolidayRegion::from_code("japan"), HolidayRegion::Jp);
    assert_eq!(HolidayRegion::from_code("none"), HolidayRegion::None);

    assert!(HolidayRegion::Kr.ical_feed_url().is_some());
    assert!(HolidayRegion::Us.ical_feed_url().is_some());
    assert!(HolidayRegion::Jp.ical_feed_url().is_some());
    assert!(HolidayRegion::None.ical_feed_url().is_none());

    // 2026 Korean holidays
    let kr_2026 = get_holidays(HolidayRegion::Kr, 2026);
    assert!(kr_2026.iter().any(|h| h.name.contains("신정")));
    assert!(kr_2026.iter().any(|h| h.name.contains("삼일절")));
    assert!(kr_2026.iter().any(|h| h.name.contains("어린이날")));
    assert!(kr_2026.iter().any(|h| h.name.contains("광복절")));
    assert!(kr_2026.iter().any(|h| h.name.contains("개천절")));
    assert!(kr_2026.iter().any(|h| h.name.contains("한글날")));
    assert!(kr_2026.iter().any(|h| h.name.contains("성탄절")));
    assert!(kr_2026.iter().any(|h| h.name.contains("설날")));
    assert!(kr_2026.iter().any(|h| h.name.contains("추석")));

    // 2026 US holidays
    let us_2026 = get_holidays(HolidayRegion::Us, 2026);
    assert!(us_2026.iter().any(|h| h.name.contains("New Year")));
    assert!(us_2026.iter().any(|h| h.name.contains("Martin Luther King")));
    assert!(us_2026.iter().any(|h| h.name.contains("Independence Day")));
    assert!(us_2026.iter().any(|h| h.name.contains("Thanksgiving")));
    assert!(us_2026.iter().any(|h| h.name.contains("Christmas")));

    // 2026 Japanese holidays
    let jp_2026 = get_holidays(HolidayRegion::Jp, 2026);
    assert!(jp_2026.iter().any(|h| h.name.contains("元日")));
    assert!(jp_2026.iter().any(|h| h.name.contains("建国記念の日")));
    assert!(jp_2026.iter().any(|h| h.name.contains("こどもの日")));
}

#[test]
fn test_holiday_cache_and_red_day_detection() -> Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    app.set_holiday_region(HolidayRegion::Kr)?;

    // 2026-08-15: Liberation Day (Saturday in 2026, but is a holiday -> is_red_day)
    let gwangbok = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    assert!(app.is_holiday(gwangbok));
    assert!(app.is_red_day(gwangbok));
    assert_eq!(app.is_blue_day(gwangbok), false); // Holiday takes precedence over Saturday blue!

    // 2026-03-01 is Sunday -> red day
    let samil = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    assert!(app.is_holiday(samil));
    assert!(app.is_red_day(samil));

    // 2026-03-02 is Monday substitute holiday in Korea -> red day!
    let samil_sub = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
    assert!(app.is_holiday(samil_sub));
    assert!(app.is_red_day(samil_sub));

    // Normal Wednesday (e.g. 2026-03-04) -> NOT red day, NOT blue day
    let wed = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
    assert_eq!(app.is_holiday(wed), false);
    assert_eq!(app.is_red_day(wed), false);
    assert_eq!(app.is_blue_day(wed), false);

    // Normal Saturday (e.g. 2026-03-07) -> blue day, NOT red day
    let sat = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
    assert_eq!(app.is_holiday(sat), false);
    assert_eq!(app.is_red_day(sat), false);
    assert_eq!(app.is_blue_day(sat), true);

    Ok(())
}

#[test]
fn test_holiday_settings_persistence_and_switching() -> Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    // Switch to US holidays
    app.set_holiday_region(HolidayRegion::Us)?;
    assert_eq!(app.holiday_region, HolidayRegion::Us);

    // Verify US Independence Day July 4 is a holiday
    let july4 = NaiveDate::from_ymd_opt(2026, 7, 4).unwrap();
    assert!(app.is_holiday(july4));
    assert!(app.is_red_day(july4));

    // Korean Liberation Day (Aug 15) should NOT be a holiday in US mode
    let aug15 = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    assert_eq!(app.is_holiday(aug15), false);

    // Turn off holidays
    app.set_holiday_region(HolidayRegion::None)?;
    assert_eq!(app.holiday_region, HolidayRegion::None);
    assert_eq!(app.is_holiday(july4), false);
    // But Sunday is still red day!
    let sun = NaiveDate::from_ymd_opt(2026, 7, 5).unwrap();
    assert!(app.is_red_day(sun));

    Ok(())
}

#[test]
fn test_calendar_toggle_disables_holiday_highlighting() -> Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    app.set_holiday_region(HolidayRegion::Kr)?;
    let aug15 = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    assert!(app.is_holiday(aug15));

    // Disable 'cal_holidays' calendar
    app.toggle_calendar("cal_holidays")?;
    let cal = app.calendars.iter().find(|c| c.id == "cal_holidays").unwrap();
    assert_eq!(cal.enabled, false);

    // Holidays cache should now be cleared
    assert_eq!(app.is_holiday(aug15), false);

    // Re-enable 'cal_holidays' calendar
    app.toggle_calendar("cal_holidays")?;
    let cal2 = app.calendars.iter().find(|c| c.id == "cal_holidays").unwrap();
    assert_eq!(cal2.enabled, true);
    assert!(app.is_holiday(aug15));

    Ok(())
}

#[test]
fn test_settings_modal_holiday_keyboard_and_mouse() -> Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    app.open_settings_modal();
    assert_eq!(app.modal, ActiveModal::Settings);

    let send_key = |app: &mut App, code: KeyCode| {
        let key_ev = KeyEvent::new(code, KeyModifiers::NONE);
        handle_crossterm_event(app, crossterm::event::Event::Key(key_ev)).unwrap();
    };

    // '2' key navigates to Holidays tab
    send_key(&mut app, KeyCode::Char('2'));
    assert_eq!(app.settings_section, SettingsSection::Holidays);

    // 'u' selects US
    send_key(&mut app, KeyCode::Char('u'));
    assert_eq!(app.holiday_region, HolidayRegion::Us);

    // 'j' selects JP
    send_key(&mut app, KeyCode::Char('j'));
    assert_eq!(app.holiday_region, HolidayRegion::Jp);

    // 'k' selects KR
    send_key(&mut app, KeyCode::Char('k'));
    assert_eq!(app.holiday_region, HolidayRegion::Kr);

    // 'x' turns Off
    send_key(&mut app, KeyCode::Char('x'));
    assert_eq!(app.holiday_region, HolidayRegion::None);

    // Mouse click selects Holiday region
    app.hit_map.register(
        ratatui::layout::Rect::new(10, 10, 5, 1),
        HitTarget::ModalSettingHoliday(HolidayRegion::Kr),
    );
    app.handle_mouse_click(11, 10).unwrap();
    assert_eq!(app.holiday_region, HolidayRegion::Kr);

    Ok(())
}

#[test]
fn test_month_view_holiday_rendering_color() -> Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;
    app.set_holiday_region(HolidayRegion::Kr)?;

    // Set view to August 2026 (Aug 15 is Liberation Day, Aug 17 is substitute holiday)
    app.selected_date = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    app.current_view = ViewMode::Month;

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| ui::render(f, &mut app))?;

    let danger_color = app.theme.danger;

    // Helper to find date cell fg color
    let find_date_fg = |app: &App,
                        terminal: &Terminal<TestBackend>,
                        target_date: NaiveDate|
     -> ratatui::style::Color {
        let hit = app
            .hit_map
            .targets()
            .iter()
            .find(|(_, t)| *t == HitTarget::MonthDay(target_date));
        let rect = hit.expect("Target date should exist in month grid").0;
        let cell = &terminal.backend().buffer()[(rect.x + 2, rect.y + 1)];
        cell.fg
    };

    // Aug 17, 2026 is Monday substitute holiday: should be colored with danger (red day)
    let aug17_fg = find_date_fg(
        &app,
        &terminal,
        NaiveDate::from_ymd_opt(2026, 8, 17).unwrap(),
    );
    assert_eq!(
        aug17_fg, danger_color,
        "Substitute holiday on Monday must be colored in red (danger) like Sunday!"
    );

    // Aug 19, 2026 is Wednesday (regular weekday): should NOT be danger color
    let aug19_fg = find_date_fg(
        &app,
        &terminal,
        NaiveDate::from_ymd_opt(2026, 8, 19).unwrap(),
    );
    assert_ne!(
        aug19_fg, danger_color,
        "Regular weekday must NOT be colored in red"
    );

    Ok(())
}
