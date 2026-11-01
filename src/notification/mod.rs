use crate::i18n::Language;
use crate::model::Event;
use crate::storage::Database;
use anyhow::Result;
use chrono::{DateTime, Local, Timelike};

/// Configuration for desktop notifications
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub lead_minutes: u32,
    pub sound: bool,
}

impl NotificationConfig {
    pub const LEAD_OPTIONS: &'static [u32] = &[0, 5, 10, 15, 30];

    pub fn load_from_db(db: &Database) -> Self {
        let enabled = db
            .get_setting("notification_enabled")
            .ok()
            .flatten()
            .map(|s| s != "false")
            .unwrap_or(true);

        let lead_minutes = db
            .get_setting("notification_lead_minutes")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(10);

        let sound = db
            .get_setting("notification_sound")
            .ok()
            .flatten()
            .map(|s| s != "false")
            .unwrap_or(true);

        Self {
            enabled,
            lead_minutes,
            sound,
        }
    }

    pub fn save_to_db(&self, db: &Database) -> Result<()> {
        db.set_setting(
            "notification_enabled",
            if self.enabled { "true" } else { "false" },
        )?;
        db.set_setting("notification_lead_minutes", &self.lead_minutes.to_string())?;
        db.set_setting(
            "notification_sound",
            if self.sound { "true" } else { "false" },
        )?;
        Ok(())
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lead_minutes: 10,
            sound: true,
        }
    }
}

/// Send a native desktop notification using notify-rust.
pub fn send_desktop_notification(summary: &str, body: &str, sound: bool) -> Result<()> {
    let mut notification = notify_rust::Notification::new();
    notification.appname("rcal").summary(summary).body(body);

    if sound {
        #[cfg(target_os = "macos")]
        notification.sound_name("Ping");
        #[cfg(not(target_os = "macos"))]
        notification.sound_name("default");
    }

    notification.show()?;
    Ok(())
}

/// Send a desktop notification safely without panicking if the OS desktop notification daemon is unavailable.
pub fn send_desktop_notification_safe(summary: &str, body: &str, sound: bool) -> bool {
    match send_desktop_notification(summary, body, sound) {
        Ok(_) => true,
        Err(e) => {
            eprintln!(
                "[rcal notification warning] Failed to display notification: {:?}",
                e
            );
            false
        }
    }
}

/// Send a test notification to verify OS notification permissions and sound.
pub fn send_test_notification(lang: Language, sound: bool) -> Result<()> {
    let (summary, body) = match lang {
        Language::Ko => (
            "🔔  rcal 테스트 알림".to_string(),
            "rcal 바탕화면 알림이 성공적으로 설정되었습니다!\n다가오는 일정이 지정된 시간에 알림으로 안내됩니다.".to_string(),
        ),
        Language::Ja => (
            "🔔  rcal テスト通知".to_string(),
            "rcal デスクトップ通知が正常に設定されました！\n今後の予定が指定された時間に通知されます。".to_string(),
        ),
        _ => (
            "🔔  rcal Test Notification".to_string(),
            "rcal desktop notifications are working properly!\nUpcoming events will be notified at your configured lead time.".to_string(),
        ),
    };

    send_desktop_notification(&summary, &body, sound)
}

/// Formats the notification title and body text based on event details, lead time, and language.
pub fn format_event_notification(
    event: &Event,
    calendar_name: &str,
    _lead_minutes: u32,
    minutes_remaining: i64,
    lang: Language,
) -> (String, String) {
    let summary = if event.all_day {
        match lang {
            Language::Ko => format!("📅  [오늘 종일 일정] {}", event.title),
            Language::Ja => format!("📅  [今日の終日予定] {}", event.title),
            _ => format!("📅  [All-Day Event Today] {}", event.title),
        }
    } else if minutes_remaining <= 0 {
        match lang {
            Language::Ko => format!("⏰  [시작] {}", event.title),
            Language::Ja => format!("⏰  [開始] {}", event.title),
            _ => format!("⏰  [Starting Now] {}", event.title),
        }
    } else {
        match lang {
            Language::Ko => format!("🔔  [곧 시작] {} ({}분 전)", event.title, minutes_remaining),
            Language::Ja => format!(
                "🔔  [まもなく開始] {} ({}分前)",
                event.title, minutes_remaining
            ),
            _ => format!("🔔  [Upcoming] {} (in {}m)", event.title, minutes_remaining),
        }
    };

    let mut body = String::new();
    if event.all_day {
        body.push_str(&format!("📅  {}", event.start_time.format("%Y-%m-%d")));
    } else {
        body.push_str(&format!(
            "⏰  {} - {}",
            event.start_time.format("%H:%M"),
            event.end_time.format("%H:%M")
        ));
    }

    if let Some(loc) = &event.location {
        let trimmed = loc.trim();
        if !trimmed.is_empty() {
            body.push_str(&format!("\n📍  {}", trimmed));
        }
    }

    if !calendar_name.trim().is_empty() {
        body.push_str(&format!("\n📁  {}", calendar_name.trim()));
    }

    if let Some(desc) = &event.description {
        let trimmed = desc.trim();
        if !trimmed.is_empty() {
            let first_line = trimmed.lines().next().unwrap_or(trimmed);
            let truncated = if first_line.chars().count() > 80 {
                let s: String = first_line.chars().take(77).collect();
                format!("{}...", s)
            } else {
                first_line.to_string()
            };
            body.push_str(&format!("\n📝  {}", truncated));
        }
    }

    (summary, body)
}

/// Checks the database for upcoming events matching the notification criteria and triggers notifications.
/// Returns a list of (Event, calendar_name) that were notified during this check.
pub fn check_and_notify_upcoming(
    db: &Database,
    config: &NotificationConfig,
    now: DateTime<Local>,
    lang: Language,
) -> Result<Vec<(Event, String)>> {
    if !config.enabled {
        return Ok(Vec::new());
    }

    let lead_mins = config.lead_minutes as i64;
    // Look ahead from (now - 1 min) to (now + lead_mins)
    let from_time = now - chrono::Duration::minutes(1);
    let to_time = now + chrono::Duration::minutes(lead_mins.max(1));

    let events_with_cal = db.get_events_for_notification(from_time, to_time)?;
    let mut notified = Vec::new();

    for (event, cal_name) in events_with_cal {
        if event.all_day {
            // For all-day events, notify once on that day if now >= 08:00
            if now.date_naive() == event.start_time.date_naive() && now.hour() >= 8 {
                let notif_key = "allday_today";
                if !db.has_notified(&event.id, notif_key)? {
                    let (summary, body) =
                        format_event_notification(&event, &cal_name, config.lead_minutes, 0, lang);
                    send_desktop_notification_safe(&summary, &body, config.sound);
                    db.mark_notified(&event.id, notif_key)?;
                    notified.push((event, cal_name));
                }
            }
        } else {
            // For timed events:
            let diff_sec = (event.start_time - now).num_seconds();
            let minutes_remaining = (diff_sec + 59) / 60; // ceiling division for remaining minutes

            // Condition to trigger:
            // Event is starting in <= lead_mins (or starting now if lead_mins == 0)
            // and event hasn't started more than 1 minute ago.
            if minutes_remaining <= lead_mins && diff_sec >= -60 {
                let notif_key = format!("lead_{}m", config.lead_minutes);
                if !db.has_notified(&event.id, &notif_key)? {
                    let (summary, body) = format_event_notification(
                        &event,
                        &cal_name,
                        config.lead_minutes,
                        minutes_remaining,
                        lang,
                    );
                    send_desktop_notification_safe(&summary, &body, config.sound);
                    db.mark_notified(&event.id, &notif_key)?;
                    notified.push((event, cal_name));
                }
            }
        }
    }

    Ok(notified)
}
