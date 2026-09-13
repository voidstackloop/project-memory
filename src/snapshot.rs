use crate::types::Memory;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub memory_count: usize,
    pub created_at: DateTime<Utc>,
    pub memories: Vec<Memory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub added: Vec<Memory>,
    pub removed: Vec<Memory>,
    pub modified: Vec<ModifiedMemory>,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedMemory {
    pub key: String,
    pub old: Memory,
    pub new: Memory,
    pub changes: Vec<String>,
}

pub fn snapshots_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(".memory").join("snapshots")
}

pub fn list_snapshots(project_dir: &Path) -> anyhow::Result<Vec<Snapshot>> {
    let dir = snapshots_dir(project_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(snapshot) = serde_json::from_str::<Snapshot>(&content) {
                snapshots.push(snapshot);
            }
        }
    }
    snapshots.sort_by_key(|a| std::cmp::Reverse(a.created_at));
    Ok(snapshots)
}

pub fn save_snapshot(project_dir: &Path, name: &str, memories: &[Memory]) -> anyhow::Result<Snapshot> {
    let dir = snapshots_dir(project_dir);
    std::fs::create_dir_all(&dir)?;

    let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let snapshot = Snapshot {
        id: id.clone(),
        name: name.to_string(),
        memory_count: memories.len(),
        created_at: Utc::now(),
        memories: memories.to_vec(),
    };

    let filename = format!("{}_{}.json", Utc::now().format("%Y%m%d_%H%M%S"), id);
    let path = dir.join(filename);
    let content = serde_json::to_string_pretty(&snapshot)?;
    std::fs::write(path, content)?;

    Ok(snapshot)
}

pub fn load_snapshot(project_dir: &Path, id_or_name: &str) -> anyhow::Result<Option<Snapshot>> {
    let snapshots = list_snapshots(project_dir)?;

    // Try by ID first
    if let Some(s) = snapshots.iter().find(|s| s.id == id_or_name) {
        return Ok(Some(s.clone()));
    }

    // Try by name
    if let Some(s) = snapshots.iter().find(|s| s.name == id_or_name) {
        return Ok(Some(s.clone()));
    }

    // Try by partial ID
    let matches: Vec<_> = snapshots.iter().filter(|s| s.id.starts_with(id_or_name)).collect();
    if matches.len() == 1 {
        return Ok(Some(matches[0].clone()));
    }

    Ok(None)
}

pub fn diff_snapshots(old: &[Memory], new: &[Memory]) -> DiffResult {
    let old_map: std::collections::HashMap<&str, &Memory> =
        old.iter().map(|m| (m.id.as_str(), m)).collect();
    let new_map: std::collections::HashMap<&str, &Memory> =
        new.iter().map(|m| (m.id.as_str(), m)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = 0;

    // Find added and modified
    for (id, new_mem) in &new_map {
        if let Some(old_mem) = old_map.get(id) {
            let changes = diff_memory(old_mem, new_mem);
            if changes.is_empty() {
                unchanged += 1;
            } else {
                modified.push(ModifiedMemory {
                    key: new_mem.key.clone(),
                    old: (*old_mem).clone(),
                    new: (*new_mem).clone(),
                    changes,
                });
            }
        } else {
            added.push((*new_mem).clone());
        }
    }

    // Find removed
    for (id, old_mem) in &old_map {
        if !new_map.contains_key(id) {
            removed.push((*old_mem).clone());
        }
    }

    DiffResult {
        added,
        removed,
        modified,
        unchanged,
    }
}

fn diff_memory(old: &Memory, new: &Memory) -> Vec<String> {
    let mut changes = Vec::new();

    if old.key != new.key {
        changes.push(format!("key: '{}' → '{}'", old.key, new.key));
    }
    if old.content != new.content {
        changes.push("content changed".to_string());
    }
    if old.kind != new.kind {
        changes.push(format!("kind: '{}' → '{}'", old.kind, new.kind));
    }
    if old.tags != new.tags {
        changes.push(format!("tags: [{}] → [{}]", old.tags.join(", "), new.tags.join(", ")));
    }

    changes
}

pub fn format_diff(diff: &DiffResult) -> String {
    let mut output = String::new();

    if diff.added.is_empty() && diff.removed.is_empty() && diff.modified.is_empty() {
        return "No changes detected.".to_string();
    }

    if !diff.added.is_empty() {
        output.push_str(&format!("{} Added ({}):\n", "+".green(), diff.added.len()));
        for m in &diff.added {
            output.push_str(&format!("  + [{}] {}: {}\n", m.kind, m.key, m.content));
        }
        output.push('\n');
    }

    if !diff.removed.is_empty() {
        output.push_str(&format!("{} Removed ({}):\n", "-".red(), diff.removed.len()));
        for m in &diff.removed {
            output.push_str(&format!("  - [{}] {}: {}\n", m.kind, m.key, m.content));
        }
        output.push('\n');
    }

    if !diff.modified.is_empty() {
        output.push_str(&format!("{} Modified ({}):\n", "~".yellow(), diff.modified.len()));
        for mm in &diff.modified {
            output.push_str(&format!("  ~ [{}] {}\n", mm.old.kind, mm.key));
            for change in &mm.changes {
                output.push_str(&format!("      {change}\n"));
            }
        }
        output.push('\n');
    }

    output.push_str(&format!("{} unchanged", diff.unchanged));

    output
}

// Add colored trait usage
use colored::Colorize;
