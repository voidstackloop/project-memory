use crate::types::Memory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub score: f64,
    pub total_issues: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub auto_fixable: usize,
    pub issues: Vec<Issue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub memory_id: Option<String>,
    pub memory_key: Option<String>,
    pub auto_fixable: bool,
}

pub fn validate_v2(memories: &[Memory]) -> ValidationReport {
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // Check for empty content
    let empty_count = memories.iter().filter(|m| m.content.trim().is_empty()).count();
    if empty_count > 0 {
        issues.push(Issue {
            severity: "error".to_string(),
            category: "content".to_string(),
            message: format!("{} memories have empty content", empty_count),
            memory_id: None,
            memory_key: None,
            auto_fixable: false,
        });
        recommendations.push("Add content to empty memories or delete them".to_string());
    }

    // Check for very short content
    let short_count = memories.iter().filter(|m| m.content.len() < 10 && !m.content.trim().is_empty()).count();
    if short_count > 0 {
        issues.push(Issue {
            severity: "warning".to_string(),
            category: "content".to_string(),
            message: format!("{} memories have very short content (<10 chars)", short_count),
            memory_id: None,
            memory_key: None,
            auto_fixable: false,
        });
    }

    // Check for duplicate tags
    let dup_tag_count = memories.iter().filter(|m| {
        let mut tags = m.tags.clone();
        tags.sort();
        tags.dedup();
        tags.len() != m.tags.len()
    }).count();
    if dup_tag_count > 0 {
        issues.push(Issue {
            severity: "warning".to_string(),
            category: "tags".to_string(),
            message: format!("{} memories have duplicate tags", dup_tag_count),
            memory_id: None,
            memory_key: None,
            auto_fixable: true,
        });
    }

    // Check for broken related_ids
    let ids: std::collections::HashSet<String> = memories.iter().map(|m| m.id.clone()).collect();
    let mut _broken_count = 0;
    for m in memories {
        for rid in &m.related_ids {
            if !ids.contains(rid) {
                _broken_count += 1;
                issues.push(Issue {
                    severity: "error".to_string(),
                    category: "relationships".to_string(),
                    message: format!("Memory '{}' has broken related_id", m.key),
                    memory_id: Some(m.id.clone()),
                    memory_key: Some(m.key.clone()),
                    auto_fixable: true,
                });
            }
        }
    }

    // Check for self-references
    let self_ref_count = memories.iter().filter(|m| m.related_ids.contains(&m.id)).count();
    if self_ref_count > 0 {
        issues.push(Issue {
            severity: "warning".to_string(),
            category: "relationships".to_string(),
            message: format!("{} memories reference themselves", self_ref_count),
            memory_id: None,
            memory_key: None,
            auto_fixable: true,
        });
    }

    // Check for duplicate keys
    let mut key_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for m in memories {
        *key_counts.entry(m.key.clone()).or_insert(0) += 1;
    }
    let dup_key_count = key_counts.values().filter(|&&c| c > 1).count();
    if dup_key_count > 0 {
        issues.push(Issue {
            severity: "warning".to_string(),
            category: "quality".to_string(),
            message: format!("{} duplicate keys found", dup_key_count),
            memory_id: None,
            memory_key: None,
            auto_fixable: false,
        });
        recommendations.push("Rename duplicate keys for uniqueness".to_string());
    }

    // Check for old memories
    let old_count = memories.iter().filter(|m| {
        (chrono::Utc::now() - m.updated_at).num_days() > 365
    }).count();
    if old_count > 0 {
        issues.push(Issue {
            severity: "info".to_string(),
            category: "freshness".to_string(),
            message: format!("{} memories haven't been updated in over a year", old_count),
            memory_id: None,
            memory_key: None,
            auto_fixable: false,
        });
        recommendations.push("Review old memories for relevance".to_string());
    }

    // Check for missing tags
    let no_tag_count = memories.iter().filter(|m| m.tags.is_empty()).count();
    if no_tag_count > memories.len() / 2 {
        issues.push(Issue {
            severity: "info".to_string(),
            category: "organization".to_string(),
            message: format!("{} memories have no tags ({:.0}%)", no_tag_count, (no_tag_count as f64 / memories.len().max(1) as f64) * 100.0),
            memory_id: None,
            memory_key: None,
            auto_fixable: false,
        });
        recommendations.push("Add tags to memories for better searchability".to_string());
    }

    // Calculate score
    let errors = issues.iter().filter(|i| i.severity == "error").count();
    let warnings = issues.iter().filter(|i| i.severity == "warning").count();
    let infos = issues.iter().filter(|i| i.severity == "info").count();
    let auto_fixable = issues.iter().filter(|i| i.auto_fixable).count();

    let score = if memories.is_empty() {
        100.0
    } else {
        let error_penalty = errors as f64 * 15.0;
        let warning_penalty = warnings as f64 * 3.0;
        (100.0 - error_penalty - warning_penalty).max(0.0)
    };

    ValidationReport {
        score,
        total_issues: issues.len(),
        errors,
        warnings,
        infos,
        auto_fixable,
        issues,
        recommendations,
    }
}

pub fn auto_fix_v2(memories: &mut Vec<Memory>) -> usize {
    let mut fixed = 0;

    // Fix duplicate tags
    for m in memories.iter_mut() {
        let original_len = m.tags.len();
        m.tags.sort();
        m.tags.dedup();
        if m.tags.len() != original_len {
            fixed += 1;
        }
    }

    // Fix self-references
    for m in memories.iter_mut() {
        let original_len = m.related_ids.len();
        m.related_ids.retain(|id| id != &m.id);
        if m.related_ids.len() != original_len {
            fixed += 1;
        }
    }

    // Remove broken related_ids
    let ids: std::collections::HashSet<String> = memories.iter().map(|m| m.id.clone()).collect();
    for m in memories.iter_mut() {
        let original_len = m.related_ids.len();
        m.related_ids.retain(|id| ids.contains(id));
        if m.related_ids.len() != original_len {
            fixed += 1;
        }
    }

    fixed
}

pub fn format_validation_report(report: &ValidationReport) -> String {
    let mut output = String::new();

    output.push_str(&format!("Validation Report\n"));
    output.push_str(&format!("=================\n\n"));
    output.push_str(&format!("Score: {:.1}/100\n\n", report.score));

    output.push_str(&format!("Issues: {}\n", report.total_issues));
    output.push_str(&format!("  Errors: {}\n", report.errors));
    output.push_str(&format!("  Warnings: {}\n", report.warnings));
    output.push_str(&format!("  Info: {}\n", report.infos));
    output.push_str(&format!("  Auto-fixable: {}\n\n", report.auto_fixable));

    if !report.issues.is_empty() {
        output.push_str("Details:\n");
        for issue in &report.issues {
            let icon = match issue.severity.as_str() {
                "error" => "[error]",
                "warning" => "[warn]",
                _ => "[info]",
            };
            output.push_str(&format!("  {} {}\n", icon, issue.message));
        }
        output.push('\n');
    }

    if !report.recommendations.is_empty() {
        output.push_str("Recommendations:\n");
        for rec in &report.recommendations {
            output.push_str(&format!("  - {}\n", rec));
        }
    }

    output
}
