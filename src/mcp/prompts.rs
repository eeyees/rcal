use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, TimeZone};

use crate::storage::Database;
use super::tools::parse_flexible_date;
use super::types::{ContentItem, Prompt, PromptArgument, PromptMessage};

pub fn get_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "daily_briefing".to_string(),
            description: Some("Generate a structured morning briefing of scheduled events for a specific day".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "date".to_string(),
                    description: Some("Target date in YYYY-MM-DD format (defaults to today)".to_string()),
                    required: Some(false),
                },
            ]),
        },
        Prompt {
            name: "schedule_event".to_string(),
            description: Some("Draft an event schedule while inspecting existing appointments for conflicts".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "title".to_string(),
                    description: Some("Title or purpose of the event to schedule".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "date".to_string(),
                    description: Some("Proposed date in YYYY-MM-DD format (defaults to today)".to_string()),
                    required: Some(false),
                },
            ]),
        },
        Prompt {
            name: "find_free_time".to_string(),
            description: Some("Review open calendar slots on a date and suggest the best meeting times".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "date".to_string(),
                    description: Some("Target date (defaults to today)".to_string()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "duration_minutes".to_string(),
                    description: Some("Meeting duration in minutes (e.g. 30, 60)".to_string()),
                    required: Some(false),
                },
            ]),
        },
    ]
}

pub fn get_prompt_messages(
    name: &str,
    arguments: Option<serde_json::Value>,
    db: &Database,
) -> Result<(String, Vec<PromptMessage>)> {
    match name {
        "daily_briefing" => {
            let date_str = arguments
                .as_ref()
                .and_then(|a| a.get("date"))
                .and_then(|v| v.as_str());

            let target_date = match date_str {
                Some(s) => parse_flexible_date(s)?,
                None => Local::now().date_naive(),
            };

            let start = Local.from_local_datetime(&target_date.and_hms_opt(0, 0, 0).unwrap()).unwrap();
            let end = Local.from_local_datetime(&target_date.and_hms_opt(23, 59, 59).unwrap()).unwrap();
            let events = db.get_events_range(start, end)?;

            let mut events_summary = String::new();
            if events.is_empty() {
                events_summary.push_str("No events scheduled for this day.\n");
            } else {
                for ev in &events {
                    events_summary.push_str(&format!(
                        "- [{}] {} ({})\n",
                        ev.time_str(),
                        ev.title,
                        ev.location.as_deref().unwrap_or("No location")
                    ));
                    if let Some(desc) = &ev.description {
                        events_summary.push_str(&format!("  Notes: {}\n", desc));
                    }
                }
            }

            let prompt_text = format!(
                "Here is my calendar schedule for {} ({}):\n\n{}\nPlease provide a concise, prioritized briefing highlighting my key commitments, preparation recommendations, and any open gaps in my day.",
                target_date.format("%Y-%m-%d"),
                target_date.weekday(),
                events_summary
            );

            Ok((
                format!("Daily briefing for {}", target_date.format("%Y-%m-%d")),
                vec![PromptMessage {
                    role: "user".to_string(),
                    content: ContentItem::Text { text: prompt_text },
                }],
            ))
        }
        "schedule_event" => {
            let title = arguments
                .as_ref()
                .and_then(|a| a.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("New Meeting");

            let date_str = arguments
                .as_ref()
                .and_then(|a| a.get("date"))
                .and_then(|v| v.as_str());

            let target_date = match date_str {
                Some(s) => parse_flexible_date(s)?,
                None => Local::now().date_naive(),
            };

            let start = Local.from_local_datetime(&target_date.and_hms_opt(0, 0, 0).unwrap()).unwrap();
            let end = Local.from_local_datetime(&target_date.and_hms_opt(23, 59, 59).unwrap()).unwrap();
            let events = db.get_events_range(start, end)?;

            let mut existing = String::new();
            for ev in &events {
                existing.push_str(&format!("- [{}] {}\n", ev.time_str(), ev.title));
            }
            if existing.is_empty() {
                existing = "No existing appointments.\n".to_string();
            }

            let prompt_text = format!(
                "I want to schedule an event titled '{}' on {}.\n\nExisting events on this date:\n{}\nPlease analyze the schedule, check for potential conflicts, recommend the best time slots, and draft the appointment details.",
                title,
                target_date.format("%Y-%m-%d"),
                existing
            );

            Ok((
                format!("Schedule '{}' on {}", title, target_date.format("%Y-%m-%d")),
                vec![PromptMessage {
                    role: "user".to_string(),
                    content: ContentItem::Text { text: prompt_text },
                }],
            ))
        }
        "find_free_time" => {
            let date_str = arguments
                .as_ref()
                .and_then(|a| a.get("date"))
                .and_then(|v| v.as_str());

            let duration = arguments
                .as_ref()
                .and_then(|a| a.get("duration_minutes"))
                .and_then(|v| v.as_i64())
                .unwrap_or(30);

            let target_date = match date_str {
                Some(s) => parse_flexible_date(s)?,
                None => Local::now().date_naive(),
            };

            let prompt_text = format!(
                "Please review my schedule for {} and suggest the best available time slots for a {}-minute meeting during standard working hours (09:00 - 18:00).",
                target_date.format("%Y-%m-%d"),
                duration
            );

            Ok((
                format!("Find free time on {}", target_date.format("%Y-%m-%d")),
                vec![PromptMessage {
                    role: "user".to_string(),
                    content: ContentItem::Text { text: prompt_text },
                }],
            ))
        }
        _ => Err(anyhow!("Unknown prompt name: '{}'", name)),
    }
}
