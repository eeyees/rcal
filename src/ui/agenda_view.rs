use chrono::{Local, NaiveDate};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(" Agenda / Upcoming Events ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.events_cache.is_empty() {
        let p =
            Paragraph::new("No upcoming events found. Press 'n' or click '+ New' to create one.")
                .style(
                    Style::default()
                        .fg(theme.text_muted)
                        .add_modifier(Modifier::ITALIC),
                );
        f.render_widget(p, inner);
        return;
    }

    let today = Local::now().date_naive();
    let lookahead_days = 60;

    // Collect occurrences for upcoming days (including recurring events)
    let mut upcoming_occurrences: Vec<(NaiveDate, &crate::model::Event)> = Vec::new();
    for d_offset in 0..=lookahead_days {
        let current_date = today + chrono::Duration::days(d_offset);
        for ev in &app.events_cache {
            if ev.occurs_on(current_date) {
                upcoming_occurrences.push((current_date, ev));
            }
        }
    }

    // Fallback: If no upcoming events in next 60 days, show all events sorted by start_time
    if upcoming_occurrences.is_empty() {
        for ev in &app.events_cache {
            upcoming_occurrences.push((ev.start_time.date_naive(), ev));
        }
    }

    // Sort by occurrence date, then all-day first, then time
    upcoming_occurrences.sort_by(|(d1, e1), (d2, e2)| {
        d1.cmp(d2)
            .then_with(|| e2.all_day.cmp(&e1.all_day))
            .then_with(|| e1.start_time.time().cmp(&e2.start_time.time()))
    });

    let mut current_y = inner.y;
    let max_y = inner.y + inner.height;
    let mut prev_date: Option<NaiveDate> = None;

    for (idx, (ev_date, ev)) in upcoming_occurrences.iter().enumerate() {
        if current_y >= max_y {
            break;
        }

        // If date changes, print a date header
        if prev_date != Some(*ev_date) {
            prev_date = Some(*ev_date);
            let today = Local::now().date_naive();
            let date_label = if *ev_date == today {
                format!("Today, {}", ev_date.format("%A, %b %d"))
            } else if *ev_date == today + chrono::Duration::days(1) {
                format!("Tomorrow, {}", ev_date.format("%A, %b %d"))
            } else {
                ev_date.format("%A, %B %d, %Y").to_string()
            };

            let date_header = Paragraph::new(Line::from(vec![Span::styled(
                format!("── {} ────────────────────────────", date_label),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]));
            f.render_widget(
                date_header,
                Rect {
                    x: inner.x,
                    y: current_y,
                    width: inner.width,
                    height: 1,
                },
            );
            current_y += 1;
            if current_y >= max_y {
                break;
            }
        }

        let is_selected = idx == app.agenda_selected_index;
        let cal = app.calendars.iter().find(|c| c.id == ev.calendar_id);
        let cal_color = cal.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);
        let cal_name = cal.map(|c| c.name.as_str()).unwrap_or("General");

        let row_rect = Rect {
            x: inner.x,
            y: current_y,
            width: inner.width,
            height: 1,
        };

        let ev_line = Line::from(vec![
            Span::raw(if is_selected { " ▶ " } else { "   " }),
            Span::styled("● ", Style::default().fg(cal_color)),
            Span::styled(
                format!("{:14} ", ev.time_str()),
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                &ev.title,
                Style::default()
                    .fg(if is_selected {
                        theme.text_highlight
                    } else {
                        theme.text_normal
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(format!("[{}]", cal_name), Style::default().fg(cal_color)),
            if let Some(loc) = &ev.location {
                Span::styled(format!(" @ {}", loc), Style::default().fg(theme.text_dim))
            } else {
                Span::raw("")
            },
        ]);

        let row_style = if is_selected {
            Style::default().bg(theme.selected_bg)
        } else {
            Style::default()
        };

        f.render_widget(Paragraph::new(ev_line).style(row_style), row_rect);
        app.hit_map
            .register(row_rect, HitTarget::EventItem(ev.id.clone()));

        current_y += 1;
    }
}
