use std::path::{Path, PathBuf};

/// Walk up from the given directory to find a .git directory,
/// returning the project root if found.
pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start.to_path_buf());
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Resolve the project directory:
/// 1. If --project is given, use it
/// 2. Otherwise, walk up from CWD to find .git
/// 3. Fall back to CWD
pub fn resolve_project_dir(cli_project: &Option<PathBuf>) -> PathBuf {
    if let Some(dir) = cli_project {
        return dir.clone();
    }

    let cwd = std::env::current_dir().expect("Cannot determine current directory");

    // Check if .memory exists in cwd
    if cwd.join(".memory").exists() {
        return cwd;
    }

    // Walk up to find git root
    if let Some(root) = find_git_root(&cwd) {
        return root;
    }

    cwd
}
