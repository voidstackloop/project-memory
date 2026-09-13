use crate::types::Memory;
use std::path::Path;

pub fn save_export(project_dir: &Path, memories: &[Memory], format: &str) -> anyhow::Result<String> {
    let dir = project_dir.join("exports");
    std::fs::create_dir_all(&dir)?;
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    match format {
        "csv" => {
            let path = dir.join(format!("memories_{}.csv", ts));
            let mut csv = String::from("id,kind,key,content,tags\n");
            for m in memories {
                csv.push_str(&format!("{},{},\"{}\",\"{}\",\"{}\"\n", m.id, m.kind, m.key, m.content.replace('"', "\"\""), m.tags.join(";")));
            }
            std::fs::write(&path, csv)?;
            Ok(path.to_string_lossy().to_string())
        }
        "yaml" | "yml" => {
            let path = dir.join(format!("memories_{}.yaml", ts));
            std::fs::write(&path, serde_yaml::to_string(memories)?)?;
            Ok(path.to_string_lossy().to_string())
        }
        _ => {
            let path = dir.join(format!("memories_{}.json", ts));
            std::fs::write(&path, serde_json::to_string_pretty(memories)?)?;
            Ok(path.to_string_lossy().to_string())
        }
    }
}
