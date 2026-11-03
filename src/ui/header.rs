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
    // 3-way horizontal layout:
    // Left: Logo + Navigation buttons ([ rcal ] [ < ] [ 오늘 ] [ > ])
    // Center: Year/Month display centered with high visibility
    // Right: View Switch Tabs (Month, Week, Day, Agenda)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(27), // [ rcal ] (8) + [ < ] (5) + [ 오늘 ] (8) + [ > ] (5) + spacing
            Constraint::Min(20),    // Center: Year / Month display
            Constraint::Length(39), // Right: 9 + 1 + 9 + 1 + 9 + 1 + 9 = 39 (Month, Week, Day, Agenda)
        ])
        .split(area);

    let lang = app.lang;
    let theme = &app.theme;

    // 1. Left Section: Logo + Navigation buttons ([ rcal ] [ < ] [ 오늘 / Today ] [ > ])
    let left_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(8), // [ rcal ]
            Constraint::Length(1), // spacing
            Constraint::Length(5), // [ < ]
            Constraint::Length(1), // spacing
            Constraint::Length(7), // [ 오늘 / Today ]
            Constraint::Length(1), // spacing
            Constraint::Length(5), // [ > ]
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let logo_p = Paragraph::new(Line::from(vec![Span::styled(
        " rcal ",
        Style::default()
            .fg(Color::Black)
            .bg(theme.primary)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(logo_p, left_chunks[0]);

    let btn_prev = Paragraph::new(Line::from(vec![Span::raw(" < ")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_prev, left_chunks[2]);
    app.hit_map.register(left_chunks[2], HitTarget::NavPrev);

    let btn_today = Paragraph::new(Line::from(vec![Span::raw(format!(
        " {} ",
        lang.btn_today()
    ))]))
    .style(theme.button_style())
    .alignment(Alignment::Center);
    f.render_widget(btn_today, left_chunks[4]);
    app.hit_map.register(left_chunks[4], HitTarget::NavToday);

    let btn_next = Paragraph::new(Line::from(vec![Span::raw(" > ")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_next, left_chunks[6]);
    app.hit_map.register(left_chunks[6], HitTarget::NavNext);

    // 2. Center Section: Year & Month Display (Enhanced visibility & presence)
    let month_year_str = lang.format_month_year(app.selected_date);
    let center_line = Line::from(vec![
        Span::styled(
            "« ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", month_year_str),
            Style::default()
                .fg(if theme.mode.is_light() {
                    Color::Black
                } else {
                    theme.text_highlight
                })
                .bg(theme.panel_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " »",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let center_p = Paragraph::new(center_line).alignment(Alignment::Center);
    f.render_widget(center_p, chunks[1]);

    // 3. Right Section: View Switch Tabs (Month, Week, Day, Agenda)
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
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    let views = [
        (
            ViewMode::Month,
            format!(" {} ", lang.view_month()),
            tab_chunks[0],
            HitTarget::TabMonth,
        ),
        (
            ViewMode::Week,
            format!(" {} ", lang.view_week()),
            tab_chunks[2],
            HitTarget::TabWeek,
        ),
        (
            ViewMode::Day,
            format!(" {} ", lang.view_day()),
            tab_chunks[4],
            HitTarget::TabDay,
        ),
        (
            ViewMode::Agenda,
            format!(" {} ", lang.view_agenda()),
            tab_chunks[6],
            HitTarget::TabAgenda,
        ),
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
}
