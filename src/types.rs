use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Convention,
    Pattern,
    Decision,
    Preference,
    Context,
}

impl std::fmt::Display for MemoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryKind::Convention => write!(f, "convention"),
            MemoryKind::Pattern => write!(f, "pattern"),
            MemoryKind::Decision => write!(f, "decision"),
            MemoryKind::Preference => write!(f, "preference"),
            MemoryKind::Context => write!(f, "context"),
        }
    }
}

impl MemoryKind {
    pub fn all() -> &'static [MemoryKind] {
        &[
            MemoryKind::Convention,
            MemoryKind::Pattern,
            MemoryKind::Decision,
            MemoryKind::Preference,
            MemoryKind::Context,
        ]
    }
}

impl std::str::FromStr for MemoryKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "convention" | "conv" => Ok(MemoryKind::Convention),
            "pattern" | "pat" => Ok(MemoryKind::Pattern),
            "decision" | "dec" => Ok(MemoryKind::Decision),
            "preference" | "pref" => Ok(MemoryKind::Preference),
            "context" | "ctx" => Ok(MemoryKind::Context),
            _ => Err(format!("Unknown kind: '{s}'. Use: convention, pattern, decision, preference, context")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub kind: MemoryKind,
    pub key: String,
    pub content: String,
    pub tags: Vec<String>,
    pub related_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInput {
    pub kind: MemoryKind,
    pub key: String,
    pub content: String,
    pub tags: Vec<String>,
    pub related_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub memory: Memory,
    pub score: f64,
}

impl std::fmt::Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.kind, self.key, self.content)
    }
}
