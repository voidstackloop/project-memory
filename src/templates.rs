use crate::types::{MemoryInput, MemoryKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template { pub name: String, pub description: String, pub memories: Vec<TemplateMemory> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMemory { pub kind: MemoryKind, pub key: String, pub content: String, pub tags: Vec<String> }
impl TemplateMemory { pub fn to_input(&self) -> MemoryInput { MemoryInput { kind: self.kind.clone(), key: self.key.clone(), content: self.content.clone(), tags: self.tags.clone(), related_ids: vec![] } } }

pub fn get_builtin_templates() -> Vec<Template> {
    vec![
        Template { name: "rust-lib".into(), description: "Rust library conventions".into(), memories: vec![
            TemplateMemory { kind: MemoryKind::Convention, key: "naming".into(), content: "Use snake_case for functions, CamelCase for types".into(), tags: vec!["rust".into()] },
        ]},
        Template { name: "rust-cli".into(), description: "Rust CLI conventions".into(), memories: vec![
            TemplateMemory { kind: MemoryKind::Pattern, key: "cli-framework".into(), content: "Use clap with derive API".into(), tags: vec!["rust".into(), "cli".into()] },
        ]},
        Template { name: "nextjs".into(), description: "Next.js conventions".into(), memories: vec![
            TemplateMemory { kind: MemoryKind::Convention, key: "file-naming".into(), content: "Use kebab-case for files".into(), tags: vec!["nextjs".into()] },
        ]},
        Template { name: "python-api".into(), description: "Python FastAPI conventions".into(), memories: vec![
            TemplateMemory { kind: MemoryKind::Convention, key: "naming".into(), content: "Use snake_case for functions".into(), tags: vec!["python".into()] },
        ]},
    ]
}
pub fn find_template(name: &str) -> Option<Template> { get_builtin_templates().into_iter().find(|t| t.name == name) }
pub fn list_template_names() -> Vec<(String, String)> { get_builtin_templates().iter().map(|t| (t.name.clone(), t.description.clone())).collect() }
