use crate::types::{MemoryInput, MemoryKind};
use std::path::Path;

pub fn import_from_obsidian(vault_path: &Path) -> anyhow::Result<Vec<MemoryInput>> {
    let mut memories = Vec::new();
    
    for entry in walkdir::WalkDir::new(vault_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let path = entry.path();
        if let Ok(content) = std::fs::read_to_string(path) {
            let file_stem = path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("untitled");
            
            let (frontmatter, body) = parse_frontmatter(&content);
            
            let tags: Vec<String> = frontmatter
                .get("tags")
                .map(|t| {
                    t.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            
            let kind = if frontmatter.get("type").is_some_and(|t| t == "convention") {
                MemoryKind::Convention
            } else if frontmatter.get("type").is_some_and(|t| t == "pattern") {
                MemoryKind::Pattern
            } else if frontmatter.get("type").is_some_and(|t| t == "decision") {
                MemoryKind::Decision
            } else {
                MemoryKind::Context
            };
            
            memories.push(MemoryInput {
                kind,
                key: file_stem.to_string(),
                content: body,
                tags,
                related_ids: Vec::new(),
            });
        }
    }
    
    Ok(memories)
}

fn parse_frontmatter(content: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut frontmatter = std::collections::HashMap::new();
    let mut body = content.to_string();
    
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let fm_str = &content[3..end + 3];
            body = content[end + 6..].trim().to_string();
            
            for line in fm_str.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    frontmatter.insert(
                        key.trim().to_string(),
                        value.trim().to_string(),
                    );
                }
            }
        }
    }
    
    (frontmatter, body)
}

pub fn import_from_json_file(json_path: &Path) -> anyhow::Result<Vec<MemoryInput>> {
    let content = std::fs::read_to_string(json_path)?;
    let memories: Vec<MemoryInput> = serde_json::from_str(&content)?;
    Ok(memories)
}

pub fn import_from_markdown_dir(dir: &Path) -> anyhow::Result<Vec<MemoryInput>> {
    let mut memories = Vec::new();
    
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let path = entry.path();
        if let Ok(content) = std::fs::read_to_string(path) {
            let file_stem = path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("untitled");
            
            let parsed = crate::markdown::parse_markdown(&content);
            if !parsed.is_empty() {
                memories.extend(parsed);
            } else {
                memories.push(MemoryInput {
                    kind: MemoryKind::Context,
                    key: file_stem.to_string(),
                    content,
                    tags: vec!["imported".to_string()],
                    related_ids: Vec::new(),
                });
            }
        }
    }
    
    Ok(memories)
}

pub fn import_from_notion_csv(csv_path: &std::path::Path) -> anyhow::Result<Vec<MemoryInput>> {
    let content = std::fs::read_to_string(csv_path)?;
    let mut memories = Vec::new();
    
    let mut lines = content.lines();
    let header = lines.next().unwrap_or("");
    
    let headers: Vec<&str> = header.split(',').collect();
    let name_idx = headers.iter().position(|h| h.trim().to_lowercase() == "name");
    let content_idx = headers.iter().position(|h| h.trim().to_lowercase() == "content");
    let tags_idx = headers.iter().position(|h| h.trim().to_lowercase() == "tags");
    let type_idx = headers.iter().position(|h| h.trim().to_lowercase() == "type");
    
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        
        let name = name_idx.and_then(|i| fields.get(i)).unwrap_or(&"").trim();
        let content_val = content_idx.and_then(|i| fields.get(i)).unwrap_or(&"").trim();
        let tags_str = tags_idx.and_then(|i| fields.get(i)).unwrap_or(&"").trim();
        let type_str = type_idx.and_then(|i| fields.get(i)).unwrap_or(&"").trim();
        
        if name.is_empty() && content_val.is_empty() {
            continue;
        }
        
        let kind = match type_str.to_lowercase().as_str() {
            "convention" | "conv" => MemoryKind::Convention,
            "pattern" | "pat" => MemoryKind::Pattern,
            "decision" | "dec" => MemoryKind::Decision,
            "preference" | "pref" => MemoryKind::Preference,
            _ => MemoryKind::Context,
        };
        
        let tags: Vec<String> = tags_str
            .split(';')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        
        memories.push(MemoryInput {
            kind,
            key: name.to_string(),
            content: content_val.to_string(),
            tags,
            related_ids: Vec::new(),
        });
    }
    
    Ok(memories)
}
