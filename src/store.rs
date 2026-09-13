use crate::types::{Memory, MemoryInput, MemoryKind, ScoredMemory};
use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo { pub tag: String, pub count: usize }

pub struct MemoryStore { conn: Connection }

impl MemoryStore {
    pub fn conn(&self) -> &Connection { &self.conn }
    pub fn open(db_path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;
        let s = Self { conn };
        s.init_tables()?;
        Ok(s)
    }
    pub fn open_in_project(project_dir: &Path) -> anyhow::Result<Self> {
        let d = project_dir.join(".memory");
        std::fs::create_dir_all(&d)?;
        Self::open(&d.join("store.db"))
    }
    fn init_tables(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (id TEXT PRIMARY KEY, kind TEXT NOT NULL, key TEXT NOT NULL, content TEXT NOT NULL, tags TEXT NOT NULL DEFAULT '[]', related_ids TEXT NOT NULL DEFAULT '[]', created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS idx_memories_kind ON memories(kind);
            CREATE INDEX IF NOT EXISTS idx_memories_key ON memories(key);
            CREATE TABLE IF NOT EXISTS memory_links (source_id TEXT NOT NULL, target_id TEXT NOT NULL, PRIMARY KEY(source_id,target_id), FOREIGN KEY(source_id) REFERENCES memories(id) ON DELETE CASCADE, FOREIGN KEY(target_id) REFERENCES memories(id) ON DELETE CASCADE);
            CREATE TABLE IF NOT EXISTS memory_versions (id INTEGER PRIMARY KEY AUTOINCREMENT, memory_id TEXT NOT NULL, content TEXT NOT NULL, tags TEXT NOT NULL, version_num INTEGER NOT NULL, created_at TEXT NOT NULL, FOREIGN KEY(memory_id) REFERENCES memories(id) ON DELETE CASCADE);
            CREATE TABLE IF NOT EXISTS audit_log (id INTEGER PRIMARY KEY AUTOINCREMENT, action TEXT NOT NULL, memory_id TEXT NOT NULL, details TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL);"
        )?;
        Ok(())
    }
    pub fn add(&self, input: MemoryInput) -> anyhow::Result<Memory> {
        let now = Utc::now();
        let m = Memory { id: Uuid::new_v4().to_string(), kind: input.kind, key: input.key, content: input.content, tags: input.tags, related_ids: input.related_ids, created_at: now, updated_at: now };
        self.conn.execute("INSERT INTO memories(id,kind,key,content,tags,related_ids,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![m.id, m.kind.to_string(), m.key, m.content, serde_json::to_string(&m.tags)?, serde_json::to_string(&m.related_ids)?, m.created_at.to_rfc3339(), m.updated_at.to_rfc3339()])?;
        self.conn.execute("INSERT INTO audit_log(action,memory_id,details,created_at) VALUES(?1,?2,?3,?4)", params!["add", m.id, format!("{}: {}", m.kind, m.key), now.to_rfc3339()])?;
        Ok(m)
    }
    pub fn get(&self, id: &str) -> anyhow::Result<Option<Memory>> {
        let mut stmt = self.conn.prepare("SELECT id,kind,key,content,tags,related_ids,created_at,updated_at FROM memories WHERE id=?1")?;
        let mut rows = stmt.query_map(params![id], |row| Ok(RawRow { id: row.get(0)?, kind: row.get(1)?, key: row.get(2)?, content: row.get(3)?, tags: row.get(4)?, related_ids: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)? }))?;
        match rows.next() { Some(r) => Ok(Some(r?.into_memory()?)), None => Ok(None) }
    }
    pub fn list(&self, kind: Option<MemoryKind>, limit: usize) -> anyhow::Result<Vec<Memory>> {
        let mut sql = String::from("SELECT id,kind,key,content,tags,related_ids,created_at,updated_at FROM memories");
        let mut pv: Vec<String> = Vec::new();
        if let Some(ref k) = kind { sql.push_str(" WHERE kind=?1"); pv.push(k.to_string()); }
        sql.push_str(" ORDER BY updated_at DESC LIMIT ?");
        pv.push(limit.to_string());
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(pv.iter()), |row| Ok(RawRow { id: row.get(0)?, kind: row.get(1)?, key: row.get(2)?, content: row.get(3)?, tags: row.get(4)?, related_ids: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)? }))?;
        Ok(rows.filter_map(|r| r.ok().and_then(|r| r.into_memory().ok())).collect())
    }
    pub fn update(&self, id: &str, input: MemoryInput) -> anyhow::Result<Option<Memory>> {
        let now = Utc::now();
        self.conn.execute("UPDATE memories SET kind=?1,key=?2,content=?3,tags=?4,related_ids=?5,updated_at=?6 WHERE id=?7", params![input.kind.to_string(), input.key, input.content, serde_json::to_string(&input.tags)?, serde_json::to_string(&input.related_ids)?, now.to_rfc3339(), id])?;
        self.get(id)
    }
    pub fn delete(&self, id: &str) -> anyhow::Result<bool> {
        Ok(self.conn.execute("DELETE FROM memories WHERE id=?1", params![id])? > 0)
    }
    pub fn link(&self, src: &str, dst: &str) -> anyhow::Result<bool> {
        if self.get(src)?.is_none() || self.get(dst)?.is_none() { return Ok(false); }
        self.conn.execute("INSERT OR IGNORE INTO memory_links(source_id,target_id) VALUES(?1,?2)", params![src, dst])?;
        if let Some(mut m) = self.get(src)? {
            if !m.related_ids.contains(&dst.to_string()) { m.related_ids.push(dst.to_string()); self.conn.execute("UPDATE memories SET related_ids=?1 WHERE id=?2", params![serde_json::to_string(&m.related_ids)?, src])?; }
        }
        Ok(true)
    }
    pub fn unlink(&self, src: &str, dst: &str) -> anyhow::Result<bool> {
        self.conn.execute("DELETE FROM memory_links WHERE source_id=?1 AND target_id=?2", params![src, dst])?;
        if let Some(mut m) = self.get(src)? { m.related_ids.retain(|id| id != dst); self.conn.execute("UPDATE memories SET related_ids=?1 WHERE id=?2", params![serde_json::to_string(&m.related_ids)?, src])?; }
        Ok(true)
    }
    pub fn get_related(&self, id: &str) -> anyhow::Result<Vec<Memory>> {
        let m = match self.get(id)? { Some(m) => m, None => return Ok(vec![]) };
        Ok(m.related_ids.iter().filter_map(|rid| self.get(rid).ok().flatten()).collect())
    }
    pub fn count(&self) -> anyhow::Result<usize> { Ok(self.conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?) }
    pub fn count_by_kind(&self) -> anyhow::Result<Vec<(MemoryKind, usize)>> {
        let mut stmt = self.conn.prepare("SELECT kind,COUNT(*) FROM memories GROUP BY kind ORDER BY COUNT(*) DESC")?;
        let mut result = Vec::new();
        for row in stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?)))? {
            if let Ok((k, c)) = row { if let Ok(kind) = k.parse::<MemoryKind>() { result.push((kind, c)); } }
        }
        Ok(result)
    }
    pub fn export_json(&self) -> anyhow::Result<Vec<Memory>> { self.list(None, 1000000) }
    pub fn import_memories(&self, memories: Vec<MemoryInput>) -> anyhow::Result<usize> {
        let mut c = 0; for i in memories { self.add(i)?; c += 1; } Ok(c)
    }
    /// Inserts a memory preserving its existing id/timestamps (used by snapshot restore,
    /// so related_ids captured in the snapshot still resolve after restore).
    pub fn add_with_id(&self, m: &Memory) -> anyhow::Result<()> {
        self.conn.execute("INSERT INTO memories(id,kind,key,content,tags,related_ids,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![m.id, m.kind.to_string(), m.key, m.content, serde_json::to_string(&m.tags)?, serde_json::to_string(&m.related_ids)?, m.created_at.to_rfc3339(), m.updated_at.to_rfc3339()])?;
        self.conn.execute("INSERT INTO audit_log(action,memory_id,details,created_at) VALUES(?1,?2,?3,?4)", params!["restore", m.id, format!("{}: {}", m.kind, m.key), Utc::now().to_rfc3339()])?;
        Ok(())
    }
    /// Deletes `remove_ids` and writes `keep` in a single transaction, so a duplicate merge
    /// never leaves the survivor un-updated while the duplicates are already gone.
    pub fn merge_duplicates(&self, keep: &Memory, remove_ids: &[String]) -> anyhow::Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let mut removed = 0usize;
        for id in remove_ids {
            removed += tx.execute("DELETE FROM memories WHERE id=?1", params![id])?;
        }
        tx.execute("UPDATE memories SET kind=?1,key=?2,content=?3,tags=?4,related_ids=?5,updated_at=?6 WHERE id=?7",
            params![keep.kind.to_string(), keep.key, keep.content, serde_json::to_string(&keep.tags)?, serde_json::to_string(&keep.related_ids)?, Utc::now().to_rfc3339(), keep.id])?;
        tx.commit()?;
        Ok(removed)
    }
    pub fn list_tags(&self) -> anyhow::Result<Vec<TagInfo>> {
        let memories = self.list(None, 10000)?;
        let mut tc: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for m in &memories { for t in &m.tags { *tc.entry(t.clone()).or_insert(0) += 1; } }
        let mut tags: Vec<TagInfo> = tc.into_iter().map(|(tag, count)| TagInfo { tag, count }).collect();
        tags.sort_by_key(|a| std::cmp::Reverse(a.count));
        Ok(tags)
    }
    pub fn rename_tag(&self, old: &str, new: &str) -> anyhow::Result<usize> {
        let memories = self.list(None, 10000)?; let mut c = 0;
        for m in &memories {
            if m.tags.contains(&old.to_string()) {
                let mut t = m.tags.clone(); t.retain(|t| t != old); if !t.contains(&new.to_string()) { t.push(new.to_string()); }
                self.conn.execute("UPDATE memories SET tags=?1,updated_at=?2 WHERE id=?3", params![serde_json::to_string(&t)?, Utc::now().to_rfc3339(), m.id])?; c += 1;
            }
        }
        Ok(c)
    }
    pub fn delete_tag(&self, tag: &str) -> anyhow::Result<usize> {
        let memories = self.list(None, 10000)?; let mut c = 0;
        for m in &memories {
            if m.tags.contains(&tag.to_string()) {
                let mut t = m.tags.clone(); t.retain(|t| t != tag);
                self.conn.execute("UPDATE memories SET tags=?1,updated_at=?2 WHERE id=?3", params![serde_json::to_string(&t)?, Utc::now().to_rfc3339(), m.id])?; c += 1;
            }
        }
        Ok(c)
    }
    pub fn add_tag(&self, id: &str, tag: &str) -> anyhow::Result<bool> {
        if let Some(mut m) = self.get(id)? {
            if !m.tags.contains(&tag.to_string()) { m.tags.push(tag.to_string()); self.conn.execute("UPDATE memories SET tags=?1,updated_at=?2 WHERE id=?3", params![serde_json::to_string(&m.tags)?, Utc::now().to_rfc3339(), id])?; }
            return Ok(true);
        }
        Ok(false)
    }
    pub fn tag_stats(&self) -> anyhow::Result<Vec<(String, usize, Vec<String>)>> {
        let memories = self.list(None, 10000)?;
        let mut ti: std::collections::HashMap<String, (usize, Vec<String>)> = std::collections::HashMap::new();
        for m in &memories { for tag in &m.tags { let e = ti.entry(tag.clone()).or_insert((0, Vec::new())); e.0 += 1; e.1.push(m.key.clone()); } }
        let mut r: Vec<_> = ti.into_iter().map(|(t, (c, k))| (t, c, k)).collect(); r.sort_by(|a, b| b.1.cmp(&a.1)); Ok(r)
    }
    pub fn batch_delete(&self, ids: &[String]) -> anyhow::Result<usize> { let mut d = 0; for id in ids { if self.delete(id)? { d += 1; } } Ok(d) }
    pub fn batch_add_tag(&self, ids: &[String], tag: &str) -> anyhow::Result<usize> { let mut u = 0; for id in ids { if self.add_tag(id, tag)? { u += 1; } } Ok(u) }
    pub fn batch_update_kind(&self, ids: &[String], kind: &MemoryKind) -> anyhow::Result<usize> {
        let mut u = 0; for id in ids { if let Some(mut m) = self.get(id)? { m.kind = kind.clone(); self.conn.execute("UPDATE memories SET kind=?1,updated_at=?2 WHERE id=?3", params![m.kind.to_string(), Utc::now().to_rfc3339(), id])?; u += 1; } } Ok(u)
    }
    pub fn pin(&self, id: &str) -> anyhow::Result<bool> {
        if let Some(mut m) = self.get(id)? { if !m.tags.contains(&"pinned".to_string()) { m.tags.push("pinned".to_string()); self.conn.execute("UPDATE memories SET tags=?1,updated_at=?2 WHERE id=?3", params![serde_json::to_string(&m.tags)?, Utc::now().to_rfc3339(), id])?; } return Ok(true); } Ok(false)
    }
    pub fn unpin(&self, id: &str) -> anyhow::Result<bool> {
        if let Some(mut m) = self.get(id)? { m.tags.retain(|t| t != "pinned"); self.conn.execute("UPDATE memories SET tags=?1,updated_at=?2 WHERE id=?3", params![serde_json::to_string(&m.tags)?, Utc::now().to_rfc3339(), id])?; return Ok(true); } Ok(false)
    }
    pub fn list_pinned(&self) -> anyhow::Result<Vec<Memory>> { Ok(self.list(None, 10000)?.into_iter().filter(|m| m.tags.contains(&"pinned".to_string())).collect()) }
    pub fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<Memory>> {
        let p = format!("%{}%", query.to_lowercase());
        let mut stmt = self.conn.prepare("SELECT id,kind,key,content,tags,related_ids,created_at,updated_at FROM memories WHERE lower(key) LIKE ?1 OR lower(content) LIKE ?1 OR lower(tags) LIKE ?1 ORDER BY updated_at DESC LIMIT ?2")?;
        let rows = stmt.query_map(params![p, limit], |row| Ok(RawRow { id: row.get(0)?, kind: row.get(1)?, key: row.get(2)?, content: row.get(3)?, tags: row.get(4)?, related_ids: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)? }))?;
        let mut memories = Vec::new();
        for row in rows {
            if let Ok(r) = row {
                if let Ok(m) = r.into_memory() {
                    memories.push(m);
                }
            }
        }
        Ok(memories)
    }
    pub fn fuzzy_search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<ScoredMemory>> {
        let all = self.list(None, 10000)?;
        let ql = query.to_lowercase(); let qw: Vec<&str> = ql.split_whitespace().collect();
        let mut scored: Vec<ScoredMemory> = all.into_iter().filter_map(|m| {
            let kl = m.key.to_lowercase(); let cl = m.content.to_lowercase();
            let mut s = 0.0;
            if kl.contains(&ql) { s += 0.5; } if cl.contains(&ql) { s += 0.3; }
            s += strsim::jaro_winkler(&ql, &kl).max(strsim::jaro_winkler(&ql, &cl)) * 0.3;
            s += qw.iter().map(|w| kl.split_whitespace().map(|k| strsim::jaro_winkler(w, k)).fold(0.0f64, f64::max).max(cl.split_whitespace().map(|c| strsim::jaro_winkler(w, c)).fold(0.0f64, f64::max))).sum::<f64>() / qw.len().max(1) as f64 * 0.4;
            if s > 0.15 { Some(ScoredMemory { memory: m, score: s }) } else { None }
        }).collect();
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit); Ok(scored)
    }
    pub fn search_by_date(&self, after: Option<chrono::DateTime<Utc>>, before: Option<chrono::DateTime<Utc>>, limit: usize) -> anyhow::Result<Vec<Memory>> {
        let mut sql = String::from("SELECT id,kind,key,content,tags,related_ids,created_at,updated_at FROM memories WHERE 1=1");
        let mut pv: Vec<String> = Vec::new();
        if let Some(a) = after { sql.push_str(&format!(" AND updated_at>=?{}", pv.len()+1)); pv.push(a.to_rfc3339()); }
        if let Some(b) = before { sql.push_str(&format!(" AND updated_at<=?{}", pv.len()+1)); pv.push(b.to_rfc3339()); }
        sql.push_str(&format!(" ORDER BY updated_at DESC LIMIT ?{}", pv.len()+1)); pv.push(limit.to_string());
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(pv.iter()), |row| Ok(RawRow { id: row.get(0)?, kind: row.get(1)?, key: row.get(2)?, content: row.get(3)?, tags: row.get(4)?, related_ids: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)? }))?;
        let mut memories = Vec::new();
        for row in rows {
            if let Ok(r) = row {
                if let Ok(m) = r.into_memory() {
                    memories.push(m);
                }
            }
        }
        Ok(memories)
    }
    pub fn faceted_search(&self, query: Option<&str>, kind: Option<&MemoryKind>, tags: &[String], pinned: Option<bool>, limit: usize) -> anyhow::Result<Vec<Memory>> {
        let mut memories = self.list(None, 10000)?;
        if let Some(k) = kind { memories.retain(|m| &m.kind == k); }
        if !tags.is_empty() { memories.retain(|m| tags.iter().any(|t| m.tags.contains(t))); }
        if let Some(true) = pinned { memories.retain(|m| m.tags.contains(&"pinned".to_string())); }
        if let Some(q) = query { let ql = q.to_lowercase(); memories.retain(|m| m.key.to_lowercase().contains(&ql) || m.content.to_lowercase().contains(&ql)); }
        memories.truncate(limit); Ok(memories)
    }
    pub fn memory_size_bytes(&self) -> anyhow::Result<u64> {
        let p: i64 = self.conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let s: i64 = self.conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        Ok((p * s) as u64)
    }
}

struct RawRow { id: String, kind: String, key: String, content: String, tags: String, related_ids: String, created_at: String, updated_at: String }
impl RawRow {
    fn into_memory(self) -> anyhow::Result<Memory> {
        Ok(Memory { id: self.id, kind: self.kind.parse().map_err(|e: String| anyhow::anyhow!(e))?, key: self.key, content: self.content, tags: serde_json::from_str(&self.tags)?, related_ids: serde_json::from_str(&self.related_ids).unwrap_or_default(), created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)?.with_timezone(&Utc), updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)?.with_timezone(&Utc) })
    }
}
