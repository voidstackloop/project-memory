use std::path::Path;
use colored::Colorize;

const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# project-memory: auto-export memories on commit
if command -v pmem >/dev/null 2>&1; then
    pmem export > .memory/memories.json 2>/dev/null || true
    if [ -f .memory/memories.json ]; then
        git add .memory/memories.json 2>/dev/null || true
    fi
fi
"#;

const POST_MERGE_HOOK: &str = r#"#!/bin/sh
# project-memory: reload after merge
if command -v pmem >/dev/null 2>&1; then
    echo "project-memory: memories synced"
fi
"#;

pub fn install_hooks(project_dir: &Path) -> anyhow::Result<()> {
    let hooks_dir = project_dir.join(".git").join("hooks");
    if !hooks_dir.exists() {
        anyhow::bail!("Not a git repository (no .git/hooks directory)");
    }

    let pre_commit_path = hooks_dir.join("pre-commit");
    if pre_commit_path.exists() {
        let existing = std::fs::read_to_string(&pre_commit_path)?;
        if existing.contains("project-memory") {
            println!("  pre-commit hook already installed");
        } else {
            let mut content = existing;
            content.push('\n');
            content.push_str(PRE_COMMIT_HOOK);
            std::fs::write(&pre_commit_path, content)?;
            make_executable(&pre_commit_path)?;
            println!("  added project-memory to existing pre-commit hook");
        }
    } else {
        std::fs::write(&pre_commit_path, PRE_COMMIT_HOOK)?;
        make_executable(&pre_commit_path)?;
        println!("  installed pre-commit hook");
    }

    let post_merge_path = hooks_dir.join("post-merge");
    if post_merge_path.exists() {
        let existing = std::fs::read_to_string(&post_merge_path)?;
        if existing.contains("project-memory") {
            println!("  post-merge hook already installed");
        } else {
            let mut content = existing;
            content.push('\n');
            content.push_str(POST_MERGE_HOOK);
            std::fs::write(&post_merge_path, content)?;
            make_executable(&post_merge_path)?;
            println!("  added project-memory to existing post-merge hook");
        }
    } else {
        std::fs::write(&post_merge_path, POST_MERGE_HOOK)?;
        make_executable(&post_merge_path)?;
        println!("  installed post-merge hook");
    }

    Ok(())
}

pub fn uninstall_hooks(project_dir: &Path) -> anyhow::Result<()> {
    let hooks_dir = project_dir.join(".git").join("hooks");

    for hook_name in &["pre-commit", "post-merge"] {
        let hook_path = hooks_dir.join(hook_name);
        if hook_path.exists() {
            let content = std::fs::read_to_string(&hook_path)?;
            if content.contains("project-memory") {
                let cleaned: String = content
                    .lines()
                    .collect::<Vec<_>>()
                    .join("\n")
                    .replace(PRE_COMMIT_HOOK, "")
                    .replace(POST_MERGE_HOOK, "")
                    .trim()
                    .to_string();

                if cleaned.is_empty() {
                    std::fs::remove_file(&hook_path)?;
                } else {
                    std::fs::write(&hook_path, cleaned)?;
                }
                println!("  removed project-memory from {}", hook_name);
            }
        }
    }

    Ok(())
}

pub fn hooks_status(project_dir: &Path) -> anyhow::Result<()> {
    let hooks_dir = project_dir.join(".git").join("hooks");
    if !hooks_dir.exists() {
        println!("  Not a git repository");
        return Ok(());
    }

    for hook_name in &["pre-commit", "post-merge"] {
        let hook_path = hooks_dir.join(hook_name);
        if hook_path.exists() {
            let content = std::fs::read_to_string(&hook_path)?;
            if content.contains("project-memory") {
                println!("  {} {}", hook_name, "✓ installed".green());
            } else {
                println!("  {} {}", hook_name, "not managed".dimmed());
            }
        } else {
            println!("  {} {}", hook_name, "not installed".dimmed());
        }
    }

    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
