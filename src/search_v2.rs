use crate::types::Memory;


#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memory: Memory,
    pub score: f64,
    pub match_type: MatchType,
    pub matched_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
    Semantic,
}

pub fn advanced_search(
    memories: &[Memory],
    query: &str,
    filters: &SearchFilters,
) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut results: Vec<SearchResult> = memories
        .iter()
        .filter_map(|m| {
            // Apply filters
            if let Some(ref kind) = filters.kind {
                if m.kind.to_string() != *kind {
                    return None;
                }
            }

            if let Some(ref tag) = filters.tag {
                if !m.tags.contains(tag) {
                    return None;
                }
            }

            if filters.pinned_only && !m.tags.contains(&"pinned".to_string()) {
                return None;
            }

            // Calculate match
            let key_lower = m.key.to_lowercase();
            let content_lower = m.content.to_lowercase();
            let tags_lower = m.tags.join(" ").to_lowercase();

            let mut score = 0.0;
            let mut match_type = MatchType::Fuzzy;
            let mut matched_fields = Vec::new();

            // Exact match
            if key_lower == query_lower {
                score += 100.0;
                match_type = MatchType::Exact;
                matched_fields.push("key".to_string());
            }
            // Prefix match
            else if key_lower.starts_with(&query_lower) {
                score += 80.0;
                match_type = MatchType::Prefix;
                matched_fields.push("key".to_string());
            }
            // Contains match
            else if key_lower.contains(&query_lower) {
                score += 60.0;
                match_type = MatchType::Contains;
                matched_fields.push("key".to_string());
            }

            // Content match
            if content_lower.contains(&query_lower) {
                score += 40.0;
                matched_fields.push("content".to_string());
            }

            // Tag match
            if tags_lower.contains(&query_lower) {
                score += 50.0;
                matched_fields.push("tags".to_string());
            }

            // Word-level matching
            for word in &query_words {
                if key_lower.contains(word) {
                    score += 10.0;
                }
                if content_lower.contains(word) {
                    score += 5.0;
                }
            }

            // Fuzzy matching with strsim
            let key_sim = strsim::jaro_winkler(&query_lower, &key_lower);
            let content_sim = strsim::jaro_winkler(&query_lower, &content_lower);
            score += key_sim * 20.0;
            score += content_sim * 10.0;

            // Recency bonus
            let age_days = (chrono::Utc::now() - m.updated_at).num_days();
            if age_days < 7 {
                score += 5.0;
            } else if age_days < 30 {
                score += 2.0;
            }

            // Tag count bonus
            score += m.tags.len() as f64 * 1.0;

            // Related memories bonus
            score += m.related_ids.len() as f64 * 2.0;

            if score > 10.0 {
                Some(SearchResult {
                    memory: m.clone(),
                    score,
                    match_type,
                    matched_fields,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(filters.limit);
    results
}

#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub kind: Option<String>,
    pub tag: Option<String>,
    pub pinned_only: bool,
    pub limit: usize,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            kind: None,
            tag: None,
            pinned_only: false,
            limit: 20,
        }
    }
}

pub fn format_search_results(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "No results found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("Found {} results:\n\n", results.len()));

    for (i, r) in results.iter().enumerate() {
        let match_icon = match r.match_type {
            MatchType::Exact => "[exact]",
            MatchType::Prefix => "[prefix]",
            MatchType::Contains => "[contains]",
            MatchType::Fuzzy => "[fuzzy]",
            MatchType::Semantic => "[semantic]",
        };

        output.push_str(&format!(
            "{}. {} {} {} (score: {:.1})\n",
            i + 1,
            match_icon,
            r.memory.kind,
            r.memory.key,
            r.score
        ));
        output.push_str(&format!("   {}\n", r.memory.content));
        output.push_str(&format!("   Fields: {}\n", r.matched_fields.join(", ")));
        output.push('\n');
    }

    output
}

pub fn search_suggestions(memories: &[Memory], partial: &str) -> Vec<String> {
    let partial_lower = partial.to_lowercase();
    let mut suggestions = Vec::new();

    // Key suggestions
    for m in memories {
        if m.key.to_lowercase().starts_with(&partial_lower) {
            suggestions.push(m.key.clone());
        }
    }

    // Tag suggestions
    let mut tags: Vec<String> = memories
        .iter()
        .flat_map(|m| m.tags.clone())
        .filter(|t| t.to_lowercase().starts_with(&partial_lower))
        .collect();
    tags.sort();
    tags.dedup();
    suggestions.extend(tags);

    suggestions.truncate(10);
    suggestions
}
