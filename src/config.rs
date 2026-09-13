use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_results: usize,
    pub mcp_port: u16,
    pub default_tags: Vec<String>,
    pub auto_link: bool,
    pub fuzzy_search: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_results: 50, mcp_port: 3777, default_tags: vec![], auto_link: false, fuzzy_search: true }
    }
}

impl Config {
    pub fn load(project_dir: &Path) -> Self {
        let path = project_dir.join(".memory").join("config.toml");
        std::fs::read_to_string(&path).ok().and_then(|c| toml::from_str(&c).ok()).unwrap_or_default()
    }
    pub fn save(&self, project_dir: &Path) -> anyhow::Result<()> {
        std::fs::write(project_dir.join(".memory").join("config.toml"), toml::to_string_pretty(self)?)?;
        Ok(())
    }
    pub fn config_path(project_dir: &Path) -> std::path::PathBuf { project_dir.join(".memory").join("config.toml") }
}
