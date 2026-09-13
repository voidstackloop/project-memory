use crate::types::Memory;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub memory: Option<WebhookMemory>,
    pub timestamp: String,
    pub project: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMemory {
    pub id: String,
    pub kind: String,
    pub key: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub fn load_webhooks(project_dir: &Path) -> Vec<Webhook> {
    let webhook_path = project_dir.join(".memory").join("webhooks.toml");
    if !webhook_path.exists() {
        return Vec::new();
    }
    
    match std::fs::read_to_string(&webhook_path) {
        Ok(content) => {
            #[derive(Deserialize)]
            struct WebhookConfig {
                webhooks: Vec<Webhook>,
            }
            toml::from_str::<WebhookConfig>(&content)
                .map(|c| c.webhooks)
                .unwrap_or_default()
        }
        Err(_) => Vec::new(),
    }
}

pub fn save_webhooks(project_dir: &Path, webhooks: &[Webhook]) -> anyhow::Result<()> {
    let webhook_path = project_dir.join(".memory").join("webhooks.toml");
    
    #[derive(Serialize)]
    struct WebhookConfig {
        webhooks: Vec<Webhook>,
    }
    
    let config = WebhookConfig {
        webhooks: webhooks.to_vec(),
    };
    
    let content = toml::to_string_pretty(&config)?;
    std::fs::write(webhook_path, content)?;
    
    Ok(())
}

pub fn add_webhook(project_dir: &Path, webhook: Webhook) -> anyhow::Result<()> {
    let mut webhooks = load_webhooks(project_dir);
    webhooks.push(webhook);
    save_webhooks(project_dir, &webhooks)
}

pub fn remove_webhook(project_dir: &Path, name: &str) -> anyhow::Result<bool> {
    let mut webhooks = load_webhooks(project_dir);
    let original_len = webhooks.len();
    webhooks.retain(|w| w.name != name);
    
    if webhooks.len() < original_len {
        save_webhooks(project_dir, &webhooks)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn trigger_webhooks(project_dir: &Path, event: &str, memory: Option<&Memory>) {
    let webhooks = load_webhooks(project_dir);
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let payload = WebhookPayload {
        event: event.to_string(),
        memory: memory.map(|m| WebhookMemory {
            id: m.id.clone(),
            kind: m.kind.to_string(),
            key: m.key.clone(),
            content: m.content.clone(),
            tags: m.tags.clone(),
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
        project: project_name,
    };
    
    for webhook in &webhooks {
        if !webhook.enabled || !webhook.events.iter().any(|e| e == event) {
            continue;
        }
        
        let client = reqwest::blocking::Client::new();
        let mut request = client.post(&webhook.url)
            .json(&payload);
        
        if let Some(ref secret) = webhook.secret {
            request = request.header("X-Webhook-Secret", secret.as_str());
        }
        
        match request.send() {
            Ok(response) => {
                if !response.status().is_success() {
                    eprintln!("Webhook '{}' returned status {}", webhook.name, response.status());
                }
            }
            Err(e) => {
                eprintln!("Failed to trigger webhook '{}': {}", webhook.name, e);
            }
        }
    }
}

pub fn list_webhooks(project_dir: &Path) -> Vec<Webhook> {
    load_webhooks(project_dir)
}
