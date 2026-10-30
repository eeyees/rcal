use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::model::{Event, SyncStatus};
use crate::sync::caldav::RemoteCalendarInfo;

#[derive(Debug, Deserialize)]
struct GoogleCalendarListResponse {
    items: Option<Vec<GoogleCalendarItem>>,
}

#[derive(Debug, Deserialize)]
struct GoogleCalendarItem {
    id: String,
    summary: String,
    #[serde(rename = "backgroundColor")]
    background_color: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleEventsListResponse {
    items: Option<Vec<GoogleEventItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleEventTime {
    #[serde(rename = "dateTime", skip_serializing_if = "Option::is_none")]
    date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleEventItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    etag: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    location: Option<String>,
    start: Option<GoogleEventTime>,
    end: Option<GoogleEventTime>,
}

pub struct GoogleCalendarClient {
    client: Client,
    #[allow(dead_code)]
    access_token: String,
}

impl GoogleCalendarClient {
    pub fn new(access_token: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rcal/0.1.0"));
        let auth_val = format!("Bearer {}", access_token);
        if let Ok(hv) = HeaderValue::from_str(&auth_val) {
            headers.insert(AUTHORIZATION, hv);
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        Self {
            client,
            access_token,
        }
    }

    pub async fn list_calendars(&self) -> Result<Vec<RemoteCalendarInfo>> {
        let url = "https://www.googleapis.com/calendar/v3/users/me/calendarList";
        let res = self.client.get(url).send().await
            .with_context(|| "Failed to query Google Calendar list")?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Google API error: {}", err));
        }

        let data: GoogleCalendarListResponse = res.json().await?;
        let mut list = Vec::new();

        if let Some(items) = data.items {
            for item in items {
                list.push(RemoteCalendarInfo {
                    name: item.summary,
                    href: item.id,
                    color_code: item.background_color.or_else(|| Some("#4285F4".to_string())),
                    ctag: None,
                });
            }
        }

        Ok(list)
    }

    pub async fn list_events(&self, calendar_id: &str, local_calendar_id: &str) -> Result<Vec<Event>> {
        let encoded_cal_id = urlencoding::encode(calendar_id);
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events?singleEvents=true&maxResults=250",
            encoded_cal_id
        );

        let res = self.client.get(&url).send().await
            .with_context(|| format!("Failed to query events for Google calendar {}", calendar_id))?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Google Events API error: {}", err));
        }

        let data: GoogleEventsListResponse = res.json().await?;
        let mut events = Vec::new();

        if let Some(items) = data.items {
            for item in items {
                if let (Some(start_time), Some(end_time), all_day) = parse_google_time(&item.start, &item.end) {
                    let ev = Event {
                        id: uuid::Uuid::new_v4().to_string(),
                        calendar_id: local_calendar_id.to_string(),
                        title: item.summary.unwrap_or_else(|| "(No title)".to_string()),
                        description: item.description,
                        location: item.location,
                        start_time,
                        end_time,
                        all_day,
                        rrule: None,
                        sync_status: SyncStatus::Synced,
                        remote_id: item.id,
                        etag: item.etag.map(|s| s.trim_matches('"').to_string()),
                        updated_at: Local::now(),
                    };
                    events.push(ev);
                }
            }
        }

        Ok(events)
    }

    pub async fn insert_event(&self, calendar_id: &str, event: &Event) -> Result<(String, Option<String>)> {
        let encoded_cal_id = urlencoding::encode(calendar_id);
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            encoded_cal_id
        );

        let payload = event_to_google_payload(event);
        let res = self.client.post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Google Insert Event error: {}", err));
        }

        let created: GoogleEventItem = res.json().await?;
        let remote_id = created.id.ok_or_else(|| anyhow!("Missing created event ID"))?;
        let etag = created.etag.map(|s| s.trim_matches('"').to_string());
        Ok((remote_id, etag))
    }

    pub async fn update_event(&self, calendar_id: &str, remote_id: &str, event: &Event) -> Result<Option<String>> {
        let encoded_cal_id = urlencoding::encode(calendar_id);
        let encoded_ev_id = urlencoding::encode(remote_id);
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            encoded_cal_id, encoded_ev_id
        );

        let payload = event_to_google_payload(event);
        let res = self.client.put(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Google Update Event error: {}", err));
        }

        let updated: GoogleEventItem = res.json().await?;
        Ok(updated.etag.map(|s| s.trim_matches('"').to_string()))
    }

    pub async fn delete_event(&self, calendar_id: &str, remote_id: &str) -> Result<()> {
        let encoded_cal_id = urlencoding::encode(calendar_id);
        let encoded_ev_id = urlencoding::encode(remote_id);
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            encoded_cal_id, encoded_ev_id
        );

        let res = self.client.delete(&url).send().await?;
        if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Google Delete Event error: {}", err));
        }
        Ok(())
    }
}

fn parse_google_time(
    start_opt: &Option<GoogleEventTime>,
    end_opt: &Option<GoogleEventTime>,
) -> (Option<DateTime<Local>>, Option<DateTime<Local>>, bool) {
    if let (Some(s), Some(e)) = (start_opt, end_opt) {
        if let (Some(s_dt), Some(e_dt)) = (&s.date_time, &e.date_time) {
            let start = DateTime::parse_from_rfc3339(s_dt).ok().map(|dt| dt.with_timezone(&Local));
            let end = DateTime::parse_from_rfc3339(e_dt).ok().map(|dt| dt.with_timezone(&Local));
            return (start, end, false);
        }

        if let (Some(s_d), Some(e_d)) = (&s.date, &e.date) {
            if let (Ok(s_naive), Ok(e_naive)) = (
                NaiveDate::parse_from_str(s_d, "%Y-%m-%d"),
                NaiveDate::parse_from_str(e_d, "%Y-%m-%d"),
            ) {
                let start = Local.from_local_datetime(&s_naive.and_hms_opt(0, 0, 0).unwrap()).single();
                let end = Local.from_local_datetime(&e_naive.and_hms_opt(23, 59, 59).unwrap()).single();
                return (start, end, true);
            }
        }
    }
    (None, None, false)
}

fn event_to_google_payload(event: &Event) -> GoogleEventItem {
    let (start, end) = if event.all_day {
        (
            GoogleEventTime {
                date_time: None,
                date: Some(event.start_time.format("%Y-%m-%d").to_string()),
            },
            GoogleEventTime {
                date_time: None,
                date: Some(event.end_time.format("%Y-%m-%d").to_string()),
            },
        )
    } else {
        (
            GoogleEventTime {
                date_time: Some(event.start_time.to_rfc3339()),
                date: None,
            },
            GoogleEventTime {
                date_time: Some(event.end_time.to_rfc3339()),
                date: None,
            },
        )
    };

    GoogleEventItem {
        id: event.remote_id.clone(),
        etag: None,
        summary: Some(event.title.clone()),
        description: event.description.clone(),
        location: event.location.clone(),
        start: Some(start),
        end: Some(end),
    }
}

mod urlencoding {
    pub fn encode(data: &str) -> String {
        let mut encoded = String::new();
        for b in data.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(b as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", b));
                }
            }
        }
        encoded
    }
}
