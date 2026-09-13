use crate::types::Memory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub score: f64,
    pub issues: Vec<String>,
    pub auto_fixable: usize,
}

pub fn validate_memories(memories: &[Memory]) -> ValidationResult {
    let mut issues = Vec::new();
    let mut auto_fixable = 0;
    for m in memories {
        if m.content.trim().is_empty() { issues.push(format!("{}: empty content", m.key)); }
        if m.content.len() < 10 { issues.push(format!("{}: very short content", m.key)); }
        let mut tags = m.tags.clone();
        tags.sort();
        tags.dedup();
        if tags.len() != m.tags.len() {
            issues.push(format!("{}: duplicate tags", m.key));
            auto_fixable += 1;
        }
    }
    let score = if memories.is_empty() { 100.0 } else { 100.0 - (issues.len() as f64 * 5.0).max(0.0) };
    ValidationResult { score, auto_fixable, issues }
}

pub fn format_validation(result: &ValidationResult) -> String {
    let mut o = format!("Score: {:.1}/100\nIssues: {}\n", result.score, result.issues.len());
    for i in &result.issues { o.push_str(&format!("  - {}\n", i)); }
    o
}

pub fn auto_fix(memories: &mut Vec<Memory>) -> usize {
    let mut fixed = 0;
    for m in memories.iter_mut() {
        let original_len = m.tags.len();
        m.tags.sort();
        m.tags.dedup();
        if m.tags.len() != original_len { fixed += 1; }
    }
    fixed
}
