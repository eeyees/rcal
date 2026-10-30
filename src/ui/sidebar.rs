use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(" My Calendars ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(app.calendars.len() as u16 + 2), // Calendar list
            Constraint::Length(1),                              // Divider
            Constraint::Min(6),                                 // Shortcuts hint
        ])
        .split(inner);

    // 1. Render Calendar items with clickable checkbox
    let mut cal_lines = Vec::new();
    for (idx, cal) in app.calendars.iter().enumerate() {
        let check_box = if cal.enabled { "[x] " } else { "[ ] " };
        let check_color = if cal.enabled { Theme::SECONDARY } else { Theme::TEXT_DIM };
        let cal_color = cal.ratatui_color();

        let line = Line::from(vec![
            Span::styled(check_box, Style::default().fg(check_color).add_modifier(Modifier::BOLD)),
            Span::styled("● ", Style::default().fg(cal_color)),
            Span::styled(&cal.name, Style::default().fg(if cal.enabled { Theme::TEXT_NORMAL } else { Theme::TEXT_MUTED })),
            Span::styled(
                format!(" ({})", cal.provider.as_str()),
                Style::default().fg(Theme::TEXT_DIM),
            ),
        ]);
        cal_lines.push(line);

        // Register hit region for this calendar row
        if (idx as u16) < chunks[0].height {
            let row_rect = Rect {
                x: chunks[0].x,
                y: chunks[0].y + idx as u16,
                width: chunks[0].width,
                height: 1,
            };
            app.hit_map.register(row_rect, HitTarget::CalendarToggle(cal.id.clone()));
        }
    }

    f.render_widget(Paragraph::new(cal_lines), chunks[0]);

    // 2. Quick Tips in sidebar
    let tips = vec![
        Line::from(Span::styled("Quick Navigation:", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(" • Click tabs to switch", Style::default().fg(Theme::TEXT_MUTED))),
        Line::from(Span::styled(" • Click date / slot to add", Style::default().fg(Theme::TEXT_MUTED))),
        Line::from(Span::styled(" • Click event to view/edit", Style::default().fg(Theme::TEXT_MUTED))),
        Line::from(Span::styled(" • [x] Click cal to toggle", Style::default().fg(Theme::TEXT_MUTED))),
        Line::from(""),
        Line::from(Span::styled("Keys:", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(" • h/j/k/l : Navigate", Style::default().fg(Theme::TEXT_MUTED))),
        Line::from(Span::styled(" • c or n  : New event", Style::default().fg(Theme::TEXT_MUTED))),
        Line::from(Span::styled(" • t       : Jump to today", Style::default().fg(Theme::TEXT_MUTED))),
        Line::from(Span::styled(" • q / Esc : Quit / Close", Style::default().fg(Theme::TEXT_MUTED))),
    ];
    f.render_widget(Paragraph::new(tips), chunks[2]);
}
