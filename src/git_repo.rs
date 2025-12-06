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
    remote_status: Option<String>,
    status: Option<String>,
    missing: bool,
    remote_url: Option<String>,
}

impl GitRepo {
    /// Create a new GitRepo from a path (branch only, async fields are None)
    pub fn new(path: PathBuf) -> Self {
        let branch = Self::read_branch(&path);

        Self {
            path,
            branch,
            remote_status: None,
            status: None,
            missing: false,
            remote_url: None,
        }
    }

    /// Create a new missing GitRepo (exists in cache but not on disk)
    pub fn new_missing(path: PathBuf, remote_url: Option<String>) -> Self {
        Self {
            path,
            branch: String::new(),
            remote_status: None,
            status: None,
            missing: true,
            remote_url,
        }
    }

    /// Check if this repo is missing from disk
    pub fn is_missing(&self) -> bool {
        self.missing
    }

    /// Mark this repository as missing (deleted)
    pub fn set_missing(&mut self) {
        self.missing = true;
    }

    /// Update the remote status
    pub fn set_remote_status(&mut self, remote_status: String) {
        self.remote_status = Some(remote_status);
    }

    /// Update the working tree status
    pub fn set_status(&mut self, status: String) {
        self.status = Some(status);
    }

    /// Check if async data is loaded
    pub fn is_loaded(&self) -> bool {
        self.remote_status.is_some() && self.status.is_some()
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
        self.remote_status.as_deref().unwrap_or("loading...")
    }

    /// Get the working tree status
    pub fn status(&self) -> &str {
        self.status.as_deref().unwrap_or("loading...")
    }

    /// Get the remote URL (origin)
    pub fn get_remote_url(&self) -> Option<String> {
        // If this is a missing repo, return cached remote URL
        if self.missing {
            return self.remote_url.clone();
        }

        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&self.path)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Clone this repository to its expected path
    pub fn clone_repository(&self) -> Result<()> {
        if !self.missing {
            return Err(color_eyre::eyre::eyre!("Repository already exists"));
        }

        let remote_url = self.remote_url.as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("No remote URL for repository"))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if it's a GitHub repository
        let is_github = remote_url.contains("github.com");

        let output = if is_github {
            // Use gh repo clone for GitHub repos
            Command::new("gh")
                .args(["repo", "clone", remote_url, &self.path.to_string_lossy()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output()
        } else {
            // Use git clone for non-GitHub repos
            Command::new("git")
                .args(["clone", remote_url, &self.path.to_string_lossy()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output()
        }?;

        if !output.status.success() {
            return Err(color_eyre::eyre::eyre!("Failed to clone repository"));
        }

        Ok(())
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
    pub fn read_remote_status(path: &Path) -> String {
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

    /// Read the working tree status (clean/dirty)
    pub fn read_status(path: &Path) -> String {
        // Run git status --porcelain to check for changes
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output();

        if let Ok(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                return "clean".to_string();
            }

            // Count staged and unstaged changes
            let mut staged = 0;
            let mut unstaged = 0;

            for line in stdout.lines() {
                if line.len() >= 2 {
                    let index_status = &line[0..1];
                    let work_tree_status = &line[1..2];

                    if index_status != " " && index_status != "?" {
                        staged += 1;
                    }
                    if work_tree_status != " " {
                        unstaged += 1;
                    }
                }
            }

            match (staged, unstaged) {
                (0, u) if u > 0 => format!("{}M", u),
                (s, 0) if s > 0 => format!("{}S", s),
                (s, u) if s > 0 && u > 0 => format!("{}S {}M", s, u),
                _ => "dirty".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }

    /// Fetch from all remotes and optionally fast-forward if possible
    pub fn fetch(path: &Path, update: bool) -> Result<()> {
        // First, fetch from all remotes
        let output = Command::new("git")
            .args(["fetch", "--all", "--prune"])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!("git fetch failed: {}", stderr));
        }

        // Try to fast-forward merge the current branch with its upstream if requested
        if update {
            // This only succeeds if it's a clean fast-forward (no divergence)
            let merge_output = Command::new("git")
                .args(["merge", "--ff-only", "@{upstream}"])
                .current_dir(path)
                .output()?;

            // If merge succeeded, also update submodules
            if merge_output.status.success() {
                let _ = Command::new("git")
                    .args(["submodule", "update", "--init", "--recursive"])
                    .current_dir(path)
                    .output();
            }
        }

        Ok(())
    }
}

/// Check if a directory is a git repository
fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Scan directory recursively and find all git repositories
pub fn find_git_repos(root: &Path) -> Vec<GitRepo> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let filename = e.file_name();

            // Skip .git directories and other hidden directories
            if filename.to_str().is_some_and(|s| s.starts_with('.')) {
                return false;
            }

            // Skip if parent is a git repo (don't descend into nested repos)
            if let Some(parent) = e.path().parent()
                && parent != root && is_git_repo(parent)
            {
                return false;
            }

            true
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir() && is_git_repo(entry.path()))
        .map(|entry| {
            let path = entry.path().canonicalize().unwrap_or_else(|_| entry.path().to_path_buf());
            GitRepo::new(path)
        })
        .collect()
}
