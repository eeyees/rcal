use std::path::{Path, PathBuf};
use std::sync::Mutex;
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{AuthType, CalDavAccount, Calendar, Event, ProviderType, SyncStatus};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(custom_path: Option<&Path>) -> Result<Self> {
        let db_path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => Self::default_db_path()?,
        };

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open SQLite database at {:?}", db_path))?;

        // Enable WAL mode for high performance concurrent reads
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;

        let db = Self { conn: Mutex::new(conn) };
        db.init_schema()?;
        db.ensure_default_data()?;

        Ok(db)
    }

    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn: Mutex::new(conn) };
        db.init_schema()?;
        db.ensure_default_data()?;
        Ok(db)
    }

    fn default_db_path() -> Result<PathBuf> {
        let mut path = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().map(|h| h.join(".local/share")).unwrap_or_else(|| PathBuf::from(".")));
        path.push("rcal");
        path.push("rcal.db");
        Ok(path)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                url TEXT NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                auth_type TEXT NOT NULL DEFAULT 'basic',
                client_id TEXT,
                access_token TEXT,
                refresh_token TEXT,
                token_expires_at TEXT,
                last_synced_at TEXT
            );

            CREATE TABLE IF NOT EXISTS calendars (
                id TEXT PRIMARY KEY,
                account_id TEXT,
                name TEXT NOT NULL,
                color_code TEXT NOT NULL,
                provider TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                is_default INTEGER NOT NULL DEFAULT 0,
                remote_url TEXT,
                ctag TEXT,
                FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                calendar_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                all_day INTEGER NOT NULL DEFAULT 0,
                rrule TEXT,
                sync_status TEXT NOT NULL,
                remote_id TEXT,
                etag TEXT,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (calendar_id) REFERENCES calendars(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_events_start_end ON events(start_time, end_time);
            CREATE INDEX IF NOT EXISTS idx_events_calendar ON events(calendar_id);
            "#,
        )?;

        let _ = conn.execute("ALTER TABLE calendars ADD COLUMN account_id TEXT", []);
        let _ = conn.execute("ALTER TABLE calendars ADD COLUMN remote_url TEXT", []);
        let _ = conn.execute("ALTER TABLE calendars ADD COLUMN ctag TEXT", []);

        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN auth_type TEXT NOT NULL DEFAULT 'basic'", []);
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN client_id TEXT", []);
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN access_token TEXT", []);
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN refresh_token TEXT", []);
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN token_expires_at TEXT", []);

        Ok(())
    }

    fn ensure_default_data(&self) -> Result<()> {
        let count: i64 = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT COUNT(*) FROM calendars",
                [],
                |row| row.get(0),
            )?
        };

        if count == 0 {
            let default_calendars = vec![
                Calendar {
                    id: "cal_personal".to_string(),
                    account_id: None,
                    name: "Personal".to_string(),
                    color_code: "#4285F4".to_string(),
                    provider: ProviderType::Local,
                    enabled: true,
                    is_default: true,
                    remote_url: None,
                    ctag: None,
                },
                Calendar {
                    id: "cal_work".to_string(),
                    account_id: None,
                    name: "Work".to_string(),
                    color_code: "#0F9D58".to_string(),
                    provider: ProviderType::Local,
                    enabled: true,
                    is_default: false,
                    remote_url: None,
                    ctag: None,
                },
                Calendar {
                    id: "cal_holidays".to_string(),
                    account_id: None,
                    name: "Holidays".to_string(),
                    color_code: "#F4B400".to_string(),
                    provider: ProviderType::Local,
                    enabled: true,
                    is_default: false,
                    remote_url: None,
                    ctag: None,
                },
            ];

            for cal in &default_calendars {
                self.save_calendar(cal)?;
            }

            // Seed sample events relative to today
            let today = Local::now().date_naive();
            
            let sample_events = vec![
                Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    calendar_id: "cal_work".to_string(),
                    title: "Weekly Team Standup".to_string(),
                    description: Some("Project roadmap & Sprint review".to_string()),
                    location: Some("Google Meet / Room A".to_string()),
                    start_time: Local.from_local_datetime(&today.and_hms_opt(10, 0, 0).unwrap()).unwrap(),
                    end_time: Local.from_local_datetime(&today.and_hms_opt(11, 0, 0).unwrap()).unwrap(),
                    all_day: false,
                    rrule: None,
                    sync_status: SyncStatus::Synced,
                    remote_id: None,
                    etag: None,
                    updated_at: Local::now(),
                },
                Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    calendar_id: "cal_personal".to_string(),
                    title: "Lunch with Alex".to_string(),
                    description: Some("Try the new ramen place nearby".to_string()),
                    location: Some("Downtown Diner".to_string()),
                    start_time: Local.from_local_datetime(&today.and_hms_opt(12, 30, 0).unwrap()).unwrap(),
                    end_time: Local.from_local_datetime(&today.and_hms_opt(13, 30, 0).unwrap()).unwrap(),
                    all_day: false,
                    rrule: None,
                    sync_status: SyncStatus::Synced,
                    remote_id: None,
                    etag: None,
                    updated_at: Local::now(),
                },
                Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    calendar_id: "cal_work".to_string(),
                    title: "Architecture Design Review".to_string(),
                    description: Some("Review TUI event loop & CalDAV sync flow".to_string()),
                    location: Some("Virtual Room 4".to_string()),
                    start_time: Local.from_local_datetime(&today.and_hms_opt(15, 0, 0).unwrap()).unwrap(),
                    end_time: Local.from_local_datetime(&today.and_hms_opt(16, 30, 0).unwrap()).unwrap(),
                    all_day: false,
                    rrule: None,
                    sync_status: SyncStatus::Synced,
                    remote_id: None,
                    etag: None,
                    updated_at: Local::now(),
                },
                Event {
                    id: uuid::Uuid::new_v4().to_string(),
                    calendar_id: "cal_personal".to_string(),
                    title: "Evening Gym / Workout".to_string(),
                    description: Some("Leg day & 5km cardio".to_string()),
                    location: Some("Fitness Center".to_string()),
                    start_time: Local.from_local_datetime(&today.and_hms_opt(19, 0, 0).unwrap()).unwrap(),
                    end_time: Local.from_local_datetime(&today.and_hms_opt(20, 30, 0).unwrap()).unwrap(),
                    all_day: false,
                    rrule: None,
                    sync_status: SyncStatus::Synced,
                    remote_id: None,
                    etag: None,
                    updated_at: Local::now(),
                },
            ];

            for ev in &sample_events {
                self.save_event(ev)?;
            }
        }

        Ok(())
    }

    pub fn get_calendars(&self) -> Result<Vec<Calendar>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, color_code, provider, enabled, is_default, remote_url, ctag FROM calendars ORDER BY is_default DESC, name ASC"
        )?;

        let iter = stmt.query_map([], |row| {
            let provider_str: String = row.get(4)?;
            Ok(Calendar {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                color_code: row.get(3)?,
                provider: ProviderType::from_str(&provider_str),
                enabled: row.get::<_, i64>(5)? != 0,
                is_default: row.get::<_, i64>(6)? != 0,
                remote_url: row.get(7)?,
                ctag: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for item in iter {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn save_calendar(&self, cal: &Calendar) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO calendars (id, account_id, name, color_code, provider, enabled, is_default, remote_url, ctag)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                account_id = excluded.account_id,
                name = excluded.name,
                color_code = excluded.color_code,
                provider = excluded.provider,
                enabled = excluded.enabled,
                is_default = excluded.is_default,
                remote_url = excluded.remote_url,
                ctag = excluded.ctag
            "#,
            params![
                cal.id,
                cal.account_id,
                cal.name,
                cal.color_code,
                cal.provider.as_str(),
                if cal.enabled { 1 } else { 0 },
                if cal.is_default { 1 } else { 0 },
                cal.remote_url,
                cal.ctag
            ],
        )?;
        Ok(())
    }

    // Account management
    pub fn get_accounts(&self) -> Result<Vec<CalDavAccount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, provider, url, username, password,
                   auth_type, client_id, access_token, refresh_token, token_expires_at, last_synced_at
            FROM accounts ORDER BY name ASC
            "#,
        )?;

        let iter = stmt.query_map([], |row| {
            let provider_str: String = row.get(2)?;
            let auth_type_str: String = row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "basic".to_string());
            let expires_str: Option<String> = row.get(10)?;
            let token_expires_at = expires_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Local))
            });

            let last_sync_str: Option<String> = row.get(11)?;
            let last_synced_at = last_sync_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Local))
            });

            Ok(CalDavAccount {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: ProviderType::from_str(&provider_str),
                url: row.get(3)?,
                username: row.get(4)?,
                password: row.get(5)?,
                auth_type: AuthType::from_str(&auth_type_str),
                client_id: row.get(7)?,
                access_token: row.get(8)?,
                refresh_token: row.get(9)?,
                token_expires_at,
                last_synced_at,
            })
        })?;

        let mut list = Vec::new();
        for item in iter {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn save_account(&self, acc: &CalDavAccount) -> Result<()> {
        let last_sync_str = acc.last_synced_at.map(|t| t.to_rfc3339());
        let expires_str = acc.token_expires_at.map(|t| t.to_rfc3339());
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO accounts (
                id, name, provider, url, username, password,
                auth_type, client_id, access_token, refresh_token, token_expires_at, last_synced_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                provider = excluded.provider,
                url = excluded.url,
                username = excluded.username,
                password = excluded.password,
                auth_type = excluded.auth_type,
                client_id = excluded.client_id,
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                token_expires_at = excluded.token_expires_at,
                last_synced_at = excluded.last_synced_at
            "#,
            params![
                acc.id,
                acc.name,
                acc.provider.as_str(),
                acc.url,
                acc.username,
                acc.password,
                acc.auth_type.as_str(),
                acc.client_id,
                acc.access_token,
                acc.refresh_token,
                expires_str,
                last_sync_str
            ],
        )?;
        Ok(())
    }

    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        conn.execute("DELETE FROM calendars WHERE account_id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_pending_sync_events(&self, calendar_id: &str) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, calendar_id, title, description, location,
                   start_time, end_time, all_day, rrule, sync_status,
                   remote_id, etag, updated_at
            FROM events
            WHERE calendar_id = ?1 AND sync_status IN ('created_locally', 'updated_locally', 'deleted_locally')
            "#,
        )?;

        let iter = stmt.query_map(params![calendar_id], |row| Self::row_to_event(row))?;
        let mut events = Vec::new();
        for ev in iter {
            events.push(ev?);
        }
        Ok(events)
    }

    pub fn toggle_calendar(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let current: i64 = conn.query_row(
            "SELECT enabled FROM calendars WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        let new_state = if current == 0 { 1 } else { 0 };
        conn.execute(
            "UPDATE calendars SET enabled = ?1 WHERE id = ?2",
            params![new_state, id],
        )?;
        Ok(new_state == 1)
    }

    pub fn get_all_events(&self) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.calendar_id, e.title, e.description, e.location,
                   e.start_time, e.end_time, e.all_day, e.rrule, e.sync_status,
                   e.remote_id, e.etag, e.updated_at
            FROM events e
            JOIN calendars c ON e.calendar_id = c.id
            WHERE c.enabled = 1 AND e.sync_status != 'deleted_locally'
            ORDER BY e.start_time ASC
            "#,
        )?;

        let iter = stmt.query_map([], |row| Self::row_to_event(row))?;
        let mut events = Vec::new();
        for ev in iter {
            events.push(ev?);
        }
        Ok(events)
    }

    #[allow(dead_code)]
    pub fn get_events_range(
        &self,
        start_inclusive: DateTime<Local>,
        end_inclusive: DateTime<Local>,
    ) -> Result<Vec<Event>> {
        let start_str = start_inclusive.to_rfc3339();
        let end_str = end_inclusive.to_rfc3339();

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.calendar_id, e.title, e.description, e.location,
                   e.start_time, e.end_time, e.all_day, e.rrule, e.sync_status,
                   e.remote_id, e.etag, e.updated_at
            FROM events e
            JOIN calendars c ON e.calendar_id = c.id
            WHERE c.enabled = 1 
              AND e.sync_status != 'deleted_locally'
              AND (e.end_time >= ?1 AND e.start_time <= ?2)
            ORDER BY e.start_time ASC
            "#,
        )?;

        let iter = stmt.query_map(params![start_str, end_str], |row| Self::row_to_event(row))?;
        let mut events = Vec::new();
        for ev in iter {
            events.push(ev?);
        }
        Ok(events)
    }

    pub fn get_event_by_id(&self, id: &str) -> Result<Option<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, calendar_id, title, description, location,
                   start_time, end_time, all_day, rrule, sync_status,
                   remote_id, etag, updated_at
            FROM events
            WHERE id = ?1
            "#,
        )?;

        let res = stmt.query_row(params![id], |row| Self::row_to_event(row)).optional()?;
        Ok(res)
    }

    pub fn save_event(&self, event: &Event) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO events (
                id, calendar_id, title, description, location,
                start_time, end_time, all_day, rrule, sync_status,
                remote_id, etag, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                calendar_id = excluded.calendar_id,
                title = excluded.title,
                description = excluded.description,
                location = excluded.location,
                start_time = excluded.start_time,
                end_time = excluded.end_time,
                all_day = excluded.all_day,
                rrule = excluded.rrule,
                sync_status = excluded.sync_status,
                remote_id = excluded.remote_id,
                etag = excluded.etag,
                updated_at = excluded.updated_at
            "#,
            params![
                event.id,
                event.calendar_id,
                event.title,
                event.description,
                event.location,
                event.start_time.to_rfc3339(),
                event.end_time.to_rfc3339(),
                if event.all_day { 1 } else { 0 },
                event.rrule,
                event.sync_status.as_str(),
                event.remote_id,
                event.etag,
                event.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn delete_event(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let remote_id: Option<String> = conn.query_row(
            "SELECT remote_id FROM events WHERE id = ?1",
            params![id],
            |r| r.get(0),
        ).optional()?.flatten();

        if remote_id.is_some() {
            conn.execute(
                "UPDATE events SET sync_status = 'deleted_locally', updated_at = ?1 WHERE id = ?2",
                params![Local::now().to_rfc3339(), id],
            )?;
        } else {
            conn.execute("DELETE FROM events WHERE id = ?1", params![id])?;
        }
        Ok(())
    }

    fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
        let start_str: String = row.get(5)?;
        let end_str: String = row.get(6)?;
        let sync_str: String = row.get(9)?;
        let updated_str: String = row.get(12)?;

        let start_time = DateTime::parse_from_rfc3339(&start_str)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());

        let end_time = DateTime::parse_from_rfc3339(&end_str)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());

        let updated_at = DateTime::parse_from_rfc3339(&updated_str)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());

        Ok(Event {
            id: row.get(0)?,
            calendar_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            location: row.get(4)?,
            start_time,
            end_time,
            all_day: row.get::<_, i64>(7)? != 0,
            rrule: row.get(8)?,
            sync_status: SyncStatus::from_str(&sync_str),
            remote_id: row.get(10)?,
            etag: row.get(11)?,
            updated_at,
        })
    }
}
