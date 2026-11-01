use chrono::{DateTime, Datelike, Local, NaiveDate};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    Local,
    Google,
    Apple,
    Naver,
    Microsoft,
    Samsung,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::Local => "Local",
            ProviderType::Google => "Google",
            ProviderType::Apple => "Apple",
            ProviderType::Naver => "Naver",
            ProviderType::Microsoft => "Microsoft",
            ProviderType::Samsung => "Samsung",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Google" => ProviderType::Google,
            "Apple" => ProviderType::Apple,
            "Naver" => ProviderType::Naver,
            "Microsoft" => ProviderType::Microsoft,
            "Samsung" => ProviderType::Samsung,
            _ => ProviderType::Local,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub account_id: Option<String>,
    pub name: String,
    pub color_code: String, // hex e.g. "#4285F4" or named color
    pub provider: ProviderType,
    pub enabled: bool,
    pub is_default: bool,
    pub remote_url: Option<String>,
    pub ctag: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeekStartDay {
    Sunday,
    Monday,
    Saturday,
}

impl WeekStartDay {
    pub fn as_str(&self) -> &'static str {
        match self {
            WeekStartDay::Sunday => "sunday",
            WeekStartDay::Monday => "monday",
            WeekStartDay::Saturday => "saturday",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "monday" | "mon" => WeekStartDay::Monday,
            "saturday" | "sat" => WeekStartDay::Saturday,
            _ => WeekStartDay::Sunday,
        }
    }

    pub fn days_offset(&self, date: NaiveDate) -> u32 {
        let dow = date.weekday();
        match self {
            WeekStartDay::Sunday => dow.num_days_from_sunday(),
            WeekStartDay::Monday => dow.num_days_from_monday(),
            WeekStartDay::Saturday => (dow.num_days_from_sunday() + 1) % 7,
        }
    }

    pub fn week_start_date(&self, date: NaiveDate) -> NaiveDate {
        let offset = self.days_offset(date);
        date - chrono::Duration::days(offset as i64)
    }

    pub fn weekday_headers(&self, lang: crate::i18n::Language) -> Vec<&'static str> {
        lang.weekday_headers_for(*self).to_vec()
    }

    pub fn label(&self, lang: crate::i18n::Language) -> &'static str {
        lang.week_start_label(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthType {
    Basic,
    OAuth2,
}

impl AuthType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthType::Basic => "basic",
            AuthType::OAuth2 => "oauth2",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "oauth2" => AuthType::OAuth2,
            _ => AuthType::Basic,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalDavAccount {
    pub id: String,
    pub name: String,
    pub provider: ProviderType,
    pub url: String,
    pub username: String,
    pub password: String,
    pub auth_type: AuthType,
    pub client_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Local>>,
    pub last_synced_at: Option<DateTime<Local>>,
}

impl CalDavAccount {
    pub fn new(
        name: String,
        provider: ProviderType,
        url: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            id: format!("acc_{}", uuid::Uuid::new_v4()),
            name,
            provider,
            url,
            username,
            password,
            auth_type: AuthType::Basic,
            client_id: None,
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            last_synced_at: None,
        }
    }

    pub fn new_oauth(
        name: String,
        provider: ProviderType,
        client_id: String,
        client_secret: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
    ) -> Self {
        let default_url = match provider {
            ProviderType::Google => "https://www.googleapis.com/calendar/v3".to_string(),
            ProviderType::Microsoft => "https://graph.microsoft.com/v1.0".to_string(),
            _ => String::new(),
        };

        Self {
            id: format!("acc_{}", uuid::Uuid::new_v4()),
            name,
            provider,
            url: default_url,
            username: client_id.clone(),
            password: client_secret,
            auth_type: AuthType::OAuth2,
            client_id: Some(client_id),
            access_token,
            refresh_token,
            token_expires_at: None,
            last_synced_at: None,
        }
    }
}

impl Calendar {
    pub fn ratatui_color(&self) -> Color {
        match self.color_code.as_str() {
            "blue" | "#4285F4" => Color::Rgb(66, 133, 244), // Google Blue
            "green" | "#0F9D58" | "#03C75A" => Color::Rgb(15, 157, 88), // Naver Green
            "yellow" | "#F4B400" => Color::Rgb(244, 180, 0),
            "red" | "#DB4437" => Color::Rgb(219, 68, 55),
            "purple" | "#9C27B0" => Color::Rgb(156, 39, 176),
            "cyan" | "#00ACC1" => Color::Rgb(0, 172, 193),
            "orange" | "#FF9800" => Color::Rgb(255, 152, 0),
            "#0078D4" => Color::Rgb(0, 120, 212), // Microsoft Blue
            "#034EA2" | "samsung" => Color::Rgb(3, 78, 162), // Samsung Iconic Blue
            _ => Color::Cyan,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    CreatedLocally,
    UpdatedLocally,
    DeletedLocally,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncStatus::Synced => "synced",
            SyncStatus::CreatedLocally => "created_locally",
            SyncStatus::UpdatedLocally => "updated_locally",
            SyncStatus::DeletedLocally => "deleted_locally",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "created_locally" => SyncStatus::CreatedLocally,
            "updated_locally" => SyncStatus::UpdatedLocally,
            "deleted_locally" => SyncStatus::DeletedLocally,
            _ => SyncStatus::Synced,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub calendar_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub all_day: bool,
    pub rrule: Option<String>,
    pub sync_status: SyncStatus,
    pub remote_id: Option<String>,
    pub etag: Option<String>,
    pub updated_at: DateTime<Local>,
}

impl Event {
    pub fn new(
        calendar_id: String,
        title: String,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
        all_day: bool,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            calendar_id,
            title,
            description: None,
            location: None,
            start_time,
            end_time,
            all_day,
            rrule: None,
            sync_status: SyncStatus::CreatedLocally,
            remote_id: None,
            etag: None,
            updated_at: Local::now(),
        }
    }

    /// Checks if the event falls on or spans across the given date
    pub fn occurs_on(&self, date: NaiveDate) -> bool {
        let event_start_date = self.start_time.date_naive();
        let event_end_date = self.end_time.date_naive();

        if self.all_day {
            date >= event_start_date && date <= event_end_date
        } else {
            // For timed events: spans if date is between start and end date
            // or if starts/ends on this date
            date >= event_start_date && date <= event_end_date
        }
    }

    /// Formatted time string, e.g. "14:00 - 15:30" or "All day"
    pub fn time_str(&self) -> String {
        if self.all_day {
            "All-day".to_string()
        } else {
            format!(
                "{} - {}",
                self.start_time.format("%H:%M"),
                self.end_time.format("%H:%M")
            )
        }
    }
}
