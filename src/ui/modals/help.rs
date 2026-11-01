use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 72.min(area.width.saturating_sub(4));
    let popup_height = 26.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);
    app.hit_map.register(popup_rect, HitTarget::ModalConsume);

    let lang = app.lang;
    let theme = &app.theme;
    let t = lang.t();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            t.modal_help_title,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(21),
            Constraint::Length(1), // [ Close ]
        ])
        .split(inner);

    let mut help_text = vec![Line::from(vec![Span::styled(
        t.help_mouse_title,
        Style::default()
            .fg(theme.secondary)
            .add_modifier(Modifier::BOLD),
    )])];
    for line in t.help_mouse_lines {
        help_text.push(Line::from(*line));
    }
    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        t.help_keyboard_title,
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    )]));
    for line in t.help_keyboard_lines {
        help_text.push(Line::from(*line));
    }

    f.render_widget(Paragraph::new(help_text), chunks[0]);

    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(16),
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(lang.btn_close())]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalClose);

    f.set_cursor_position((btn_chunks[1].x + btn_chunks[1].width / 2, btn_chunks[1].y));
}
