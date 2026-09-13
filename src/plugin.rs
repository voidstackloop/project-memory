use crate::types::Memory;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub hooks: Vec<PluginHook>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    pub event: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugins: Vec<Plugin>,
}

pub fn load_plugins(project_dir: &Path) -> Vec<Plugin> {
    let plugin_dir = project_dir.join(".memory").join("plugins");
    if !plugin_dir.exists() {
        return Vec::new();
    }
    
    let mut plugins = Vec::new();
    for entry in std::fs::read_dir(&plugin_dir).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "toml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(plugin) = toml::from_str::<Plugin>(&content) {
                        plugins.push(plugin);
                    }
                }
            }
        }
    }
    
    plugins
}

pub fn save_plugin(project_dir: &Path, plugin: &Plugin) -> anyhow::Result<()> {
    let plugin_dir = project_dir.join(".memory").join("plugins");
    std::fs::create_dir_all(&plugin_dir)?;
    
    let filename = format!("{}.toml", plugin.name);
    let path = plugin_dir.join(filename);
    let content = toml::to_string_pretty(plugin)?;
    std::fs::write(path, content)?;
    
    Ok(())
}

pub fn remove_plugin(project_dir: &Path, name: &str) -> anyhow::Result<bool> {
    let plugin_dir = project_dir.join(".memory").join("plugins");
    let path = plugin_dir.join(format!("{}.toml", name));
    
    if path.exists() {
        std::fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn run_hook(project_dir: &Path, event: &str, memory: Option<&Memory>) -> anyhow::Result<()> {
    let plugins = load_plugins(project_dir);
    
    for plugin in &plugins {
        if !plugin.enabled {
            continue;
        }
        
        for hook in &plugin.hooks {
            if hook.event == event {
                let mut cmd = Command::new(&hook.command);
                cmd.args(&hook.args);
                
                if let Some(mem) = memory {
                    cmd.env("PMEM_MEMORY_ID", &mem.id);
                    cmd.env("PMEM_MEMORY_KEY", &mem.key);
                    cmd.env("PMEM_MEMORY_KIND", mem.kind.to_string());
                    cmd.env("PMEM_MEMORY_CONTENT", &mem.content);
                    cmd.env("PMEM_MEMORY_TAGS", mem.tags.join(","));
                }
                
                cmd.current_dir(project_dir);
                
                match cmd.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            eprintln!("Plugin '{}' hook '{}' failed", plugin.name, event);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to run plugin '{}': {}", plugin.name, e);
                    }
                }
            }
        }
    }
    
    Ok(())
}

pub fn list_plugins(project_dir: &Path) -> Vec<Plugin> {
    load_plugins(project_dir)
}

pub fn create_example_plugin() -> Plugin {
    Plugin {
        name: "example".to_string(),
        description: "Example plugin that logs memory changes".to_string(),
        version: "0.1.0".to_string(),
        author: "You".to_string(),
        hooks: vec![
            PluginHook {
                event: "memory_added".to_string(),
                command: "echo".to_string(),
                args: vec!["Memory added: $PMEM_MEMORY_KEY".to_string()],
            },
            PluginHook {
                event: "memory_deleted".to_string(),
                command: "echo".to_string(),
                args: vec!["Memory deleted: $PMEM_MEMORY_ID".to_string()],
            },
        ],
        enabled: true,
    }
}
