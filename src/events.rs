use anyhow::Result;
use chrono::Duration;
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::app::{ActiveModal, App, EventFormState, ViewMode};
use crate::model::{CalDavAccount, ProviderType};

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
                form.active_field = (form.active_field + 1) % (EventFormState::TOTAL_FIELDS + 1);
            }
            KeyCode::BackTab | KeyCode::Up => {
                if form.active_field == 0 {
                    form.active_field = EventFormState::TOTAL_FIELDS;
                } else {
                    form.active_field -= 1;
                }
            }
            KeyCode::Enter => {
                form_action_save = Some(form.clone());
            }
            KeyCode::Char(' ') if form.active_field == 1 => {
                let current_idx = app.calendars.iter().position(|c| c.id == form.calendar_id).unwrap_or(0);
                let next_idx = (current_idx + 1) % app.calendars.len();
                if let Some(next_cal) = app.calendars.get(next_idx) {
                    form.calendar_id = next_cal.id.clone();
                }
            }
            KeyCode::Char(' ') if form.active_field == 5 => {
                form.all_day = !form.all_day;
            }
            KeyCode::Backspace => {
                match form.active_field {
                    0 => { form.title.pop(); }
                    2 => { form.date_str.pop(); }
                    3 => { form.start_time_str.pop(); }
                    4 => { form.end_time_str.pop(); }
                    6 => { form.location.pop(); }
                    7 => { form.description.pop(); }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match form.active_field {
                    0 => form.title.push(c),
                    2 => form.date_str.push(c),
                    3 => form.start_time_str.push(c),
                    4 => form.end_time_str.push(c),
                    6 => form.location.push(c),
                    7 => form.description.push(c),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    if form_action_close {
        app.close_modal();
        return Ok(());
    } else if let Some(form) = form_action_save {
        app.save_event_from_form(form)?;
        return Ok(());
    }

    // 2. Handle Accounts modal
    let mut acc_action_close = false;
    let mut acc_action_sync = false;
    let mut acc_action_save: Option<CalDavAccount> = None;

    if let ActiveModal::Accounts(ref mut state) = app.modal {
        if !state.is_adding {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => acc_action_close = true,
                KeyCode::Char('a') | KeyCode::Char('n') | KeyCode::Enter => state.is_adding = true,
                KeyCode::Char('r') => acc_action_sync = true,
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Esc => state.is_adding = false,
                KeyCode::Tab | KeyCode::Down => {
                    state.active_field = (state.active_field + 1) % 5;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    if state.active_field == 0 {
                        state.active_field = 4;
                    } else {
                        state.active_field -= 1;
                    }
                }
                KeyCode::Enter => {
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
                KeyCode::Char(' ') if state.active_field == 0 => {
                    let next_idx = (state.provider_idx + 1) % 6;
                    state.set_preset(next_idx);
                }
                KeyCode::Backspace => {
                    match state.active_field {
                        1 => { state.name.pop(); }
                        2 => { state.url.pop(); }
                        3 => { state.username.pop(); }
                        4 => { state.password.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
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
    } else if let Some(new_acc) = acc_action_save {
        app.db.save_account(&new_acc)?;
        app.set_status(format!("Added account '{}'", new_acc.name));
        app.trigger_sync();
        app.close_modal();
        return Ok(());
    }

    // 3. Handle Other modals
    match &mut app.modal {
        ActiveModal::CalendarManager(ref mut cal_state) => {
            if !cal_state.is_creating {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => app.close_modal(),
                    KeyCode::Char('n') | KeyCode::Char('c') | KeyCode::Enter => cal_state.is_creating = true,
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Esc => cal_state.is_creating = false,
                    KeyCode::Tab | KeyCode::Char(' ') => {
                        cal_state.color_idx = (cal_state.color_idx + 1) % 7;
                    }
                    KeyCode::Enter => {
                        if !cal_state.name.trim().is_empty() {
                            let color = crate::app::CalendarModalState::PALETTE[cal_state.color_idx % 7];
                            let new_cal = crate::model::Calendar {
                                id: format!("cal_custom_{}", uuid::Uuid::new_v4()),
                                account_id: None,
                                name: cal_state.name.clone(),
                                color_code: color.to_string(),
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
                        }
                    }
                    KeyCode::Backspace => {
                        cal_state.name.pop();
                    }
                    KeyCode::Char(c) => {
                        cal_state.name.push(c);
                    }
                    _ => {}
                }
            }
            return Ok(());
        }
        ActiveModal::EventDetail(ref id) => {
            let id_clone = id.clone();
            match key.code {
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    app.open_edit_event_modal(&id_clone);
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    app.delete_event(&id_clone)?;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }
        ActiveModal::Help => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter) {
                app.close_modal();
            }
            return Ok(());
        }
        ActiveModal::Settings => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => app.close_modal(),
                KeyCode::Char('1') | KeyCode::Char('s') => {
                    app.set_week_start_day(crate::model::WeekStartDay::Sunday);
                }
                KeyCode::Char('2') | KeyCode::Char('m') | KeyCode::Char('M') => {
                    app.set_week_start_day(crate::model::WeekStartDay::Monday);
                }
                KeyCode::Char('3') | KeyCode::Char('t') | KeyCode::Char('T') => {
                    app.set_week_start_day(crate::model::WeekStartDay::Saturday);
                }
                KeyCode::Tab | KeyCode::Right | KeyCode::Down => {
                    let next = match app.week_start_day {
                        crate::model::WeekStartDay::Sunday => crate::model::WeekStartDay::Monday,
                        crate::model::WeekStartDay::Monday => crate::model::WeekStartDay::Saturday,
                        crate::model::WeekStartDay::Saturday => crate::model::WeekStartDay::Sunday,
                    };
                    app.set_week_start_day(next);
                }
                KeyCode::BackTab | KeyCode::Left | KeyCode::Up => {
                    let prev = match app.week_start_day {
                        crate::model::WeekStartDay::Sunday => crate::model::WeekStartDay::Saturday,
                        crate::model::WeekStartDay::Monday => crate::model::WeekStartDay::Sunday,
                        crate::model::WeekStartDay::Saturday => crate::model::WeekStartDay::Monday,
                    };
                    app.set_week_start_day(prev);
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    app.toggle_language();
                }
                KeyCode::Enter => {
                    app.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }
        ActiveModal::ConfirmDelete(ref id) => {
            let id_clone = id.clone();
            if matches!(key.code, KeyCode::Char('y') | KeyCode::Enter) {
                app.delete_event(&id_clone)?;
            } else if matches!(key.code, KeyCode::Char('n') | KeyCode::Esc) {
                app.close_modal();
            }
            return Ok(());
        }
        ActiveModal::EventForm(_) | ActiveModal::Accounts(_) => return Ok(()),
        ActiveModal::None => {}
    }

    // 4. Normal Mode Navigation & Shortcuts
    match key.code {
        // App termination
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.running = false,

        // View switching
        KeyCode::Char('1') | KeyCode::Char('m') => app.current_view = ViewMode::Month,
        KeyCode::Char('2') | KeyCode::Char('w') => app.current_view = ViewMode::Week,
        KeyCode::Char('3') | KeyCode::Char('d') => app.current_view = ViewMode::Day,
        KeyCode::Char('4') | KeyCode::Char('a') => app.current_view = ViewMode::Agenda,

        // Navigation
        KeyCode::Char('h') | KeyCode::Left => {
            app.selected_date = app.selected_date - Duration::days(1);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.selected_date = app.selected_date + Duration::days(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            match app.current_view {
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
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            match app.current_view {
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
            }
        }
        KeyCode::Char('[') => app.nav_prev(),
        KeyCode::Char(']') => app.nav_next(),
        KeyCode::Char('t') => app.nav_today(),

        // Actions
        KeyCode::Char('c') | KeyCode::Char('n') => app.open_new_event_modal(None),
        KeyCode::Char('C') => app.open_calendar_modal(),
        KeyCode::Char('r') => app.trigger_sync(),
        KeyCode::Char('A') => app.open_accounts_modal(),
        KeyCode::Char('o') | KeyCode::Char('O') | KeyCode::Char('S') => app.open_settings_modal(),
        KeyCode::Char('L') => app.toggle_language(),
        KeyCode::Char('?') => app.open_help_modal(),

        // Inspect/Select
        KeyCode::Enter => {
            if app.current_view == ViewMode::Month {
                app.current_view = ViewMode::Day;
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
