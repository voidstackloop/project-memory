use crate::types::{MemoryInput, MemoryKind};
use std::path::Path;

#[derive(Debug)]
struct Comment {
    file: String,
    line: usize,
    tag: String,
    content: String,
}

pub fn scan_directory(dir: &Path, extensions: &[&str]) -> anyhow::Result<Vec<MemoryInput>> {
    let mut comments = Vec::new();

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if !extensions.contains(&ext) {
            continue;
        }

        let path_str = path.to_string_lossy();
        if path_str.contains("/node_modules/")
            || path_str.contains("/target/")
            || path_str.contains("/.git/")
            || path_str.contains("/dist/")
            || path_str.contains("/build/")
        {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let file_comments = extract_comments(&content, &path.to_string_lossy());
            comments.extend(file_comments);
        }
    }

    let memories = comments_to_memories(comments);
    Ok(memories)
}

fn extract_comments(content: &str, filename: &str) -> Vec<Comment> {
    let mut comments = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        let tags = ["TODO", "FIXME", "HACK", "XXX", "NOTE", "WARN", "SAFETY"];

        for tag in &tags {
            if let Some(pos) = trimmed.find(&format!("// {}:", tag)) {
                let comment_content = trimmed[pos + tag.len() + 3..].trim().to_string();
                if !comment_content.is_empty() {
                    comments.push(Comment {
                        file: filename.to_string(),
                        line: line_num + 1,
                        tag: tag.to_string(),
                        content: comment_content,
                    });
                }
            } else if let Some(pos) = trimmed.find(&format!("# {}:", tag)) {
                let comment_content = trimmed[pos + tag.len() + 2..].trim().to_string();
                if !comment_content.is_empty() {
                    comments.push(Comment {
                        file: filename.to_string(),
                        line: line_num + 1,
                        tag: tag.to_string(),
                        content: comment_content,
                    });
                }
            }
        }
    }

    comments
}

fn comments_to_memories(comments: Vec<Comment>) -> Vec<MemoryInput> {
    let mut memories = Vec::new();

    let mut by_tag: std::collections::HashMap<String, Vec<Comment>> = std::collections::HashMap::new();
    for comment in comments {
        by_tag.entry(comment.tag.clone()).or_default().push(comment);
    }

    for (tag, group) in &by_tag {
        let mut seen = std::collections::HashSet::new();
        for comment in group {
            let key = format!("{}:{}", tag, truncate(&comment.content, 30));
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key.clone());

            let kind = match tag.as_str() {
                "TODO" => MemoryKind::Pattern,
                "FIXME" => MemoryKind::Decision,
                "HACK" => MemoryKind::Decision,
                "XXX" => MemoryKind::Decision,
                "NOTE" => MemoryKind::Context,
                "WARN" => MemoryKind::Convention,
                "SAFETY" => MemoryKind::Convention,
                _ => MemoryKind::Context,
            };

            let content = format!("{} ({}:{})", comment.content, comment.file, comment.line);

            memories.push(MemoryInput {
                kind,
                key: format!("{}: {}", tag.to_lowercase(), truncate(&comment.content, 50)),
                content,
                tags: vec!["code-comment".to_string(), tag.to_lowercase()],
                related_ids: Vec::new(),
            });
        }
    }

    memories
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
