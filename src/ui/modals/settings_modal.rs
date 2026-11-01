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
use crate::ui::theme::{Theme, ThemeCategory, ThemeMode};

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 86.min(area.width.saturating_sub(2));
    let popup_height = 26.min(area.height.saturating_sub(1));

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
        .style(Style::default().bg(theme.panel_bg))
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
            Constraint::Length(1), // Subtitle 2 & Active Theme Indicator
            Constraint::Length(1), // Category Tabs & [◀ Prev] [Next ▶]
            Constraint::Length(1), // Theme Row 1 (up to 5 buttons)
            Constraint::Length(1), // Theme Row 2 (up to 5 buttons)
            Constraint::Length(1), // Config file path hint
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Title 3: Language
            Constraint::Length(6), // Buttons for Language (4 cols x 6 rows grid)
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Keyboard navigation guide
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Close button
        ])
        .split(inner);

    // =========================================================================
    // Section 1: Week Start Day
    // =========================================================================
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
                .fg(if theme.mode.is_light() { Color::White } else { Color::Black })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.button_style()
        };

        let rect = week_btn_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingWeekStart(*day));
    }

    // =========================================================================
    // Section 2: Color Themes (37 Themes across 4 Categories)
    // =========================================================================
    let all_themes = ThemeMode::all();
    let current_idx = all_themes.iter().position(|m| *m == app.theme.mode).unwrap_or(0);

    let theme_header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Length(26),
        ])
        .split(chunks[4]);

    f.render_widget(
        Paragraph::new(Span::styled(t.settings_theme_title, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))),
        theme_header_chunks[0],
    );

    let open_modal_label = match lang {
        crate::i18n::Language::Ko => "[ 🎨 전체 테마 모달 (T) ]",
        crate::i18n::Language::Ja => "[ 🎨 テーマ一覧 (T) ]",
        _ => "[ 🎨 Theme Modal (T) ]",
    };
    let btn_open_modal = Paragraph::new(Line::from(vec![Span::raw(open_modal_label)]))
        .style(theme.button_style())
        .alignment(Alignment::Right);
    f.render_widget(btn_open_modal, theme_header_chunks[1]);
    app.hit_map.register(theme_header_chunks[1], HitTarget::BtnTheme);

    let current_theme_badge = format!("  [ {}/{} ] {}", current_idx + 1, all_themes.len(), app.theme.mode.label(lang));
    let theme_desc_line = Line::from(vec![
        Span::styled(t.settings_theme_desc, Style::default().fg(theme.text_muted)),
        Span::styled(current_theme_badge, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(theme_desc_line), chunks[5]);

    // Row: [◀ Prev] | [1:Dark] [2:Nature] [3:Vibrant] [4:Light] | [Next ▶]
    let cat_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // [◀ Prev]
            Constraint::Length(1),
            Constraint::Length(13), // Dark
            Constraint::Length(1),
            Constraint::Length(13), // Nature
            Constraint::Length(1),
            Constraint::Length(13), // Vibrant
            Constraint::Length(1),
            Constraint::Length(13), // Light
            Constraint::Length(1),
            Constraint::Length(10), // [Next ▶]
            Constraint::Min(0),
        ])
        .split(chunks[6]);

    // Prev Button
    let prev_btn = Paragraph::new(Line::from(vec![Span::raw("◀ 이전 (c)")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(prev_btn, cat_chunks[0]);
    app.hit_map.register(cat_chunks[0], HitTarget::ModalSettingThemePrev);

    // 4 Category Tabs
    let categories = [
        (ThemeCategory::Dark, "다크 (10)"),
        (ThemeCategory::Nature, "내추럴 (9)"),
        (ThemeCategory::Vibrant, "비비드 (8)"),
        (ThemeCategory::Light, "라이트 (10)"),
    ];

    for (idx, (cat, name)) in categories.iter().enumerate() {
        let is_active_cat = app.theme_category == *cat;
        let prefix = if is_active_cat { "● " } else { "○ " };
        let text = format!("{}{}", prefix, name);
        let style = if is_active_cat {
            Style::default()
                .fg(if theme.mode.is_light() { Color::White } else { Color::Black })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.button_style()
        };

        let rect = cat_chunks[2 + idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingThemeCategory(*cat));
    }

    // Next Button
    let next_btn = Paragraph::new(Line::from(vec![Span::raw("다음 (C) ▶")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(next_btn, cat_chunks[10]);
    app.hit_map.register(cat_chunks[10], HitTarget::ModalSettingThemeNext);

    // Active Category Themes Grid (2 rows x up to 5 buttons)
    let cat_themes = app.theme_category.themes();
    let num_in_cat = cat_themes.len();
    let row1_count = (num_in_cat + 1) / 2;
    let row2_count = num_in_cat.saturating_sub(row1_count);

    // Row 1 themes
    let row1_themes = &cat_themes[0..row1_count.min(num_in_cat)];
    let mut row1_constraints = Vec::with_capacity(row1_themes.len() * 2);
    for _ in 0..row1_themes.len() {
        row1_constraints.push(Constraint::Length(16));
        row1_constraints.push(Constraint::Length(1));
    }
    row1_constraints.push(Constraint::Min(0));

    let row1_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(row1_constraints)
        .split(chunks[7]);

    for (idx, mode) in row1_themes.iter().enumerate() {
        let is_selected = app.theme.mode == *mode;
        let prefix = if is_selected { "● " } else { "○ " };
        let label = format!("{}{}", prefix, mode.short_name(lang));
        let sample = Theme::from_mode(*mode);

        let style = if is_selected {
            Style::default()
                .fg(if mode.is_light() { Color::White } else { Color::Black })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(sample.primary)
                .bg(theme.button_bg)
        };

        let rect = row1_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingTheme(*mode));
    }

    // Row 2 themes
    if row2_count > 0 {
        let row2_themes = &cat_themes[row1_count..num_in_cat];
        let mut row2_constraints = Vec::with_capacity(row2_themes.len() * 2);
        for _ in 0..row2_themes.len() {
            row2_constraints.push(Constraint::Length(16));
            row2_constraints.push(Constraint::Length(1));
        }
        row2_constraints.push(Constraint::Min(0));

        let row2_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(row2_constraints)
            .split(chunks[8]);

        for (idx, mode) in row2_themes.iter().enumerate() {
            let is_selected = app.theme.mode == *mode;
            let prefix = if is_selected { "● " } else { "○ " };
            let label = format!("{}{}", prefix, mode.short_name(lang));
            let sample = Theme::from_mode(*mode);

            let style = if is_selected {
                Style::default()
                    .fg(if mode.is_light() { Color::White } else { Color::Black })
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(sample.primary)
                    .bg(theme.button_bg)
            };

            let rect = row2_chunks[idx * 2];
            let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
                .style(style)
                .alignment(Alignment::Center);
            f.render_widget(p, rect);
            app.hit_map.register(rect, HitTarget::ModalSettingTheme(*mode));
        }
    }

    // Hint: Config file location for customization & contrast adjustments
    let cfg_hint = match lang {
        crate::i18n::Language::Ko => "📁 설정 파일: ~/.config/rcal/theme.toml (색상 오버라이드 및 대비 조절 가능)",
        crate::i18n::Language::Ja => "📁 設定ファイル: ~/.config/rcal/theme.toml (色の上書き・コントラスト調整可能)",
        _ => "📁 Config file: ~/.config/rcal/theme.toml (Color overrides & contrast options)",
    };
    f.render_widget(Paragraph::new(Span::styled(cfg_hint, Style::default().fg(theme.text_dim))), chunks[9]);

    // =========================================================================
    // Section 3: Language (4 columns x 6 rows grid for all supported languages)
    // =========================================================================
    f.render_widget(
        Paragraph::new(Span::styled(t.settings_lang_title, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))),
        chunks[11],
    );

    let all_langs = crate::i18n::Language::ALL;
    let grid_area = chunks[12];
    let num_cols = 4;
    let num_rows = (all_langs.len() + num_cols - 1) / num_cols;
    let col_w = (grid_area.width.saturating_sub(num_cols as u16 - 1)) / (num_cols as u16);

    for r in 0..num_rows {
        if r as u16 >= grid_area.height {
            break;
        }
        for c in 0..num_cols {
            let idx = r * num_cols + c;
            if idx >= all_langs.len() {
                break;
            }
            let lang_opt = all_langs[idx];
            let is_selected = app.lang == lang_opt;
            let prefix = if is_selected { "● " } else { "○ " };
            let text = format!(" {}{}", prefix, lang_opt.label());

            let style = if is_selected {
                Style::default()
                    .fg(if theme.mode.is_light() { Color::White } else { Color::Black })
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_normal).bg(theme.button_bg)
            };

            let x = grid_area.x + (c as u16) * (col_w + 1);
            let y = grid_area.y + (r as u16);
            let w = if c == num_cols - 1 {
                grid_area.width.saturating_sub((c as u16) * (col_w + 1))
            } else {
                col_w
            };
            let rect = Rect {
                x,
                y,
                width: w,
                height: 1,
            };

            let p = Paragraph::new(Line::from(vec![Span::raw(text)])).style(style);
            f.render_widget(p, rect);
            app.hit_map.register(rect, HitTarget::ModalSelectLanguage(lang_opt));
        }
    }

    // =========================================================================
    // Section 4: Keyboard shortcuts guide & Close
    // =========================================================================
    let hint_line = Line::from(vec![
        Span::styled(t.settings_hint_shortcuts, Style::default().fg(theme.text_muted)),
        Span::styled("[1/2/3]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(t.settings_hint_day_select),
        Span::styled("[T/c/C]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" Theme  "),
        Span::styled("[Tab/←/→]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(t.settings_hint_cycle),
        Span::styled("[L]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(t.settings_hint_lang),
    ]);
    f.render_widget(Paragraph::new(hint_line).alignment(Alignment::Center), chunks[14]);

    // Close button
    let close_btn = Paragraph::new(Line::from(vec![Span::raw(t.settings_btn_close)]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(close_btn, chunks[16]);
    app.hit_map.register(chunks[16], HitTarget::ModalClose);

    // Place cursor on active week start option
    let active_idx = match app.week_start_day {
        WeekStartDay::Sunday => 0,
        WeekStartDay::Monday => 1,
        WeekStartDay::Saturday => 2,
    };
    let active_rect = week_btn_chunks[active_idx * 2];
    f.set_cursor_position((active_rect.x + active_rect.width / 2, active_rect.y));
}
