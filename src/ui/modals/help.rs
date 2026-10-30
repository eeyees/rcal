use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 68.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(" Keyboard & Mouse Shortcuts ", Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(14),
            Constraint::Length(1), // [ Close ]
        ])
        .split(inner);

    let help_text = vec![
        Line::from(vec![
            Span::styled("🖱  Mouse Controls:", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  • Click View Tabs       : Switch between Month, Week, Day, Agenda"),
        Line::from("  • Click [<] / [>]       : Navigate backward / forward in time"),
        Line::from("  • Click [Today]         : Jump straight to today's date"),
        Line::from("  • Click Day Cell        : Select date in Month view"),
        Line::from("  • Double Click Day      : Open Day view for that date"),
        Line::from("  • Click Time Slot       : Create new event at that hour"),
        Line::from("  • Click Event Block     : View event details & edit/delete"),
        Line::from("  • Click [x] Checkbox    : Toggle calendar visibility ON/OFF"),
        Line::from(""),
        Line::from(vec![
            Span::styled("⌨  Keyboard Shortcuts:", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  • 1 / 2 / 3 / 4         : Month / Week / Day / Agenda view"),
        Line::from("  • m / w / d / a         : Alternate view shortcuts"),
        Line::from("  • h / j / k / l         : Navigate days / weeks (Left/Down/Up/Right)"),
        Line::from("  • [ / ]                 : Previous / Next period"),
        Line::from("  • t                     : Jump to today"),
        Line::from("  • c / n                 : Create new event"),
        Line::from("  • ?                     : Toggle this help screen"),
        Line::from("  • q / Esc               : Close modal or quit"),
    ];

    f.render_widget(Paragraph::new(help_text), chunks[0]);

    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(14),
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(" [ Close (Esc) ] ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalClose);
}
