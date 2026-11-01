use std::cmp::{max, min};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::model::{Event, SyncStatus};
use crate::storage::Database;
use crate::sync::SyncEngine;
use super::types::{Tool, ToolCallResult};

/// Returns all MCP tool definitions supported by rcal.
pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_calendars".to_string(),
            description: "List all calendars configured in rcal, including IDs, names, color codes, providers (Local, Google, Apple, Naver, Microsoft, Samsung), and enabled status.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "list_events".to_string(),
            description: "Query calendar events within a date or time range. If neither start nor end is provided, defaults to the upcoming 7 days.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_date": {
                        "type": "string",
                        "description": "Start date or ISO datetime (e.g. '2026-11-01' or '2026-11-01T09:00:00'). Inclusive."
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date or ISO datetime (e.g. '2026-11-07' or '2026-11-07T23:59:59'). Inclusive."
                    },
                    "calendar_id": {
                        "type": "string",
                        "description": "Optional calendar ID to filter events by. If omitted, returns events from all enabled calendars."
                    },
                    "query": {
                        "type": "string",
                        "description": "Optional search term to filter events by title, description, or location."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Optional maximum number of events to return (default: 50)."
                    }
                }
            }),
        },
        Tool {
            name: "get_event".to_string(),
            description: "Get detailed information about a single event by its unique ID.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The unique event ID."
                    }
                },
                "required": ["id"]
            }),
        },
        Tool {
            name: "create_event".to_string(),
            description: "Create a new calendar event. Time fields accept ISO 8601 strings or YYYY-MM-DD for all-day events.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "The title or summary of the event (e.g., 'Team Sprint Planning')."
                    },
                    "start_time": {
                        "type": "string",
                        "description": "Event start time in ISO 8601 format (e.g., '2026-11-02T14:00:00' or '2026-11-02' for all-day)."
                    },
                    "end_time": {
                        "type": "string",
                        "description": "Event end time in ISO 8601 format. If omitted for timed events, defaults to 1 hour after start_time."
                    },
                    "all_day": {
                        "type": "boolean",
                        "description": "True if this is an all-day event (default: false)."
                    },
                    "calendar_id": {
                        "type": "string",
                        "description": "The calendar ID to add this event to. Defaults to the user's primary/default calendar."
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional notes or detailed description for the event."
                    },
                    "location": {
                        "type": "string",
                        "description": "Optional location or virtual meeting URL."
                    }
                },
                "required": ["title", "start_time"]
            }),
        },
        Tool {
            name: "update_event".to_string(),
            description: "Update an existing event. Only fields provided in arguments will be updated.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The ID of the event to update."
                    },
                    "title": {
                        "type": "string",
                        "description": "New title for the event."
                    },
                    "start_time": {
                        "type": "string",
                        "description": "New start time in ISO 8601 format."
                    },
                    "end_time": {
                        "type": "string",
                        "description": "New end time in ISO 8601 format."
                    },
                    "all_day": {
                        "type": "boolean",
                        "description": "Update all-day flag."
                    },
                    "calendar_id": {
                        "type": "string",
                        "description": "Move event to a different calendar ID."
                    },
                    "description": {
                        "type": "string",
                        "description": "New description."
                    },
                    "location": {
                        "type": "string",
                        "description": "New location."
                    }
                },
                "required": ["id"]
            }),
        },
        Tool {
            name: "delete_event".to_string(),
            description: "Delete an event from rcal by its ID.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The ID of the event to delete."
                    }
                },
                "required": ["id"]
            }),
        },
        Tool {
            name: "search_events".to_string(),
            description: "Search across events for keywords matching title, description, or location.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query or keyword."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of search results (default: 20)."
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "find_free_slots".to_string(),
            description: "Find free time slots for a specified date and duration within working hours (e.g. 09:00 - 18:00).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "description": "Date to find free slots on (format: YYYY-MM-DD). Defaults to today."
                    },
                    "duration_minutes": {
                        "type": "integer",
                        "description": "Required duration in minutes (default: 30)."
                    },
                    "start_hour": {
                        "type": "integer",
                        "description": "Working day start hour, 0-23 (default: 9)."
                    },
                    "end_hour": {
                        "type": "integer",
                        "description": "Working day end hour, 1-24 (default: 18)."
                    }
                }
            }),
        },
        Tool {
            name: "get_current_time".to_string(),
            description: "Returns current local date, time, day of week, and timezone of the host system. Essential for resolving relative dates like 'today', 'tomorrow', 'next Monday'.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "sync_calendars".to_string(),
            description: "Trigger immediate two-way synchronization with all configured cloud calendar accounts (Google, Apple, Naver, Microsoft, Samsung, CalDAV).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

/// Executes a tool call given the tool name and arguments.
pub async fn call_tool(name: &str, args: serde_json::Value, db: &Database) -> ToolCallResult {
    match name {
        "list_calendars" => handle_list_calendars(db),
        "list_events" => handle_list_events(args, db),
        "get_event" => handle_get_event(args, db),
        "create_event" => handle_create_event(args, db),
        "update_event" => handle_update_event(args, db),
        "delete_event" => handle_delete_event(args, db),
        "search_events" => handle_search_events(args, db),
        "find_free_slots" => handle_find_free_slots(args, db),
        "get_current_time" => handle_get_current_time(),
        "sync_calendars" => handle_sync_calendars(db).await,
        _ => ToolCallResult::error(format!("Unknown tool: '{}'", name)),
    }
}

// ----------------------------------------------------------------------------
// Tool Handlers
// ----------------------------------------------------------------------------

fn handle_list_calendars(db: &Database) -> ToolCallResult {
    match db.get_calendars() {
        Ok(cals) => {
            #[derive(Serialize)]
            struct CalDto {
                id: String,
                name: String,
                color_code: String,
                provider: &'static str,
                enabled: bool,
                is_default: bool,
            }

            let dtos: Vec<CalDto> = cals
                .into_iter()
                .map(|c| CalDto {
                    id: c.id,
                    name: c.name,
                    color_code: c.color_code,
                    provider: c.provider.as_str(),
                    enabled: c.enabled,
                    is_default: c.is_default,
                })
                .collect();

            match ToolCallResult::json(&dtos) {
                Ok(res) => res,
                Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to retrieve calendars: {}", e)),
    }
}

#[derive(Deserialize)]
struct ListEventsArgs {
    start_date: Option<String>,
    end_date: Option<String>,
    calendar_id: Option<String>,
    query: Option<String>,
    limit: Option<usize>,
}

fn handle_list_events(args: serde_json::Value, db: &Database) -> ToolCallResult {
    let args: ListEventsArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => return ToolCallResult::error(format!("Invalid arguments for list_events: {}", e)),
    };

    let start_time = match &args.start_date {
        Some(s) => match parse_flexible_datetime(s, false) {
            Ok(dt) => dt,
            Err(e) => return ToolCallResult::error(format!("Invalid start_date: {}", e)),
        },
        None => {
            let today = Local::now().date_naive();
            match Local.from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap()) {
                chrono::LocalResult::Single(dt) => dt,
                _ => Local::now(),
            }
        }
    };

    let end_time = match &args.end_date {
        Some(s) => match parse_flexible_datetime(s, true) {
            Ok(dt) => dt,
            Err(e) => return ToolCallResult::error(format!("Invalid end_date: {}", e)),
        },
        None => {
            let next_week = start_time.date_naive() + chrono::Duration::days(7);
            match Local.from_local_datetime(&next_week.and_hms_opt(23, 59, 59).unwrap()) {
                chrono::LocalResult::Single(dt) => dt,
                _ => start_time + chrono::Duration::days(7),
            }
        }
    };

    match db.get_events_filtered(
        Some(start_time),
        Some(end_time),
        args.calendar_id.as_deref(),
        args.query.as_deref(),
        args.limit.or(Some(50)),
    ) {
        Ok(events) => {
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

            let result = json!({
                "count": event_dtos.len(),
                "start_range": start_time.to_rfc3339(),
                "end_range": end_time.to_rfc3339(),
                "events": event_dtos,
            });

            match ToolCallResult::json(&result) {
                Ok(res) => res,
                Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to query events: {}", e)),
    }
}

#[derive(Deserialize)]
struct GetEventArgs {
    id: String,
}

fn handle_get_event(args: serde_json::Value, db: &Database) -> ToolCallResult {
    let args: GetEventArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => return ToolCallResult::error(format!("Invalid arguments for get_event: {}", e)),
    };

    match db.get_event_by_id(&args.id) {
        Ok(Some(ev)) => {
            let dto = json!({
                "id": ev.id,
                "calendar_id": ev.calendar_id,
                "title": ev.title,
                "description": ev.description,
                "location": ev.location,
                "start_time": ev.start_time.to_rfc3339(),
                "end_time": ev.end_time.to_rfc3339(),
                "all_day": ev.all_day,
                "time_str": ev.time_str(),
                "updated_at": ev.updated_at.to_rfc3339(),
            });
            match ToolCallResult::json(&dto) {
                Ok(res) => res,
                Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
            }
        }
        Ok(None) => ToolCallResult::error(format!("Event with ID '{}' not found", args.id)),
        Err(e) => ToolCallResult::error(format!("Database error retrieving event: {}", e)),
    }
}

#[derive(Deserialize)]
struct CreateEventArgs {
    title: String,
    start_time: String,
    end_time: Option<String>,
    all_day: Option<bool>,
    calendar_id: Option<String>,
    description: Option<String>,
    location: Option<String>,
}

fn handle_create_event(args: serde_json::Value, db: &Database) -> ToolCallResult {
    let args: CreateEventArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => return ToolCallResult::error(format!("Invalid arguments for create_event: {}", e)),
    };

    let title = args.title.trim().to_string();
    if title.is_empty() {
        return ToolCallResult::error("Event title cannot be empty");
    }

    let all_day = args.all_day.unwrap_or(false);

    let start_time = match parse_flexible_datetime(&args.start_time, false) {
        Ok(dt) => dt,
        Err(e) => return ToolCallResult::error(format!("Invalid start_time: {}", e)),
    };

    let end_time = match &args.end_time {
        Some(s) => match parse_flexible_datetime(s, all_day) {
            Ok(dt) => dt,
            Err(e) => return ToolCallResult::error(format!("Invalid end_time: {}", e)),
        },
        None => {
            if all_day {
                match parse_flexible_datetime(&args.start_time, true) {
                    Ok(dt) => dt,
                    Err(_) => start_time,
                }
            } else {
                start_time + chrono::Duration::hours(1)
            }
        }
    };

    if end_time < start_time {
        return ToolCallResult::error("end_time cannot be earlier than start_time");
    }

    // Resolve calendar_id
    let calendar_id = match args.calendar_id {
        Some(id) => {
            match db.get_calendars() {
                Ok(cals) => {
                    if !cals.iter().any(|c| c.id == id) {
                        return ToolCallResult::error(format!("Calendar with ID '{}' not found", id));
                    }
                    id
                }
                Err(e) => return ToolCallResult::error(format!("Failed to verify calendar: {}", e)),
            }
        }
        None => match db.get_default_calendar() {
            Ok(Some(cal)) => cal.id,
            Ok(None) => return ToolCallResult::error("No calendar available in database to add event to"),
            Err(e) => return ToolCallResult::error(format!("Failed to get default calendar: {}", e)),
        },
    };

    let mut event = Event::new(calendar_id, title, start_time, end_time, all_day);
    event.description = args.description;
    event.location = args.location;

    match db.save_event(&event) {
        Ok(_) => {
            let dto = json!({
                "status": "success",
                "message": "Event created successfully",
                "event": {
                    "id": event.id,
                    "calendar_id": event.calendar_id,
                    "title": event.title,
                    "description": event.description,
                    "location": event.location,
                    "start_time": event.start_time.to_rfc3339(),
                    "end_time": event.end_time.to_rfc3339(),
                    "all_day": event.all_day,
                    "time_str": event.time_str(),
                }
            });
            match ToolCallResult::json(&dto) {
                Ok(res) => res,
                Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to save event: {}", e)),
    }
}

#[derive(Deserialize)]
struct UpdateEventArgs {
    id: String,
    title: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    all_day: Option<bool>,
    calendar_id: Option<String>,
    description: Option<String>,
    location: Option<String>,
}

fn handle_update_event(args: serde_json::Value, db: &Database) -> ToolCallResult {
    let args: UpdateEventArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => return ToolCallResult::error(format!("Invalid arguments for update_event: {}", e)),
    };

    let mut event = match db.get_event_by_id(&args.id) {
        Ok(Some(ev)) => ev,
        Ok(None) => return ToolCallResult::error(format!("Event with ID '{}' not found", args.id)),
        Err(e) => return ToolCallResult::error(format!("Database error: {}", e)),
    };

    if let Some(title) = args.title {
        let t = title.trim();
        if t.is_empty() {
            return ToolCallResult::error("Event title cannot be empty");
        }
        event.title = t.to_string();
    }

    if let Some(all_day) = args.all_day {
        event.all_day = all_day;
    }

    if let Some(st_str) = args.start_time {
        match parse_flexible_datetime(&st_str, false) {
            Ok(dt) => event.start_time = dt,
            Err(e) => return ToolCallResult::error(format!("Invalid start_time: {}", e)),
        }
    }

    if let Some(et_str) = args.end_time {
        match parse_flexible_datetime(&et_str, event.all_day) {
            Ok(dt) => event.end_time = dt,
            Err(e) => return ToolCallResult::error(format!("Invalid end_time: {}", e)),
        }
    }

    if event.end_time < event.start_time {
        return ToolCallResult::error("end_time cannot be earlier than start_time");
    }

    if let Some(cal_id) = args.calendar_id {
        match db.get_calendars() {
            Ok(cals) => {
                if !cals.iter().any(|c| c.id == cal_id) {
                    return ToolCallResult::error(format!("Calendar with ID '{}' not found", cal_id));
                }
                event.calendar_id = cal_id;
            }
            Err(e) => return ToolCallResult::error(format!("Failed to verify calendar: {}", e)),
        }
    }

    if args.description.is_some() {
        event.description = args.description;
    }

    if args.location.is_some() {
        event.location = args.location;
    }

    event.updated_at = Local::now();
    if event.sync_status == SyncStatus::Synced {
        event.sync_status = SyncStatus::UpdatedLocally;
    }

    match db.save_event(&event) {
        Ok(_) => {
            let dto = json!({
                "status": "success",
                "message": "Event updated successfully",
                "event": {
                    "id": event.id,
                    "calendar_id": event.calendar_id,
                    "title": event.title,
                    "description": event.description,
                    "location": event.location,
                    "start_time": event.start_time.to_rfc3339(),
                    "end_time": event.end_time.to_rfc3339(),
                    "all_day": event.all_day,
                    "time_str": event.time_str(),
                }
            });
            match ToolCallResult::json(&dto) {
                Ok(res) => res,
                Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to update event: {}", e)),
    }
}

#[derive(Deserialize)]
struct DeleteEventArgs {
    id: String,
}

fn handle_delete_event(args: serde_json::Value, db: &Database) -> ToolCallResult {
    let args: DeleteEventArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => return ToolCallResult::error(format!("Invalid arguments for delete_event: {}", e)),
    };

    match db.get_event_by_id(&args.id) {
        Ok(Some(_)) => match db.delete_event(&args.id) {
            Ok(_) => {
                let dto = json!({
                    "status": "success",
                    "message": "Event deleted successfully",
                    "deleted_id": args.id,
                });
                match ToolCallResult::json(&dto) {
                    Ok(res) => res,
                    Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
                }
            }
            Err(e) => ToolCallResult::error(format!("Failed to delete event: {}", e)),
        },
        Ok(None) => ToolCallResult::error(format!("Event with ID '{}' not found", args.id)),
        Err(e) => ToolCallResult::error(format!("Database error: {}", e)),
    }
}

#[derive(Deserialize)]
struct SearchEventsArgs {
    query: String,
    limit: Option<usize>,
}

fn handle_search_events(args: serde_json::Value, db: &Database) -> ToolCallResult {
    let args: SearchEventsArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => return ToolCallResult::error(format!("Invalid arguments for search_events: {}", e)),
    };

    let query = args.query.trim();
    if query.is_empty() {
        return ToolCallResult::error("Search query cannot be empty");
    }

    match db.search_events(query, args.limit.or(Some(20))) {
        Ok(events) => {
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

            let dto = json!({
                "query": query,
                "count": event_dtos.len(),
                "events": event_dtos,
            });

            match ToolCallResult::json(&dto) {
                Ok(res) => res,
                Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to search events: {}", e)),
    }
}

#[derive(Deserialize)]
struct FindFreeSlotsArgs {
    date: Option<String>,
    duration_minutes: Option<i64>,
    start_hour: Option<u32>,
    end_hour: Option<u32>,
}

fn handle_find_free_slots(args: serde_json::Value, db: &Database) -> ToolCallResult {
    let args: FindFreeSlotsArgs = match serde_json::from_value(args) {
        Ok(a) => a,
        Err(e) => return ToolCallResult::error(format!("Invalid arguments for find_free_slots: {}", e)),
    };

    let target_date = match &args.date {
        Some(s) => match parse_flexible_date(s) {
            Ok(d) => d,
            Err(e) => return ToolCallResult::error(format!("Invalid date: {}", e)),
        },
        None => Local::now().date_naive(),
    };

    let duration_mins = args.duration_minutes.unwrap_or(30).clamp(5, 720);
    let start_hour = args.start_hour.unwrap_or(9).clamp(0, 23);
    let end_hour = args.end_hour.unwrap_or(18).clamp(start_hour + 1, 24);

    let window_start = match Local.from_local_datetime(&target_date.and_hms_opt(start_hour, 0, 0).unwrap()) {
        chrono::LocalResult::Single(dt) => dt,
        _ => return ToolCallResult::error("Failed to construct local window start time"),
    };

    let window_end = if end_hour == 24 {
        match Local.from_local_datetime(&target_date.and_hms_opt(23, 59, 59).unwrap()) {
            chrono::LocalResult::Single(dt) => dt,
            _ => return ToolCallResult::error("Failed to construct local window end time"),
        }
    } else {
        match Local.from_local_datetime(&target_date.and_hms_opt(end_hour, 0, 0).unwrap()) {
            chrono::LocalResult::Single(dt) => dt,
            _ => return ToolCallResult::error("Failed to construct local window end time"),
        }
    };

    let events = match db.get_events_filtered(Some(window_start), Some(window_end), None, None, None) {
        Ok(evs) => evs,
        Err(e) => return ToolCallResult::error(format!("Failed to retrieve events: {}", e)),
    };

    // Filter to timed events that intersect with [window_start, window_end]
    let mut busy_intervals: Vec<(DateTime<Local>, DateTime<Local>)> = Vec::new();
    for ev in events {
        if ev.all_day {
            continue;
        }
        let clamped_start = max(ev.start_time, window_start);
        let clamped_end = min(ev.end_time, window_end);
        if clamped_start < clamped_end {
            busy_intervals.push((clamped_start, clamped_end));
        }
    }

    // Sort intervals by start time
    busy_intervals.sort_by_key(|&(s, _)| s);

    // Merge overlapping/adjacent intervals
    let mut merged: Vec<(DateTime<Local>, DateTime<Local>)> = Vec::new();
    for interval in busy_intervals {
        if let Some(last) = merged.last_mut() {
            if interval.0 <= last.1 {
                if interval.1 > last.1 {
                    last.1 = interval.1;
                }
                continue;
            }
        }
        merged.push(interval);
    }

    // Compute gaps
    let mut free_slots = Vec::new();
    let mut cursor = window_start;

    for (busy_start, busy_end) in merged {
        if busy_start > cursor {
            let gap_mins = (busy_start - cursor).num_minutes();
            if gap_mins >= duration_mins {
                free_slots.push(json!({
                    "start_time": cursor.to_rfc3339(),
                    "end_time": busy_start.to_rfc3339(),
                    "start_formatted": cursor.format("%H:%M").to_string(),
                    "end_formatted": busy_start.format("%H:%M").to_string(),
                    "duration_minutes": gap_mins,
                }));
            }
        }
        cursor = max(cursor, busy_end);
    }

    if window_end > cursor {
        let gap_mins = (window_end - cursor).num_minutes();
        if gap_mins >= duration_mins {
            free_slots.push(json!({
                "start_time": cursor.to_rfc3339(),
                "end_time": window_end.to_rfc3339(),
                "start_formatted": cursor.format("%H:%M").to_string(),
                "end_formatted": window_end.format("%H:%M").to_string(),
                "duration_minutes": gap_mins,
            }));
        }
    }

    let dto = json!({
        "date": target_date.format("%Y-%m-%d").to_string(),
        "requested_duration_minutes": duration_mins,
        "working_window": format!("{:02}:00 - {:02}:00", start_hour, end_hour),
        "free_slots_count": free_slots.len(),
        "free_slots": free_slots,
    });

    match ToolCallResult::json(&dto) {
        Ok(res) => res,
        Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
    }
}

fn handle_get_current_time() -> ToolCallResult {
    let now = Local::now();
    let dto = json!({
        "iso": now.to_rfc3339(),
        "date": now.format("%Y-%m-%d").to_string(),
        "time": now.format("%H:%M:%S").to_string(),
        "weekday": now.weekday().to_string(),
        "timezone": now.offset().to_string(),
        "unix_timestamp": now.timestamp(),
    });
    match ToolCallResult::json(&dto) {
        Ok(res) => res,
        Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
    }
}

async fn handle_sync_calendars(db: &Database) -> ToolCallResult {
    let res = SyncEngine::sync_all(db).await;
    let dto = json!({
        "status": "success",
        "accounts_synced": res.accounts_synced,
        "events_pushed": res.events_pushed,
        "events_pulled": res.events_pulled,
        "errors": res.errors,
    });
    match ToolCallResult::json(&dto) {
        Ok(r) => r,
        Err(e) => ToolCallResult::error(format!("Serialization error: {}", e)),
    }
}

// ----------------------------------------------------------------------------
// Date Parsing Helpers
// ----------------------------------------------------------------------------

/// Parses a date or datetime string with flexible formatting into `DateTime<Local>`.
pub fn parse_flexible_datetime(s: &str, is_end_of_day: bool) -> Result<DateTime<Local>> {
    let s = s.trim();

    // 1. RFC3339 format with offset (e.g. 2026-11-01T20:00:00+09:00 or 2026-11-01T20:00:00Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Local));
    }

    // 2. Naive DateTime formats
    let formats = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M",
    ];
    for fmt in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
            match Local.from_local_datetime(&naive) {
                chrono::LocalResult::Single(dt) => return Ok(dt),
                chrono::LocalResult::Ambiguous(dt, _) => return Ok(dt),
                chrono::LocalResult::None => {
                    let adjusted = naive + chrono::Duration::hours(1);
                    if let chrono::LocalResult::Single(dt) = Local.from_local_datetime(&adjusted) {
                        return Ok(dt);
                    }
                }
            }
        }
    }

    // 3. Date-only format (e.g. "2026-11-01")
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let naive = if is_end_of_day {
            date.and_hms_opt(23, 59, 59).unwrap()
        } else {
            date.and_hms_opt(0, 0, 0).unwrap()
        };
        match Local.from_local_datetime(&naive) {
            chrono::LocalResult::Single(dt) => return Ok(dt),
            chrono::LocalResult::Ambiguous(dt, _) => return Ok(dt),
            chrono::LocalResult::None => {
                let adjusted = naive + chrono::Duration::hours(1);
                if let chrono::LocalResult::Single(dt) = Local.from_local_datetime(&adjusted) {
                    return Ok(dt);
                }
            }
        }
    }

    Err(anyhow!(
        "Failed to parse datetime string: '{}'. Expected ISO 8601 (e.g. 'YYYY-MM-DDTHH:MM:SS' or 'YYYY-MM-DD').",
        s
    ))
}

/// Parses a date string into `NaiveDate`.
pub fn parse_flexible_date(s: &str) -> Result<NaiveDate> {
    let s = s.trim();
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date);
    }
    if let Ok(dt) = parse_flexible_datetime(s, false) {
        return Ok(dt.date_naive());
    }
    Err(anyhow!(
        "Failed to parse date string: '{}'. Expected 'YYYY-MM-DD'.",
        s
    ))
}
