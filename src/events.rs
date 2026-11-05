use anyhow::Result;
use chrono::{Duration, Timelike};
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};

use crate::app::{ActiveModal, App, EventFormState, SettingsSection, ViewMode};
use crate::i18n::hangul_to_qwerty;
use crate::model::{CalDavAccount, ProviderType};
use crate::ui::theme::ThemeCategory;

/// Normalize key events so that Korean 2-beolsik keystrokes (Jamo / Syllables)
/// transparently map to standard English QWERTY shortcuts during navigation & commands.
fn normalize_key_event(key: KeyEvent) -> KeyCode {
    match key.code {
        KeyCode::Char(c) => {
            // 한글 키보드에서 'L' 자리는 모음 'ㅣ'(U+3163, U+1175).
            // 한글 입력기 상태에서 L 버튼을 누르면 Shift 여부와 무관하게 'ㅣ'가 들어오며,
            // macOS/Linux IME는 'ㅣ'에 대해 Shift modifier를 터미널로 전달하지 않고 삼킵니다.
            // 또한 이전 타이핑으로 '이'(U+C774) 음절이 들어올 수 있으므로,
            // 'ㅣ' 및 '이'는 언어 선택 모달 단축키인 대문자 'L'로 매핑합니다.
            if c == 'ㅣ' || c == '\u{1175}' || c == '이' {
                return KeyCode::Char('L');
            }

            let mut mapped = hangul_to_qwerty(c);
            if key.modifiers.contains(KeyModifiers::SHIFT) || c.is_ascii_uppercase() {
                mapped = mapped.to_ascii_uppercase();
            }
            KeyCode::Char(mapped)
        }
        other => other,
    }
}

pub fn handle_crossterm_event(app: &mut App, event: CrosstermEvent) -> Result<()> {
    match event {
        CrosstermEvent::Key(key) => handle_key_event(app, key)?,
        CrosstermEvent::Mouse(mouse) => handle_mouse_event(app, mouse)?,
        _ => {}
    }
    Ok(())
}

fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> Result<()> {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            app.handle_mouse_click(mouse.column, mouse.row)?;
        }
        MouseEventKind::ScrollDown => {
            app.nav_next();
        }
        MouseEventKind::ScrollUp => {
            app.nav_prev();
        }
        _ => {}
    }
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    // 1. Handle EventForm modal
    let mut form_action_save: Option<EventFormState> = None;
    let mut form_action_close = false;

    if let ActiveModal::EventForm(ref mut form) = app.modal {
        match key.code {
            KeyCode::Esc => {
                form_action_close = true;
            }
            KeyCode::Tab | KeyCode::Down => {
                form.active_field = (form.active_field + 1) % 10;
            }
            KeyCode::BackTab | KeyCode::Up => {
                if form.active_field == 0 {
                    form.active_field = 9;
                } else {
                    form.active_field -= 1;
                }
            }
            KeyCode::Left => {
                if form.active_field == 1 {
                    let current_idx = app
                        .calendars
                        .iter()
                        .position(|c| c.id == form.calendar_id)
                        .unwrap_or(0);
                    let prev_idx = if current_idx == 0 {
                        app.calendars.len().saturating_sub(1)
                    } else {
                        current_idx - 1
                    };
                    if let Some(c) = app.calendars.get(prev_idx) {
                        form.calendar_id = c.id.clone();
                    }
                } else if form.active_field == 9 {
                    form.active_field = 8;
                }
            }
            KeyCode::Right => {
                if form.active_field == 1 {
                    let current_idx = app
                        .calendars
                        .iter()
                        .position(|c| c.id == form.calendar_id)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % app.calendars.len().max(1);
                    if let Some(c) = app.calendars.get(next_idx) {
                        form.calendar_id = c.id.clone();
                    }
                } else if form.active_field == 8 {
                    form.active_field = 9;
                }
            }
            KeyCode::Enter => {
                if form.active_field == 9 {
                    form_action_close = true;
                } else {
                    form_action_save = Some(form.clone());
                }
            }
            KeyCode::Char(' ') if form.active_field == 1 => {
                let current_idx = app
                    .calendars
                    .iter()
                    .position(|c| c.id == form.calendar_id)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % app.calendars.len().max(1);
                if let Some(next_cal) = app.calendars.get(next_idx) {
                    form.calendar_id = next_cal.id.clone();
                }
            }
            KeyCode::Char(' ') if form.active_field == 5 => {
                form.all_day = !form.all_day;
            }
            KeyCode::Char(' ') if form.active_field == 8 => {
                form_action_save = Some(form.clone());
            }
            KeyCode::Char(' ') if form.active_field == 9 => {
                form_action_close = true;
            }
            KeyCode::Backspace => match form.active_field {
                0 => {
                    form.title.pop();
                }
                2 => {
                    form.date_str.pop();
                }
                3 => {
                    form.start_time_str.pop();
                }
                4 => {
                    form.end_time_str.pop();
                }
                6 => {
                    form.location.pop();
                }
                7 => {
                    form.description.pop();
                }
                _ => {}
            },
            KeyCode::Char(c) => match form.active_field {
                0 => form.title.push(c),
                2 => form.date_str.push(c),
                3 => form.start_time_str.push(c),
                4 => form.end_time_str.push(c),
                6 => form.location.push(c),
                7 => form.description.push(c),
                _ => {}
            },
            _ => {}
        }
    }

    if form_action_close {
        app.close_modal();
        return Ok(());
    } else if let Some(form) = form_action_save {
        app.save_event_from_form(form)?;
        return Ok(());
    } else if matches!(app.modal, ActiveModal::EventForm(_)) {
        return Ok(());
    }

    // 2. Handle Accounts modal
    let mut acc_action_close = false;
    let mut acc_action_sync = false;
    let mut acc_action_save: Option<CalDavAccount> = None;
    let mut acc_action_delete_idx: Option<usize> = None;

    if let ActiveModal::Accounts(ref mut state) = app.modal {
        if !state.is_adding {
            let norm_code = normalize_key_event(key);
            match norm_code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => acc_action_close = true,
                KeyCode::Char('+')
                | KeyCode::Char('=')
                | KeyCode::Char('a')
                | KeyCode::Char('A')
                | KeyCode::Char('n')
                | KeyCode::Char('N')
                | KeyCode::Char('c')
                | KeyCode::Char('C')
                | KeyCode::Enter => {
                    state.is_adding = true;
                    state.active_field = 0;
                }
                KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::F(5) => acc_action_sync = true,
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete | KeyCode::Backspace => {
                    acc_action_delete_idx = Some(state.selected_account_idx);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let count = app.db.get_accounts().unwrap_or_default().len();
                    if count > 0 {
                        state.selected_account_idx = (state.selected_account_idx + 1) % count;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let count = app.db.get_accounts().unwrap_or_default().len();
                    if count > 0 {
                        if state.selected_account_idx == 0 {
                            state.selected_account_idx = count - 1;
                        } else {
                            state.selected_account_idx -= 1;
                        }
                    }
                }
                _ => {}
            }
        } else {
            let is_oauth = state.provider_idx == 2 || state.provider_idx == 3;
            let max_fields = if is_oauth { 8 } else { 7 };

            match key.code {
                KeyCode::Esc => state.is_adding = false,
                KeyCode::Tab | KeyCode::Down => {
                    state.active_field = (state.active_field + 1) % max_fields;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    if state.active_field == 0 {
                        state.active_field = max_fields - 1;
                    } else {
                        state.active_field -= 1;
                    }
                }
                KeyCode::Left => {
                    if state.active_field == 0 {
                        let prev_idx = if state.provider_idx == 0 {
                            5
                        } else {
                            state.provider_idx - 1
                        };
                        state.set_preset(prev_idx);
                    } else if is_oauth {
                        if state.active_field == 7 {
                            state.active_field = 6;
                        } else if state.active_field == 6 {
                            state.active_field = 5;
                        }
                    } else if state.active_field == 6 {
                        state.active_field = 5;
                    }
                }
                KeyCode::Right => {
                    if state.active_field == 0 {
                        let next_idx = (state.provider_idx + 1) % 6;
                        state.set_preset(next_idx);
                    } else if is_oauth {
                        if state.active_field == 5 {
                            state.active_field = 6;
                        } else if state.active_field == 6 {
                            state.active_field = 7;
                        }
                    } else if state.active_field == 5 {
                        state.active_field = 6;
                    }
                }
                // Hotkey 'b' / 'B' on presets or buttons triggers 1-click browser OAuth login
                KeyCode::Char('b') | KeyCode::Char('B')
                    if is_oauth && (state.active_field == 0 || state.active_field >= 5) =>
                {
                    app.trigger_oauth_login();
                }
                KeyCode::Enter => {
                    let is_cancel = (is_oauth && state.active_field == 7)
                        || (!is_oauth && state.active_field == 6);
                    let is_oauth_btn = is_oauth && state.active_field == 5;
                    let is_connect = (is_oauth && state.active_field == 6)
                        || (!is_oauth && (state.active_field == 5 || state.active_field == 4));

                    if is_cancel {
                        state.is_adding = false;
                    } else if is_oauth_btn {
                        app.trigger_oauth_login();
                    } else if is_connect {
                        if is_oauth && state.password.trim().is_empty() {
                            // Smart fallback: if OAuth provider and token is empty, trigger browser login
                            app.trigger_oauth_login();
                        } else if state.username.trim().is_empty() {
                            app.set_status(app.lang.msg_account_username_empty());
                        } else {
                            let (provider, auth_type) = match state.provider_idx {
                                0 => (ProviderType::Naver, crate::model::AuthType::Basic),
                                1 => (ProviderType::Apple, crate::model::AuthType::Basic),
                                2 => (ProviderType::Google, crate::model::AuthType::OAuth2),
                                3 => (ProviderType::Microsoft, crate::model::AuthType::OAuth2),
                                4 => (ProviderType::Samsung, crate::model::AuthType::Basic),
                                _ => (ProviderType::Local, crate::model::AuthType::Basic),
                            };
                            let mut new_acc = CalDavAccount::new(
                                state.name.clone(),
                                provider,
                                state.url.clone(),
                                state.username.clone(),
                                state.password.clone(),
                            );
                            new_acc.auth_type = auth_type;
                            if auth_type == crate::model::AuthType::OAuth2 {
                                new_acc.access_token = Some(state.password.clone());
                            }
                            acc_action_save = Some(new_acc);
                        }
                    } else {
                        // Advance to next field
                        state.active_field = (state.active_field + 1) % max_fields;
                    }
                }
                KeyCode::Char(' ') if state.active_field == 0 => {
                    let next_idx = (state.provider_idx + 1) % 6;
                    state.set_preset(next_idx);
                }
                KeyCode::Char(' ') if is_oauth && state.active_field == 5 => {
                    app.trigger_oauth_login();
                }
                KeyCode::Char(' ')
                    if (is_oauth && state.active_field == 6)
                        || (!is_oauth && state.active_field == 5) =>
                {
                    if is_oauth && state.password.trim().is_empty() {
                        app.trigger_oauth_login();
                    } else if !state.username.trim().is_empty() {
                        let (provider, auth_type) = match state.provider_idx {
                            0 => (ProviderType::Naver, crate::model::AuthType::Basic),
                            1 => (ProviderType::Apple, crate::model::AuthType::Basic),
                            2 => (ProviderType::Google, crate::model::AuthType::OAuth2),
                            3 => (ProviderType::Microsoft, crate::model::AuthType::OAuth2),
                            4 => (ProviderType::Samsung, crate::model::AuthType::Basic),
                            _ => (ProviderType::Local, crate::model::AuthType::Basic),
                        };
                        let mut new_acc = CalDavAccount::new(
                            state.name.clone(),
                            provider,
                            state.url.clone(),
                            state.username.clone(),
                            state.password.clone(),
                        );
                        new_acc.auth_type = auth_type;
                        if auth_type == crate::model::AuthType::OAuth2 {
                            new_acc.access_token = Some(state.password.clone());
                        }
                        acc_action_save = Some(new_acc);
                    }
                }
                KeyCode::Char(' ')
                    if (is_oauth && state.active_field == 7)
                        || (!is_oauth && state.active_field == 6) =>
                {
                    state.is_adding = false;
                }
                KeyCode::Backspace => {
                    state.error_msg = None;
                    match state.active_field {
                        1 => {
                            state.name.pop();
                        }
                        2 => {
                            state.url.pop();
                        }
                        3 => {
                            state.username.pop();
                        }
                        4 => {
                            state.password.pop();
                        }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
                    state.error_msg = None;
                    match state.active_field {
                        1 => state.name.push(c),
                        2 => state.url.push(c),
                        3 => state.username.push(c),
                        4 => state.password.push(c),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    if acc_action_close {
        app.close_modal();
        return Ok(());
    } else if acc_action_sync {
        app.trigger_sync();
        return Ok(());
    } else if let Some(idx) = acc_action_delete_idx {
        let accounts = app.db.get_accounts().unwrap_or_default();
        if let Some(acc) = accounts.get(idx) {
            app.request_delete_account(&acc.id);
        }
        return Ok(());
    } else if let Some(new_acc) = acc_action_save {
        let is_update = app
            .db
            .find_account_by_provider_and_username(&new_acc.provider, &new_acc.username)?
            .is_some();
        app.db.save_account(&new_acc)?;
        let status_msg = if is_update {
            app.lang.msg_account_updated(&new_acc.name)
        } else {
            app.lang.msg_account_added(&new_acc.name)
        };
        app.set_status(status_msg);
        app.trigger_sync();
        app.close_modal();
        return Ok(());
    } else if matches!(app.modal, ActiveModal::Accounts(_)) {
        return Ok(());
    }

    // 3. Handle CalendarManager modal
    let mut cal_action_close = false;
    let mut cal_action_save_visibility = false;
    let mut cal_action_delete_idx: Option<usize> = None;
    let mut cal_action_save: Option<(String, String)> = None;

    if let ActiveModal::CalendarManager(ref mut cal_state) = app.modal {
        if !cal_state.is_creating {
            let norm_code = normalize_key_event(key);
            match norm_code {
                KeyCode::Esc | KeyCode::Char('q') => cal_action_close = true,
                // Space: toggle checkbox for current calendar
                KeyCode::Char(' ') => {
                    if let Some(cal) = app.calendars.get(cal_state.selected_calendar_idx) {
                        cal_state.toggle_calendar_id(&cal.id, cal.enabled);
                    }
                }
                // Number 1-9: directly toggle calendar by 1-based index
                KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                    let num = (c as usize) - ('1' as usize);
                    if let Some(cal) = app.calendars.get(num) {
                        cal_state.toggle_calendar_id(&cal.id, cal.enabled);
                        cal_state.selected_calendar_idx = num;
                    }
                }
                // Save and apply visibility changes: Enter, 's', or 'S'
                KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S') => {
                    cal_action_save_visibility = true;
                }
                // Add new calendar: 'n', 'a', 'A', '+'
                KeyCode::Char('n')
                | KeyCode::Char('a')
                | KeyCode::Char('A')
                | KeyCode::Char('+') => {
                    cal_state.is_creating = true;
                    cal_state.active_field = 0;
                    cal_state.name.clear();
                }
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete | KeyCode::Backspace => {
                    cal_action_delete_idx = Some(cal_state.selected_calendar_idx);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let count = app.calendars.len();
                    if count > 0 {
                        cal_state.selected_calendar_idx =
                            (cal_state.selected_calendar_idx + 1) % count;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let count = app.calendars.len();
                    if count > 0 {
                        if cal_state.selected_calendar_idx == 0 {
                            cal_state.selected_calendar_idx = count - 1;
                        } else {
                            cal_state.selected_calendar_idx -= 1;
                        }
                    }
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Esc => cal_state.is_creating = false,
                KeyCode::Tab | KeyCode::Down => {
                    cal_state.active_field = (cal_state.active_field + 1) % 4;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    if cal_state.active_field == 0 {
                        cal_state.active_field = 3;
                    } else {
                        cal_state.active_field -= 1;
                    }
                }
                KeyCode::Left => {
                    if cal_state.active_field == 1 {
                        if cal_state.color_idx == 0 {
                            cal_state.color_idx = 6;
                        } else {
                            cal_state.color_idx -= 1;
                        }
                    } else if cal_state.active_field == 3 {
                        cal_state.active_field = 2;
                    }
                }
                KeyCode::Right => {
                    if cal_state.active_field == 1 {
                        cal_state.color_idx = (cal_state.color_idx + 1) % 7;
                    } else if cal_state.active_field == 2 {
                        cal_state.active_field = 3;
                    }
                }
                KeyCode::Char(' ') if cal_state.active_field == 1 => {
                    cal_state.color_idx = (cal_state.color_idx + 1) % 7;
                }
                KeyCode::Char(' ') if cal_state.active_field == 2 => {
                    if !cal_state.name.trim().is_empty() {
                        let color =
                            crate::app::CalendarModalState::PALETTE[cal_state.color_idx % 7];
                        cal_action_save = Some((cal_state.name.clone(), color.to_string()));
                    }
                }
                KeyCode::Char(' ') if cal_state.active_field == 3 => {
                    cal_state.is_creating = false;
                }
                KeyCode::Enter => {
                    if cal_state.active_field == 3 {
                        cal_state.is_creating = false;
                    } else if !cal_state.name.trim().is_empty() {
                        let color =
                            crate::app::CalendarModalState::PALETTE[cal_state.color_idx % 7];
                        cal_action_save = Some((cal_state.name.clone(), color.to_string()));
                    }
                }
                KeyCode::Backspace => {
                    if cal_state.active_field == 0 {
                        cal_state.name.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if cal_state.active_field == 0 {
                        cal_state.name.push(c);
                    }
                }
                _ => {}
            }
        }
    }

    if cal_action_save_visibility {
        app.save_calendar_visibility()?;
        return Ok(());
    } else if cal_action_close {
        app.close_modal();
        return Ok(());
    } else if let Some(idx) = cal_action_delete_idx {
        let target_cal = app.calendars.get(idx).cloned();
        if let Some(cal) = target_cal {
            if !cal.is_default {
                app.request_delete_calendar(&cal.id);
            }
        }
        return Ok(());
    } else if let Some((name, color)) = cal_action_save {
        let new_cal = crate::model::Calendar {
            id: format!("cal_custom_{}", uuid::Uuid::new_v4()),
            account_id: None,
            name: name.clone(),
            color_code: color,
            provider: ProviderType::Local,
            enabled: true,
            is_default: false,
            remote_url: None,
            ctag: None,
        };
        let _ = app.db.save_calendar(&new_cal);
        let _ = app.reload_calendars();
        app.set_status(format!("Created calendar '{}'", new_cal.name));
        app.close_modal();
        return Ok(());
    } else if matches!(app.modal, ActiveModal::CalendarManager(_)) {
        return Ok(());
    }

    // 4. Handle other modals
    match &mut app.modal {
        ActiveModal::EventDetail(ref id) => {
            let id_clone = id.clone();
            let norm_code = normalize_key_event(key);
            match norm_code {
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    app.open_edit_event_modal(&id_clone);
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    app.request_delete_event(&id_clone);
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    app.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }
        ActiveModal::Help => {
            let norm_code = normalize_key_event(key);
            if matches!(
                norm_code,
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter
            ) {
                app.close_modal();
            }
            return Ok(());
        }
        ActiveModal::Settings => {
            let norm_code = normalize_key_event(key);
            match norm_code {
                KeyCode::Esc | KeyCode::Char('q') => app.close_modal(),
                // Tab / Shift-Tab / [ / ]: Navigate between major tabs
                KeyCode::Tab | KeyCode::Char(']') => {
                    app.settings_section = app.settings_section.next();
                }
                KeyCode::BackTab | KeyCode::Char('[') => {
                    app.settings_section = app.settings_section.prev();
                }

                // Direct tab shortcuts:
                KeyCode::Char('1') => {
                    app.settings_section = SettingsSection::WeekStart;
                    app.set_week_start_day(crate::model::WeekStartDay::Sunday);
                }
                KeyCode::Char('2') => {
                    app.settings_section = SettingsSection::Holidays;
                }
                KeyCode::Char('3') => {
                    app.settings_section = SettingsSection::Notifications;
                }
                KeyCode::Char('4') => {
                    app.settings_section = SettingsSection::Theme;
                }
                KeyCode::Char('5') => {
                    app.settings_section = SettingsSection::Language;
                }

                // Global direct shortcuts in Settings modal:
                KeyCode::Char('H') => {
                    app.settings_section = SettingsSection::Holidays;
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    app.settings_section = SettingsSection::Notifications;
                    app.toggle_notifications();
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    app.settings_section = SettingsSection::Notifications;
                    app.send_test_notification();
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    app.settings_section = SettingsSection::Notifications;
                    app.toggle_notification_sound();
                }
                KeyCode::Char('T') => {
                    app.open_theme_modal();
                }
                KeyCode::Char('c') => {
                    app.settings_section = SettingsSection::Theme;
                    app.cycle_theme();
                }
                KeyCode::Char('C') | KeyCode::Char('p') | KeyCode::Char('P') => {
                    app.settings_section = SettingsSection::Theme;
                    app.cycle_theme_prev();
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    app.open_language_modal();
                }

                // Contextual keys based on current section
                _ => match app.settings_section {
                    SettingsSection::WeekStart => match norm_code {
                        KeyCode::Right
                        | KeyCode::Char('l')
                        | KeyCode::Down
                        | KeyCode::Char('j') => {
                            let next = match app.week_start_day {
                                crate::model::WeekStartDay::Sunday => {
                                    crate::model::WeekStartDay::Monday
                                }
                                crate::model::WeekStartDay::Monday => {
                                    crate::model::WeekStartDay::Saturday
                                }
                                crate::model::WeekStartDay::Saturday => {
                                    crate::model::WeekStartDay::Sunday
                                }
                            };
                            app.set_week_start_day(next);
                        }
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Up | KeyCode::Char('k') => {
                            let prev = match app.week_start_day {
                                crate::model::WeekStartDay::Sunday => {
                                    crate::model::WeekStartDay::Saturday
                                }
                                crate::model::WeekStartDay::Monday => {
                                    crate::model::WeekStartDay::Sunday
                                }
                                crate::model::WeekStartDay::Saturday => {
                                    crate::model::WeekStartDay::Monday
                                }
                            };
                            app.set_week_start_day(prev);
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            app.set_week_start_day(crate::model::WeekStartDay::Sunday);
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            app.set_week_start_day(crate::model::WeekStartDay::Monday);
                        }
                        KeyCode::Char('t') => {
                            app.set_week_start_day(crate::model::WeekStartDay::Saturday);
                        }
                        KeyCode::Enter => {
                            app.close_modal();
                        }
                        _ => {}
                    },
                    SettingsSection::Holidays => match norm_code {
                        KeyCode::Right | KeyCode::Down => {
                            let all = crate::holidays::HolidayRegion::ALL;
                            let cur_pos = all.iter().position(|r| *r == app.holiday_region).unwrap_or(0);
                            let next_pos = (cur_pos + 1) % all.len();
                            let _ = app.set_holiday_region(all[next_pos]);
                        }
                        KeyCode::Left | KeyCode::Up => {
                            let all = crate::holidays::HolidayRegion::ALL;
                            let cur_pos = all.iter().position(|r| *r == app.holiday_region).unwrap_or(0);
                            let prev_pos = if cur_pos == 0 { all.len() - 1 } else { cur_pos - 1 };
                            let _ = app.set_holiday_region(all[prev_pos]);
                        }
                        KeyCode::Char('k') | KeyCode::Char('K') => {
                            let _ = app.set_holiday_region(crate::holidays::HolidayRegion::Kr);
                        }
                        KeyCode::Char('u') | KeyCode::Char('U') => {
                            let _ = app.set_holiday_region(crate::holidays::HolidayRegion::Us);
                        }
                        KeyCode::Char('j') | KeyCode::Char('J') => {
                            let _ = app.set_holiday_region(crate::holidays::HolidayRegion::Jp);
                        }
                        KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Char('0') => {
                            let _ = app.set_holiday_region(crate::holidays::HolidayRegion::None);
                        }
                        KeyCode::Enter => {
                            app.close_modal();
                        }
                        _ => {}
                    },
                    SettingsSection::Notifications => match norm_code {
                        KeyCode::Left | KeyCode::Char('h') => {
                            let prev_mins = match app.notification_config.lead_minutes {
                                30 => 15,
                                15 => 10,
                                10 => 5,
                                5 => 0,
                                _ => 30,
                            };
                            app.set_notification_lead_minutes(prev_mins);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            let next_mins = match app.notification_config.lead_minutes {
                                0 => 5,
                                5 => 10,
                                10 => 15,
                                15 => 30,
                                _ => 0,
                            };
                            app.set_notification_lead_minutes(next_mins);
                        }
                        KeyCode::Char(' ')
                        | KeyCode::Down
                        | KeyCode::Char('j')
                        | KeyCode::Up
                        | KeyCode::Char('k') => {
                            app.toggle_notifications();
                        }
                        KeyCode::Enter => {
                            app.close_modal();
                        }
                        _ => {}
                    },
                    SettingsSection::Theme => match norm_code {
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.cycle_theme();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.cycle_theme_prev();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let next_cat = match app.theme_category {
                                ThemeCategory::Dark => ThemeCategory::Nature,
                                ThemeCategory::Nature => ThemeCategory::Vibrant,
                                ThemeCategory::Vibrant => ThemeCategory::Light,
                                ThemeCategory::Light => ThemeCategory::Dark,
                            };
                            app.set_theme_category(next_cat);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let prev_cat = match app.theme_category {
                                ThemeCategory::Dark => ThemeCategory::Light,
                                ThemeCategory::Nature => ThemeCategory::Dark,
                                ThemeCategory::Vibrant => ThemeCategory::Nature,
                                ThemeCategory::Light => ThemeCategory::Vibrant,
                            };
                            app.set_theme_category(prev_cat);
                        }
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            app.open_theme_modal();
                        }
                        _ => {}
                    },
                    SettingsSection::Language => {
                        let all_len = crate::i18n::Language::ALL.len();
                        let num_cols = 4;
                        match norm_code {
                            KeyCode::Left | KeyCode::Char('h') => {
                                if app.settings_lang_idx > 0 {
                                    app.settings_lang_idx -= 1;
                                } else {
                                    app.settings_lang_idx = all_len - 1;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                if app.settings_lang_idx + 1 < all_len {
                                    app.settings_lang_idx += 1;
                                } else {
                                    app.settings_lang_idx = 0;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.settings_lang_idx >= num_cols {
                                    app.settings_lang_idx -= num_cols;
                                } else {
                                    let target =
                                        app.settings_lang_idx + (all_len / num_cols) * num_cols;
                                    app.settings_lang_idx = if target < all_len {
                                        target
                                    } else {
                                        target - num_cols
                                    };
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.settings_lang_idx + num_cols < all_len {
                                    app.settings_lang_idx += num_cols;
                                } else {
                                    app.settings_lang_idx = app.settings_lang_idx % num_cols;
                                }
                            }
                            KeyCode::Char(' ') | KeyCode::Enter => {
                                let selected_lang =
                                    crate::i18n::Language::ALL[app.settings_lang_idx];
                                app.set_language(selected_lang);
                            }
                            _ => {}
                        }
                    }
                    SettingsSection::Close => match norm_code {
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            app.close_modal();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.settings_section = SettingsSection::Language;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.settings_section = SettingsSection::WeekStart;
                        }
                        _ => {}
                    },
                },
            }
            return Ok(());
        }
        ActiveModal::Language(ref mut state) => {
            let norm_code = normalize_key_event(key);
            let all_len = crate::i18n::Language::ALL.len();
            match norm_code {
                KeyCode::Esc | KeyCode::Char('q') => app.close_modal(),
                KeyCode::Left | KeyCode::Char('h') => {
                    if state.selected_idx > 0 {
                        state.selected_idx -= 1;
                    } else {
                        state.selected_idx = all_len - 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                    if state.selected_idx + 1 < all_len {
                        state.selected_idx += 1;
                    } else {
                        state.selected_idx = 0;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.selected_idx >= 3 {
                        state.selected_idx -= 3;
                    } else if state.selected_idx + 18 < all_len {
                        state.selected_idx += 18;
                    } else if state.selected_idx + 15 < all_len {
                        state.selected_idx += 15;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.selected_idx + 3 < all_len {
                        state.selected_idx += 3;
                    } else {
                        state.selected_idx = state.selected_idx % 3;
                    }
                }
                KeyCode::Tab => {
                    state.selected_idx = (state.selected_idx + 1) % all_len;
                }
                KeyCode::BackTab => {
                    if state.selected_idx == 0 {
                        state.selected_idx = all_len - 1;
                    } else {
                        state.selected_idx -= 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let selected_lang = crate::i18n::Language::ALL[state.selected_idx];
                    app.set_language(selected_lang);
                    app.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }
        ActiveModal::Theme(ref mut state) => {
            let norm_code = normalize_key_event(key);
            let mut selection_changed = false;

            match norm_code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    let orig = state.original_mode;
                    app.set_theme(orig);
                    app.close_modal();
                    return Ok(());
                }
                KeyCode::Char('0') | KeyCode::Char('a') | KeyCode::Char('A') => {
                    state.category_filter = None;
                    let filtered = state.filtered_themes();
                    state.selected_idx = filtered
                        .iter()
                        .position(|m| *m == app.theme.mode)
                        .unwrap_or(0);
                    selection_changed = true;
                }
                KeyCode::Char('1') => {
                    state.category_filter = Some(crate::ui::theme::ThemeCategory::Dark);
                    let filtered = state.filtered_themes();
                    state.selected_idx = filtered
                        .iter()
                        .position(|m| *m == app.theme.mode)
                        .unwrap_or(0);
                    selection_changed = true;
                }
                KeyCode::Char('2') => {
                    state.category_filter = Some(crate::ui::theme::ThemeCategory::Nature);
                    let filtered = state.filtered_themes();
                    state.selected_idx = filtered
                        .iter()
                        .position(|m| *m == app.theme.mode)
                        .unwrap_or(0);
                    selection_changed = true;
                }
                KeyCode::Char('3') => {
                    state.category_filter = Some(crate::ui::theme::ThemeCategory::Vibrant);
                    let filtered = state.filtered_themes();
                    state.selected_idx = filtered
                        .iter()
                        .position(|m| *m == app.theme.mode)
                        .unwrap_or(0);
                    selection_changed = true;
                }
                KeyCode::Char('4') => {
                    state.category_filter = Some(crate::ui::theme::ThemeCategory::Light);
                    let filtered = state.filtered_themes();
                    state.selected_idx = filtered
                        .iter()
                        .position(|m| *m == app.theme.mode)
                        .unwrap_or(0);
                    selection_changed = true;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    let count = state.filtered_themes().len();
                    if count > 0 {
                        if state.selected_idx > 0 {
                            state.selected_idx -= 1;
                        } else {
                            state.selected_idx = count - 1;
                        }
                        selection_changed = true;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                    let count = state.filtered_themes().len();
                    if count > 0 {
                        if state.selected_idx + 1 < count {
                            state.selected_idx += 1;
                        } else {
                            state.selected_idx = 0;
                        }
                        selection_changed = true;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let count = state.filtered_themes().len();
                    if count > 0 && state.selected_idx >= 3 {
                        state.selected_idx -= 3;
                        selection_changed = true;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let count = state.filtered_themes().len();
                    if count > 0 && state.selected_idx + 3 < count {
                        state.selected_idx += 3;
                        selection_changed = true;
                    }
                }
                KeyCode::Tab => {
                    let count = state.filtered_themes().len();
                    if count > 0 {
                        state.selected_idx = (state.selected_idx + 1) % count;
                        selection_changed = true;
                    }
                }
                KeyCode::BackTab => {
                    let count = state.filtered_themes().len();
                    if count > 0 {
                        if state.selected_idx == 0 {
                            state.selected_idx = count - 1;
                        } else {
                            state.selected_idx -= 1;
                        }
                        selection_changed = true;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let filtered = state.filtered_themes();
                    if let Some(mode) = filtered.get(state.selected_idx) {
                        app.set_theme(*mode);
                    }
                    app.close_modal();
                    return Ok(());
                }
                _ => {}
            }

            if selection_changed {
                let filtered = state.filtered_themes();
                if let Some(mode) = filtered.get(state.selected_idx) {
                    app.set_theme(*mode);
                }
            }
            return Ok(());
        }
        ActiveModal::ConfirmDelete(_) => {
            let norm_code = normalize_key_event(key);
            if matches!(
                norm_code,
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter
            ) {
                app.confirm_delete()?;
            } else if matches!(
                norm_code,
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q')
            ) {
                app.cancel_delete();
            }
            return Ok(());
        }
        ActiveModal::EventForm(_) | ActiveModal::Accounts(_) | ActiveModal::CalendarManager(_) => {
            return Ok(())
        }
        ActiveModal::None => {}
    }

    // 4. Normal Mode Navigation & Shortcuts
    let norm_code = normalize_key_event(key);
    match norm_code {
        // App termination
        KeyCode::Char('q') | KeyCode::Char('Q') => app.running = false,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.running = false,

        // View switching
        KeyCode::Char('1') | KeyCode::Char('m') | KeyCode::Char('M') => {
            app.current_view = ViewMode::Month
        }
        KeyCode::Char('2') | KeyCode::Char('w') | KeyCode::Char('W') => {
            app.current_view = ViewMode::Week
        }
        KeyCode::Char('3') | KeyCode::Char('d') | KeyCode::Char('D') => {
            app.current_view = ViewMode::Day
        }
        KeyCode::Char('4') | KeyCode::Char('a') => app.current_view = ViewMode::Agenda,

        // Navigation
        KeyCode::Char('h') | KeyCode::Left => {
            app.selected_date = app.selected_date - Duration::days(1);
            app.day_selected_event_idx = None;
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.selected_date = app.selected_date + Duration::days(1);
            app.day_selected_event_idx = None;
        }
        KeyCode::Char('k') | KeyCode::Up => match app.current_view {
            ViewMode::Month | ViewMode::Week => {
                app.selected_date = app.selected_date - Duration::days(7);
            }
            ViewMode::Day => {
                if app.selected_hour > 0 {
                    app.selected_hour -= 1;
                }
            }
            ViewMode::Agenda => {
                if app.agenda_selected_index > 0 {
                    app.agenda_selected_index -= 1;
                }
            }
        },
        KeyCode::Char('j') | KeyCode::Down => match app.current_view {
            ViewMode::Month | ViewMode::Week => {
                app.selected_date = app.selected_date + Duration::days(7);
            }
            ViewMode::Day => {
                if app.selected_hour < 23 {
                    app.selected_hour += 1;
                }
            }
            ViewMode::Agenda => {
                app.agenda_selected_index += 1;
            }
        },
        KeyCode::Char('[') => app.nav_prev(),
        KeyCode::Char(']') => app.nav_next(),
        KeyCode::Char('t') => app.nav_today(),

        // Actions
        KeyCode::Char('b') | KeyCode::Char('B') => app.toggle_sidebar(),
        KeyCode::Char('c') | KeyCode::Char('n') => app.open_new_event_modal(None),
        KeyCode::Char('C') | KeyCode::Char('v') | KeyCode::Char('V') => app.open_calendar_modal(),
        KeyCode::Char('r') | KeyCode::Char('R') => app.trigger_sync(),
        KeyCode::Char('A') => app.open_accounts_modal(),
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('o') | KeyCode::Char('O') => {
            app.open_settings_modal()
        }
        KeyCode::Char('L') => app.open_language_modal(),
        KeyCode::Char('T') => app.open_theme_modal(),
        KeyCode::Char('?') => app.open_help_modal(),

        // Selection / Cycling in Day View
        KeyCode::Tab => {
            if app.current_view == ViewMode::Day {
                let day_events = app.events_for_day(app.selected_date);
                if !day_events.is_empty() {
                    let next_idx = match app.day_selected_event_idx {
                        Some(idx) => (idx + 1) % day_events.len(),
                        None => 0,
                    };
                    app.day_selected_event_idx = Some(next_idx);
                }
            }
        }
        KeyCode::BackTab => {
            if app.current_view == ViewMode::Day {
                let day_events = app.events_for_day(app.selected_date);
                if !day_events.is_empty() {
                    let prev_idx = match app.day_selected_event_idx {
                        Some(0) | None => day_events.len() - 1,
                        Some(idx) => idx - 1,
                    };
                    app.day_selected_event_idx = Some(prev_idx);
                }
            }
        }

        // Inspect/Select
        KeyCode::Enter => {
            if app.current_view == ViewMode::Month {
                app.open_new_event_modal(None);
            } else if app.current_view == ViewMode::Day {
                let day_events = app.events_for_day(app.selected_date);
                if let Some(idx) = app.day_selected_event_idx {
                    if let Some(ev) = day_events.get(idx) {
                        app.open_event_detail_modal(ev.id.clone());
                    }
                } else {
                    // If no event selected with Tab, check if selected_hour has any events
                    if let Some(ev) = day_events
                        .iter()
                        .find(|e| !e.all_day && e.start_time.hour() == app.selected_hour)
                    {
                        app.open_event_detail_modal(ev.id.clone());
                    }
                }
            } else if app.current_view == ViewMode::Agenda {
                let mut sorted: Vec<&_> = app.events_cache.iter().collect();
                sorted.sort_by_key(|e| e.start_time);
                if let Some(ev) = sorted.get(app.agenda_selected_index) {
                    app.open_event_detail_modal(ev.id.clone());
                }
            }
        }

        _ => {}
    }

    Ok(())
}
