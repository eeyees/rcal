use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, TimeZone};
use serde_json::json;

use super::types::{Resource, ResourceContent};
use crate::storage::Database;

pub fn get_resources() -> Vec<Resource> {
    vec![
        Resource {
            uri: "rcal://today".to_string(),
            name: "Today's Agenda".to_string(),
            description: Some("All scheduled events and commitments for today".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "rcal://upcoming".to_string(),
            name: "Upcoming Events (7 Days)".to_string(),
            description: Some("Scheduled events over the next 7 days".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "rcal://calendars".to_string(),
            name: "Configured Calendars".to_string(),
            description: Some("List of all calendars and their enabled statuses".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ]
}

pub fn read_resource(uri: &str, db: &Database) -> Result<ResourceContent> {
    match uri {
        "rcal://today" => {
            let today = Local::now().date_naive();
            let start = Local
                .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
                .unwrap();
            let end = Local
                .from_local_datetime(&today.and_hms_opt(23, 59, 59).unwrap())
                .unwrap();

            let events = db.get_events_range(start, end)?;
            let event_dtos: Vec<serde_json::Value> = events
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "calendar_id": e.calendar_id,
                        "title": e.title,
                        "description": e.description,
                        "location": e.location,
                        "start_time": e.start_time.to_rfc3339(),
                        "end_time": e.end_time.to_rfc3339(),
                        "all_day": e.all_day,
                        "time_str": e.time_str(),
                    })
                })
                .collect();

            let payload = json!({
                "date": today.format("%Y-%m-%d").to_string(),
                "weekday": today.weekday().to_string(),
                "count": event_dtos.len(),
                "events": event_dtos,
            });

            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: serde_json::to_string_pretty(&payload)?,
            })
        }
        "rcal://upcoming" => {
            let now = Local::now();
            let today = now.date_naive();
            let start = Local
                .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
                .unwrap();
            let next_week = today + chrono::Duration::days(7);
            let end = Local
                .from_local_datetime(&next_week.and_hms_opt(23, 59, 59).unwrap())
                .unwrap();

            let events = db.get_events_range(start, end)?;
            let event_dtos: Vec<serde_json::Value> = events
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "calendar_id": e.calendar_id,
                        "title": e.title,
                        "description": e.description,
                        "location": e.location,
                        "start_time": e.start_time.to_rfc3339(),
                        "end_time": e.end_time.to_rfc3339(),
                        "all_day": e.all_day,
                        "time_str": e.time_str(),
                    })
                })
                .collect();

            let payload = json!({
                "start_date": today.format("%Y-%m-%d").to_string(),
                "end_date": next_week.format("%Y-%m-%d").to_string(),
                "count": event_dtos.len(),
                "events": event_dtos,
            });

            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: serde_json::to_string_pretty(&payload)?,
            })
        }
        "rcal://calendars" => {
            let cals = db.get_calendars()?;
            let dtos: Vec<serde_json::Value> = cals
                .into_iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "name": c.name,
                        "color_code": c.color_code,
                        "provider": c.provider.as_str(),
                        "enabled": c.enabled,
                        "is_default": c.is_default,
                    })
                })
                .collect();

            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: serde_json::to_string_pretty(&dtos)?,
            })
        }
        _ => Err(anyhow!("Unknown resource URI: '{}'", uri)),
    }
}
