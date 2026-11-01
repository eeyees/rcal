use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::model::{Event, SyncStatus};
use crate::sync::caldav::RemoteCalendarInfo;

#[derive(Debug, Deserialize)]
struct GraphCalendarListResponse {
    value: Option<Vec<GraphCalendarItem>>,
}

#[derive(Debug, Deserialize)]
struct GraphCalendarItem {
    id: String,
    name: String,
    #[serde(rename = "hexColor")]
    hex_color: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphEventsResponse {
    value: Option<Vec<GraphEventItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphDateTimeTimeZone {
    #[serde(rename = "dateTime")]
    date_time: String,
    #[serde(rename = "timeZone")]
    time_zone: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphLocation {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphEventItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "changeKey", skip_serializing_if = "Option::is_none")]
    change_key: Option<String>,
    subject: Option<String>,
    #[serde(rename = "bodyPreview", skip_serializing_if = "Option::is_none")]
    body_preview: Option<String>,
    location: Option<GraphLocation>,
    #[serde(rename = "isAllDay")]
    is_all_day: Option<bool>,
    start: Option<GraphDateTimeTimeZone>,
    end: Option<GraphDateTimeTimeZone>,
}

pub struct MicrosoftCalendarClient {
    client: Client,
    #[allow(dead_code)]
    access_token: String,
}

impl MicrosoftCalendarClient {
    pub fn new(access_token: String) -> Self {
        let mut headers = HeaderMap::new();
        let ua = format!("rcal/{}", crate::version());
        if let Ok(hv) = HeaderValue::from_str(&ua) {
            headers.insert(USER_AGENT, hv);
        }
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
        let url = "https://graph.microsoft.com/v1.0/me/calendars";
        let res = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| "Failed to query Microsoft Graph calendars")?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Microsoft Graph API error: {}", err));
        }

        let data: GraphCalendarListResponse = res.json().await?;
        let mut list = Vec::new();

        if let Some(items) = data.value {
            for item in items {
                list.push(RemoteCalendarInfo {
                    name: item.name,
                    href: item.id,
                    color_code: item.hex_color.or_else(|| Some("#0078D4".to_string())),
                    ctag: None,
                });
            }
        }

        Ok(list)
    }

    pub async fn list_events(
        &self,
        calendar_id: &str,
        local_calendar_id: &str,
    ) -> Result<Vec<Event>> {
        let url = format!(
            "https://graph.microsoft.com/v1.0/me/calendars/{}/events?$top=250",
            calendar_id
        );

        let res = self.client.get(&url).send().await.with_context(|| {
            format!(
                "Failed to query events from Microsoft calendar {}",
                calendar_id
            )
        })?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Microsoft Events API error: {}", err));
        }

        let data: GraphEventsResponse = res.json().await?;
        let mut events = Vec::new();

        if let Some(items) = data.value {
            for item in items {
                if let (Some(start_time), Some(end_time)) = parse_graph_time(&item.start, &item.end)
                {
                    let all_day = item.is_all_day.unwrap_or(false);
                    let location = item.location.and_then(|l| l.display_name);

                    let ev = Event {
                        id: uuid::Uuid::new_v4().to_string(),
                        calendar_id: local_calendar_id.to_string(),
                        title: item.subject.unwrap_or_else(|| "(No title)".to_string()),
                        description: item.body_preview,
                        location,
                        start_time,
                        end_time,
                        all_day,
                        rrule: None,
                        sync_status: SyncStatus::Synced,
                        remote_id: item.id,
                        etag: item.change_key,
                        updated_at: Local::now(),
                    };
                    events.push(ev);
                }
            }
        }

        Ok(events)
    }

    pub async fn insert_event(
        &self,
        calendar_id: &str,
        event: &Event,
    ) -> Result<(String, Option<String>)> {
        let url = format!(
            "https://graph.microsoft.com/v1.0/me/calendars/{}/events",
            calendar_id
        );

        let payload = event_to_graph_payload(event);
        let res = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Microsoft Insert Event error: {}", err));
        }

        let created: GraphEventItem = res.json().await?;
        let remote_id = created
            .id
            .ok_or_else(|| anyhow!("Missing created event ID"))?;
        Ok((remote_id, created.change_key))
    }

    pub async fn update_event(
        &self,
        calendar_id: &str,
        remote_id: &str,
        event: &Event,
    ) -> Result<Option<String>> {
        let url = format!(
            "https://graph.microsoft.com/v1.0/me/calendars/{}/events/{}",
            calendar_id, remote_id
        );

        let payload = event_to_graph_payload(event);
        let res = self
            .client
            .patch(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Microsoft Update Event error: {}", err));
        }

        let updated: GraphEventItem = res.json().await?;
        Ok(updated.change_key)
    }

    pub async fn delete_event(&self, calendar_id: &str, remote_id: &str) -> Result<()> {
        let url = format!(
            "https://graph.microsoft.com/v1.0/me/calendars/{}/events/{}",
            calendar_id, remote_id
        );

        let res = self.client.delete(&url).send().await?;
        if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
            let err = res.text().await.unwrap_or_default();
            return Err(anyhow!("Microsoft Delete Event error: {}", err));
        }
        Ok(())
    }
}

fn parse_graph_time(
    start_opt: &Option<GraphDateTimeTimeZone>,
    end_opt: &Option<GraphDateTimeTimeZone>,
) -> (Option<DateTime<Local>>, Option<DateTime<Local>>) {
    if let (Some(s), Some(e)) = (start_opt, end_opt) {
        let s_dt = DateTime::parse_from_rfc3339(&s.date_time)
            .ok()
            .map(|dt| dt.with_timezone(&Local))
            .or_else(|| {
                NaiveDateTime::parse_from_str(&s.date_time, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .and_then(|ndt| Local.from_local_datetime(&ndt).single())
            });

        let e_dt = DateTime::parse_from_rfc3339(&e.date_time)
            .ok()
            .map(|dt| dt.with_timezone(&Local))
            .or_else(|| {
                NaiveDateTime::parse_from_str(&e.date_time, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .and_then(|ndt| Local.from_local_datetime(&ndt).single())
            });

        return (s_dt, e_dt);
    }
    (None, None)
}

fn event_to_graph_payload(event: &Event) -> GraphEventItem {
    GraphEventItem {
        id: event.remote_id.clone(),
        change_key: None,
        subject: Some(event.title.clone()),
        body_preview: event.description.clone(),
        location: event.location.as_ref().map(|l| GraphLocation {
            display_name: Some(l.clone()),
        }),
        is_all_day: Some(event.all_day),
        start: Some(GraphDateTimeTimeZone {
            date_time: event.start_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            time_zone: "UTC".to_string(),
        }),
        end: Some(GraphDateTimeTimeZone {
            date_time: event.end_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            time_zone: "UTC".to_string(),
        }),
    }
}
