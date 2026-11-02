use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, ViewMode};
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let border_color = app.theme.border;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(app.lang.sidebar_calendars());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let is_week_or_day = matches!(app.current_view, ViewMode::Week | ViewMode::Day);

    if is_week_or_day {
        let cal_section_height = (app.calendars.len() as u16 + 2).min(6);

        if inner.height >= 17 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),                  // Mini Calendar
                    Constraint::Length(1),                  // Divider
                    Constraint::Length(cal_section_height), // Calendar list + manage button
                    Constraint::Length(1),                  // Divider
                    Constraint::Min(1),                     // Shortcuts hint
                    Constraint::Length(1),                  // Hide sidebar button
                ])
                .split(inner);

            render_mini_calendar(f, app, chunks[0]);
            render_divider(f, chunks[1], border_color);
            render_calendars(f, app, chunks[2]);
            render_divider(f, chunks[3], border_color);
            render_shortcuts(f, app, chunks[4]);
            render_hide_button(f, app, chunks[5]);
        } else if inner.height >= 12 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),                  // Mini Calendar
                    Constraint::Length(1),                  // Divider
                    Constraint::Length(cal_section_height), // Calendar list + manage button
                    Constraint::Length(1),                  // Hide sidebar button
                ])
                .split(inner);

            render_mini_calendar(f, app, chunks[0]);
            render_divider(f, chunks[1], border_color);
            render_calendars(f, app, chunks[2]);
            render_hide_button(f, app, chunks[3]);
        } else {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(8),    // Mini Calendar
                    Constraint::Length(1), // Hide sidebar button
                ])
                .split(inner);

            render_mini_calendar(f, app, chunks[0]);
            render_hide_button(f, app, chunks[1]);
        }
    } else {
        let cal_section_height = (app.calendars.len() as u16 + 2).min(8);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(cal_section_height), // Calendar list + manage button
                Constraint::Length(1),                  // Divider / Spacer
                Constraint::Min(6),                     // Shortcuts hint
                Constraint::Length(1),                  // Hide sidebar button
            ])
            .split(inner);

        render_calendars(f, app, chunks[0]);
        render_shortcuts(f, app, chunks[2]);
        render_hide_button(f, app, chunks[3]);
    }
}

fn render_mini_calendar(f: &mut Frame, app: &mut App, chunk: Rect) {
    if chunk.height < 7 || chunk.width < 21 {
        return;
    }
    let theme = &app.theme;
    let selected_date = app.selected_date;
    let display_year = selected_date.year();
    let display_month = selected_date.month();
    let today = Local::now().date_naive();

    let first_day = NaiveDate::from_ymd_opt(display_year, display_month, 1).unwrap();
    let start_offset = app.week_start_day.days_offset(first_day);
    let grid_start_date = first_day - ChronoDuration::days(start_offset as i64);

    let pad_left = (chunk.width.saturating_sub(21)) / 2;
    let mut lines = Vec::new();

    // Line 0: Header with ◀ Month Year ▶
    let month_title = app.lang.format_month_year(first_day);
    let title_w = Span::raw(&month_title).width();
    let total_btn_w = 4;
    let space_avail = 21_usize.saturating_sub(title_w + total_btn_w);
    let left_sp = space_avail / 2;
    let right_sp = space_avail - left_sp;

    let header_line = Line::from(vec![
        Span::raw(" ".repeat(pad_left as usize)),
        Span::styled(
            "◀",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(left_sp + 1)),
        Span::styled(
            &month_title,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(right_sp + 1)),
        Span::styled(
            "▶",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    lines.push(header_line);

    // Register hit targets for ◀ and ▶
    let prev_rect = Rect {
        x: chunk.x + pad_left,
        y: chunk.y,
        width: 3,
        height: 1,
    };
    app.hit_map.register(prev_rect, HitTarget::MiniCalPrev);

    let next_rect = Rect {
        x: chunk.x + pad_left + 18,
        y: chunk.y,
        width: 3,
        height: 1,
    };
    app.hit_map.register(next_rect, HitTarget::MiniCalNext);

    // Line 1: Weekday headers (DOW)
    let dows = app.week_start_day.weekday_headers(app.lang);
    let mut dow_spans = vec![Span::raw(" ".repeat(pad_left as usize))];
    for (i, dow) in dows.iter().enumerate() {
        let col_date = grid_start_date + ChronoDuration::days(i as i64);
        let weekday = col_date.weekday();
        let is_weekend = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;
        let color = if is_weekend {
            theme.accent
        } else {
            theme.text_muted
        };
        let dow_style = Style::default().fg(color).add_modifier(Modifier::BOLD);

        let w = Span::raw(*dow).width();
        let formatted_dow = if w == 2 {
            format!("{} ", dow)
        } else if w >= 3 {
            let short: String = dow.chars().take(2).collect();
            format!("{} ", short)
        } else {
            format!("{}  ", dow)
        };
        dow_spans.push(Span::styled(formatted_dow, dow_style));
    }
    lines.push(Line::from(dow_spans));

    // Lines 2..=7: 6 Weeks of dates
    for row in 0..6 {
        let mut row_spans = vec![Span::raw(" ".repeat(pad_left as usize))];
        for col in 0..7 {
            let day_idx = row * 7 + col;
            let current_cell_date = grid_start_date + ChronoDuration::days(day_idx as i64);

            let is_current_month = current_cell_date.month() == display_month;
            let is_today = current_cell_date == today;
            let is_selected = current_cell_date == selected_date;

            let day_num_str = format!("{:2}", current_cell_date.day());

            let day_style = if is_today {
                theme.today_style()
            } else if is_selected {
                theme.selected_day_style()
            } else if is_current_month {
                let weekday = current_cell_date.weekday();
                let is_weekend = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;
                let color = if is_weekend {
                    theme.accent
                } else {
                    theme.text_normal
                };
                Style::default().fg(color)
            } else {
                Style::default().fg(theme.text_dim)
            };

            row_spans.push(Span::styled(day_num_str, day_style));
            row_spans.push(Span::raw(" "));

            // Register mouse hit target for this day cell
            let cell_y = chunk.y + 2 + row as u16;
            if cell_y < chunk.y + chunk.height {
                let day_rect = Rect {
                    x: chunk.x + pad_left + (col as u16) * 3,
                    y: cell_y,
                    width: 3,
                    height: 1,
                };
                app.hit_map
                    .register(day_rect, HitTarget::MiniCalendarDay(current_cell_date));
            }
        }
        lines.push(Line::from(row_spans));
    }

    f.render_widget(Paragraph::new(lines), chunk);
}

fn render_divider(f: &mut Frame, area: Rect, border_color: ratatui::style::Color) {
    if area.height > 0 && area.width > 0 {
        let line = Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(border_color),
        ));
        f.render_widget(Paragraph::new(line), area);
    }
}

fn render_calendars(f: &mut Frame, app: &mut App, chunk: Rect) {
    let theme = &app.theme;
    let mut cal_lines = Vec::new();
    for (idx, cal) in app.calendars.iter().enumerate() {
        let check_box = if cal.enabled { "[x] " } else { "[ ] " };
        let check_color = if cal.enabled {
            theme.secondary
        } else {
            theme.text_dim
        };
        let cal_color = cal.ratatui_color();

        let max_name_len = (chunk.width as usize).saturating_sub(16);
        let display_name = if cal.name.chars().count() > max_name_len && max_name_len > 3 {
            let truncated: String = cal
                .name
                .chars()
                .take(max_name_len.saturating_sub(2))
                .collect();
            format!("{}..", truncated)
        } else {
            cal.name.clone()
        };

        let line = Line::from(vec![
            Span::styled(
                check_box,
                Style::default()
                    .fg(check_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("● ", Style::default().fg(cal_color)),
            Span::styled(
                display_name,
                Style::default().fg(if cal.enabled {
                    theme.text_normal
                } else {
                    theme.text_muted
                }),
            ),
            Span::styled(
                format!(" ({})", cal.provider.as_str()),
                Style::default().fg(theme.text_dim),
            ),
        ]);
        cal_lines.push(line);

        // Register hit region for this calendar row
        if (idx as u16) < chunk.height {
            let row_rect = Rect {
                x: chunk.x,
                y: chunk.y + idx as u16,
                width: chunk.width,
                height: 1,
            };
            app.hit_map
                .register(row_rect, HitTarget::CalendarToggle(cal.id.clone()));
        }
    }

    // Add "[ + 관리/추가 (C) ]" button under calendars
    let manage_label = app.lang.sidebar_manage();
    cal_lines.push(Line::from(""));
    cal_lines.push(Line::from(vec![Span::styled(
        manage_label,
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    )]));

    let btn_row = app.calendars.len() as u16 + 1;
    if btn_row < chunk.height {
        let btn_rect = Rect {
            x: chunk.x,
            y: chunk.y + btn_row,
            width: chunk.width,
            height: 1,
        };
        app.hit_map
            .register(btn_rect, HitTarget::BtnManageCalendars);
    }

    f.render_widget(Paragraph::new(cal_lines), chunk);
}

fn render_shortcuts(f: &mut Frame, app: &mut App, chunk: Rect) {
    let theme = &app.theme;
    let mut tips = Vec::new();
    let mut current_line_idx: u16 = 0;

    tips.push(Line::from(Span::styled(
        app.lang.sidebar_quick_nav(),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    current_line_idx += 1;

    for t in app.lang.sidebar_tips() {
        tips.push(Line::from(Span::styled(
            t,
            Style::default().fg(theme.text_muted),
        )));
        if current_line_idx < chunk.height {
            let row_rect = Rect {
                x: chunk.x,
                y: chunk.y + current_line_idx,
                width: chunk.width,
                height: 1,
            };
            if t.contains("T 키") || t.contains("'T'") || t.contains("T키") {
                app.hit_map.register(row_rect, HitTarget::BtnTheme);
            } else if t.contains("L 키") || t.contains("'L'") || t.contains("L키") {
                app.hit_map.register(row_rect, HitTarget::BtnLanguage);
            }
        }
        current_line_idx += 1;
    }

    tips.push(Line::from(""));
    current_line_idx += 1;

    tips.push(Line::from(Span::styled(
        app.lang.sidebar_keys(),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    current_line_idx += 1;

    for k in app.lang.sidebar_key_tips() {
        tips.push(Line::from(Span::styled(
            k,
            Style::default().fg(theme.text_muted),
        )));
        if current_line_idx < chunk.height {
            let row_rect = Rect {
                x: chunk.x,
                y: chunk.y + current_line_idx,
                width: chunk.width,
                height: 1,
            };
            if k.contains("v/C:") {
                app.hit_map
                    .register(row_rect, HitTarget::BtnManageCalendars);
            } else if k.contains("T:") {
                app.hit_map.register(row_rect, HitTarget::BtnTheme);
            } else if k.contains("A:") {
                app.hit_map.register(row_rect, HitTarget::BtnAccounts);
            } else if k.contains("c/n:") {
                app.hit_map.register(row_rect, HitTarget::BtnNewEvent);
            } else if k.contains("r:") {
                app.hit_map.register(row_rect, HitTarget::BtnSync);
            } else if k.contains("t:") {
                app.hit_map.register(row_rect, HitTarget::NavToday);
            } else if k.contains("b:") {
                app.hit_map.register(row_rect, HitTarget::BtnToggleSidebar);
            }
        }
        current_line_idx += 1;
    }

    f.render_widget(Paragraph::new(tips), chunk);
}

fn render_hide_button(f: &mut Frame, app: &mut App, chunk: Rect) {
    let theme = &app.theme;
    let hide_label = app.lang.sidebar_hide();
    let hide_btn = Paragraph::new(Line::from(vec![Span::styled(
        hide_label,
        Style::default().fg(theme.text_muted),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(hide_btn, chunk);
    app.hit_map.register(chunk, HitTarget::BtnToggleSidebar);
}
