use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, ViewMode};
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28), // Title & Month display
            Constraint::Length(24), // Navigation buttons (<, Today, >)
            Constraint::Min(40),    // View tabs (Month, Week, Day, Agenda)
            Constraint::Length(22), // Action buttons (+ New, ? Help)
        ])
        .split(area);

    // 1. Title & Month Year display
    let month_year_str = format!(" {} ", app.selected_date.format("%B %Y"));
    let title_line = Line::from(vec![
        Span::styled(
            " rcal ",
            Style::default()
                .fg(Color::Black)
                .bg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            month_year_str,
            Style::default()
                .fg(Theme::TEXT_HIGHLIGHT)
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
            Constraint::Length(9), // [ Today ]
            Constraint::Length(1),
            Constraint::Length(5), // [ > ]
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    let btn_prev = Paragraph::new(Line::from(vec![Span::raw(" < ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_prev, nav_chunks[0]);
    app.hit_map.register(nav_chunks[0], HitTarget::NavPrev);

    let btn_today = Paragraph::new(Line::from(vec![Span::raw(" Today ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_today, nav_chunks[2]);
    app.hit_map.register(nav_chunks[2], HitTarget::NavToday);

    let btn_next = Paragraph::new(Line::from(vec![Span::raw(" > ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_next, nav_chunks[4]);
    app.hit_map.register(nav_chunks[4], HitTarget::NavNext);

    // 3. View Switch Tabs: Month, Week, Day, Agenda
    let tab_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11), // Month
            Constraint::Length(1),
            Constraint::Length(10), // Week
            Constraint::Length(1),
            Constraint::Length(9),  // Day
            Constraint::Length(1),
            Constraint::Length(12), // Agenda
            Constraint::Min(1),
        ])
        .split(chunks[2]);

    let views = [
        (ViewMode::Month, " 1:Month ", tab_chunks[0], HitTarget::TabMonth),
        (ViewMode::Week, " 2:Week ", tab_chunks[2], HitTarget::TabWeek),
        (ViewMode::Day, " 3:Day ", tab_chunks[4], HitTarget::TabDay),
        (ViewMode::Agenda, " 4:Agenda ", tab_chunks[6], HitTarget::TabAgenda),
    ];

    for (mode, label, rect, hit) in views {
        let style = if app.current_view == mode {
            Theme::header_tab_active()
        } else {
            Theme::header_tab_inactive()
        };
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, hit);
    }

    // 4. Action buttons: [+ New Event] and [? Help]
    let action_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // [+ New]
            Constraint::Length(1),
            Constraint::Length(9),  // [? Help]
            Constraint::Min(0),
        ])
        .split(chunks[3]);

    let btn_new = Paragraph::new(Line::from(vec![Span::raw(" + New ")]))
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_new, action_chunks[0]);
    app.hit_map.register(action_chunks[0], HitTarget::BtnNewEvent);

    let btn_help = Paragraph::new(Line::from(vec![Span::raw(" ? Help ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_help, action_chunks[2]);
    app.hit_map.register(action_chunks[2], HitTarget::BtnHelp);
}
