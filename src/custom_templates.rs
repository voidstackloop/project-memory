use crate::types::{MemoryInput, MemoryKind};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTemplate {
    pub name: String,
    pub description: String,
    pub memories: Vec<CustomTemplateMemory>,
    pub created_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTemplateMemory {
    pub kind: String,
    pub key: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub fn save_custom_template(project_dir: &Path, template: &CustomTemplate) -> anyhow::Result<()> {
    let template_dir = project_dir.join(".memory").join("custom_templates");
    std::fs::create_dir_all(&template_dir)?;
    
    let filename = format!("{}.json", template.name);
    let path = template_dir.join(filename);
    let content = serde_json::to_string_pretty(template)?;
    std::fs::write(path, content)?;
    
    Ok(())
}

pub fn load_custom_templates(project_dir: &Path) -> Vec<CustomTemplate> {
    let template_dir = project_dir.join(".memory").join("custom_templates");
    if !template_dir.exists() {
        return Vec::new();
    }
    
    let mut templates = Vec::new();
    for entry in std::fs::read_dir(&template_dir).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(template) = serde_json::from_str::<CustomTemplate>(&content) {
                        templates.push(template);
                    }
                }
            }
        }
    }
    
    templates
}

pub fn find_custom_template(project_dir: &Path, name: &str) -> Option<CustomTemplate> {
    load_custom_templates(project_dir)
        .into_iter()
        .find(|t| t.name == name)
}

pub fn delete_custom_template(project_dir: &Path, name: &str) -> anyhow::Result<bool> {
    let template_dir = project_dir.join(".memory").join("custom_templates");
    let path = template_dir.join(format!("{}.json", name));
    
    if path.exists() {
        std::fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn create_template_from_memories(
    name: &str,
    description: &str,
    memories: &[crate::types::Memory],
) -> CustomTemplate {
    CustomTemplate {
        name: name.to_string(),
        description: description.to_string(),
        memories: memories.iter().map(|m| CustomTemplateMemory {
            kind: m.kind.to_string(),
            key: m.key.clone(),
            content: m.content.clone(),
            tags: m.tags.clone(),
        }).collect(),
        created_at: chrono::Utc::now().to_rfc3339(),
        tags: Vec::new(),
    }
}

pub fn apply_custom_template(project_dir: &Path, name: &str) -> anyhow::Result<usize> {
    let template = find_custom_template(project_dir, name)
        .ok_or_else(|| anyhow::anyhow!("Custom template '{}' not found", name))?;
    
    let store = crate::store::MemoryStore::open_in_project(project_dir)?;
    let mut count = 0;
    
    for mem in &template.memories {
        let kind: MemoryKind = mem.kind.parse().unwrap_or(MemoryKind::Context);
        store.add(MemoryInput {
            kind,
            key: mem.key.clone(),
            content: mem.content.clone(),
            tags: mem.tags.clone(),
            related_ids: Vec::new(),
        })?;
        count += 1;
    }
    
    Ok(count)
}

pub fn list_custom_templates(project_dir: &Path) -> Vec<(String, String, usize)> {
    load_custom_templates(project_dir)
        .iter()
        .map(|t| (t.name.clone(), t.description.clone(), t.memories.len()))
        .collect()
}
