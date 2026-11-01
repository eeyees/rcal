use std::time::{Duration, Instant};
use anyhow::Result;
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveTime, TimeZone};

use crate::i18n::Language;
use crate::model::{CalDavAccount, Calendar, Event, ProviderType, SyncStatus, WeekStartDay};
use crate::mouse::{HitMap, HitTarget};
use crate::notification::{check_and_notify_upcoming, send_test_notification, NotificationConfig};
use crate::storage::Database;
use crate::ui::theme::{Theme, ThemeCategory, ThemeMode};
use crate::ui::theme_config::ThemeConfigFile;

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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub const TOTAL_FIELDS: usize = 9;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarModalState {
    pub is_creating: bool,
    pub name: String,
    pub color_idx: usize,
    pub selected_calendar_idx: usize,
    pub active_field: usize,
}

impl CalendarModalState {
    pub const PALETTE: [&'static str; 7] = [
        "#4285F4", // Blue
        "#0F9D58", // Green
        "#F4B400", // Yellow
        "#DB4437", // Red
        "#9C27B0", // Purple
        "#00ACC1", // Cyan
        "#FF9800", // Orange
    ];

    pub fn new() -> Self {
        Self {
            is_creating: false,
            name: String::new(),
            color_idx: 0,
            selected_calendar_idx: 0,
            active_field: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountModalState {
    pub is_adding: bool,
    pub provider_idx: usize, // 0: Naver, 1: Apple iCloud, 2: CalDAV
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub active_field: usize, // 0..=6
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageModalState {
    pub selected_idx: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeModalState {
    pub selected_idx: usize,
    pub category_filter: Option<ThemeCategory>,
    pub original_mode: ThemeMode,
}

impl ThemeModalState {
    pub fn filtered_themes(&self) -> Vec<ThemeMode> {
        if let Some(cat) = self.category_filter {
            cat.themes().to_vec()
        } else {
            ThemeMode::all().to_vec()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    EventForm(EventFormState),
    EventDetail(String /* event_id */),
    Accounts(AccountModalState),
    CalendarManager(CalendarModalState),
    Settings,
    Language(LanguageModalState),
    Theme(ThemeModalState),
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
    pub week_start_day: WeekStartDay,
    pub theme: Theme,
    pub theme_config: ThemeConfigFile,
    pub theme_category: ThemeCategory,
    pub show_sidebar: bool,
    pub notification_config: NotificationConfig,

    // Synchronization state
    pub is_syncing: bool,
    pub last_sync_status: Option<String>,
    pub sync_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl App {
    pub fn new(db: Database) -> Result<Self> {
        let today = Local::now().date_naive();
        let lang = db.get_setting("language")
            .ok()
            .flatten()
            .and_then(|s| Language::from_code(&s))
            .unwrap_or_else(Language::detect);
        let week_start_day = db.get_setting("week_start_day")
            .ok()
            .flatten()
            .map(|s| WeekStartDay::from_str(&s))
            .unwrap_or(WeekStartDay::Sunday);
        let (theme_config, _cfg_path) = ThemeConfigFile::load_or_create();
        let theme_mode = db.get_setting("theme")
            .ok()
            .flatten()
            .map(|s| ThemeMode::from_str(&s))
            .or_else(|| theme_config.active_theme.as_ref().map(|s| ThemeMode::from_str(s)))
            .unwrap_or(ThemeMode::ModernDark);
        let mut theme = Theme::from_mode(theme_mode);
        theme_config.apply_to(&mut theme);
        let theme_category = theme_mode.category();
        let notification_config = NotificationConfig::load_from_db(&db);

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
            week_start_day,
            theme,
            theme_config,
            theme_category,
            show_sidebar: true,
            notification_config,
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

    pub fn set_language(&mut self, lang: Language) {
        self.lang = lang;
        let _ = self.db.set_setting("language", lang.code());
        self.set_status(self.lang.msg_lang_changed());
    }

    pub fn toggle_language(&mut self) {
        self.set_language(self.lang.toggle());
    }

    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
        let msg = self.lang.msg_sidebar_toggle(self.show_sidebar);
        self.set_status(msg);
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

    pub fn open_new_all_day_event_modal(&mut self, date: Option<NaiveDate>) {
        if let Some(d) = date {
            self.selected_date = d;
        }
        let cal_id = self.default_calendar_id();
        let mut form = EventFormState::new_with_date(
            self.selected_date,
            cal_id,
            None,
        );
        form.all_day = true;
        self.modal = ActiveModal::EventForm(form);
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

    pub fn open_calendar_modal(&mut self) {
        self.modal = ActiveModal::CalendarManager(CalendarModalState::new());
    }

    pub fn open_settings_modal(&mut self) {
        self.modal = ActiveModal::Settings;
    }

    pub fn open_language_modal(&mut self) {
        let current_idx = Language::ALL.iter().position(|l| *l == self.lang).unwrap_or(0);
        self.modal = ActiveModal::Language(LanguageModalState {
            selected_idx: current_idx,
        });
    }

    pub fn open_theme_modal(&mut self) {
        let all = ThemeMode::all();
        let current_idx = all.iter().position(|m| *m == self.theme.mode).unwrap_or(0);
        let orig = self.theme.mode;
        self.modal = ActiveModal::Theme(ThemeModalState {
            selected_idx: current_idx,
            category_filter: None,
            original_mode: orig,
        });
    }

    pub fn set_week_start_day(&mut self, day: WeekStartDay) {
        self.week_start_day = day;
        let _ = self.db.set_setting("week_start_day", day.as_str());
        let label = day.label(self.lang);
        let msg = self.lang.msg_week_start_changed(label);
        self.set_status(msg);
    }

    pub fn toggle_notifications(&mut self) {
        self.notification_config.enabled = !self.notification_config.enabled;
        let _ = self.notification_config.save_to_db(&self.db);
        let msg = match self.lang {
            Language::Ko => {
                if self.notification_config.enabled {
                    format!("🔔 바탕화면 알림 활성화 ({}분 전 안내)", self.notification_config.lead_minutes)
                } else {
                    "🔕 바탕화면 알림 비활성화".to_string()
                }
            }
            Language::Ja => {
                if self.notification_config.enabled {
                    format!("🔔 デスクトップ通知有効 ({}分前)", self.notification_config.lead_minutes)
                } else {
                    "🔕 デスクトップ通知無効".to_string()
                }
            }
            _ => {
                if self.notification_config.enabled {
                    format!("🔔 Desktop notifications enabled ({}m before)", self.notification_config.lead_minutes)
                } else {
                    "🔕 Desktop notifications disabled".to_string()
                }
            }
        };
        self.set_status(msg);
    }

    pub fn set_notification_lead_minutes(&mut self, minutes: u32) {
        self.notification_config.lead_minutes = minutes;
        let _ = self.notification_config.save_to_db(&self.db);
        let label = if minutes == 0 {
            match self.lang {
                Language::Ko => "정시 (0분)".to_string(),
                Language::Ja => "開始時 (0分)".to_string(),
                _ => "At start (0m)".to_string(),
            }
        } else {
            match self.lang {
                Language::Ko => format!("{}분 전", minutes),
                Language::Ja => format!("{}分前", minutes),
                _ => format!("{}m before", minutes),
            }
        };
        let msg = match self.lang {
            Language::Ko => format!("🔔 알림 시간이 '{}'로 설정되었습니다.", label),
            Language::Ja => format!("🔔 通知時間が「{}」に設定されました。", label),
            _ => format!("🔔 Notification lead time set to '{}'.", label),
        };
        self.set_status(msg);
    }

    pub fn toggle_notification_sound(&mut self) {
        self.notification_config.sound = !self.notification_config.sound;
        let _ = self.notification_config.save_to_db(&self.db);
        let msg = match self.lang {
            Language::Ko => {
                if self.notification_config.sound {
                    "🔊 알림 소리 켜짐"
                } else {
                    "🔇 알림 소리 꺼짐 (무음)"
                }
            }
            Language::Ja => {
                if self.notification_config.sound {
                    "🔊 通知音オン"
                } else {
                    "🔇 通知音オフ"
                }
            }
            _ => {
                if self.notification_config.sound {
                    "🔊 Notification sound: ON"
                } else {
                    "🔇 Notification sound: OFF"
                }
            }
        };
        self.set_status(msg);
    }

    pub fn send_test_notification(&mut self) {
        match send_test_notification(self.lang, self.notification_config.sound) {
            Ok(_) => {
                let msg = match self.lang {
                    Language::Ko => "🔔 바탕화면 테스트 알림을 발송했습니다.",
                    Language::Ja => "🔔 デスクトップテスト通知を送信しました。",
                    _ => "🔔 Desktop test notification sent.",
                };
                self.set_status(msg);
            }
            Err(e) => {
                let msg = match self.lang {
                    Language::Ko => format!("⚠ 알림 발송 실패: {}", e),
                    Language::Ja => format!("⚠ 通知送信失敗: {}", e),
                    _ => format!("⚠ Notification failed: {}", e),
                };
                self.set_status(msg);
            }
        }
    }

    pub fn check_upcoming_notifications(&mut self) -> Result<Vec<(Event, String)>> {
        let notified = check_and_notify_upcoming(&self.db, &self.notification_config, Local::now(), self.lang)?;
        if let Some((ev, _)) = notified.first() {
            let msg = match self.lang {
                Language::Ko => format!("🔔 알림 발송: {}", ev.title),
                Language::Ja => format!("🔔 通知送信: {}", ev.title),
                _ => format!("🔔 Notification sent: {}", ev.title),
            };
            self.set_status(msg);
        }
        Ok(notified)
    }

    pub fn set_theme(&mut self, mode: ThemeMode) {
        let mut theme = Theme::from_mode(mode);
        self.theme_config.apply_to(&mut theme);
        self.theme = theme;
        self.theme_category = mode.category();
        let _ = self.db.set_setting("theme", mode.as_str());
        let all = ThemeMode::all();
        let idx = all.iter().position(|m| *m == mode).unwrap_or(0) + 1;
        let label = mode.label(self.lang);
        let msg = match self.lang {
            Language::Ko => format!("🎨 테마가 '{}' [{}/{}] 로 변경되었습니다.", label, idx, all.len()),
            Language::Ja => format!("🎨 テーマが「{}」[{}/{}] に変更されました。", label, idx, all.len()),
            _ => format!("🎨 Theme changed to '{}' [{}/{}].", label, idx, all.len()),
        };
        self.set_status(msg);
    }

    pub fn cycle_theme(&mut self) {
        let next_mode = self.theme.mode.next();
        self.set_theme(next_mode);
    }

    pub fn cycle_theme_prev(&mut self) {
        let prev_mode = self.theme.mode.prev();
        self.set_theme(prev_mode);
    }

    pub fn set_theme_category(&mut self, cat: ThemeCategory) {
        self.theme_category = cat;
        let themes = cat.themes();
        if !themes.contains(&self.theme.mode) && !themes.is_empty() {
            self.set_theme(themes[0]);
        }
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
            // When a modal is active, clicks on background targets must NEVER execute!
            if !matches!(self.modal, ActiveModal::None) && !target.is_modal() {
                return Ok(());
            }

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
                HitTarget::BtnLanguage => self.open_language_modal(),
                HitTarget::BtnTheme => self.open_theme_modal(),
                HitTarget::BtnManageCalendars => self.open_calendar_modal(),
                HitTarget::BtnSettings => self.open_settings_modal(),
                HitTarget::BtnToggleSidebar => self.toggle_sidebar(),
                HitTarget::ModalBackdrop => self.close_modal(),
                HitTarget::ModalConsume => {}

                HitTarget::ModalSettingWeekStart(day) => {
                    self.set_week_start_day(day);
                }

                HitTarget::ModalSettingNotificationToggle => {
                    self.toggle_notifications();
                }

                HitTarget::ModalSettingNotificationLead(mins) => {
                    self.set_notification_lead_minutes(mins);
                }

                HitTarget::ModalSettingNotificationSound => {
                    self.toggle_notification_sound();
                }

                HitTarget::ModalSettingNotificationTest => {
                    self.send_test_notification();
                }

                HitTarget::ModalSettingTheme(mode) => {
                    self.set_theme(mode);
                }

                HitTarget::ModalSettingThemeCategory(cat) => {
                    self.set_theme_category(cat);
                }

                HitTarget::ModalSettingThemePrev => {
                    self.cycle_theme_prev();
                }

                HitTarget::ModalSettingThemeNext => {
                    self.cycle_theme();
                }

                HitTarget::ModalSelectLanguage(l) => {
                    self.set_language(l);
                    if matches!(self.modal, ActiveModal::Language(_)) {
                        self.close_modal();
                    }
                }

                HitTarget::ModalSelectTheme(mode) => {
                    self.set_theme(mode);
                    if matches!(self.modal, ActiveModal::Theme(_)) {
                        self.close_modal();
                    }
                }

                HitTarget::ModalThemeCategory(cat) => {
                    if let ActiveModal::Theme(ref mut state) = self.modal {
                        state.category_filter = cat;
                        let filtered = state.filtered_themes();
                        let pos = filtered.iter().position(|m| *m == self.theme.mode).unwrap_or(0);
                        state.selected_idx = pos;
                        if let Some(mode) = filtered.get(pos) {
                            let m = *mode;
                            self.set_theme(m);
                        }
                    }
                }

                HitTarget::ModalCalNameField => {}
                HitTarget::ModalCalNew => {
                    if let ActiveModal::CalendarManager(ref mut cal_state) = self.modal {
                        cal_state.is_creating = true;
                    }
                }

                HitTarget::ModalCalColor(idx) => {
                    if let ActiveModal::CalendarManager(ref mut cal_state) = self.modal {
                        cal_state.color_idx = idx % CalendarModalState::PALETTE.len();
                    }
                }

                HitTarget::ModalCalSave => {
                    if let ActiveModal::CalendarManager(ref cal_state) = self.modal {
                        if !cal_state.name.trim().is_empty() {
                            let color = CalendarModalState::PALETTE[cal_state.color_idx % CalendarModalState::PALETTE.len()];
                            let new_cal = Calendar {
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
                            self.db.save_calendar(&new_cal)?;
                            self.reload_calendars()?;
                            self.set_status(self.lang.msg_calendar_created(&new_cal.name));
                            self.close_modal();
                        }
                    }
                }

                HitTarget::ModalCalDelete(id) => {
                    self.db.delete_calendar(&id)?;
                    self.reload_calendars()?;
                    self.reload_events()?;
                    self.set_status(self.lang.msg_calendar_deleted());
                    if let ActiveModal::CalendarManager(ref mut cal_state) = self.modal {
                        cal_state.is_creating = false;
                    }
                }

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
                        if acc_state.username.trim().is_empty() {
                            self.set_status(self.lang.msg_account_username_empty());
                            return Ok(());
                        }

                        let (provider, auth_type) = match acc_state.provider_idx {
                            0 => (ProviderType::Naver, crate::model::AuthType::Basic),
                            1 => (ProviderType::Apple, crate::model::AuthType::Basic),
                            2 => (ProviderType::Google, crate::model::AuthType::OAuth2),
                            3 => (ProviderType::Microsoft, crate::model::AuthType::OAuth2),
                            4 => (ProviderType::Samsung, crate::model::AuthType::Basic),
                            _ => (ProviderType::Local, crate::model::AuthType::Basic),
                        };

                        let existing = self.db.find_account_by_provider_and_username(&provider, &acc_state.username)?;
                        let is_update = existing.is_some();
                        let account_id = existing.map(|a| a.id).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                        let mut new_acc = CalDavAccount::new(
                            acc_state.name.clone(),
                            provider,
                            acc_state.url.clone(),
                            acc_state.username.clone(),
                            acc_state.password.clone(),
                        );
                        new_acc.id = account_id;
                        new_acc.auth_type = auth_type;
                        if auth_type == crate::model::AuthType::OAuth2 {
                            new_acc.access_token = Some(acc_state.password.clone());
                        }

                        self.db.save_account(&new_acc)?;
                        let status_msg = if is_update {
                            self.lang.msg_account_updated(&new_acc.name)
                        } else {
                            self.lang.msg_account_added(&new_acc.name)
                        };
                        self.set_status(status_msg);
                        self.trigger_sync();
                        self.close_modal();
                    }
                }

                HitTarget::ModalAccountDelete(id) => {
                    self.db.delete_account(&id)?;
                    self.reload_calendars()?;
                    self.reload_events()?;
                    self.set_status(self.lang.msg_account_removed());
                    if let ActiveModal::Accounts(ref mut acc_state) = self.modal {
                        acc_state.is_adding = false;
                        acc_state.selected_account_idx = 0;
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

                HitTarget::WeekAllDaySlot(date) => {
                    self.open_new_all_day_event_modal(Some(date));
                }

                HitTarget::DaySlot(hour, _) => {
                    self.selected_hour = hour;
                    self.open_new_event_modal(Some(hour));
                }

                HitTarget::DayAllDaySlot(date) => {
                    self.open_new_all_day_event_modal(Some(date));
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
                    if let ActiveModal::Theme(ref state) = self.modal {
                        let orig = state.original_mode;
                        self.set_theme(orig);
                    }
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
