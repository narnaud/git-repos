use color_eyre::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a Git repository with its path
#[derive(Debug, Clone)]
pub struct GitRepo {
    path: PathBuf,
}

impl GitRepo {
    /// Create a new GitRepo from a path
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Get the repository path
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the repository name
    pub fn name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }

    /// Get the parent directory name
    pub fn parent_name(&self) -> Option<&str> {
        self.path.parent()?.file_name()?.to_str()
    }

    /// Get a formatted display string in the form "parent/repo"
    pub fn display_short(&self) -> String {
        match (self.parent_name(), self.name()) {
            (Some(parent), Some(name)) => format!("{}/{}", parent, name),
            (None, Some(name)) => name.to_string(),
            _ => self.path.display().to_string(),
        }
    }
}

/// Check if a directory is a git repository
fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Scan directory recursively and find all git repositories
pub fn find_git_repos(root: &Path) -> Result<Vec<GitRepo>> {
    let mut repos = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        // Always skip .git directories themselves
        if e.file_name() == ".git" {
            return false;
        }
        // Skip hidden directories (starting with .)
        if e.file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
            return false;
        }
        // Skip if parent directory is a git repo
        if let Some(parent) = e.path().parent()
            && parent != root
            && is_git_repo(parent)
        {
            return false;
        }
        true
    }) {
        // Skip entries with errors (e.g., permission denied)
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // Check if this directory is a git repository
        if entry.file_type().is_dir() && is_git_repo(path) {
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            repos.push(GitRepo::new(canonical_path));
        }
    }

    Ok(repos)
}
