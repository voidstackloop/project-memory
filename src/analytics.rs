use crate::types::{Memory, MemoryKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analytics { pub total: usize, pub by_kind: Vec<(String, usize)>, pub by_tag: Vec<(String, usize)>, pub health_score: f64 }

pub fn compute_analytics(memories: &[Memory]) -> Analytics {
    let total = memories.len();
    let mut kc: HashMap<String, usize> = HashMap::new();
    let mut tc: HashMap<String, usize> = HashMap::new();
    for m in memories { *kc.entry(m.kind.to_string()).or_default() += 1; for t in &m.tags { *tc.entry(t.clone()).or_default() += 1; } }
    let mut by_kind: Vec<_> = kc.into_iter().collect(); by_kind.sort_by(|a,b| b.1.cmp(&a.1));
    let mut by_tag: Vec<_> = tc.into_iter().collect(); by_tag.sort_by(|a,b| b.1.cmp(&a.1));
    let health = if total == 0 { 100.0 } else { 100.0 - (memories.iter().filter(|m| m.content.trim().is_empty()).count() as f64 / total as f64 * 20.0) };
    Analytics { total, by_kind, by_tag, health_score: health }
}

pub fn format_analytics(a: &Analytics) -> String {
    let mut o = String::new();
    o.push_str(&format!("Total: {}\nHealth: {:.1}/100\n\n", a.total, a.health_score));
    o.push_str("By Kind:\n"); for (k,c) in &a.by_kind { o.push_str(&format!("  {}: {}\n", k, c)); }
    o.push_str("\nTop Tags:\n"); for (t,c) in a.by_tag.iter().take(10) { o.push_str(&format!("  {}: {}\n", t, c)); }
    o
}
