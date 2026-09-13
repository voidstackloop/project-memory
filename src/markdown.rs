use crate::types::{MemoryInput, MemoryKind};

pub fn parse_markdown(content: &str) -> Vec<MemoryInput> {
    let mut memories = Vec::new();
    let mut current_section: Option<String> = None;
    let mut current_content = String::new();
    let mut in_code_block = false;

    for line in content.lines() {
        // Track code blocks
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip code blocks
        if in_code_block {
            continue;
        }

        // Detect headers as section names
        if let Some(stripped) = line.strip_prefix("# ") {
            // Save previous section
            if let Some(section) = current_section.take() {
                let content = current_content.trim().to_string();
                if !content.is_empty() && content.len() > 10 {
                    memories.push(MemoryInput {
                        kind: guess_kind(&section, &content),
                        key: section,
                        content,
                        tags: vec!["imported".to_string(), "markdown".to_string()],
                        related_ids: Vec::new(),
                    });
                }
            }
            current_section = Some(stripped.trim().to_string());
            current_content.clear();
        } else if let Some(stripped) = line.strip_prefix("## ") {
            // Save previous section
            if let Some(section) = current_section.take() {
                let content = current_content.trim().to_string();
                if !content.is_empty() && content.len() > 10 {
                    memories.push(MemoryInput {
                        kind: guess_kind(&section, &content),
                        key: section,
                        content,
                        tags: vec!["imported".to_string(), "markdown".to_string()],
                        related_ids: Vec::new(),
                    });
                }
            }
            current_section = Some(stripped.trim().to_string());
            current_content.clear();
        } else if !line.trim().is_empty() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if let Some(section) = current_section {
        let content = current_content.trim().to_string();
        if !content.is_empty() && content.len() > 10 {
            memories.push(MemoryInput {
                kind: guess_kind(&section, &content),
                key: section,
                content,
                tags: vec!["imported".to_string(), "markdown".to_string()],
                related_ids: Vec::new(),
            });
        }
    }

    memories
}

pub fn parse_bullet_list(content: &str) -> Vec<MemoryInput> {
    let mut memories = Vec::new();
    let mut current_key = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            // Save previous
            if !current_key.is_empty() {
                let content = current_content.trim().to_string();
                if !content.is_empty() {
                    memories.push(MemoryInput {
                        kind: guess_kind(&current_key, &content),
                        key: current_key.clone(),
                        content,
                        tags: vec!["imported".to_string()],
                        related_ids: Vec::new(),
                    });
                }
            }
            let bullet = &trimmed[2..];
            if let Some((key, rest)) = bullet.split_once(':') {
                current_key = key.trim().to_string();
                current_content = rest.trim().to_string();
            } else {
                current_key = bullet.to_string();
                current_content.clear();
            }
        } else if !trimmed.is_empty() {
            if !current_content.is_empty() {
                current_content.push(' ');
            }
            current_content.push_str(trimmed);
        }
    }

    // Save last
    if !current_key.is_empty() {
        let content = current_content.trim().to_string();
        if !content.is_empty() {
            memories.push(MemoryInput {
                kind: guess_kind(&current_key, &content),
                key: current_key,
                content,
                tags: vec!["imported".to_string()],
                related_ids: Vec::new(),
            });
        }
    }

    memories
}

fn guess_kind(key: &str, content: &str) -> MemoryKind {
    let lower_key = key.to_lowercase();
    let lower_content = content.to_lowercase();

    // Convention indicators
    if lower_key.contains("naming") || lower_key.contains("style")
        || lower_key.contains("format") || lower_key.contains("convention")
        || lower_content.contains("always use") || lower_content.contains("never use")
        || lower_content.contains("should be")
    {
        return MemoryKind::Convention;
    }

    // Pattern indicators
    if lower_key.contains("pattern") || lower_key.contains("architecture")
        || lower_key.contains("structure") || lower_key.contains("approach")
        || lower_content.contains("use the") || lower_content.contains("implement")
    {
        return MemoryKind::Pattern;
    }

    // Decision indicators
    if lower_key.contains("decision") || lower_key.contains("chose")
        || lower_key.contains("selected") || lower_key.contains("why")
        || lower_content.contains("we decided") || lower_content.contains("we chose")
        || lower_content.contains("rationale")
    {
        return MemoryKind::Decision;
    }

    // Preference indicators
    if lower_key.contains("prefer") || lower_key.contains("favorite")
        || lower_content.contains("we prefer") || lower_content.contains("preferred")
    {
        return MemoryKind::Preference;
    }

    // Default to context
    MemoryKind::Context
}
