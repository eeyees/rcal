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
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let today = Local::now().date_naive();
    let selected_date = app.selected_date;

    // Determine week start (Sunday)
    let offset_from_sun = selected_date.weekday().num_days_from_sunday();
    let week_start = selected_date - Duration::days(offset_from_sun as i64);

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
        Constraint::Length(2); 1 // Header row for Date
    ];
    let mut all_time_constraints = time_row_constraints;
    all_time_constraints.extend(vec![Constraint::Ratio(1, num_hours as u32); num_hours as usize]);

    let time_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(all_time_constraints)
        .split(time_area);

    // Render Time column labels
    for h in start_hour..=end_hour {
        let idx = (h - start_hour + 1) as usize;
        let time_str = format!("{:02}:00", h);
        let p = Paragraph::new(Line::from(vec![Span::styled(
            time_str,
            Style::default().fg(Theme::TEXT_MUTED),
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
                Constraint::Min(10),   // Timeline Body
            ])
            .split(col_rect);

        // Header for this day
        let dow_str = app.lang.weekday_name_short(col as usize);
        let date_header_text = format!("{} {:02}/{:02}", dow_str, current_date.month(), current_date.day());
        let header_style = if is_today {
            Theme::today_style()
        } else if is_selected {
            Theme::selected_day_style()
        } else {
            Style::default().fg(Theme::TEXT_NORMAL).add_modifier(Modifier::BOLD)
        };

        let p_head = Paragraph::new(Line::from(vec![Span::raw(date_header_text)]))
            .style(header_style)
            .alignment(Alignment::Center);
        f.render_widget(p_head, col_chunks[0]);

        // Register hit for day header
        app.hit_map.register(col_chunks[0], HitTarget::MonthDay(current_date));

        // Timeline Body with borders
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(if is_selected { Theme::PRIMARY } else { Theme::BORDER }));
        let body_inner = block.inner(col_chunks[1]);
        f.render_widget(block, col_chunks[1]);

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

        // Render Events that fall on this day
        let events_on_day: Vec<&_> = app.events_cache.iter()
            .filter(|ev| ev.occurs_on(current_date))
            .collect();

        for ev in events_on_day {
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
