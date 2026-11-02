use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, DeleteTarget};
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, target: &DeleteTarget, area: Rect) {
    let popup_width = 58.min(area.width.saturating_sub(4));
    let popup_height = 13.min(area.height.saturating_sub(2));

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
        .border_style(Style::default().fg(theme.danger))
        .title(Span::styled(
            t.modal_confirm_delete_title,
            Style::default()
                .fg(theme.danger)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Question prompt
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Target item badge & name
            Constraint::Length(1), // Warning text
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Action buttons
        ])
        .split(inner);

    // 1. Question prompt
    let prompt_p = Paragraph::new(Line::from(vec![Span::styled(
        t.confirm_delete_prompt,
        Style::default()
            .fg(theme.text_highlight)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(prompt_p, chunks[0]);

    // 2. Item details
    let (type_label, item_name, type_color) = match target {
        DeleteTarget::Event { title, .. } => (t.label_type_event, title.as_str(), theme.accent),
        DeleteTarget::Calendar { name, .. } => {
            (t.label_type_calendar, name.as_str(), theme.secondary)
        }
        DeleteTarget::Account { name, .. } => (t.label_type_account, name.as_str(), theme.primary),
    };

    let item_line = Line::from(vec![
        Span::styled(
            format!(" [ {} ] ", type_label),
            Style::default()
                .fg(Color::Black)
                .bg(type_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("\"{}\"", item_name),
            Style::default()
                .fg(theme.text_normal)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let item_p = Paragraph::new(item_line).alignment(Alignment::Center);
    f.render_widget(item_p, chunks[2]);

    // 3. Warning text
    let warn_p = Paragraph::new(Line::from(vec![Span::styled(
        t.confirm_delete_warning,
        Style::default().fg(theme.danger),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(warn_p, chunks[3]);

    // 4. Action Buttons: [ ✕ 삭제 확인 (y/Enter) ]  [ 취소 (n/Esc) ]
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(24), // [ ✕ 삭제 확인 (y/Enter) ]
            Constraint::Length(3),
            Constraint::Length(18), // [ 취소 (n/Esc) ]
            Constraint::Min(1),
        ])
        .split(chunks[5]);

    let btn_yes = Paragraph::new(Line::from(vec![Span::raw(t.btn_confirm_yes)]))
        .style(
            Style::default()
                .fg(Color::White)
                .bg(theme.danger)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_yes, btn_chunks[1]);
    app.hit_map
        .register(btn_chunks[1], HitTarget::ModalConfirmYes);

    let btn_no = Paragraph::new(Line::from(vec![Span::raw(t.btn_confirm_no)]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_no, btn_chunks[3]);
    app.hit_map
        .register(btn_chunks[3], HitTarget::ModalConfirmNo);

    // Position cursor on the confirm button
    f.set_cursor_position((btn_chunks[1].x + btn_chunks[1].width / 2, btn_chunks[1].y));
}
