use std::path::Path;
use std::time::Duration;

pub struct WatchConfig {
    pub interval: Duration,
    pub auto_sync: bool,
    pub auto_backup: bool,
    pub notify: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            auto_sync: false,
            auto_backup: false,
            notify: true,
        }
    }
}

pub fn watch_project(project_dir: &Path, config: &WatchConfig) -> anyhow::Result<()> {
    let memory_dir = project_dir.join(".memory");
    if !memory_dir.exists() {
        anyhow::bail!("No .memory directory found");
    }

    let mut last_count = get_memory_count(project_dir)?;
    let mut last_backup = std::time::Instant::now();

    println!("Watching for changes (interval: {}s)", config.interval.as_secs());
    println!("Press Ctrl+C to stop\n");

    loop {
        std::thread::sleep(config.interval);

        let current_count = get_memory_count(project_dir)?;

        if current_count != last_count {
            let diff = current_count as i64 - last_count as i64;
            if diff > 0 {
                println!("[+] {} new memories added", diff);
            } else {
                println!("[-] {} memories removed", diff.abs());
            }
            last_count = current_count;

            if config.auto_sync {
                println!("  Auto-syncing...");
                // TODO: Implement auto-sync
            }
        }

        if config.auto_backup && last_backup.elapsed() > Duration::from_secs(3600) {
            println!("  Creating auto-backup...");
            match crate::backup::create_backup(project_dir) {
                Ok(path) => println!("  Backup created: {}", path),
                Err(e) => eprintln!("  Backup failed: {}", e),
            }
            last_backup = std::time::Instant::now();
        }
    }
}

fn get_memory_count(project_dir: &Path) -> anyhow::Result<usize> {
    let store = crate::store::MemoryStore::open_in_project(project_dir)?;
    store.count()
}

pub fn watch_status(project_dir: &Path) -> anyhow::Result<WatchStatus> {
    let memory_dir = project_dir.join(".memory");
    let db_path = memory_dir.join("store.db");

    Ok(WatchStatus {
        is_watching: false,
        memory_count: get_memory_count(project_dir).unwrap_or(0),
        db_size: std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0),
        last_modified: std::fs::metadata(&db_path)
            .and_then(|m| m.modified())
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t)),
    })
}

#[derive(Debug)]
pub struct WatchStatus {
    pub is_watching: bool,
    pub memory_count: usize,
    pub db_size: u64,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn format_watch_status(status: &WatchStatus) -> String {
    let mut output = String::new();
    output.push_str("Watch Status\n");
    output.push_str("============\n\n");
    output.push_str(&format!("Watching: {}\n", if status.is_watching { "yes" } else { "no" }));
    output.push_str(&format!("Memories: {}\n", status.memory_count));
    output.push_str(&format!("DB size: {:.2} KB\n", status.db_size as f64 / 1024.0));
    if let Some(dt) = &status.last_modified {
        output.push_str(&format!("Last modified: {}\n", dt.format("%Y-%m-%d %H:%M:%S")));
    }
    output
}
