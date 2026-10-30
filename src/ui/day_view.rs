use chrono::{Local, Timelike};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_date = app.selected_date;
    let today = Local::now().date_naive();
    let is_today = selected_date == today;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Day Header Banner
            Constraint::Min(15),   // 24h / Workday Schedule
        ])
        .split(area);

    // 1. Header Banner
    let day_name = selected_date.format("%A, %B %d, %Y").to_string();
    let header_line = Line::from(vec![
        Span::styled(
            format!(" {} ", day_name),
            if is_today {
                Theme::today_style()
            } else {
                Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)
            },
        ),
        Span::raw("  "),
        Span::styled(
            "(Click any time slot to add an event)",
            Style::default().fg(Theme::TEXT_MUTED).add_modifier(Modifier::ITALIC),
        ),
    ]);

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Theme::BORDER));
    f.render_widget(Paragraph::new(header_line).block(header_block), chunks[0]);

    // 2. Schedule Grid: Hours 07:00 to 22:00
    let start_hour = 7;
    let end_hour = 22;
    let num_hours = end_hour - start_hour + 1;

    let hour_constraints = vec![Constraint::Ratio(1, num_hours as u32); num_hours as usize];
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(hour_constraints)
        .split(chunks[1]);

    // Filter events for this day
    let events_today: Vec<&_> = app.events_cache.iter()
        .filter(|ev| ev.occurs_on(selected_date))
        .collect();

    for h in start_hour..=end_hour {
        let idx = (h - start_hour) as usize;
        let row_area = row_chunks[idx];

        let slot_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8), // Time label (e.g. 09:00)
                Constraint::Min(20),   // Events content / slot area
            ])
            .split(row_area);

        // Time label
        let time_str = format!("{:02}:00", h);
        let p_time = Paragraph::new(Line::from(vec![Span::styled(
            time_str,
            Style::default().fg(Theme::TEXT_MUTED),
        )]))
        .alignment(Alignment::Right);
        f.render_widget(p_time, slot_cols[0]);

        // Slot block
        let is_selected_slot = app.selected_hour == h;
        let slot_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(if is_selected_slot { Theme::PRIMARY } else { Theme::BORDER }));
        let slot_inner = slot_block.inner(slot_cols[1]);
        f.render_widget(slot_block, slot_cols[1]);

        // Register hit for clicking empty slot
        app.hit_map.register(slot_cols[1], HitTarget::DaySlot(h, 0));

        // Find events starting in this hour
        let events_in_hour: Vec<&_> = events_today.iter()
            .filter(|ev| !ev.all_day && ev.start_time.hour() == h)
            .copied()
            .collect();

        if !events_in_hour.is_empty() {
            let ev_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Ratio(1, events_in_hour.len() as u32); events_in_hour.len()])
                .split(slot_inner);

            for (ev_i, ev) in events_in_hour.iter().enumerate() {
                let cal_color = app.calendars.iter()
                    .find(|c| c.id == ev.calendar_id)
                    .map(|c| c.ratatui_color())
                    .unwrap_or(Color::Cyan);

                let ev_area = ev_chunks[ev_i];
                let title_text = format!(" {} ({})", ev.title, ev.time_str());
                let p_ev = Paragraph::new(Line::from(vec![
                    Span::styled(
                        title_text,
                        Style::default()
                            .fg(Color::Black)
                            .bg(cal_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    if let Some(loc) = &ev.location {
                        Span::styled(format!(" @ {}", loc), Style::default().fg(Color::Black).bg(cal_color))
                    } else {
                        Span::raw("")
                    }
                ]));

                f.render_widget(p_ev, ev_area);
                app.hit_map.register(ev_area, HitTarget::EventItem(ev.id.clone()));
            }
        }
    }
}
