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
            Constraint::Length(22), // Title & Month display
            Constraint::Length(18), // Navigation buttons (<, Today, >)
            Constraint::Min(40),    // View tabs (Month, Week, Day, Agenda: 9+1+9+1+9+1+9 = 39)
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

    let btn_today = Paragraph::new(Line::from(vec![Span::raw(format!(
        " {} ",
        lang.btn_today()
    ))]))
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
