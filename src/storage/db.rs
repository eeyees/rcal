use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        db.ensure_default_data()?;

        Ok(db)
    }

    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        db.ensure_default_data()?;
        Ok(db)
    }

    fn default_db_path() -> Result<PathBuf> {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".local/share"))
                .unwrap_or_else(|| PathBuf::from("."))
        });
        path.push("rcal");
        path.push("rcal.db");
        Ok(path)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

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

            CREATE TABLE IF NOT EXISTS notified_events (
                event_id TEXT NOT NULL,
                notification_type TEXT NOT NULL,
                notified_at TEXT NOT NULL,
                PRIMARY KEY (event_id, notification_type)
            );
            CREATE INDEX IF NOT EXISTS idx_notified_events_time ON notified_events(notified_at);
            "#,
        )?;

        let _ = conn.execute("ALTER TABLE calendars ADD COLUMN account_id TEXT", []);
        let _ = conn.execute("ALTER TABLE calendars ADD COLUMN remote_url TEXT", []);
        let _ = conn.execute("ALTER TABLE calendars ADD COLUMN ctag TEXT", []);

        let _ = conn.execute(
            "ALTER TABLE accounts ADD COLUMN auth_type TEXT NOT NULL DEFAULT 'basic'",
            [],
        );
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN client_id TEXT", []);
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN access_token TEXT", []);
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN refresh_token TEXT", []);
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN token_expires_at TEXT", []);

        // Clean up any existing duplicate accounts
        let _ = conn.execute(
            "DELETE FROM accounts WHERE rowid NOT IN (SELECT min(rowid) FROM accounts GROUP BY provider, username)",
            [],
        );

        Ok(())
    }

    fn ensure_default_data(&self) -> Result<()> {
        let count: i64 = {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM calendars", [], |row| row.get(0))?
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
                    start_time: Local
                        .from_local_datetime(&today.and_hms_opt(10, 0, 0).unwrap())
                        .unwrap(),
                    end_time: Local
                        .from_local_datetime(&today.and_hms_opt(11, 0, 0).unwrap())
                        .unwrap(),
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
                    start_time: Local
                        .from_local_datetime(&today.and_hms_opt(12, 30, 0).unwrap())
                        .unwrap(),
                    end_time: Local
                        .from_local_datetime(&today.and_hms_opt(13, 30, 0).unwrap())
                        .unwrap(),
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
                    start_time: Local
                        .from_local_datetime(&today.and_hms_opt(15, 0, 0).unwrap())
                        .unwrap(),
                    end_time: Local
                        .from_local_datetime(&today.and_hms_opt(16, 30, 0).unwrap())
                        .unwrap(),
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
                    start_time: Local
                        .from_local_datetime(&today.and_hms_opt(19, 0, 0).unwrap())
                        .unwrap(),
                    end_time: Local
                        .from_local_datetime(&today.and_hms_opt(20, 30, 0).unwrap())
                        .unwrap(),
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

    pub fn get_default_calendar(&self) -> Result<Option<Calendar>> {
        let calendars = self.get_calendars()?;
        if let Some(cal) = calendars
            .iter()
            .find(|c| c.is_default && c.enabled)
            .cloned()
        {
            return Ok(Some(cal));
        }
        if let Some(cal) = calendars.iter().find(|c| c.enabled).cloned() {
            return Ok(Some(cal));
        }
        Ok(calendars.into_iter().next())
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
    fn row_to_account(row: &rusqlite::Row) -> rusqlite::Result<CalDavAccount> {
        let provider_str: String = row.get(2)?;
        let auth_type_str: String = row
            .get::<_, Option<String>>(6)?
            .unwrap_or_else(|| "basic".to_string());
        let expires_str: Option<String> = row.get(10)?;
        let token_expires_at = expires_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Local))
        });

        let last_sync_str: Option<String> = row.get(11)?;
        let last_synced_at = last_sync_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Local))
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
    }

    pub fn get_accounts(&self) -> Result<Vec<CalDavAccount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, provider, url, username, password,
                   auth_type, client_id, access_token, refresh_token, token_expires_at, last_synced_at
            FROM accounts ORDER BY name ASC
            "#,
        )?;

        let iter = stmt.query_map([], |row| Self::row_to_account(row))?;

        let mut list = Vec::new();
        for item in iter {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn find_account_by_provider_and_username(
        &self,
        provider: &ProviderType,
        username: &str,
    ) -> Result<Option<CalDavAccount>> {
        let trimmed_user = username.trim();
        let clean_user = trimmed_user
            .strip_suffix("@naver.com")
            .unwrap_or(trimmed_user)
            .trim();
        let full_user = format!("{}@naver.com", clean_user);

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, provider, url, username, password,
                   auth_type, client_id, access_token, refresh_token, token_expires_at, last_synced_at
            FROM accounts
            WHERE (provider = ?1 OR LOWER(provider) = LOWER(?1)) AND (
                username = ?2
                OR username = ?3
                OR (LOWER(?1) = 'naver' AND (username = ?4 OR username = ?5))
            )
            ORDER BY rowid ASC
            LIMIT 1
            "#,
        )?;

        let account = stmt
            .query_row(
                params![
                    provider.as_str(),
                    trimmed_user,
                    username,
                    clean_user,
                    full_user
                ],
                |row| Self::row_to_account(row),
            )
            .optional()?;

        Ok(account)
    }

    pub fn save_account(&self, acc: &CalDavAccount) -> Result<()> {
        // Check if an account with matching (provider, username) or (url, username) already exists.
        // If found with a different id, UPDATE that existing account (preserve the existing ID)
        let existing = self.find_account_by_provider_and_username(&acc.provider, &acc.username)?;
        let target_id = if let Some(found) = existing {
            found.id
        } else {
            let existing_by_url: Option<String> = if !acc.url.trim().is_empty() {
                let conn = self.conn.lock().unwrap();
                conn.query_row(
                    "SELECT id FROM accounts WHERE url = ?1 AND username = ?2 LIMIT 1",
                    params![acc.url.trim(), acc.username.trim()],
                    |row| row.get(0),
                )
                .optional()?
            } else {
                None
            };
            existing_by_url.unwrap_or_else(|| acc.id.clone())
        };

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
                target_id,
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

    pub fn delete_calendar(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM events WHERE calendar_id = ?1", params![id])?;
        conn.execute("DELETE FROM calendars WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Settings
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let res = stmt.query_row(params![key], |r| r.get(0)).optional()?;
        Ok(res)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO settings (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
            params![key, value],
        )?;
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

    pub fn set_calendar_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE calendars SET enabled = ?1 WHERE id = ?2",
            params![if enabled { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn update_calendars_enabled(&self, states: &[(String, bool)]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for (id, enabled) in states {
            conn.execute(
                "UPDATE calendars SET enabled = ?1 WHERE id = ?2",
                params![if *enabled { 1 } else { 0 }, id],
            )?;
        }
        Ok(())
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

    pub fn search_events(&self, query: &str, limit: Option<usize>) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let max_limit = limit.unwrap_or(50) as i64;
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.calendar_id, e.title, e.description, e.location,
                   e.start_time, e.end_time, e.all_day, e.rrule, e.sync_status,
                   e.remote_id, e.etag, e.updated_at
            FROM events e
            JOIN calendars c ON e.calendar_id = c.id
            WHERE c.enabled = 1 
              AND e.sync_status != 'deleted_locally'
              AND (
                  e.title LIKE ?1 
                  OR (e.description IS NOT NULL AND e.description LIKE ?1)
                  OR (e.location IS NOT NULL AND e.location LIKE ?1)
              )
            ORDER BY e.start_time ASC
            LIMIT ?2
            "#,
        )?;

        let iter = stmt.query_map(params![pattern, max_limit], |row| Self::row_to_event(row))?;
        let mut events = Vec::new();
        for ev in iter {
            events.push(ev?);
        }
        Ok(events)
    }

    pub fn get_events_filtered(
        &self,
        start_inclusive: Option<DateTime<Local>>,
        end_inclusive: Option<DateTime<Local>>,
        calendar_id: Option<&str>,
        query: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let start_str = start_inclusive.map(|dt| dt.to_rfc3339());
        let end_str = end_inclusive.map(|dt| dt.to_rfc3339());
        let pattern = query.map(|q| format!("%{}%", q));
        let max_limit = limit.unwrap_or(100) as i64;

        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.calendar_id, e.title, e.description, e.location,
                   e.start_time, e.end_time, e.all_day, e.rrule, e.sync_status,
                   e.remote_id, e.etag, e.updated_at
            FROM events e
            JOIN calendars c ON e.calendar_id = c.id
            WHERE e.sync_status != 'deleted_locally'
              AND (?1 IS NOT NULL OR c.enabled = 1)
              AND (?1 IS NULL OR e.calendar_id = ?1)
              AND (?2 IS NULL OR e.end_time >= ?2)
              AND (?3 IS NULL OR e.start_time <= ?3)
              AND (?4 IS NULL OR (
                  e.title LIKE ?4 
                  OR (e.description IS NOT NULL AND e.description LIKE ?4)
                  OR (e.location IS NOT NULL AND e.location LIKE ?4)
              ))
            ORDER BY e.start_time ASC
            LIMIT ?5
            "#,
        )?;

        let iter = stmt.query_map(
            params![calendar_id, start_str, end_str, pattern, max_limit],
            |row| Self::row_to_event(row),
        )?;
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

        let res = stmt
            .query_row(params![id], |row| Self::row_to_event(row))
            .optional()?;
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

        // Reset notification history for modified event so updated times trigger reminders
        let _ = conn.execute(
            "DELETE FROM notified_events WHERE event_id = ?1",
            params![event.id],
        );

        Ok(())
    }

    pub fn delete_event(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let remote_id: Option<String> = conn
            .query_row(
                "SELECT remote_id FROM events WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();

        if remote_id.is_some() {
            conn.execute(
                "UPDATE events SET sync_status = 'deleted_locally', updated_at = ?1 WHERE id = ?2",
                params![Local::now().to_rfc3339(), id],
            )?;
        } else {
            conn.execute("DELETE FROM events WHERE id = ?1", params![id])?;
        }

        let _ = conn.execute(
            "DELETE FROM notified_events WHERE event_id = ?1",
            params![id],
        );
        Ok(())
    }

    pub fn delete_events_by_calendar(&self, calendar_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM events WHERE calendar_id = ?1",
            params![calendar_id],
        )?;
        Ok(())
    }

    pub fn has_notified(&self, event_id: &str, notification_type: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT 1 FROM notified_events WHERE event_id = ?1 AND notification_type = ?2 LIMIT 1",
        )?;
        let exists = stmt.exists(params![event_id, notification_type])?;
        Ok(exists)
    }

    pub fn mark_notified(&self, event_id: &str, notification_type: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO notified_events (event_id, notification_type, notified_at) VALUES (?1, ?2, ?3)",
            params![event_id, notification_type, now],
        )?;
        Ok(())
    }

    pub fn clear_notifications_for_event(&self, event_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM notified_events WHERE event_id = ?1",
            params![event_id],
        )?;
        Ok(())
    }

    pub fn cleanup_old_notifications(&self, older_than: DateTime<Local>) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let older_str = older_than.to_rfc3339();
        let count = conn.execute(
            "DELETE FROM notified_events WHERE notified_at < ?1",
            params![older_str],
        )?;
        Ok(count)
    }

    pub fn get_events_for_notification(
        &self,
        from_time: DateTime<Local>,
        to_time: DateTime<Local>,
    ) -> Result<Vec<(Event, String)>> {
        let from_str = from_time.to_rfc3339();
        let to_str = to_time.to_rfc3339();

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.calendar_id, e.title, e.description, e.location,
                   e.start_time, e.end_time, e.all_day, e.rrule, e.sync_status,
                   e.remote_id, e.etag, e.updated_at, c.name
            FROM events e
            JOIN calendars c ON e.calendar_id = c.id
            WHERE c.enabled = 1
              AND e.sync_status != 'deleted_locally'
              AND (
                  (e.all_day = 0 AND e.start_time >= ?1 AND e.start_time <= ?2)
                  OR
                  (e.all_day = 1 AND substr(e.start_time, 1, 10) = substr(?1, 1, 10))
                  OR
                  (e.rrule IS NOT NULL AND e.rrule != '')
              )
            ORDER BY e.start_time ASC
            "#,
        )?;

        let iter = stmt.query_map(params![from_str, to_str], |row| {
            let ev = Self::row_to_event(row)?;
            let cal_name: String = row.get(13)?;
            Ok((ev, cal_name))
        })?;

        let mut results = Vec::new();
        for item in iter {
            results.push(item?);
        }
        Ok(results)
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
