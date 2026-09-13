use rusqlite::Connection;

pub fn init_fts(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
            id,
            key,
            content,
            tags,
            content='memories',
            content_rowid='rowid'
        );
        
        CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
            INSERT INTO memories_fts(rowid, id, key, content, tags)
            VALUES (new.rowid, new.id, new.key, new.content, new.tags);
        END;
        
        CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
            INSERT INTO memories_fts(memories_fts, rowid, id, key, content, tags)
            VALUES ('delete', old.rowid, old.id, old.key, old.content, old.tags);
        END;
        
        CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
            INSERT INTO memories_fts(memories_fts, rowid, id, key, content, tags)
            VALUES ('delete', old.rowid, old.id, old.key, old.content, old.tags);
            INSERT INTO memories_fts(rowid, id, key, content, tags)
            VALUES (new.rowid, new.id, new.key, new.content, new.tags);
        END;",
    )?;
    
    Ok(())
}

pub fn rebuild_fts(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("INSERT INTO memories_fts(memories_fts) VALUES('rebuild');")?;
    Ok(())
}

pub fn fts_search(conn: &Connection, query: &str, limit: usize) -> anyhow::Result<Vec<FtsResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, key, content, tags, rank
         FROM memories_fts
         WHERE memories_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2"
    )?;
    
    let rows = stmt.query_map(rusqlite::params![query, limit], |row| {
        Ok(FtsResult {
            id: row.get(0)?,
            key: row.get(1)?,
            content: row.get(2)?,
            tags: row.get(3)?,
            rank: row.get(4)?,
        })
    })?;
    
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    
    Ok(results)
}

#[derive(Debug)]
pub struct FtsResult {
    pub id: String,
    pub key: String,
    pub content: String,
    pub tags: String,
    pub rank: f64,
}

pub fn format_fts_results(results: &[FtsResult]) -> String {
    if results.is_empty() {
        return "No results found.".to_string();
    }
    
    let mut output = String::new();
    output.push_str(&format!("Found {} results:\n\n", results.len()));
    
    for r in results {
        output.push_str(&format!("  {} (score: {:.2})\n", r.key, r.rank.abs()));
        output.push_str(&format!("    {}\n", r.content));
        if !r.tags.is_empty() {
            output.push_str(&format!("    Tags: {}\n", r.tags));
        }
        output.push('\n');
    }
    
    output
}
