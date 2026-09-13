use crate::types::{Memory, MemoryInput};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct SyncState { pub memories_synced: usize, pub conflicts: Vec<String> }

pub fn sync_memories(local: &Path, remote: &Path, _strategy: &str) -> anyhow::Result<SyncState> {
    let local_store = crate::store::MemoryStore::open_in_project(local)?;
    let remote_store = crate::store::MemoryStore::open_in_project(remote)?;
    let remote_memories = remote_store.export_json()?;
    let local_keys: std::collections::HashSet<String> = local_store.export_json()?.iter().map(|m| m.key.clone()).collect();
    let mut synced = 0;
    for m in &remote_memories {
        if !local_keys.contains(&m.key) {
            local_store.add(MemoryInput { kind: m.kind.clone(), key: m.key.clone(), content: m.content.clone(), tags: m.tags.clone(), related_ids: m.related_ids.clone() })?;
            synced += 1;
        }
    }
    Ok(SyncState { memories_synced: synced, conflicts: vec![] })
}

pub fn export_sync_bundle(project_dir: &Path) -> anyhow::Result<String> {
    let store = crate::store::MemoryStore::open_in_project(project_dir)?;
    Ok(serde_json::to_string_pretty(&store.export_json()?)?)
}

pub fn import_sync_bundle(project_dir: &Path, json: &str, strategy: &str) -> anyhow::Result<usize> {
    let store = crate::store::MemoryStore::open_in_project(project_dir)?;
    let memories: Vec<Memory> = serde_json::from_str(json)?;
    let existing: std::collections::HashSet<String> = store.export_json()?.iter().map(|m| m.key.clone()).collect();
    let mut count = 0;
    for m in &memories {
        if strategy == "skip" && existing.contains(&m.key) { continue; }
        store.add(MemoryInput { kind: m.kind.clone(), key: m.key.clone(), content: m.content.clone(), tags: m.tags.clone(), related_ids: m.related_ids.clone() })?;
        count += 1;
    }
    Ok(count)
}
