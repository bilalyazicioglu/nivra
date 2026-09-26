use crate::model::{Event, Snapshot, now_ms};
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, params};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{path::PathBuf, time::Duration};

pub struct Store {
    pub conn: Connection,
    pub path: PathBuf,
}
impl Store {
    pub fn open() -> Result<Self> {
        let dir = match std::env::var_os("NIVRA_DATA_DIR") {
            Some(path) => PathBuf::from(path),
            None => std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or(
                    PathBuf::from(std::env::var_os("HOME").context("HOME is unset")?)
                        .join(".local/share"),
                )
                .join("nivra"),
        };
        std::fs::create_dir_all(&dir)?;
        #[cfg(unix)]
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        let path = dir.join("nivra.db");
        if path
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            bail!("refusing symlink database");
        }
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(false);
        #[cfg(unix)]
        options.mode(0o600);
        options.open(&path)?;
        #[cfg(unix)]
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        let conn = Connection::open(&path)?;
        conn.busy_timeout(Duration::from_millis(50))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > 1 {
            bail!("database created by newer Nivra; upgrade this binary");
        }
        if version == 0 {
            conn.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE sessions(id TEXT PRIMARY KEY, started_ms INTEGER NOT NULL, shell TEXT NOT NULL);
                CREATE TABLE events(id TEXT PRIMARY KEY, session TEXT NOT NULL REFERENCES sessions(id), command TEXT NOT NULL, started_ms INTEGER NOT NULL, finished_ms INTEGER, exit_code INTEGER, before_json TEXT NOT NULL, after_json TEXT);
                CREATE INDEX events_time ON events(started_ms);
                CREATE TABLE marks(name TEXT NOT NULL, repo TEXT NOT NULL, created_ms INTEGER NOT NULL, snapshot_json TEXT NOT NULL, PRIMARY KEY(name,repo));
                CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
                PRAGMA user_version=1;
                COMMIT;")?;
        }
        Ok(Self { conn, path })
    }
    pub fn setting(&self, key: &str) -> Result<bool> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key=?", [key], |r| {
                r.get::<_, String>(0)
            })
            .optional()?
            .as_deref()
            == Some("1"))
    }
    pub fn set(&self, key: &str, value: bool) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings VALUES (?,?)",
            params![key, if value { "1" } else { "0" }],
        )?;
        Ok(())
    }
    pub fn should_capture(&mut self) -> Result<bool> {
        let tx = self.conn.transaction()?;
        let value = |key: &str| -> Result<bool> {
            Ok(tx
                .query_row("SELECT value FROM settings WHERE key=?", [key], |r| {
                    r.get::<_, String>(0)
                })
                .optional()?
                .as_deref()
                == Some("1"))
        };
        if value("paused")? {
            return Ok(false);
        }
        if value("ignore-next")? {
            tx.execute("DELETE FROM settings WHERE key='ignore-next'", [])?;
            tx.commit()?;
            return Ok(false);
        }
        tx.commit()?;
        Ok(true)
    }
    pub fn begin(&self, session: &str, command: &str, before: &Snapshot) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_ms();
        self.conn.execute(
            "INSERT OR IGNORE INTO sessions VALUES (?,?,?)",
            params![session, now, "zsh/explicit"],
        )?;
        self.conn.execute(
            "INSERT INTO events(id,session,command,started_ms,before_json) VALUES (?,?,?,?,?)",
            params![id, session, command, now, serde_json::to_string(before)?],
        )?;
        Ok(id)
    }
    pub fn finish(&self, id: &str, exit: i32, after: &Snapshot, finished_ms: i64) -> Result<()> {
        let count = self.conn.execute("UPDATE events SET finished_ms=?,exit_code=?,after_json=? WHERE id=? AND finished_ms IS NULL", params![finished_ms,exit,serde_json::to_string(after)?,id])?;
        if count != 1 {
            bail!("unknown or already completed event");
        }
        Ok(())
    }
    pub fn events(&self, limit: usize) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare("SELECT id,session,command,started_ms,finished_ms,exit_code,before_json,after_json FROM events ORDER BY started_ms DESC,rowid DESC LIMIT ?")?;
        let rows = stmt.query_map([limit as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i32>>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, Option<String>>(7)?,
            ))
        })?;
        rows.map(|row| {
            let (id, session, command, started_ms, finished_ms, exit_code, before, after) = row?;
            Ok(Event {
                id,
                session,
                command,
                started_ms,
                finished_ms,
                exit_code,
                before: serde_json::from_str(&before)?,
                after: after.map(|v| serde_json::from_str(&v)).transpose()?,
            })
        })
        .collect()
    }
    pub fn mark(&self, name: &str, snap: &Snapshot) -> Result<()> {
        let repo = snap
            .repo
            .as_ref()
            .context("marks require a Git working tree")?;
        self.conn.execute(
            "INSERT OR REPLACE INTO marks VALUES (?,?,?,?)",
            params![name, repo, now_ms(), serde_json::to_string(snap)?],
        )?;
        Ok(())
    }
    pub fn get_mark(&self, name: &str, repo: &str) -> Result<(i64, Snapshot)> {
        let (time, json) = self
            .conn
            .query_row(
                "SELECT created_ms,snapshot_json FROM marks WHERE name=? AND repo=?",
                params![name, repo],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
            .context("mark not found in this repository")?;
        Ok((time, serde_json::from_str(&json)?))
    }
}
use rusqlite::OptionalExtension;
