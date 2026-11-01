use anyhow::{Context, Result};
use chrono::Local;

use crate::model::{AuthType, CalDavAccount, Calendar, ProviderType, SyncStatus};
use crate::storage::Database;
use crate::sync::caldav::CalDavClient;
use crate::sync::google::GoogleCalendarClient;
use crate::sync::ical;
use crate::sync::microsoft::MicrosoftCalendarClient;

#[derive(Debug, Default, Clone)]
pub struct SyncResult {
    pub accounts_synced: usize,
    pub events_pushed: usize,
    pub events_pulled: usize,
    pub errors: Vec<String>,
}

pub struct SyncEngine;

impl SyncEngine {
    /// Synchronize all registered accounts with local database
    pub async fn sync_all(db: &Database) -> SyncResult {
        let mut result = SyncResult::default();

        let accounts = match db.get_accounts() {
            Ok(accs) => accs,
            Err(e) => {
                result.errors.push(format!("Failed to read accounts: {}", e));
                return result;
            }
        };

        if accounts.is_empty() {
            return result;
        }

        for acc in &accounts {
            match Self::sync_account(db, acc).await {
                Ok((pushed, pulled)) => {
                    result.accounts_synced += 1;
                    result.events_pushed += pushed;
                    result.events_pulled += pulled;

                    let mut updated_acc = acc.clone();
                    updated_acc.last_synced_at = Some(Local::now());
                    let _ = db.save_account(&updated_acc);
                }
                Err(err) => {
                    result.errors.push(format!("{}: {}", acc.name, err));
                }
            }
        }

        result
    }

    pub async fn sync_account(db: &Database, account: &CalDavAccount) -> Result<(usize, usize)> {
        match account.provider {
            ProviderType::Google => Self::sync_google(db, account).await,
            ProviderType::Microsoft => Self::sync_microsoft(db, account).await,
            ProviderType::Samsung => {
                if account.auth_type == AuthType::OAuth2 {
                    Self::sync_microsoft(db, account).await
                } else {
                    Self::sync_caldav(db, account).await
                }
            }
            ProviderType::Apple | ProviderType::Naver | ProviderType::Local => {
                Self::sync_caldav(db, account).await
            }
        }
    }

    // Google Calendar Sync
    async fn sync_google(db: &Database, account: &CalDavAccount) -> Result<(usize, usize)> {
        let token = account.access_token.as_deref().unwrap_or(&account.password);
        let client = GoogleCalendarClient::new(token.to_string());

        let remote_cals = client.list_calendars().await
            .with_context(|| "Google calendar list failed")?;

        let local_cals = db.get_calendars()?;
        let mut active_calendar_mappings = Vec::new();

        for r_cal in remote_cals {
            let existing = local_cals.iter().find(|c| {
                c.account_id.as_deref() == Some(&account.id)
                    && c.remote_url.as_deref() == Some(&r_cal.href)
            });

            let local_cal_id = if let Some(found) = existing {
                found.id.clone()
            } else {
                let new_cal = Calendar {
                    id: format!("cal_google_{}", uuid::Uuid::new_v4()),
                    account_id: Some(account.id.clone()),
                    name: format!("{} (Google)", r_cal.name),
                    color_code: r_cal.color_code.unwrap_or_else(|| "#4285F4".to_string()),
                    provider: ProviderType::Google,
                    enabled: true,
                    is_default: false,
                    remote_url: Some(r_cal.href.clone()),
                    ctag: None,
                };
                db.save_calendar(&new_cal)?;
                new_cal.id
            };

            active_calendar_mappings.push((local_cal_id, r_cal.href));
        }

        let mut total_pushed = 0;
        let mut total_pulled = 0;

        for (local_cal_id, google_cal_id) in active_calendar_mappings {
            // Push pending local events
            let pending_events = db.get_pending_sync_events(&local_cal_id)?;
            for mut ev in pending_events {
                match ev.sync_status {
                    SyncStatus::CreatedLocally => {
                        if let Ok((remote_id, etag)) = client.insert_event(&google_cal_id, &ev).await {
                            ev.remote_id = Some(remote_id);
                            ev.etag = etag;
                            ev.sync_status = SyncStatus::Synced;
                            db.save_event(&ev)?;
                            total_pushed += 1;
                        }
                    }
                    SyncStatus::UpdatedLocally => {
                        if let Some(ref r_id) = ev.remote_id {
                            if let Ok(etag) = client.update_event(&google_cal_id, r_id, &ev).await {
                                ev.etag = etag;
                                ev.sync_status = SyncStatus::Synced;
                                db.save_event(&ev)?;
                                total_pushed += 1;
                            }
                        }
                    }
                    SyncStatus::DeletedLocally => {
                        if let Some(ref r_id) = ev.remote_id {
                            let _ = client.delete_event(&google_cal_id, r_id).await;
                        }
                        db.delete_event(&ev.id)?;
                        total_pushed += 1;
                    }
                    SyncStatus::Synced => {}
                }
            }

            // Pull events from Google
            let events = client.list_events(&google_cal_id, &local_cal_id).await
                .with_context(|| format!("Failed to list events for Google calendar {}", google_cal_id))?;
            for g_ev in events {
                let all_events = db.get_all_events()?;
                let existing = all_events.iter().find(|e| {
                    e.calendar_id == local_cal_id && e.remote_id == g_ev.remote_id
                });

                if let Some(existing_ev) = existing {
                    if existing_ev.etag != g_ev.etag {
                        let mut updated = g_ev;
                        updated.id = existing_ev.id.clone();
                        db.save_event(&updated)?;
                        total_pulled += 1;
                    }
                } else {
                    db.save_event(&g_ev)?;
                    total_pulled += 1;
                }
            }
        }

        Ok((total_pushed, total_pulled))
    }

    // Microsoft 365 Sync
    async fn sync_microsoft(db: &Database, account: &CalDavAccount) -> Result<(usize, usize)> {
        let token = account.access_token.as_deref().unwrap_or(&account.password);
        let client = MicrosoftCalendarClient::new(token.to_string());

        let remote_cals = client.list_calendars().await
            .with_context(|| "Microsoft calendar list failed")?;

        let local_cals = db.get_calendars()?;
        let mut active_calendar_mappings = Vec::new();

        for r_cal in remote_cals {
            let existing = local_cals.iter().find(|c| {
                c.account_id.as_deref() == Some(&account.id)
                    && c.remote_url.as_deref() == Some(&r_cal.href)
            });

            let local_cal_id = if let Some(found) = existing {
                found.id.clone()
            } else {
                let default_color = if account.provider == ProviderType::Samsung {
                    "#034EA2"
                } else {
                    "#0078D4"
                };
                let suffix = if account.provider == ProviderType::Samsung { "Samsung" } else { "MS 365" };
                let new_cal = Calendar {
                    id: format!("cal_ms_{}", uuid::Uuid::new_v4()),
                    account_id: Some(account.id.clone()),
                    name: format!("{} ({})", r_cal.name, suffix),
                    color_code: r_cal.color_code.unwrap_or_else(|| default_color.to_string()),
                    provider: account.provider,
                    enabled: true,
                    is_default: false,
                    remote_url: Some(r_cal.href.clone()),
                    ctag: None,
                };
                db.save_calendar(&new_cal)?;
                new_cal.id
            };

            active_calendar_mappings.push((local_cal_id, r_cal.href));
        }

        let mut total_pushed = 0;
        let mut total_pulled = 0;

        for (local_cal_id, ms_cal_id) in active_calendar_mappings {
            // Push pending local events
            let pending_events = db.get_pending_sync_events(&local_cal_id)?;
            for mut ev in pending_events {
                match ev.sync_status {
                    SyncStatus::CreatedLocally => {
                        if let Ok((remote_id, change_key)) = client.insert_event(&ms_cal_id, &ev).await {
                            ev.remote_id = Some(remote_id);
                            ev.etag = change_key;
                            ev.sync_status = SyncStatus::Synced;
                            db.save_event(&ev)?;
                            total_pushed += 1;
                        }
                    }
                    SyncStatus::UpdatedLocally => {
                        if let Some(ref r_id) = ev.remote_id {
                            if let Ok(change_key) = client.update_event(&ms_cal_id, r_id, &ev).await {
                                ev.etag = change_key;
                                ev.sync_status = SyncStatus::Synced;
                                db.save_event(&ev)?;
                                total_pushed += 1;
                            }
                        }
                    }
                    SyncStatus::DeletedLocally => {
                        if let Some(ref r_id) = ev.remote_id {
                            let _ = client.delete_event(&ms_cal_id, r_id).await;
                        }
                        db.delete_event(&ev.id)?;
                        total_pushed += 1;
                    }
                    SyncStatus::Synced => {}
                }
            }

            // Pull events from Microsoft
            let events = client.list_events(&ms_cal_id, &local_cal_id).await
                .with_context(|| format!("Failed to list events for Microsoft calendar {}", ms_cal_id))?;
            for ms_ev in events {
                let all_events = db.get_all_events()?;
                let existing = all_events.iter().find(|e| {
                    e.calendar_id == local_cal_id && e.remote_id == ms_ev.remote_id
                });

                if let Some(existing_ev) = existing {
                    if existing_ev.etag != ms_ev.etag {
                        let mut updated = ms_ev;
                        updated.id = existing_ev.id.clone();
                        db.save_event(&updated)?;
                        total_pulled += 1;
                    }
                } else {
                    db.save_event(&ms_ev)?;
                    total_pulled += 1;
                }
            }
        }

        Ok((total_pushed, total_pulled))
    }

    // CalDAV Sync (Apple iCloud, Naver, Samsung CalDAV, Generic)
    async fn sync_caldav(db: &Database, account: &CalDavAccount) -> Result<(usize, usize)> {
        let client = CalDavClient::new(&account.url, account.username.clone(), account.password.clone())
            .with_context(|| format!("Failed to create client for {}", account.name))?;

        let remote_cals = client.discover_calendars().await
            .with_context(|| "Calendar discovery failed")?;

        let local_cals = db.get_calendars()?;
        let mut active_calendar_mappings = Vec::new();

        for r_cal in remote_cals {
            let existing = local_cals.iter().find(|c| {
                c.account_id.as_deref() == Some(&account.id)
                    && c.remote_url.as_deref() == Some(&r_cal.href)
            });

            let local_cal_id = if let Some(found) = existing {
                found.id.clone()
            } else {
                let default_color = match account.provider {
                    ProviderType::Naver => "#03C75A",
                    ProviderType::Apple => "#4285F4",
                    ProviderType::Samsung => "#034EA2",
                    _ => "#00ACC1",
                };
                let new_cal = Calendar {
                    id: format!("cal_remote_{}", uuid::Uuid::new_v4()),
                    account_id: Some(account.id.clone()),
                    name: format!("{} ({})", r_cal.name, account.name),
                    color_code: r_cal.color_code.unwrap_or_else(|| default_color.to_string()),
                    provider: account.provider,
                    enabled: true,
                    is_default: false,
                    remote_url: Some(r_cal.href.clone()),
                    ctag: r_cal.ctag.clone(),
                };
                db.save_calendar(&new_cal)?;
                new_cal.id
            };

            active_calendar_mappings.push((local_cal_id, r_cal.href));
        }

        let mut total_pushed = 0;
        let mut total_pulled = 0;

        for (local_cal_id, remote_href) in active_calendar_mappings {
            // Push Local Changes to Remote
            let pending_events = db.get_pending_sync_events(&local_cal_id)?;
            for mut ev in pending_events {
                match ev.sync_status {
                    SyncStatus::CreatedLocally => {
                        let uid = ev.remote_id.clone().unwrap_or_else(|| ev.id.clone());
                        let ics = ical::event_to_ics(&ev);
                        if let Ok(new_etag) = client.upload_event(&remote_href, &uid, &ics, None).await {
                            ev.remote_id = Some(uid);
                            ev.etag = new_etag;
                            ev.sync_status = SyncStatus::Synced;
                            db.save_event(&ev)?;
                            total_pushed += 1;
                        }
                    }
                    SyncStatus::UpdatedLocally => {
                        let uid = ev.remote_id.clone().unwrap_or_else(|| ev.id.clone());
                        let ics = ical::event_to_ics(&ev);
                        if let Ok(new_etag) = client.upload_event(&remote_href, &uid, &ics, ev.etag.as_deref()).await {
                            ev.etag = new_etag;
                            ev.sync_status = SyncStatus::Synced;
                            db.save_event(&ev)?;
                            total_pushed += 1;
                        }
                    }
                    SyncStatus::DeletedLocally => {
                        if let Some(ref r_id) = ev.remote_id {
                            let event_href = if remote_href.ends_with('/') {
                                format!("{}{}.ics", remote_href, r_id)
                            } else {
                                format!("{}/{}.ics", remote_href, r_id)
                            };
                            let _ = client.delete_event(&event_href, ev.etag.as_deref()).await;
                        }
                        db.delete_event(&ev.id)?;
                        total_pushed += 1;
                    }
                    SyncStatus::Synced => {}
                }
            }

            // Pull Remote Changes from Server
            let remote_events = client.fetch_events(&remote_href).await
                .with_context(|| format!("Failed to fetch events from {}", remote_href))?;

            for r_ev in remote_events {
                let parsed_events = ical::ics_to_events(&r_ev.ics_content, &local_cal_id, &r_ev.href, r_ev.etag.as_deref());
                for p_ev in parsed_events {
                    let all_events = db.get_all_events()?;
                    let existing = all_events.iter().find(|e| {
                        e.calendar_id == local_cal_id && e.remote_id == p_ev.remote_id
                    });

                    if let Some(existing_ev) = existing {
                        if existing_ev.etag != p_ev.etag {
                            let mut updated = p_ev;
                            updated.id = existing_ev.id.clone();
                            db.save_event(&updated)?;
                            total_pulled += 1;
                        }
                    } else {
                        db.save_event(&p_ev)?;
                        total_pulled += 1;
                    }
                }
            }
        }

        Ok((total_pushed, total_pulled))
    }
}
