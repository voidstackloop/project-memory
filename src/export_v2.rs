use crate::types::Memory;


pub fn export_html_report(memories: &[Memory], title: &str) -> String {
    let mut body = String::new();

    body.push_str(&format!("<h1>{}</h1>\n", title));
    body.push_str(&format!("<p>Total: {} memories</p>\n", memories.len()));

    // Summary
    let mut kind_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for m in memories {
        *kind_counts.entry(m.kind.to_string()).or_insert(0) += 1;
    }

    body.push_str("<div class='summary'>\n");
    for (kind, count) in &kind_counts {
        body.push_str(&format!("  <span class='badge badge-{}'>{}: {}</span>\n", kind, kind, count));
    }
    body.push_str("</div>\n\n");

    // Memories by kind
    let kinds = ["convention", "pattern", "decision", "preference", "context"];
    for kind in &kinds {
        let filtered: Vec<&Memory> = memories.iter().filter(|m| m.kind.to_string() == *kind).collect();
        if !filtered.is_empty() {
            body.push_str(&format!("<h2>{}</h2>\n", kind));
            body.push_str("<div class='cards'>\n");
            for m in filtered {
                body.push_str(&format!("  <div class='card card-{}'>\n", kind));
                body.push_str(&format!("    <h3>{}</h3>\n", m.key));
                body.push_str(&format!("    <p>{}</p>\n", m.content));
                if !m.tags.is_empty() {
                    body.push_str(&format!("    <div class='tags'>{}</div>\n", m.tags.join(", ")));
                }
                body.push_str("  </div>\n");
            }
            body.push_str("</div>\n");
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, sans-serif; background: #0f172a; color: #e2e8f0; padding: 2rem; max-width: 900px; margin: 0 auto; }}
        h1 {{ color: #f8fafc; }}
        h2 {{ color: #94a3b8; border-bottom: 1px solid #334155; padding-bottom: 0.5rem; }}
        .summary {{ margin: 1rem 0; }}
        .badge {{ display: inline-block; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.75rem; margin-right: 0.5rem; }}
        .badge-convention {{ background: #06b6d4; }}
        .badge-pattern {{ background: #22c55e; }}
        .badge-decision {{ background: #eab308; }}
        .badge-preference {{ background: #a855f7; }}
        .badge-context {{ background: #3b82f6; }}
        .cards {{ display: grid; gap: 1rem; }}
        .card {{ background: #1e293b; padding: 1rem; border-radius: 8px; border-left: 4px solid; }}
        .card-convention {{ border-color: #06b6d4; }}
        .card-pattern {{ border-color: #22c55e; }}
        .card-decision {{ border-color: #eab308; }}
        .card-preference {{ border-color: #a855f7; }}
        .card-context {{ border-color: #3b82f6; }}
        .tags {{ margin-top: 0.5rem; font-size: 0.75rem; color: #94a3b8; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        title, body
    )
}

pub fn export_markdown_v2(memories: &[Memory], title: &str) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {}\n\n", title));
    output.push_str(&format!("*{} memories*\n\n", memories.len()));

    // Table of contents
    output.push_str("## Table of Contents\n\n");
    let mut kind_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for m in memories {
        *kind_counts.entry(m.kind.to_string()).or_insert(0) += 1;
    }
    for (kind, count) in &kind_counts {
        output.push_str(&format!("- [{}](#{}) ({})\n", kind, kind, count));
    }
    output.push_str("\n---\n\n");

    // Sections
    let kinds = ["convention", "pattern", "decision", "preference", "context"];
    for kind in &kinds {
        let filtered: Vec<&Memory> = memories.iter().filter(|m| m.kind.to_string() == *kind).collect();
        if !filtered.is_empty() {
            output.push_str(&format!("## {}\n\n", kind));
            for m in filtered {
                output.push_str(&format!("### {}\n\n", m.key));
                output.push_str(&format!("{}\n\n", m.content));
                if !m.tags.is_empty() {
                    output.push_str(&format!("*Tags:* {}\n\n", m.tags.join(", ")));
                }
            }
        }
    }

    output.push_str("---\n\n");
    output.push_str(&format!("*Generated: {}*\n", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")));
    output
}

pub fn export_csv_v2(memories: &[Memory]) -> String {
    let mut output = String::new();
    output.push_str("id,kind,key,content,tags,related_count,created_at,updated_at\n");

    for m in memories {
        let content = m.content.replace('"', "\"\"");
        let key = m.key.replace('"', "\"\"");
        output.push_str(&format!(
            "{},{},\"{}\",\"{}\",\"{}\",{},{},{}\n",
            m.id,
            m.kind,
            key,
            content,
            m.tags.join(";"),
            m.related_ids.len(),
            m.created_at.to_rfc3339(),
            m.updated_at.to_rfc3339()
        ));
    }

    output
}

pub fn export_json_schema() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "memories": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "kind": { "type": "string", "enum": ["convention", "pattern", "decision", "preference", "context"] },
                        "key": { "type": "string" },
                        "content": { "type": "string" },
                        "tags": { "type": "array", "items": { "type": "string" } },
                        "related_ids": { "type": "array", "items": { "type": "string" } },
                        "created_at": { "type": "string", "format": "date-time" },
                        "updated_at": { "type": "string", "format": "date-time" }
                    },
                    "required": ["id", "kind", "key", "content"]
                }
            }
        },
        "required": ["memories"]
    })).unwrap_or_default()
}
