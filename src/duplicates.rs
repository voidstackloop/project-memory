use crate::types::Memory;
use strsim::jaro_winkler;

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub memories: Vec<Memory>,
    pub similarity: f64,
    pub reason: String,
}

pub fn find_duplicates(memories: &[Memory], threshold: f64) -> Vec<DuplicateGroup> {
    let mut groups: Vec<DuplicateGroup> = Vec::new();
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 0..memories.len() {
        if used.contains(&memories[i].id) {
            continue;
        }

        let mut group_memories = vec![memories[i].clone()];
        used.insert(memories[i].id.clone());

        for j in (i + 1)..memories.len() {
            if used.contains(&memories[j].id) {
                continue;
            }

            let (similarity, _reason) = calculate_similarity(&memories[i], &memories[j]);
            if similarity >= threshold {
                group_memories.push(memories[j].clone());
                used.insert(memories[j].id.clone());
            }
        }

        if group_memories.len() > 1 {
            let avg_sim = group_memories.windows(2)
                .map(|w| calculate_similarity(&w[0], &w[1]).0)
                .sum::<f64>()
                / (group_memories.len() - 1) as f64;

            groups.push(DuplicateGroup {
                memories: group_memories,
                similarity: avg_sim,
                reason: "similar content".to_string(),
            });
        }
    }

    groups.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
    groups
}

fn calculate_similarity(a: &Memory, b: &Memory) -> (f64, String) {
    let key_sim = jaro_winkler(&a.key.to_lowercase(), &b.key.to_lowercase());
    let content_sim = jaro_winkler(&a.content.to_lowercase(), &b.content.to_lowercase());

    // Exact key match is strong signal
    if a.key.to_lowercase() == b.key.to_lowercase() {
        return (0.95, "exact key match".to_string());
    }

    // Same kind + high content similarity
    if a.kind == b.kind && content_sim > 0.85 {
        return (content_sim, "same kind + similar content".to_string());
    }

    // High key similarity
    if key_sim > 0.9 {
        return (key_sim, "very similar key".to_string());
    }

    // Combined score
    let combined = key_sim * 0.4 + content_sim * 0.6;
    (combined, "similar content".to_string())
}

pub fn merge_group(group: &DuplicateGroup) -> Memory {
    // Use the most recently updated memory as base
    let base = group.memories.iter()
        .max_by(|a, b| a.updated_at.cmp(&b.updated_at))
        .unwrap()
        .clone();

    // Collect all unique tags
    let mut all_tags: Vec<String> = Vec::new();
    for m in &group.memories {
        for tag in &m.tags {
            if !all_tags.contains(tag) {
                all_tags.push(tag.clone());
            }
        }
    }

    // Collect all unique related IDs
    let mut all_related: Vec<String> = Vec::new();
    for m in &group.memories {
        for rid in &m.related_ids {
            if !all_related.contains(rid) && !group.memories.iter().any(|gm| &gm.id == rid) {
                all_related.push(rid.clone());
            }
        }
    }

    Memory {
        id: base.id,
        kind: base.kind,
        key: base.key,
        content: base.content,
        tags: all_tags,
        related_ids: all_related,
        created_at: group.memories.iter()
            .map(|m| m.created_at)
            .min()
            .unwrap_or(base.created_at),
        updated_at: chrono::Utc::now(),
    }
}

pub fn format_duplicates(groups: &[DuplicateGroup]) -> String {
    if groups.is_empty() {
        return "No duplicates found.".to_string();
    }

    let mut output = format!("Found {} duplicate group(s):\n\n", groups.len());

    for (i, group) in groups.iter().enumerate() {
        output.push_str(&format!("Group {} (similarity: {:.0}%):\n", i + 1, group.similarity * 100.0));
        for m in &group.memories {
            output.push_str(&format!("  - [{}] {}: {} ({})\n", m.kind, m.key, &m.id[..8], group.reason));
        }
        output.push('\n');
    }

    output.push_str("Use `pmem dedupe merge` to merge duplicates.\n");
    output.push_str("Use `pmem dedupe --threshold 0.9` to adjust sensitivity.\n");

    output
}
