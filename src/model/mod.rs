use chrono::{DateTime, Local, NaiveDate};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    Local,
    Google,
    Apple,
    Naver,
    Microsoft,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::Local => "Local",
            ProviderType::Google => "Google",
            ProviderType::Apple => "Apple",
            ProviderType::Naver => "Naver",
            ProviderType::Microsoft => "Microsoft",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Google" => ProviderType::Google,
            "Apple" => ProviderType::Apple,
            "Naver" => ProviderType::Naver,
            "Microsoft" => ProviderType::Microsoft,
            _ => ProviderType::Local,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub color_code: String, // hex e.g. "#4285F4" or named color
    pub provider: ProviderType,
    pub enabled: bool,
    pub is_default: bool,
}

impl Calendar {
    pub fn ratatui_color(&self) -> Color {
        match self.color_code.as_str() {
            "blue" | "#4285F4" => Color::Rgb(66, 133, 244),
            "green" | "#0F9D58" => Color::Rgb(15, 157, 88),
            "yellow" | "#F4B400" => Color::Rgb(244, 180, 0),
            "red" | "#DB4437" => Color::Rgb(219, 68, 55),
            "purple" | "#9C27B0" => Color::Rgb(156, 39, 176),
            "cyan" | "#00ACC1" => Color::Rgb(0, 172, 193),
            "orange" | "#FF9800" => Color::Rgb(255, 152, 0),
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
