
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
        if path.extension().is_some_and(|e| e == "db") {
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
    crate::store::MemoryStore::open(&backup_path)
        .map_err(|e| anyhow::anyhow!("Backup '{}' is not a valid memory store, refusing to restore: {}", backup_name, e))?;

    let db_path = project_dir.join(".memory").join("store.db");
    if db_path.exists() {
        // Safety copy of current state so a bad restore can still be undone.
        let safety_path = backup_dir.join(format!("pre_restore_{}.db", chrono::Utc::now().format("%Y%m%d_%H%M%S")));
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
