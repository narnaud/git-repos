use color_eyre::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Represents a Git repository with its path
#[derive(Debug, Clone)]
pub struct GitRepo {
    path: PathBuf,
    branch: String,
    remote_status: String,
}

impl GitRepo {
    /// Create a new GitRepo from a path
    pub fn new(path: PathBuf) -> Self {
        let branch = Self::read_branch(&path);
        let remote_status = Self::read_remote_status(&path);

        Self {
            path,
            branch,
            remote_status,
        }
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

    /// Get the current branch name
    pub fn branch(&self) -> &str {
        &self.branch
    }

    /// Get the remote tracking status (ahead/behind)
    pub fn remote_status(&self) -> &str {
        &self.remote_status
    }

    /// Read the current branch name from .git/HEAD
    fn read_branch(path: &Path) -> String {
        // Try to read .git/HEAD to get the current branch
        let head_path = path.join(".git").join("HEAD");

        if let Ok(content) = fs::read_to_string(&head_path) {
            let content = content.trim();

            // HEAD typically contains "ref: refs/heads/branch-name"
            if let Some(branch_ref) = content.strip_prefix("ref: refs/heads/") {
                return branch_ref.to_string();
            }

            // If it's a detached HEAD, show first 7 chars of commit hash
            if content.len() >= 7 {
                return format!("detached@{}", &content[..7]);
            }
        }

        // Fallback if we can't determine the branch
        "unknown".to_string()
    }

    /// Read the remote tracking status (ahead/behind)
    fn read_remote_status(path: &Path) -> String {
        // Check if there are any remotes configured
        let has_remote = Command::new("git")
            .args(["remote"])
            .current_dir(path)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(!output.stdout.is_empty())
                } else {
                    None
                }
            })
            .unwrap_or(false);

        if !has_remote {
            return "local-only".to_string();
        }

        // Get ahead/behind count
        let output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(path)
            .output();

        if let Ok(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.split_whitespace().collect();

            if parts.len() == 2
                && let (Ok(ahead), Ok(behind)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>())
            {
                if ahead == 0 && behind == 0 {
                    return "up-to-date".to_string();
                }
                return format!("↑{} ↓{}", ahead, behind);
            }
        }

        // No tracking branch or error
        "no-tracking".to_string()
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
