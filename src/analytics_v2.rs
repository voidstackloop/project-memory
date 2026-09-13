use crate::types::Memory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsV2 {
    pub overview: OverviewStats,
    pub distribution: DistributionStats,
    pub trends: TrendStats,
    pub quality: QualityStats,
    pub insights: Vec<Insight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewStats {
    pub total_memories: usize,
    pub total_tags: usize,
    pub total_links: usize,
    pub avg_content_length: usize,
    pub oldest_memory: Option<String>,
    pub newest_memory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub by_kind: Vec<(String, usize)>,
    pub by_tag: Vec<(String, usize)>,
    pub by_month: Vec<(String, usize)>,
    pub kind_percentages: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendStats {
    pub memories_added_7d: usize,
    pub memories_added_30d: usize,
    pub memories_updated_7d: usize,
    pub memories_updated_30d: usize,
    pub growth_rate: f64,
    pub most_active_day: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStats {
    pub avg_key_length: usize,
    pub avg_content_length: usize,
    pub memories_with_tags_pct: f64,
    pub memories_with_links_pct: f64,
    pub duplicate_keys: usize,
    pub empty_content: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub category: String,
    pub message: String,
    pub severity: String,
}

pub fn compute_analytics_v2(memories: &[Memory]) -> AnalyticsV2 {
    let now = chrono::Utc::now();
    
    // Overview
    let total_tags: usize = memories.iter().map(|m| m.tags.len()).sum();
    let total_links: usize = memories.iter().map(|m| m.related_ids.len()).sum();
    let avg_content_length = if memories.is_empty() {
        0
    } else {
        memories.iter().map(|m| m.content.len()).sum::<usize>() / memories.len()
    };
    
    let oldest = memories.iter().min_by_key(|m| m.created_at).map(|m| m.created_at.format("%Y-%m-%d").to_string());
    let newest = memories.iter().max_by_key(|m| m.created_at).map(|m| m.created_at.format("%Y-%m-%d").to_string());
    
    let overview = OverviewStats {
        total_memories: memories.len(),
        total_tags,
        total_links,
        avg_content_length,
        oldest_memory: oldest,
        newest_memory: newest,
    };
    
    // Distribution
    let mut kind_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut month_counts: HashMap<String, usize> = HashMap::new();
    
    for m in memories {
        *kind_counts.entry(m.kind.to_string()).or_insert(0) += 1;
        for tag in &m.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
        let month = m.created_at.format("%Y-%m").to_string();
        *month_counts.entry(month).or_insert(0) += 1;
    }
    
    let mut by_kind: Vec<(String, usize)> = kind_counts.into_iter().collect();
    by_kind.sort_by(|a, b| b.1.cmp(&a.1));
    
    let mut by_tag: Vec<(String, usize)> = tag_counts.into_iter().collect();
    by_tag.sort_by(|a, b| b.1.cmp(&a.1));
    
    let mut by_month: Vec<(String, usize)> = month_counts.into_iter().collect();
    by_month.sort_by(|a, b| a.0.cmp(&b.0));
    
    let mut kind_percentages = HashMap::new();
    for (kind, count) in &by_kind {
        kind_percentages.insert(kind.clone(), (*count as f64 / memories.len().max(1) as f64) * 100.0);
    }
    
    let distribution = DistributionStats {
        by_kind,
        by_tag,
        by_month,
        kind_percentages,
    };
    
    // Trends
    let memories_added_7d = memories.iter().filter(|m| (now - m.created_at).num_days() <= 7).count();
    let memories_added_30d = memories.iter().filter(|m| (now - m.created_at).num_days() <= 30).count();
    let memories_updated_7d = memories.iter().filter(|m| (now - m.updated_at).num_days() <= 7).count();
    let memories_updated_30d = memories.iter().filter(|m| (now - m.updated_at).num_days() <= 30).count();
    
    let growth_rate = if memories_added_30d > 0 {
        (memories_added_7d as f64 / memories_added_30d as f64) * 100.0
    } else {
        0.0
    };
    
    let trends = TrendStats {
        memories_added_7d,
        memories_added_30d,
        memories_updated_7d,
        memories_updated_30d,
        growth_rate,
        most_active_day: "N/A".to_string(),
    };
    
    // Quality
    let avg_key_length = if memories.is_empty() {
        0
    } else {
        memories.iter().map(|m| m.key.len()).sum::<usize>() / memories.len()
    };
    
    let memories_with_tags = memories.iter().filter(|m| !m.tags.is_empty()).count();
    let memories_with_links = memories.iter().filter(|m| !m.related_ids.is_empty()).count();
    
    let mut key_counts: HashMap<String, usize> = HashMap::new();
    for m in memories {
        *key_counts.entry(m.key.clone()).or_insert(0) += 1;
    }
    let duplicate_keys = key_counts.values().filter(|&&c| c > 1).count();
    let empty_content = memories.iter().filter(|m| m.content.trim().is_empty()).count();
    
    let quality = QualityStats {
        avg_key_length,
        avg_content_length,
        memories_with_tags_pct: (memories_with_tags as f64 / memories.len().max(1) as f64) * 100.0,
        memories_with_links_pct: (memories_with_links as f64 / memories.len().max(1) as f64) * 100.0,
        duplicate_keys,
        empty_content,
    };
    
    // Insights
    let mut insights = Vec::new();
    
    if quality.memories_with_tags_pct < 50.0 {
        insights.push(Insight {
            category: "Organization".to_string(),
            message: "Less than 50% of memories have tags. Consider adding tags for better searchability.".to_string(),
            severity: "warning".to_string(),
        });
    }
    
    if quality.memories_with_links_pct < 20.0 && memories.len() > 10 {
        insights.push(Insight {
            category: "Relationships".to_string(),
            message: "Few memories are linked. Consider linking related memories for better context.".to_string(),
            severity: "info".to_string(),
        });
    }
    
    if quality.duplicate_keys > 0 {
        insights.push(Insight {
            category: "Quality".to_string(),
            message: format!("{} duplicate keys found. Consider renaming for uniqueness.", quality.duplicate_keys),
            severity: "warning".to_string(),
        });
    }
    
    if trends.memories_added_7d == 0 && memories.len() > 0 {
        insights.push(Insight {
            category: "Activity".to_string(),
            message: "No memories added in the last 7 days. Keep your project memory up to date!".to_string(),
            severity: "info".to_string(),
        });
    }
    
    AnalyticsV2 {
        overview,
        distribution,
        trends,
        quality,
        insights,
    }
}

pub fn format_analytics_v2(analytics: &AnalyticsV2) -> String {
    let mut output = String::new();
    
    output.push_str("Analytics Report v2\n");
    output.push_str("===================\n\n");
    
    // Overview
    output.push_str("Overview:\n");
    output.push_str(&format!("  Total memories: {}\n", analytics.overview.total_memories));
    output.push_str(&format!("  Total tags: {}\n", analytics.overview.total_tags));
    output.push_str(&format!("  Total links: {}\n", analytics.overview.total_links));
    output.push_str(&format!("  Avg content length: {} chars\n", analytics.overview.avg_content_length));
    if let Some(ref oldest) = analytics.overview.oldest_memory {
        output.push_str(&format!("  Oldest: {}\n", oldest));
    }
    if let Some(ref newest) = analytics.overview.newest_memory {
        output.push_str(&format!("  Newest: {}\n", newest));
    }
    output.push_str("\n");
    
    // Distribution
    output.push_str("Kind Distribution:\n");
    for (kind, count) in &analytics.distribution.by_kind {
        let pct = analytics.distribution.kind_percentages.get(kind).unwrap_or(&0.0);
        output.push_str(&format!("  {}: {} ({:.1}%)\n", kind, count, pct));
    }
    output.push_str("\n");
    
    output.push_str("Top Tags:\n");
    for (tag, count) in analytics.distribution.by_tag.iter().take(10) {
        output.push_str(&format!("  {}: {}\n", tag, count));
    }
    output.push_str("\n");
    
    // Trends
    output.push_str("Trends:\n");
    output.push_str(&format!("  Added (7d): {}\n", analytics.trends.memories_added_7d));
    output.push_str(&format!("  Added (30d): {}\n", analytics.trends.memories_added_30d));
    output.push_str(&format!("  Updated (7d): {}\n", analytics.trends.memories_updated_7d));
    output.push_str(&format!("  Growth rate: {:.1}%\n", analytics.trends.growth_rate));
    output.push_str("\n");
    
    // Quality
    output.push_str("Quality:\n");
    output.push_str(&format!("  With tags: {:.1}%\n", analytics.quality.memories_with_tags_pct));
    output.push_str(&format!("  With links: {:.1}%\n", analytics.quality.memories_with_links_pct));
    output.push_str(&format!("  Duplicate keys: {}\n", analytics.quality.duplicate_keys));
    output.push_str(&format!("  Empty content: {}\n", analytics.quality.empty_content));
    output.push_str("\n");
    
    // Insights
    if !analytics.insights.is_empty() {
        output.push_str("Insights:\n");
        for insight in &analytics.insights {
            output.push_str(&format!("  [{}] {}\n", insight.severity, insight.message));
        }
    }
    
    output
}
