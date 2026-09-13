use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub action: String,
    pub memory_id: String,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

pub fn init_table(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS audit_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            action      TEXT NOT NULL,
            memory_id   TEXT NOT NULL,
            details     TEXT NOT NULL DEFAULT '',
            created_at  TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_audit_memory ON audit_log(memory_id);
        CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log(action);",
    )?;
    Ok(())
}

pub fn log(conn: &Connection, action: &str, memory_id: &str, details: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO audit_log (action, memory_id, details, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![action, memory_id, details, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn list(conn: &Connection, limit: usize) -> anyhow::Result<Vec<AuditEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, action, memory_id, details, created_at FROM audit_log ORDER BY id DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit], |row| {
        Ok(AuditEntry {
            id: row.get(0)?,
            action: row.get(1)?,
            memory_id: row.get(2)?,
            details: row.get(3)?,
            created_at: chrono::DateTime::parse_from_rfc3339(
                &row.get::<_, String>(4)?,
            )
            .unwrap_or_default()
            .with_timezone(&Utc),
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

pub fn list_for_memory(conn: &Connection, memory_id: &str, limit: usize) -> anyhow::Result<Vec<AuditEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, action, memory_id, details, created_at FROM audit_log WHERE memory_id = ?1 ORDER BY id DESC LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![memory_id, limit], |row| {
        Ok(AuditEntry {
            id: row.get(0)?,
            action: row.get(1)?,
            memory_id: row.get(2)?,
            details: row.get(3)?,
            created_at: chrono::DateTime::parse_from_rfc3339(
                &row.get::<_, String>(4)?,
            )
            .unwrap_or_default()
            .with_timezone(&Utc),
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

pub fn count(conn: &Connection) -> anyhow::Result<usize> {
    let count: usize = conn.query_row(
        "SELECT COUNT(*) FROM audit_log",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}
