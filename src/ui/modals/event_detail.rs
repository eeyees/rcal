use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, event_id: &str, area: Rect) {
    let popup_width = 58.min(area.width.saturating_sub(4));
    let popup_height = 14.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);
    app.hit_map.register(popup_rect, HitTarget::ModalConsume);

    let event = match app.events_cache.iter().find(|e| e.id == event_id) {
        Some(ev) => ev,
        None => return,
    };

    let lang = app.lang;
    let theme = &app.theme;
    let cal = app.calendars.iter().find(|c| c.id == event.calendar_id);
    let cal_color = cal.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);
    let cal_name = cal.map(|c| c.name.as_str()).unwrap_or("Default");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(cal_color))
        .title(Span::styled(lang.modal_event_details(), Style::default().fg(theme.text_highlight).add_modifier(Modifier::BOLD)));
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
        Span::styled(&event.title, Style::default().fg(theme.text_highlight).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // Calendar
    let cal_line = Line::from(vec![
        Span::styled("● ", Style::default().fg(cal_color)),
        Span::styled(cal_name, Style::default().fg(cal_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" ({})", event.sync_status.as_str()), Style::default().fg(theme.text_dim)),
    ]);
    f.render_widget(Paragraph::new(cal_line), chunks[1]);

    // Time
    let date_str = lang.format_date_long(event.start_time.date_naive());
    let time_line = Line::from(vec![
        Span::styled("📅 ", Style::default().fg(theme.accent)),
        Span::styled(format!("{}, {}", date_str, event.time_str()), Style::default().fg(theme.text_normal)),
    ]);
    f.render_widget(Paragraph::new(time_line), chunks[2]);

    // Location
    let loc_text = event.location.as_deref().unwrap_or(lang.t().no_location);
    let loc_line = Line::from(vec![
        Span::styled("📍 ", Style::default().fg(theme.secondary)),
        Span::styled(loc_text, Style::default().fg(theme.text_muted)),
    ]);
    f.render_widget(Paragraph::new(loc_line), chunks[3]);

    // Description
    let desc_text = event.description.as_deref().unwrap_or(lang.t().no_description);
    let desc_p = Paragraph::new(desc_text)
        .style(Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC));
    f.render_widget(desc_p, chunks[4]);

    // Buttons: [ Edit (e) ] [ Delete (d) ] [ Close (Esc) ]
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14), // [ Edit ]
            Constraint::Length(2),
            Constraint::Length(14), // [ Delete ]
            Constraint::Min(1),
            Constraint::Length(14), // [ Close ]
        ])
        .split(chunks[6]);

    let btn_edit = Paragraph::new(Line::from(vec![Span::raw(lang.btn_edit())]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_edit, btn_chunks[0]);
    app.hit_map.register(btn_chunks[0], HitTarget::ModalEdit);

    let btn_del = Paragraph::new(Line::from(vec![Span::raw(lang.btn_delete())]))
        .style(Style::default().fg(Color::White).bg(theme.danger).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_del, btn_chunks[2]);
    app.hit_map.register(btn_chunks[2], HitTarget::ModalDelete);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(lang.btn_close())]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[4]);
    app.hit_map.register(btn_chunks[4], HitTarget::ModalClose);

    // Terminal cursor on Close button
    f.set_cursor_position((btn_chunks[4].x + btn_chunks[4].width / 2, btn_chunks[4].y));
}
