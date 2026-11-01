use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, ViewMode};
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(24), // Title & Month display
            Constraint::Length(20), // Navigation buttons (<, Today, >)
            Constraint::Min(24),    // View tabs (Month, Week, Day, Agenda)
            Constraint::Length(80), // Action buttons (+ New, ⟳ Sync, Accounts, Settings, Theme, L:Lang, ? Help)
        ])
        .split(area);

    let lang = app.lang;
    let theme = &app.theme;

    // 1. Title & Month Year display
    let month_year_str = format!(" {} ", lang.format_month_year(app.selected_date));
    let title_line = Line::from(vec![
        Span::styled(
            " rcal ",
            Style::default()
                .fg(Color::Black)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            month_year_str,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // 2. Navigation buttons (< Today >)
    let nav_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(5), // [ < ]
            Constraint::Length(1),
            Constraint::Length(8), // [ 오늘 / Today ]
            Constraint::Length(1),
            Constraint::Length(5), // [ > ]
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    let btn_prev = Paragraph::new(Line::from(vec![Span::raw(" < ")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_prev, nav_chunks[0]);
    app.hit_map.register(nav_chunks[0], HitTarget::NavPrev);

    let btn_today = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_today()))]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_today, nav_chunks[2]);
    app.hit_map.register(nav_chunks[2], HitTarget::NavToday);

    let btn_next = Paragraph::new(Line::from(vec![Span::raw(" > ")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_next, nav_chunks[4]);
    app.hit_map.register(nav_chunks[4], HitTarget::NavNext);

    // 3. View Switch Tabs: Month, Week, Day, Agenda
    let tab_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(9), // Month / 월간
            Constraint::Length(1),
            Constraint::Length(9), // Week / 주간
            Constraint::Length(1),
            Constraint::Length(9), // Day / 일간
            Constraint::Length(1),
            Constraint::Length(9), // Agenda / 목록
            Constraint::Min(1),
        ])
        .split(chunks[2]);

    let views = [
        (ViewMode::Month, format!(" {} ", lang.view_month()), tab_chunks[0], HitTarget::TabMonth),
        (ViewMode::Week, format!(" {} ", lang.view_week()), tab_chunks[2], HitTarget::TabWeek),
        (ViewMode::Day, format!(" {} ", lang.view_day()), tab_chunks[4], HitTarget::TabDay),
        (ViewMode::Agenda, format!(" {} ", lang.view_agenda()), tab_chunks[6], HitTarget::TabAgenda),
    ];

    for (mode, label, rect, hit) in views {
        let style = if app.current_view == mode {
            theme.header_tab_active()
        } else {
            theme.header_tab_inactive()
        };
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, hit);
    }

    // 4. Action buttons: [+ New Event], [⟳ Sync], [Accounts], [⚙ Settings], [🎨 Theme], [L: Language], [? Help]
    let action_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11), // [+ 새 일정]
            Constraint::Length(1),
            Constraint::Length(14), // [⟳  동기화] / [⟳  동기화중]
            Constraint::Length(1),
            Constraint::Length(10), // [계정연동]
            Constraint::Length(1),
            Constraint::Length(10), // [⚙ 설정]
            Constraint::Length(1),
            Constraint::Length(10), // [🎨 테마]
            Constraint::Length(1),
            Constraint::Length(8),  // [L: 한/EN]
            Constraint::Length(1),
            Constraint::Length(9),  // [? 도움말]
            Constraint::Min(0),
        ])
        .split(chunks[3]);

    let btn_new = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_new_event()))]))
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_new, action_chunks[0]);
    app.hit_map.register(action_chunks[0], HitTarget::BtnNewEvent);

    let sync_label = if app.is_syncing {
        lang.btn_syncing()
    } else {
        lang.btn_sync()
    };
    let sync_text = if sync_label.starts_with("⟳ ") && !sync_label.starts_with("⟳  ") {
        sync_label.replacen("⟳ ", "⟳  ", 1)
    } else {
        sync_label.to_string()
    };
    let sync_bg = if app.is_syncing { theme.accent } else { Color::Rgb(45, 90, 160) };
    let btn_sync = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", sync_text))]))
        .style(
            Style::default()
                .fg(Color::White)
                .bg(sync_bg)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_sync, action_chunks[2]);
    app.hit_map.register(action_chunks[2], HitTarget::BtnSync);

    let btn_acc = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_accounts()))]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_acc, action_chunks[4]);
    app.hit_map.register(action_chunks[4], HitTarget::BtnAccounts);

    // Settings button: [ ⚙ 설정 ]
    let btn_settings = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_settings()))]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_settings, action_chunks[6]);
    app.hit_map.register(action_chunks[6], HitTarget::BtnSettings);

    // Theme button: [ 🎨 테마 ]
    let btn_theme = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_theme()))]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_theme, action_chunks[8]);
    app.hit_map.register(action_chunks[8], HitTarget::BtnTheme);

    // Language toggle button: [ L: 한/EN/日 ]
    let lang_btn_text = format!(" L: {} ", lang.short_label());
    let btn_lang = Paragraph::new(Line::from(vec![Span::raw(lang_btn_text)]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_lang, action_chunks[10]);
    app.hit_map.register(action_chunks[10], HitTarget::BtnLanguage);

    let btn_help = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_help()))]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_help, action_chunks[12]);
    app.hit_map.register(action_chunks[12], HitTarget::BtnHelp);
}
