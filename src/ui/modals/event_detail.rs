use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, event_id: &str, area: Rect) {
    let popup_width = 56.min(area.width.saturating_sub(4));
    let popup_height = 14.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);

    let event = match app.events_cache.iter().find(|e| e.id == event_id) {
        Some(ev) => ev,
        None => return,
    };

    let cal = app.calendars.iter().find(|c| c.id == event.calendar_id);
    let cal_color = cal.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);
    let cal_name = cal.map(|c| c.name.as_str()).unwrap_or("Default");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(cal_color))
        .title(Span::styled(" Event Details ", Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Calendar badge
            Constraint::Length(1), // Time
            Constraint::Length(1), // Location
            Constraint::Length(3), // Description
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Buttons: [ Edit ] [ Delete ] [ Close ]
        ])
        .split(inner);

    // Title
    let title_line = Line::from(vec![
        Span::styled(&event.title, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // Calendar
    let cal_line = Line::from(vec![
        Span::styled("● ", Style::default().fg(cal_color)),
        Span::styled(cal_name, Style::default().fg(cal_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" ({})", event.sync_status.as_str()), Style::default().fg(Theme::TEXT_DIM)),
    ]);
    f.render_widget(Paragraph::new(cal_line), chunks[1]);

    // Time
    let date_str = event.start_time.format("%A, %B %d, %Y").to_string();
    let time_line = Line::from(vec![
        Span::styled("📅 ", Style::default().fg(Theme::ACCENT)),
        Span::styled(format!("{}, {}", date_str, event.time_str()), Style::default().fg(Theme::TEXT_NORMAL)),
    ]);
    f.render_widget(Paragraph::new(time_line), chunks[2]);

    // Location
    let loc_text = event.location.as_deref().unwrap_or("No location specified");
    let loc_line = Line::from(vec![
        Span::styled("📍 ", Style::default().fg(Theme::SECONDARY)),
        Span::styled(loc_text, Style::default().fg(Theme::TEXT_MUTED)),
    ]);
    f.render_widget(Paragraph::new(loc_line), chunks[3]);

    // Description
    let desc_text = event.description.as_deref().unwrap_or("No description");
    let desc_p = Paragraph::new(desc_text)
        .style(Style::default().fg(Theme::TEXT_MUTED).add_modifier(Modifier::ITALIC));
    f.render_widget(desc_p, chunks[4]);

    // Buttons: [ Edit (e) ] [ Delete (d) ] [ Close (Esc) ]
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // [ Edit ]
            Constraint::Length(2),
            Constraint::Length(14), // [ Delete ]
            Constraint::Min(1),
            Constraint::Length(12), // [ Close ]
        ])
        .split(chunks[6]);

    let btn_edit = Paragraph::new(Line::from(vec![Span::raw(" [ Edit (e) ] ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_edit, btn_chunks[0]);
    app.hit_map.register(btn_chunks[0], HitTarget::ModalEdit);

    let btn_del = Paragraph::new(Line::from(vec![Span::raw(" [ Delete (d) ] ")]))
        .style(Style::default().fg(Color::White).bg(Theme::DANGER).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_del, btn_chunks[2]);
    app.hit_map.register(btn_chunks[2], HitTarget::ModalDelete);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(" [ Close (q) ] ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[4]);
    app.hit_map.register(btn_chunks[4], HitTarget::ModalClose);
}
