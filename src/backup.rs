
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

pub fn create_backup(project_dir: &Path) -> anyhow::Result<String> {
    let memory_dir = project_dir.join(".memory");
    let db_path = memory_dir.join("store.db");
    
    if !db_path.exists() {
        anyhow::bail!("No memory database found");
    }
    
    let backup_dir = project_dir.join(".memory").join("backups");
    std::fs::create_dir_all(&backup_dir)?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("backup_{}.db", timestamp));
    
    std::fs::copy(&db_path, &backup_path)?;
    
    let store = crate::store::MemoryStore::open(&db_path)?;
    let memories = store.export_json()?;
    let json_path = backup_dir.join(format!("backup_{}.json", timestamp));
    std::fs::write(&json_path, serde_json::to_string_pretty(&memories)?)?;
    
    Ok(backup_path.to_string_lossy().to_string())
}

pub fn list_backups(project_dir: &Path) -> anyhow::Result<Vec<BackupInfo>> {
    let backup_dir = project_dir.join(".memory").join("backups");
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut backups = Vec::new();
    for entry in std::fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        // Only match the backup_<timestamp>.db convention — excludes pre_restore_*.db safety
        // copies, which aren't regular backups and shouldn't be listed or swept by cleanup.
        let is_backup_file = path.extension().is_some_and(|e| e == "db")
            && path.file_stem().and_then(|n| n.to_str()).is_some_and(|s| s.starts_with("backup_"));
        if is_backup_file {
            let metadata = std::fs::metadata(&path)?;
            let name = path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            backups.push(BackupInfo {
                name,
                path: path.to_string_lossy().to_string(),
                size: metadata.len(),
                created: chrono::DateTime::<chrono::Utc>::from(metadata.modified()?).with_timezone(&chrono::Utc),
            });
        }
    }
    
    backups.sort_by(|a, b| b.created.cmp(&a.created));
    Ok(backups)
}

pub fn restore_backup(project_dir: &Path, backup_name: &str) -> anyhow::Result<()> {
    let backup_dir = project_dir.join(".memory").join("backups");
    let backup_path = backup_dir.join(format!("{}.db", backup_name));

    if !backup_path.exists() {
        anyhow::bail!("Backup '{}' not found", backup_name);
    }

    // Validate the backup is actually a readable memory store before touching live data.
    // Read-only open + a query against the memories table: a plain MemoryStore::open would
    // silently CREATE TABLE IF NOT EXISTS into an empty/foreign file and "validate" it wrongly.
    {
        let conn = Connection::open_with_flags(&backup_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| anyhow::anyhow!("Backup '{}' is not a valid memory store, refusing to restore: {}", backup_name, e))?;
        conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get::<_, i64>(0))
            .map_err(|e| anyhow::anyhow!("Backup '{}' is not a valid memory store, refusing to restore: {}", backup_name, e))?;
    }

    let db_path = project_dir.join(".memory").join("store.db");
    if db_path.exists() {
        // Safety copy of current state so a bad restore can still be undone. Named outside
        // the backup_* naming convention so list_backups/cleanup_backups don't sweep it up.
        let safety_path = backup_dir.join(format!("pre_restore_{}_{}.db", chrono::Utc::now().format("%Y%m%d_%H%M%S"), &uuid::Uuid::new_v4().to_string()[..8]));
        std::fs::copy(&db_path, &safety_path)?;
    }
    std::fs::copy(&backup_path, &db_path)?;

    Ok(())
}

pub fn cleanup_backups(project_dir: &Path, keep: usize) -> anyhow::Result<usize> {
    let backups = list_backups(project_dir)?;
    
    if backups.len() <= keep {
        return Ok(0);
    }
    
    let to_delete = &backups[keep..];
    let mut deleted = 0;
    
    for backup in to_delete {
        let path = Path::new(&backup.path);
        if path.exists() {
            std::fs::remove_file(path)?;
            deleted += 1;
        }
        
        let json_path = path.with_extension("json");
        if json_path.exists() {
            std::fs::remove_file(json_path)?;
        }
    }
    
    Ok(deleted)
}

#[derive(Debug)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created: chrono::DateTime<chrono::Utc>,
}
