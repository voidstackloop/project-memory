use crate::types::Memory;

pub struct HighlightedMemory {
    pub memory: Memory,
    pub highlighted_key: String,
    pub highlighted_content: String,
    pub match_score: f64,
}

pub fn highlight_search(memories: &[Memory], query: &str) -> Vec<HighlightedMemory> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    
    let mut results: Vec<HighlightedMemory> = memories
        .iter()
        .filter_map(|m| {
            let key_lower = m.key.to_lowercase();
            let content_lower = m.content.to_lowercase();
            
            let mut score = 0.0;
            let mut matched = false;
            
            for word in &query_words {
                if key_lower.contains(word) {
                    score += 2.0;
                    matched = true;
                }
                if content_lower.contains(word) {
                    score += 1.0;
                    matched = true;
                }
            }
            
            if !matched {
                return None;
            }
            
            let highlighted_key = highlight_text(&m.key, &query_words);
            let highlighted_content = highlight_text(&m.content, &query_words);
            
            Some(HighlightedMemory {
                memory: m.clone(),
                highlighted_key,
                highlighted_content,
                match_score: score,
            })
        })
        .collect();
    
    results.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

fn highlight_text(text: &str, query_words: &[&str]) -> String {
    let mut result = text.to_string();
    
    for word in query_words {
        if word.is_empty() {
            continue;
        }
        
        let lower_text = result.to_lowercase();
        let mut new_result = String::new();
        let mut last_end = 0;
        
        for (start, _) in lower_text.match_indices(word) {
            new_result.push_str(&result[last_end..start]);
            new_result.push_str("**");
            new_result.push_str(&result[start..start + word.len()]);
            new_result.push_str("**");
            last_end = start + word.len();
        }
        
        new_result.push_str(&result[last_end..]);
        result = new_result;
    }
    
    result
}

pub fn format_highlighted(memory: &HighlightedMemory) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("[{}] {}\n", memory.memory.kind, memory.highlighted_key));
    output.push_str(&format!("  {}\n", memory.highlighted_content));
    
    if !memory.memory.tags.is_empty() {
        output.push_str(&format!("  Tags: {}\n", memory.memory.tags.join(", ")));
    }
    
    output.push_str(&format!("  Score: {:.1}\n", memory.match_score));
    
    output
}
