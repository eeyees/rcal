use chrono::{Datelike, Duration, Local, Timelike};
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
    let today = Local::now().date_naive();
    let selected_date = app.selected_date;
    let theme = &app.theme;

    // Determine week start based on week_start_day setting
    let week_start = app.week_start_day.week_start_date(selected_date);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(8), // Time labels (08:00, 09:00...)
            Constraint::Min(40),   // 7 Day columns
        ])
        .split(area);

    let time_area = main_chunks[0];
    let days_area = main_chunks[1];

    let day_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 7); 7])
        .split(days_area);

    // Display hours from 08:00 to 22:00 (15 hours)
    let start_hour = 8;
    let end_hour = 22;
    let num_hours = end_hour - start_hour + 1;

    let time_row_constraints = vec![
        Constraint::Length(2), // Header row for Date
        Constraint::Length(1), // All-Day row
    ];
    let mut all_time_constraints = time_row_constraints;
    all_time_constraints.extend(vec![Constraint::Ratio(1, num_hours as u32); num_hours as usize]);

    let time_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(all_time_constraints)
        .split(time_area);

    // All-day label
    let all_day_lbl = app.lang.t().all_day_short;
    let p_all = Paragraph::new(Line::from(vec![Span::styled(
        all_day_lbl,
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Right);
    f.render_widget(p_all, time_rows[1]);

    // Render Time column labels
    for h in start_hour..=end_hour {
        let idx = (h - start_hour + 2) as usize;
        let time_str = format!("{:02}:00", h);
        let p = Paragraph::new(Line::from(vec![Span::styled(
            time_str,
            Style::default().fg(theme.text_muted),
        )]))
        .alignment(Alignment::Right);
        f.render_widget(p, time_rows[idx]);
    }

    // Render each day column
    for col in 0..7 {
        let current_date = week_start + Duration::days(col as i64);
        let col_rect = day_columns[col as usize];

        let is_today = current_date == today;
        let is_selected = current_date == selected_date;

        let col_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Date Header
                Constraint::Length(1), // All-Day Events Slot
                Constraint::Min(10),   // Timeline Body
            ])
            .split(col_rect);

        // Header for this day
        let dows = app.week_start_day.weekday_headers(app.lang);
        let dow_str = dows[col as usize];
        let date_header_text = format!("{} {:02}/{:02}", dow_str, current_date.month(), current_date.day());
        let is_weekend = current_date.weekday() == chrono::Weekday::Sat || current_date.weekday() == chrono::Weekday::Sun;
        let header_style = if is_today {
            theme.today_style()
        } else if is_selected {
            theme.selected_day_style()
        } else if is_weekend {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_normal).add_modifier(Modifier::BOLD)
        };

        let p_head = Paragraph::new(Line::from(vec![Span::raw(date_header_text)]))
            .style(header_style)
            .alignment(Alignment::Center);
        f.render_widget(p_head, col_chunks[0]);

        // Register hit for day header
        app.hit_map.register(col_chunks[0], HitTarget::MonthDay(current_date));

        // Render All-day events row
        let all_day_events_on_day: Vec<&_> = app.events_cache.iter()
            .filter(|ev| ev.all_day && ev.occurs_on(current_date))
            .collect();

        if let Some(first_ev) = all_day_events_on_day.first() {
            let ev_cal = app.calendars.iter().find(|c| c.id == first_ev.calendar_id);
            let cal_color = ev_cal.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);
            let count_suffix = if all_day_events_on_day.len() > 1 {
                format!(" +{}", all_day_events_on_day.len() - 1)
            } else {
                String::new()
            };
            let all_day_p = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!(" ■ {}{}", first_ev.title, count_suffix),
                    Style::default().fg(Color::Black).bg(cal_color).add_modifier(Modifier::BOLD),
                )
            ]));
            f.render_widget(all_day_p, col_chunks[1]);
            app.hit_map.register(col_chunks[1], HitTarget::EventItem(first_ev.id.clone()));
        } else {
            let empty_block = Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border));
            f.render_widget(empty_block, col_chunks[1]);
            app.hit_map.register(col_chunks[1], HitTarget::WeekAllDaySlot(current_date));
        }

        // Timeline Body with borders
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(if is_selected { theme.primary } else { theme.border }));
        let body_inner = block.inner(col_chunks[2]);
        f.render_widget(block, col_chunks[2]);

        let hour_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Ratio(1, num_hours as u32); num_hours as usize])
            .split(body_inner);

        // Register slots for clicking
        for h in start_hour..=end_hour {
            let idx = (h - start_hour) as usize;
            let slot_rect = hour_chunks[idx];
            app.hit_map.register(slot_rect, HitTarget::WeekSlot(current_date, h));
        }

        // Render Timed Events that fall on this day
        let timed_events_on_day: Vec<&_> = app.events_cache.iter()
            .filter(|ev| !ev.all_day && ev.occurs_on(current_date))
            .collect();

        for ev in timed_events_on_day {
            let ev_cal = app.calendars.iter().find(|c| c.id == ev.calendar_id);
            let cal_color = ev_cal.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);

            let ev_hour = ev.start_time.hour();
            if ev_hour >= start_hour && ev_hour <= end_hour {
                let slot_idx = (ev_hour - start_hour) as usize;
                let target_rect = hour_chunks[slot_idx];

                let ev_block = Paragraph::new(Line::from(vec![
                    Span::styled(
                        format!(" {}", ev.title),
                        Style::default()
                            .fg(Color::Black)
                            .bg(cal_color)
                            .add_modifier(Modifier::BOLD),
                    )
                ]));
                f.render_widget(ev_block, target_rect);
                app.hit_map.register(target_rect, HitTarget::EventItem(ev.id.clone()));
            }
        }
    }
}
