use crate::types::Memory;
use std::collections::HashMap;

pub fn get_suggestions(memories: &[Memory], partial: &str) -> Vec<SearchSuggestion> {
    let partial_lower = partial.to_lowercase();
    let mut suggestions: Vec<SearchSuggestion> = Vec::new();
    
    // Suggest by key prefix
    for m in memories {
        if m.key.to_lowercase().starts_with(&partial_lower) {
            suggestions.push(SearchSuggestion {
                text: m.key.clone(),
                kind: SuggestionKind::Key,
                score: 100.0,
            });
        }
    }
    
    // Suggest by tag prefix
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for m in memories {
        for tag in &m.tags {
            if tag.to_lowercase().starts_with(&partial_lower) {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }
    for (tag, count) in tag_counts {
        suggestions.push(SearchSuggestion {
            text: tag,
            kind: SuggestionKind::Tag,
            score: 50.0 + count as f64,
        });
    }
    
    // Suggest by kind
    for kind in crate::types::MemoryKind::all() {
        if kind.to_string().starts_with(&partial_lower) {
            suggestions.push(SearchSuggestion {
                text: kind.to_string(),
                kind: SuggestionKind::Kind,
                score: 30.0,
            });
        }
    }
    
    // Sort by score
    suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    suggestions.truncate(10);
    suggestions
}

pub fn get_popular_searches(memories: &[Memory]) -> Vec<String> {
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    
    for m in memories {
        for word in m.key.split_whitespace() {
            if word.len() > 3 {
                *word_counts.entry(word.to_lowercase()).or_insert(0) += 1;
            }
        }
        for tag in &m.tags {
            *word_counts.entry(tag.to_lowercase()).or_insert(0) += 2;
        }
    }
    
    let mut words: Vec<(String, usize)> = word_counts.into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1));
    words.into_iter().take(10).map(|(w, _)| w).collect()
}

#[derive(Debug)]
pub struct SearchSuggestion {
    pub text: String,
    pub kind: SuggestionKind,
    pub score: f64,
}

#[derive(Debug)]
pub enum SuggestionKind {
    Key,
    Tag,
    Kind,
    Content,
}

pub fn format_suggestions(suggestions: &[SearchSuggestion]) -> String {
    if suggestions.is_empty() {
        return "No suggestions found.".to_string();
    }
    
    let mut output = String::new();
    output.push_str("Suggestions:\n");
    
    for s in suggestions {
        let icon = match s.kind {
            SuggestionKind::Key => "[key]",
            SuggestionKind::Tag => "[tag]",
            SuggestionKind::Kind => "[kind]",
            SuggestionKind::Content => "[txt]",
        };
        output.push_str(&format!("  {} {} ({:.0})\n", icon, s.text, s.score));
    }
    
    output
}
