use clap::Parser;
use color_eyre::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// CLI tool for managing git repositories
#[derive(Parser, Debug)]
#[command(name = "git-repos")]
#[command(about = "Scan and manage git repositories", long_about = None)]
struct Args {
    /// Path to scan for git repositories (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

/// Check if a directory is a git repository
fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Scan directory recursively and find all git repositories
fn find_git_repos(root: &Path) -> Result<Vec<PathBuf>> {
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
        if let Some(parent) = e.path().parent() {
            if parent != root && is_git_repo(parent) {
                return false;
            }
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
            repos.push(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
        }
    }

    Ok(repos)
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let scan_path = args.path.canonicalize()?;

    println!("Scanning for git repositories in: {}", scan_path.display());
    println!();

    let repos = find_git_repos(&scan_path)?;

    if repos.is_empty() {
        println!("No git repositories found.");
    } else {
        println!("Found {} git repository(ies):", repos.len());
        for repo in repos {
            // Extract parent and repo folder names
            if let (Some(repo_name), Some(parent_path)) = (repo.file_name(), repo.parent()) {
                if let Some(parent_name) = parent_path.file_name() {
                    println!("  {}/{}", parent_name.to_string_lossy(), repo_name.to_string_lossy());
                } else {
                    // Fallback if no parent name (e.g., root directory)
                    println!("  {}", repo_name.to_string_lossy());
                }
            } else {
                // Fallback to full path if extraction fails
                println!("  {}", repo.display());
            }
        }
    }

    Ok(())
}

