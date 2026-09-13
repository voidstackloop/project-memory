use crate::types::Memory;
use std::path::Path;

pub fn export_dot(memories: &[Memory], path: &Path) -> anyhow::Result<()> {
    let mut dot = String::from("digraph memory_graph {\n");
    for m in memories {
        dot.push_str(&format!("  \"{}\" [label=\"{}\"];\"n", m.id, m.key));
        for rid in &m.related_ids {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", m.id, rid));
        }
    }
    dot.push_str("}
");
    std::fs::write(path, dot)?;
    Ok(())
}

pub fn export_json_graph(memories: &[Memory]) -> serde_json::Value {
    let nodes: Vec<_> = memories.iter().map(|m| serde_json::json!({"id": m.id, "label": m.key})).collect();
    let mut edges = Vec::new();
    for m in memories {
        for rid in &m.related_ids {
            edges.push(serde_json::json!({"source": m.id, "target": rid}));
        }
    }
    serde_json::json!({"nodes": nodes, "edges": edges})
}
