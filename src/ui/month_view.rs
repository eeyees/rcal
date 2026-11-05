use chrono::{Datelike, Duration, Local, NaiveDate};
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
    let selected_year = app.selected_date.year();
    let selected_month = app.selected_date.month();
    let today = Local::now().date_naive();
    let theme = &app.theme;

    let first_day = NaiveDate::from_ymd_opt(selected_year, selected_month, 1).unwrap();

    // Day of week offset for 1st day based on week start setting
    let start_offset = app.week_start_day.days_offset(first_day);

    // 6 rows of weeks
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Days of week header
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
        ])
        .split(area);

    // Determine calendar date for grid index (0..42)
    let grid_start_date = first_day - Duration::days(start_offset as i64);

    // 1. Render Weekday Header
    let dow_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 7); 7])
        .split(vertical_chunks[0]);

    let dows = app.week_start_day.weekday_headers(app.lang);
    for (i, dow) in dows.iter().enumerate() {
        let col_date = grid_start_date + Duration::days(i as i64);
        let weekday = col_date.weekday();
        let style = if weekday == chrono::Weekday::Sun {
            Style::default()
                .fg(theme.danger)
                .add_modifier(Modifier::BOLD)
        } else if weekday == chrono::Weekday::Sat {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.text_muted)
                .add_modifier(Modifier::BOLD)
        };
        let p = Paragraph::new(Line::from(vec![Span::raw(*dow)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, dow_chunks[i]);
    }

    for row in 0..6 {
        let week_area = vertical_chunks[row + 1];
        let day_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 7); 7])
            .split(week_area);

        for col in 0..7 {
            let day_idx = row * 7 + col;
            let current_cell_date = grid_start_date + Duration::days(day_idx as i64);
            let cell_area = day_chunks[col];

            let is_current_month = current_cell_date.month() == selected_month;
            let is_today = current_cell_date == today;
            let is_selected = current_cell_date == app.selected_date;

            // Day cell border & styling
            let border_color = if is_selected {
                theme.primary
            } else if is_today {
                theme.secondary
            } else {
                theme.border
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));

            let inner = block.inner(cell_area);
            f.render_widget(block, cell_area);

            // Register mouse hit for this entire cell
            app.hit_map
                .register(cell_area, HitTarget::MonthDay(current_cell_date));

            // Content inside cell: Day number header + events
            let mut cell_lines = Vec::new();

            // Day Number - consistent format across all states so text never shifts on focus/selection change
            let day_num_str = format!("{}", current_cell_date.day());
            let day_text = format!(" {} ", day_num_str);
            let is_red = app.is_red_day(current_cell_date);
            let is_blue = app.is_blue_day(current_cell_date);
            let holiday_name = app.get_holiday(current_cell_date);

            let day_span = if is_today {
                Span::styled(day_text, theme.today_style())
            } else if is_selected {
                Span::styled(day_text, theme.selected_day_style())
            } else if is_current_month {
                let color = if is_red {
                    theme.danger
                } else if is_blue {
                    theme.accent
                } else {
                    theme.text_normal
                };
                Span::styled(
                    day_text,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )
            } else {
                let color = if is_red {
                    theme.danger
                } else if is_blue {
                    theme.accent
                } else {
                    theme.text_dim
                };
                Span::styled(day_text, Style::default().fg(color).add_modifier(Modifier::DIM))
            };

            if let Some(h_name) = holiday_name {
                let max_badge_len = (inner.width as usize).saturating_sub(day_num_str.len() + 4);
                let badge_text = if h_name.chars().count() > max_badge_len && max_badge_len > 2 {
                    let truncated: String = h_name.chars().take(max_badge_len - 1).collect();
                    format!(" {truncated}…")
                } else {
                    format!(" {h_name}")
                };
                cell_lines.push(Line::from(vec![
                    day_span,
                    Span::styled(
                        badge_text,
                        Style::default()
                            .fg(theme.danger)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else {
                cell_lines.push(Line::from(vec![day_span]));
            }

            // Filter events for this cell (holidays already highlighted above)
            let events_for_day: Vec<&_> = app
                .events_cache
                .iter()
                .filter(|ev| ev.occurs_on(current_cell_date) && ev.calendar_id != "cal_holidays")
                .collect();

            let max_displayable = inner.height.saturating_sub(1) as usize;
            for (ev_idx, ev) in events_for_day.iter().take(max_displayable).enumerate() {
                let cal_color = app
                    .calendars
                    .iter()
                    .find(|c| c.id == ev.calendar_id)
                    .map(|c| c.ratatui_color())
                    .unwrap_or(Color::Cyan);

                let ev_line_rect = Rect {
                    x: inner.x,
                    y: inner.y + 1 + ev_idx as u16,
                    width: inner.width,
                    height: 1,
                };
                app.hit_map
                    .register(ev_line_rect, HitTarget::EventItem(ev.id.clone()));

                let prefix = if ev.all_day { "■ " } else { "• " };
                let time_or_title = if ev.all_day {
                    ev.title.clone()
                } else {
                    format!("{} {}", ev.start_time.format("%H:%M"), ev.title)
                };

                cell_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(cal_color)),
                    Span::styled(time_or_title, Style::default().fg(theme.text_normal)),
                ]));
            }

            if events_for_day.len() > max_displayable {
                let more_count = events_for_day.len() - max_displayable;
                let more_label = app.lang.more_events(more_count);
                cell_lines.push(Line::from(vec![Span::styled(
                    more_label,
                    Style::default()
                        .fg(theme.text_muted)
                        .add_modifier(Modifier::ITALIC),
                )]));
            }

            f.render_widget(Paragraph::new(cell_lines), inner);
        }
    }
}
