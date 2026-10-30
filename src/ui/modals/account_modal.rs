use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{AccountModalState, App};
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    let popup_width = 72.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);

    let title = if state.is_adding {
        " Add CalDAV Account (Naver / Apple iCloud) "
    } else {
        " CalDAV Accounts & Cloud Sync "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(title, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    if state.is_adding {
        render_add_form(f, app, state, inner);
    } else {
        render_account_list(f, app, state, inner);
    }
}

fn render_add_form(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Preset buttons
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Name
            Constraint::Length(1), // URL
            Constraint::Length(1), // Username
            Constraint::Length(1), // Password
            Constraint::Min(2),    // Notes/help
            Constraint::Length(1), // Buttons
        ])
        .split(area);

    // 1. Presets (Naver, Apple iCloud, Generic)
    let preset_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // Naver
            Constraint::Length(1),
            Constraint::Length(18), // Apple
            Constraint::Length(1),
            Constraint::Length(18), // Generic
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let presets = [
        (0, " [ Naver CalDAV ] ", preset_chunks[0]),
        (1, " [ Apple iCloud ] ", preset_chunks[2]),
        (2, " [ Custom CalDAV ] ", preset_chunks[4]),
    ];

    for (idx, label, rect) in presets {
        let is_selected = state.provider_idx == idx;
        let style = if is_selected {
            Theme::header_tab_active()
        } else {
            Theme::header_tab_inactive()
        };
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalAccountPreset(idx));
    }

    let active_style = Style::default().fg(Theme::TEXT_HIGHLIGHT).bg(Color::Rgb(40, 60, 100)).add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Theme::TEXT_NORMAL);

    // Name
    let name_line = Line::from(vec![
        Span::styled(" Name:     ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.name.is_empty() { "(Account name)" } else { &state.name },
            if state.active_field == 1 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(name_line), chunks[2]);
    app.hit_map.register(chunks[2], HitTarget::ModalAccountField(1));

    // URL
    let url_line = Line::from(vec![
        Span::styled(" URL:      ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            &state.url,
            if state.active_field == 2 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(url_line), chunks[3]);
    app.hit_map.register(chunks[3], HitTarget::ModalAccountField(2));

    // Username
    let user_line = Line::from(vec![
        Span::styled(" User/ID:  ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.username.is_empty() { "(e.g. naver_id or apple_id@icloud.com)" } else { &state.username },
            if state.active_field == 3 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(user_line), chunks[4]);
    app.hit_map.register(chunks[4], HitTarget::ModalAccountField(3));

    // Password
    let masked_pw = "•".repeat(state.password.len());
    let pw_line = Line::from(vec![
        Span::styled(" Password: ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.password.is_empty() { "(App password or 2-factor app password)" } else { &masked_pw },
            if state.active_field == 4 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(pw_line), chunks[5]);
    app.hit_map.register(chunks[5], HitTarget::ModalAccountField(4));

    // Help note
    let note = vec![
        Line::from(Span::styled(" Tip: For Apple/Naver, use a generated App-Specific Password.", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::ITALIC))),
    ];
    f.render_widget(Paragraph::new(note), chunks[6]);

    // Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(18), // [ Connect & Sync ]
            Constraint::Length(3),
            Constraint::Length(12), // [ Cancel ]
            Constraint::Min(1),
        ])
        .split(chunks[7]);

    let btn_conn = Paragraph::new(Line::from(vec![Span::raw(" [ Connect & Sync ] ")]))
        .style(Style::default().fg(Color::Black).bg(Theme::SECONDARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_conn, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalAccountConnect);

    let btn_cancel = Paragraph::new(Line::from(vec![Span::raw(" [ Cancel ] ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_cancel, btn_chunks[3]);
    app.hit_map.register(btn_chunks[3], HitTarget::ModalCancel);
}

fn render_account_list(f: &mut Frame, app: &mut App, _state: &AccountModalState, area: Rect) {
    let accounts = app.db.get_accounts().unwrap_or_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // List of accounts
            Constraint::Length(1), // Buttons
        ])
        .split(area);

    let mut lines = Vec::new();
    if accounts.is_empty() {
        lines.push(Line::from(Span::styled(
            "No CalDAV accounts configured yet. Add your Naver or Apple iCloud calendar below!",
            Style::default().fg(Theme::TEXT_MUTED).add_modifier(Modifier::ITALIC),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Configured Sync Accounts:",
            Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD),
        )));

        for (idx, acc) in accounts.iter().enumerate() {
            let last_sync = acc.last_synced_at
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Never".to_string());

            let line = Line::from(vec![
                Span::styled(" • ", Style::default().fg(Theme::PRIMARY)),
                Span::styled(&acc.name, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" [{}]", acc.provider.as_str()), Style::default().fg(Theme::SECONDARY)),
                Span::styled(format!(" - User: {} | Last synced: {}", acc.username, last_sync), Style::default().fg(Theme::TEXT_MUTED)),
                Span::raw("  "),
                Span::styled("[ Delete ]", Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD)),
            ]);
            lines.push(line);

            // Register delete button hit for this row
            let row_y = chunks[0].y + 1 + idx as u16;
            if row_y < chunks[0].y + chunks[0].height {
                let del_rect = Rect {
                    x: chunks[0].x + chunks[0].width.saturating_sub(14),
                    y: row_y,
                    width: 12,
                    height: 1,
                };
                app.hit_map.register(del_rect, HitTarget::ModalAccountDelete(acc.id.clone()));
            }
        }
    }

    f.render_widget(Paragraph::new(lines), chunks[0]);

    // Bottom Action Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // [ + Add CalDAV Account ]
            Constraint::Length(2),
            Constraint::Length(14), // [ ⟳ Sync Now ]
            Constraint::Min(1),
            Constraint::Length(12), // [ Close ]
        ])
        .split(chunks[1]);

    let btn_add = Paragraph::new(Line::from(vec![Span::raw(" [ + Add Account ] ")]))
        .style(Style::default().fg(Color::Black).bg(Theme::PRIMARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_add, btn_chunks[0]);
    app.hit_map.register(btn_chunks[0], HitTarget::ModalAccountAdd);

    let btn_sync = Paragraph::new(Line::from(vec![Span::raw(" [ ⟳ Sync Now ] ")]))
        .style(Style::default().fg(Color::Black).bg(Theme::SECONDARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_sync, btn_chunks[2]);
    app.hit_map.register(btn_chunks[2], HitTarget::BtnSync);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(" [ Close (q) ] ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[4]);
    app.hit_map.register(btn_chunks[4], HitTarget::ModalClose);
}
