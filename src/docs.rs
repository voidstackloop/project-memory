use crate::types::Memory;
use std::path::Path;

pub fn generate_markdown(memories: &[Memory], title: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!("# {}\n\n", title));
    output.push_str(&format!("*Generated from {} memories*\n\n", memories.len()));
    output.push_str("---\n\n");

    let mut by_kind: std::collections::HashMap<String, Vec<&Memory>> = std::collections::HashMap::new();
    for m in memories {
        by_kind.entry(m.kind.to_string()).or_default().push(m);
    }

    let sections = [
        ("convention", "Conventions", "Coding standards and naming rules."),
        ("pattern", "Patterns", "Architectural patterns and common approaches."),
        ("decision", "Decisions", "Technical decisions and rationale."),
        ("preference", "Preferences", "Team preferences and tool choices."),
        ("context", "Context", "Domain knowledge and business rules."),
    ];

    for (kind, label, desc) in &sections {
        if let Some(mems) = by_kind.get(*kind) {
            output.push_str(&format!("## {}\n\n", label));
            output.push_str(&format!("{}\n\n", desc));
            for m in mems {
                output.push_str(&format!("### {}\n\n", m.key));
                output.push_str(&format!("{}\n", m.content));
                if !m.tags.is_empty() {
                    let tags: Vec<String> = m.tags.iter().filter(|t| *t != "pinned" && *t != "group").cloned().collect();
                    if !tags.is_empty() {
                        output.push_str(&format!("\n*Tags: {}*\n", tags.join(", ")));
                    }
                }
                output.push('\n');
            }
        }
    }

    output.push_str("---\n\n");
    output.push_str(&format!("*Last updated: {}*\n", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")));
    output
}

pub fn generate_html(memories: &[Memory], title: &str) -> String {
    let mut body = String::new();
    body.push_str(&format!("<h1>{}</h1>\n", title));
    body.push_str(&format!("<p><em>Generated from {} memories</em></p>\n", memories.len()));

    let mut by_kind: std::collections::HashMap<String, Vec<&Memory>> = std::collections::HashMap::new();
    for m in memories {
        by_kind.entry(m.kind.to_string()).or_default().push(m);
    }

    let kind_labels = [
        ("convention", "Conventions", "#06b6d4"),
        ("pattern", "Patterns", "#22c55e"),
        ("decision", "Decisions", "#eab308"),
        ("preference", "Preferences", "#a855f7"),
        ("context", "Context", "#3b82f6"),
    ];

    for (kind, label, color) in &kind_labels {
        if let Some(mems) = by_kind.get(*kind) {
            body.push_str(&format!("<h2 style='color:{}'>{}</h2>\n", color, label));
            body.push_str("<div style='display:grid;gap:1rem;'>\n");
            for m in mems {
                body.push_str(&format!(
                    "<div style='background:#1e293b;padding:1rem;border-radius:8px;border-left:4px solid {}'>\n",
                    color
                ));
                body.push_str(&format!("  <strong>{}</strong><br>\n", m.key));
                body.push_str(&format!("  <p>{}</p>\n", m.content));
                body.push_str("</div>\n");
            }
            body.push_str("</div>\n");
        }
    }

    format!(
        "<!DOCTYPE html>\n<html><head><title>{}</title></head><body>{}</body></html>",
        title, body
    )
}

pub fn save_documentation(project_dir: &Path, memories: &[Memory], format: &str) -> anyhow::Result<String> {
    let docs_dir = project_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    let title = project_dir.file_name().and_then(|n| n.to_str()).unwrap_or("Project Memory");

    match format {
        "markdown" | "md" => {
            let content = generate_markdown(memories, title);
            let path = docs_dir.join("memories.md");
            std::fs::write(&path, content)?;
            Ok(path.to_string_lossy().to_string())
        }
        "html" => {
            let content = generate_html(memories, title);
            let path = docs_dir.join("memories.html");
            std::fs::write(&path, content)?;
            Ok(path.to_string_lossy().to_string())
        }
        _ => anyhow::bail!("Unknown format: {}. Use 'markdown' or 'html'", format),
    }
}
