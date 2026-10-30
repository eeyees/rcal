use std::time::{Duration, Instant};
use anyhow::Result;
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveTime, TimeZone};

use crate::i18n::Language;
use crate::model::{CalDavAccount, Calendar, Event, ProviderType, SyncStatus};
use crate::mouse::{HitMap, HitTarget};
use crate::storage::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Month,
    Week,
    Day,
    Agenda,
}

impl ViewMode {
    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            ViewMode::Month => "Month",
            ViewMode::Week => "Week",
            ViewMode::Day => "Day",
            ViewMode::Agenda => "Agenda",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventFormState {
    pub is_new: bool,
    pub event_id: Option<String>,
    pub calendar_id: String,
    pub title: String,
    pub date_str: String,
    pub start_time_str: String,
    pub end_time_str: String,
    pub all_day: bool,
    pub location: String,
    pub description: String,
    pub active_field: usize, // 0..=7
}

impl EventFormState {
    pub const TOTAL_FIELDS: usize = 7;

    pub fn new_with_date(date: NaiveDate, default_cal_id: String, default_hour: Option<u32>) -> Self {
        let hour = default_hour.unwrap_or(9);
        Self {
            is_new: true,
            event_id: None,
            calendar_id: default_cal_id,
            title: String::new(),
            date_str: date.format("%Y-%m-%d").to_string(),
            start_time_str: format!("{:02}:00", hour),
            end_time_str: format!("{:02}:00", (hour + 1).min(23)),
            all_day: false,
            location: String::new(),
            description: String::new(),
            active_field: 0,
        }
    }

    pub fn from_event(ev: &Event) -> Self {
        Self {
            is_new: false,
            event_id: Some(ev.id.clone()),
            calendar_id: ev.calendar_id.clone(),
            title: ev.title.clone(),
            date_str: ev.start_time.format("%Y-%m-%d").to_string(),
            start_time_str: ev.start_time.format("%H:%M").to_string(),
            end_time_str: ev.end_time.format("%H:%M").to_string(),
            all_day: ev.all_day,
            location: ev.location.clone().unwrap_or_default(),
            description: ev.description.clone().unwrap_or_default(),
            active_field: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountModalState {
    pub is_adding: bool,
    pub provider_idx: usize, // 0: Naver, 1: Apple iCloud, 2: CalDAV
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub active_field: usize, // 0..=4
    pub selected_account_idx: usize,
    pub error_msg: Option<String>,
}

impl AccountModalState {
    pub fn new() -> Self {
        Self {
            is_adding: false,
            provider_idx: 0,
            name: "Naver Calendar".to_string(),
            url: "https://caldav.calendar.naver.com/".to_string(),
            username: String::new(),
            password: String::new(),
            active_field: 0,
            selected_account_idx: 0,
            error_msg: None,
        }
    }

    pub fn set_preset(&mut self, idx: usize) {
        self.provider_idx = idx % 6;
        match self.provider_idx {
            0 => {
                self.name = "Naver Calendar".to_string();
                self.url = "https://caldav.calendar.naver.com/".to_string();
            }
            1 => {
                self.name = "Apple iCloud".to_string();
                self.url = "https://caldav.icloud.com/".to_string();
            }
            2 => {
                self.name = "Google Calendar".to_string();
                self.url = "https://www.googleapis.com/calendar/v3".to_string();
            }
            3 => {
                self.name = "Microsoft 365".to_string();
                self.url = "https://graph.microsoft.com/v1.0".to_string();
            }
            4 => {
                self.name = "Samsung Calendar".to_string();
                self.url = "https://caldav.samsung.com/".to_string();
            }
            _ => {
                self.name = "CalDAV Server".to_string();
                self.url = "https://".to_string();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActiveModal {
    None,
    EventForm(EventFormState),
    EventDetail(String /* event_id */),
    Accounts(AccountModalState),
    Help,
    #[allow(dead_code)]
    ConfirmDelete(String /* event_id */),
}

pub struct App {
    pub running: bool,
    pub current_view: ViewMode,
    pub selected_date: NaiveDate,
    pub selected_hour: u32,
    pub agenda_selected_index: usize,

    pub db: Database,
    pub calendars: Vec<Calendar>,
    pub events_cache: Vec<Event>,

    pub modal: ActiveModal,
    pub hit_map: HitMap,
    pub status_message: Option<(String, Instant)>,
    pub lang: Language,

    // Synchronization state
    pub is_syncing: bool,
    pub last_sync_status: Option<String>,
    pub sync_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl App {
    pub fn new(db: Database) -> Result<Self> {
        let today = Local::now().date_naive();
        let lang = Language::detect();
        let mut app = Self {
            running: true,
            current_view: ViewMode::Month,
            selected_date: today,
            selected_hour: 9,
            agenda_selected_index: 0,
            db,
            calendars: Vec::new(),
            events_cache: Vec::new(),
            modal: ActiveModal::None,
            hit_map: HitMap::new(),
            status_message: Some((lang.msg_welcome().to_string(), Instant::now())),
            lang,
            is_syncing: false,
            last_sync_status: None,
            sync_tx: None,
        };

        app.reload_calendars()?;
        app.reload_events()?;
        Ok(app)
    }

    pub fn reload_calendars(&mut self) -> Result<()> {
        self.calendars = self.db.get_calendars()?;
        Ok(())
    }

    pub fn reload_events(&mut self) -> Result<()> {
        self.events_cache = self.db.get_all_events()?;
        Ok(())
    }

    pub fn default_calendar_id(&self) -> String {
        self.calendars
            .iter()
            .find(|c| c.is_default)
            .map(|c| c.id.clone())
            .unwrap_or_else(|| {
                self.calendars
                    .first()
                    .map(|c| c.id.clone())
                    .unwrap_or_else(|| "cal_personal".to_string())
            })
    }

    pub fn toggle_language(&mut self) {
        self.lang = self.lang.toggle();
        self.set_status(self.lang.msg_lang_changed());
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn get_active_status(&self) -> Option<&str> {
        if let Some((msg, time)) = &self.status_message {
            if time.elapsed() < Duration::from_secs(5) {
                return Some(msg.as_str());
            }
        }
        None
    }

    // Navigation methods
    pub fn nav_prev(&mut self) {
        match self.current_view {
            ViewMode::Month => {
                // Go to previous month
                let (year, month) = if self.selected_date.month() == 1 {
                    (self.selected_date.year() - 1, 12)
                } else {
                    (self.selected_date.year(), self.selected_date.month() - 1)
                };
                let day = self.selected_date.day().min(days_in_month(year, month));
                if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
                    self.selected_date = d;
                }
            }
            ViewMode::Week => {
                self.selected_date = self.selected_date - ChronoDuration::days(7);
            }
            ViewMode::Day => {
                self.selected_date = self.selected_date - ChronoDuration::days(1);
            }
            ViewMode::Agenda => {
                if self.agenda_selected_index > 0 {
                    self.agenda_selected_index -= 1;
                }
            }
        }
    }

    pub fn nav_next(&mut self) {
        match self.current_view {
            ViewMode::Month => {
                // Go to next month
                let (year, month) = if self.selected_date.month() == 12 {
                    (self.selected_date.year() + 1, 1)
                } else {
                    (self.selected_date.year(), self.selected_date.month() + 1)
                };
                let day = self.selected_date.day().min(days_in_month(year, month));
                if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
                    self.selected_date = d;
                }
            }
            ViewMode::Week => {
                self.selected_date = self.selected_date + ChronoDuration::days(7);
            }
            ViewMode::Day => {
                self.selected_date = self.selected_date + ChronoDuration::days(1);
            }
            ViewMode::Agenda => {
                self.agenda_selected_index += 1;
            }
        }
    }

    pub fn nav_today(&mut self) {
        self.selected_date = Local::now().date_naive();
        let formatted = self.lang.format_month_year(self.selected_date);
        self.set_status(self.lang.msg_jump_today(&formatted));
    }

    // Modal triggers
    pub fn open_new_event_modal(&mut self, default_hour: Option<u32>) {
        let cal_id = self.default_calendar_id();
        self.modal = ActiveModal::EventForm(EventFormState::new_with_date(
            self.selected_date,
            cal_id,
            default_hour,
        ));
    }

    pub fn open_edit_event_modal(&mut self, event_id: &str) {
        if let Ok(Some(ev)) = self.db.get_event_by_id(event_id) {
            self.modal = ActiveModal::EventForm(EventFormState::from_event(&ev));
        }
    }

    pub fn open_event_detail_modal(&mut self, event_id: String) {
        self.modal = ActiveModal::EventDetail(event_id);
    }

    pub fn open_help_modal(&mut self) {
        self.modal = ActiveModal::Help;
    }

    pub fn open_accounts_modal(&mut self) {
        self.modal = ActiveModal::Accounts(AccountModalState::new());
    }

    pub fn trigger_sync(&mut self) {
        if self.is_syncing {
            return;
        }
        self.is_syncing = true;
        self.set_status("⟳ Synchronizing with CalDAV servers...");
        if let Some(ref tx) = self.sync_tx {
            let _ = tx.try_send(());
        }
    }

    pub fn close_modal(&mut self) {
        self.modal = ActiveModal::None;
    }

    // Save event from form state
    pub fn save_event_from_form(&mut self, form: EventFormState) -> Result<()> {
        let date = NaiveDate::parse_from_str(&form.date_str, "%Y-%m-%d")
            .unwrap_or(self.selected_date);

        let (start_time, end_time) = if form.all_day {
            let start = Local.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).unwrap();
            let end = Local.from_local_datetime(&date.and_hms_opt(23, 59, 59).unwrap()).unwrap();
            (start, end)
        } else {
            let start_time_part = NaiveTime::parse_from_str(&form.start_time_str, "%H:%M")
                .unwrap_or_else(|_| NaiveTime::from_hms_opt(9, 0, 0).unwrap());
            let end_time_part = NaiveTime::parse_from_str(&form.end_time_str, "%H:%M")
                .unwrap_or_else(|_| NaiveTime::from_hms_opt(10, 0, 0).unwrap());

            let start = Local.from_local_datetime(&date.and_time(start_time_part)).unwrap();
            let end = Local.from_local_datetime(&date.and_time(end_time_part)).unwrap();
            (start, end)
        };

        let event = if let Some(id) = form.event_id {
            // Updating existing event
            let mut ev = self.db.get_event_by_id(&id)?.unwrap_or_else(|| {
                Event::new(form.calendar_id.clone(), form.title.clone(), start_time, end_time, form.all_day)
            });
            ev.calendar_id = form.calendar_id;
            ev.title = if form.title.trim().is_empty() { "(No title)".to_string() } else { form.title };
            ev.description = if form.description.trim().is_empty() { None } else { Some(form.description) };
            ev.location = if form.location.trim().is_empty() { None } else { Some(form.location) };
            ev.start_time = start_time;
            ev.end_time = end_time;
            ev.all_day = form.all_day;
            ev.sync_status = SyncStatus::UpdatedLocally;
            ev.updated_at = Local::now();
            ev
        } else {
            // New event
            let mut ev = Event::new(
                form.calendar_id,
                if form.title.trim().is_empty() { "(No title)".to_string() } else { form.title },
                start_time,
                end_time,
                form.all_day,
            );
            ev.description = if form.description.trim().is_empty() { None } else { Some(form.description) };
            ev.location = if form.location.trim().is_empty() { None } else { Some(form.location) };
            ev
        };

        self.db.save_event(&event)?;
        self.reload_events()?;
        self.close_modal();
        self.set_status(self.lang.msg_saved());
        Ok(())
    }

    pub fn delete_event(&mut self, event_id: &str) -> Result<()> {
        self.db.delete_event(event_id)?;
        self.reload_events()?;
        self.close_modal();
        self.set_status(self.lang.msg_deleted());
        Ok(())
    }

    pub fn toggle_calendar(&mut self, cal_id: &str) -> Result<()> {
        self.db.toggle_calendar(cal_id)?;
        self.reload_calendars()?;
        self.reload_events()?;
        self.set_status("Calendar visibility toggled");
        Ok(())
    }

    // Process mouse click
    pub fn handle_mouse_click(&mut self, x: u16, y: u16) -> Result<()> {
        if let Some(target) = self.hit_map.check(x, y) {
            match target {
                HitTarget::TabMonth => self.current_view = ViewMode::Month,
                HitTarget::TabWeek => self.current_view = ViewMode::Week,
                HitTarget::TabDay => self.current_view = ViewMode::Day,
                HitTarget::TabAgenda => self.current_view = ViewMode::Agenda,
                HitTarget::NavPrev => self.nav_prev(),
                HitTarget::NavToday => self.nav_today(),
                HitTarget::NavNext => self.nav_next(),
                HitTarget::BtnNewEvent => self.open_new_event_modal(None),
                HitTarget::BtnHelp => self.open_help_modal(),
                HitTarget::BtnSync => self.trigger_sync(),
                HitTarget::BtnAccounts => self.open_accounts_modal(),
                HitTarget::BtnLanguage => self.toggle_language(),

                HitTarget::ModalAccountPreset(idx) => {
                    if let ActiveModal::Accounts(ref mut acc_state) = self.modal {
                        acc_state.set_preset(idx);
                    }
                }

                HitTarget::ModalAccountField(idx) => {
                    if let ActiveModal::Accounts(ref mut acc_state) = self.modal {
                        acc_state.active_field = idx;
                    }
                }

                HitTarget::ModalAccountAdd => {
                    if let ActiveModal::Accounts(ref mut acc_state) = self.modal {
                        acc_state.is_adding = true;
                    }
                }

                HitTarget::ModalAccountConnect => {
                    if let ActiveModal::Accounts(ref mut acc_state) = self.modal {
                        let (provider, auth_type) = match acc_state.provider_idx {
                            0 => (ProviderType::Naver, crate::model::AuthType::Basic),
                            1 => (ProviderType::Apple, crate::model::AuthType::Basic),
                            2 => (ProviderType::Google, crate::model::AuthType::OAuth2),
                            3 => (ProviderType::Microsoft, crate::model::AuthType::OAuth2),
                            4 => (ProviderType::Samsung, crate::model::AuthType::Basic),
                            _ => (ProviderType::Local, crate::model::AuthType::Basic),
                        };
                        let mut new_acc = CalDavAccount::new(
                            acc_state.name.clone(),
                            provider,
                            acc_state.url.clone(),
                            acc_state.username.clone(),
                            acc_state.password.clone(),
                        );
                        new_acc.auth_type = auth_type;
                        if auth_type == crate::model::AuthType::OAuth2 {
                            new_acc.access_token = Some(acc_state.password.clone());
                        }

                        self.db.save_account(&new_acc)?;
                        self.set_status(format!("Added account '{}'", new_acc.name));
                        self.trigger_sync();
                        self.close_modal();
                    }
                }

                HitTarget::ModalAccountDelete(id) => {
                    self.db.delete_account(&id)?;
                    self.reload_calendars()?;
                    self.reload_events()?;
                    self.set_status("Account removed");
                    if let ActiveModal::Accounts(ref mut acc_state) = self.modal {
                        acc_state.is_adding = false;
                    }
                }

                HitTarget::CalendarToggle(cal_id) => {
                    self.toggle_calendar(&cal_id)?;
                }

                HitTarget::MonthDay(date) => {
                    if self.selected_date == date {
                        // Double click on already selected date opens Day view
                        self.current_view = ViewMode::Day;
                    } else {
                        self.selected_date = date;
                    }
                }

                HitTarget::WeekSlot(date, hour) => {
                    self.selected_date = date;
                    self.selected_hour = hour;
                    self.open_new_event_modal(Some(hour));
                }

                HitTarget::DaySlot(hour, _) => {
                    self.selected_hour = hour;
                    self.open_new_event_modal(Some(hour));
                }

                HitTarget::EventItem(id) => {
                    self.open_event_detail_modal(id);
                }

                HitTarget::ModalField(idx) => {
                    if let ActiveModal::EventForm(ref mut form) = self.modal {
                        form.active_field = idx;
                    }
                }

                HitTarget::ModalSave => {
                    if let ActiveModal::EventForm(form) = self.modal.clone() {
                        self.save_event_from_form(form)?;
                    }
                }

                HitTarget::ModalCancel | HitTarget::ModalClose => {
                    self.close_modal();
                }

                HitTarget::ModalEdit => {
                    if let ActiveModal::EventDetail(ref id) = self.modal {
                        let id_clone = id.clone();
                        self.open_edit_event_modal(&id_clone);
                    }
                }

                HitTarget::ModalDelete => {
                    if let ActiveModal::EventDetail(ref id) = self.modal {
                        let id_clone = id.clone();
                        self.delete_event(&id_clone)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    if let Some(next) = next_month_first {
        let current_first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        (next - current_first).num_days() as u32
    } else {
        30
    }
}
