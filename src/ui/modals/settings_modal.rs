use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::model::WeekStartDay;
use crate::mouse::HitTarget;
use crate::ui::theme::{Theme, ThemeMode};

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 74.min(area.width.saturating_sub(4));
    let popup_height = 23.min(area.height.saturating_sub(4));

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
    let title = t.modal_settings_title;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(title, Style::default().fg(theme.text_highlight).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title 1: Week start day
            Constraint::Length(1), // Subtitle 1
            Constraint::Length(1), // 3 Buttons for Week start day
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Title 2: Color Themes
            Constraint::Length(1), // Subtitle 2
            Constraint::Length(1), // Row 1 of Themes (4 themes)
            Constraint::Length(1), // Row 2 of Themes (3 themes)
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Title 3: Language
            Constraint::Length(1), // Buttons for Language (dynamically rendered)
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Keyboard navigation guide
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Close button
        ])
        .split(inner);

    // Section 1: Week Start Day
    f.render_widget(
        Paragraph::new(Span::styled(t.settings_week_start_title, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))),
        chunks[0],
    );

    f.render_widget(
        Paragraph::new(Span::styled(t.settings_week_start_desc, Style::default().fg(theme.text_muted))),
        chunks[1],
    );

    let week_btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // Sunday
            Constraint::Length(2),
            Constraint::Length(18), // Monday
            Constraint::Length(2),
            Constraint::Length(18), // Saturday
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    let options = [
        (WeekStartDay::Sunday, "일요일 (Sun)", "[1]"),
        (WeekStartDay::Monday, "월요일 (Mon)", "[2]"),
        (WeekStartDay::Saturday, "토요일 (Sat)", "[3]"),
    ];

    for (idx, (day, label, shortcut)) in options.iter().enumerate() {
        let is_selected = app.week_start_day == *day;
        let prefix = if is_selected { "● " } else { "○ " };
        let text = format!("{}{} {}", prefix, label, shortcut);

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.text_normal)
                .bg(Color::Rgb(40, 45, 55))
        };

        let rect = week_btn_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingWeekStart(*day));
    }

    // Section 2: Color Themes
    f.render_widget(
        Paragraph::new(Span::styled(t.settings_theme_title, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))),
        chunks[4],
    );

    f.render_widget(
        Paragraph::new(Span::styled(t.settings_theme_desc, Style::default().fg(theme.text_muted))),
        chunks[5],
    );

    // Row 1: Modern Dark, Nord, Dracula, Forest
    let theme_row1 = [
        ThemeMode::ModernDark,
        ThemeMode::Nord,
        ThemeMode::Dracula,
        ThemeMode::Forest,
    ];
    let row1_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(15),
            Constraint::Length(1),
            Constraint::Length(15),
            Constraint::Length(1),
            Constraint::Length(15),
            Constraint::Length(1),
            Constraint::Length(15),
            Constraint::Min(0),
        ])
        .split(chunks[6]);

    for (idx, mode) in theme_row1.iter().enumerate() {
        let is_selected = app.theme.mode == *mode;
        let prefix = if is_selected { "● " } else { "○ " };
        let label = format!("{}{}", prefix, mode.short_name(lang));
        let sample = Theme::from_mode(*mode);

        let style = if is_selected {
            Style::default().fg(Color::Black).bg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(sample.primary).bg(Color::Rgb(40, 45, 55))
        };

        let rect = row1_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingTheme(*mode));
    }

    // Row 2: Sunset, Monokai, Solarized
    let theme_row2 = [
        ThemeMode::Sunset,
        ThemeMode::Monokai,
        ThemeMode::Solarized,
    ];
    let row2_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(15),
            Constraint::Length(1),
            Constraint::Length(15),
            Constraint::Length(1),
            Constraint::Length(15),
            Constraint::Min(0),
        ])
        .split(chunks[7]);

    for (idx, mode) in theme_row2.iter().enumerate() {
        let is_selected = app.theme.mode == *mode;
        let prefix = if is_selected { "● " } else { "○ " };
        let label = format!("{}{}", prefix, mode.short_name(lang));
        let sample = Theme::from_mode(*mode);

        let style = if is_selected {
            Style::default().fg(Color::Black).bg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(sample.primary).bg(Color::Rgb(40, 45, 55))
        };

        let rect = row2_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingTheme(*mode));
    }

    // Section 3: Language (dynamically rendered for all supported languages in Language::ALL)
    f.render_widget(
        Paragraph::new(Span::styled(t.settings_lang_title, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))),
        chunks[9],
    );

    let all_langs = crate::i18n::Language::ALL;
    let mut lang_constraints = Vec::with_capacity(all_langs.len() * 2 + 1);
    for _ in all_langs {
        lang_constraints.push(Constraint::Length(18));
        lang_constraints.push(Constraint::Length(2));
    }
    lang_constraints.push(Constraint::Min(0));

    let lang_btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(lang_constraints)
        .split(chunks[10]);

    for (idx, lang_opt) in all_langs.iter().enumerate() {
        let is_selected = app.lang == *lang_opt;
        let prefix = if is_selected { "● " } else { "○ " };
        let text = format!("{}{} [L]", prefix, lang_opt.label());
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_normal).bg(Color::Rgb(40, 45, 55))
        };
        let rect = lang_btn_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSelectLanguage(*lang_opt));
    }

    // Section 4: Keyboard shortcuts guide
    let hint_line = Line::from(vec![
        Span::styled(t.settings_hint_shortcuts, Style::default().fg(theme.text_muted)),
        Span::styled("[1/2/3]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(t.settings_hint_day_select),
        Span::styled("[T/c]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" Theme  "),
        Span::styled("[Tab/←/→]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(t.settings_hint_cycle),
        Span::styled("[L]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(t.settings_hint_lang),
    ]);
    f.render_widget(Paragraph::new(hint_line).alignment(Alignment::Center), chunks[12]);

    // Close button
    let close_btn = Paragraph::new(Line::from(vec![Span::raw(t.settings_btn_close)]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(close_btn, chunks[14]);
    app.hit_map.register(chunks[14], HitTarget::ModalClose);

    // Place cursor on active week start option
    let active_idx = match app.week_start_day {
        WeekStartDay::Sunday => 0,
        WeekStartDay::Monday => 1,
        WeekStartDay::Saturday => 2,
    };
    let active_rect = week_btn_chunks[active_idx * 2];
    f.set_cursor_position((active_rect.x + active_rect.width / 2, active_rect.y));
}
