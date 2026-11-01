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

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_date = app.selected_date;
    let today = Local::now().date_naive();
    let is_today = selected_date == today;
    let theme = &app.theme;

    // Filter events for this day
    let events_today: Vec<&_> = app
        .events_cache
        .iter()
        .filter(|ev| ev.occurs_on(selected_date))
        .collect();

    let all_day_events: Vec<&_> = events_today
        .iter()
        .filter(|ev| ev.all_day)
        .copied()
        .collect();

    let all_day_height = if all_day_events.len() > 3 { 3 } else { 2 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),              // 1. Day Header Banner
            Constraint::Length(all_day_height), // 2. All-Day Schedule Section
            Constraint::Min(12),                // 3. 24h / Workday Schedule
        ])
        .split(area);

    // 1. Header Banner
    let day_name = app.lang.format_date_long(selected_date);
    let hint_text = app.lang.t().day_view_hint;
    let header_line = Line::from(vec![
        Span::styled(
            format!(" {} ", day_name),
            if is_today {
                theme.today_style()
            } else {
                Style::default()
                    .fg(theme.text_highlight)
                    .add_modifier(Modifier::BOLD)
            },
        ),
        Span::raw("  "),
        Span::styled(
            hint_text,
            Style::default()
                .fg(theme.text_muted)
                .add_modifier(Modifier::ITALIC),
        ),
    ]);

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));
    f.render_widget(Paragraph::new(header_line).block(header_block), chunks[0]);

    // 2. All-Day Events Section
    let all_day_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(8), // Time label column width
            Constraint::Min(20),   // All-day content / slot area
        ])
        .split(chunks[1]);

    let all_day_label = app.lang.t().all_day_label;
    let p_all_day_lbl = Paragraph::new(Line::from(vec![Span::styled(
        all_day_label,
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Right);
    f.render_widget(p_all_day_lbl, all_day_cols[0]);

    let all_day_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));
    let all_day_inner = all_day_block.inner(all_day_cols[1]);
    f.render_widget(all_day_block, all_day_cols[1]);

    if all_day_events.is_empty() {
        let empty_tip = app.lang.t().add_all_day_tip;
        let p_empty = Paragraph::new(Line::from(vec![Span::styled(
            empty_tip,
            Style::default().fg(theme.text_dim),
        )]));
        f.render_widget(p_empty, all_day_inner);
        app.hit_map
            .register(all_day_cols[1], HitTarget::DayAllDaySlot(selected_date));
    } else {
        // Register slot background hit
        app.hit_map
            .register(all_day_cols[1], HitTarget::DayAllDaySlot(selected_date));

        let max_display = 4;
        let visible_events = &all_day_events[..all_day_events.len().min(max_display)];
        let has_more = all_day_events.len() > max_display;
        let chunk_count = if has_more {
            visible_events.len() + 1
        } else {
            visible_events.len()
        };

        let ev_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, chunk_count as u32); chunk_count])
            .split(all_day_inner);

        for (ev_i, ev) in visible_events.iter().enumerate() {
            let cal_color = app
                .calendars
                .iter()
                .find(|c| c.id == ev.calendar_id)
                .map(|c| c.ratatui_color())
                .unwrap_or(Color::Cyan);

            let ev_area = ev_chunks[ev_i];
            let loc_str = if let Some(loc) = &ev.location {
                format!(" @ {}", loc)
            } else {
                String::new()
            };
            let title_text = format!(" ■ {}{}", ev.title, loc_str);

            let p_ev = Paragraph::new(Line::from(vec![Span::styled(
                title_text,
                Style::default()
                    .fg(Color::Black)
                    .bg(cal_color)
                    .add_modifier(Modifier::BOLD),
            )]));

            f.render_widget(p_ev, ev_area);
            app.hit_map
                .register(ev_area, HitTarget::EventItem(ev.id.clone()));
        }

        if has_more {
            let more_count = all_day_events.len() - max_display;
            let more_area = ev_chunks[visible_events.len()];
            let p_more = Paragraph::new(Line::from(vec![Span::styled(
                format!(" +{} more ", more_count),
                Style::default()
                    .fg(theme.text_muted)
                    .add_modifier(Modifier::ITALIC),
            )]));
            f.render_widget(p_more, more_area);
        }
    }

    // 3. Schedule Grid: Hours 07:00 to 22:00
    let start_hour = 7;
    let end_hour = 22;
    let num_hours = end_hour - start_hour + 1;

    let hour_constraints = vec![Constraint::Ratio(1, num_hours as u32); num_hours as usize];
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(hour_constraints)
        .split(chunks[2]);

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
            Style::default().fg(theme.text_muted),
        )]))
        .alignment(Alignment::Right);
        f.render_widget(p_time, slot_cols[0]);

        // Slot block
        let is_selected_slot = app.selected_hour == h;
        let slot_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(if is_selected_slot {
                theme.primary
            } else {
                theme.border
            }));
        let slot_inner = slot_block.inner(slot_cols[1]);
        f.render_widget(slot_block, slot_cols[1]);

        // Register hit for clicking empty slot
        app.hit_map.register(slot_cols[1], HitTarget::DaySlot(h, 0));

        // Find events starting in this hour
        let events_in_hour: Vec<&_> = events_today
            .iter()
            .filter(|ev| !ev.all_day && ev.start_time.hour() == h)
            .copied()
            .collect();

        if !events_in_hour.is_empty() {
            let ev_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Ratio(1, events_in_hour.len() as u32);
                    events_in_hour.len()
                ])
                .split(slot_inner);

            for (ev_i, ev) in events_in_hour.iter().enumerate() {
                let cal_color = app
                    .calendars
                    .iter()
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
                        Span::styled(
                            format!(" @ {}", loc),
                            Style::default().fg(Color::Black).bg(cal_color),
                        )
                    } else {
                        Span::raw("")
                    },
                ]));

                f.render_widget(p_ev, ev_area);
                app.hit_map
                    .register(ev_area, HitTarget::EventItem(ev.id.clone()));
            }
        }
    }
}
