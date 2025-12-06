use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod app;
mod config;
mod event;
mod git_repo;
mod ui;

use app::App;
use config::{load_repo_cache, save_repo_cache, CachedRepo, Settings};
use git_repo::{find_git_repos, GitRepo};

/// CLI tool for managing git repositories
#[derive(Parser, Debug)]
#[command(name = "git-repos")]
#[command(about = "Scan and manage git repositories", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to scan for git repositories (defaults to current directory or configured root)
    path: Option<PathBuf>,

    /// Skip automatic fetching of repositories with remotes
    #[arg(long)]
    no_fetch: bool,

    /// Update local branches with fast-forward merge after fetch
    #[arg(short, long)]
    update: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Configure git-repos settings
    Set {
        #[command(subcommand)]
        setting: SetCommand,
    },
}

#[derive(Subcommand, Debug)]
enum SetCommand {
    /// Set the default root directory to scan
    Root {
        /// Path to use as the default root directory
        path: PathBuf,
    },
    /// Enable fast-forward updates by default
    Update {
        /// Enable or disable auto-update (true or false)
        enabled: String,
    },
}

fn handle_set_root(path: PathBuf) -> Result<()> {
    let canonical_path = path.canonicalize()?;
    let mut settings = Settings::load()?;
    settings.set_root_path(canonical_path.clone())?;

    // Display the cleaned path (without \\?\ prefix)
    let display_path = if let Some(root) = &settings.root_path {
        root.display().to_string()
    } else {
        canonical_path.display().to_string()
    };
    println!("Root path set to: {}", display_path);
    Ok(())
}

fn handle_set_update(enabled: String) -> Result<()> {
    let enabled_bool = enabled
        .to_lowercase()
        .parse::<bool>()
        .map_err(|_| color_eyre::eyre::eyre!("Invalid value '{}'. Use 'true' or 'false'", enabled))?;

    let mut settings = Settings::load()?;
    settings.set_update(enabled_bool)?;
    println!("Auto-update set to: {}", enabled_bool);
    Ok(())
}

fn determine_scan_path(args_path: Option<PathBuf>, settings: &Settings) -> Result<PathBuf> {
    if let Some(path) = args_path {
        Ok(path.canonicalize()?)
    } else if let Some(root_path) = &settings.root_path {
        Ok(root_path.clone())
    } else {
        Ok(PathBuf::from(".").canonicalize()?)
    }
}

/// Clean a path by removing Windows \\?\ prefix
fn clean_path(path: &Path) -> PathBuf {
    if let Some(path_str) = path.to_str()
        && let Some(stripped) = path_str.strip_prefix(r"\\?\")
    {
        return PathBuf::from(stripped);
    }
    path.to_path_buf()
}

/// Get relative path from root, handling \\?\ prefix
fn get_relative_path(repo_path: &Path, root_path: &Path) -> Option<PathBuf> {
    let cleaned_path = clean_path(repo_path);
    cleaned_path.strip_prefix(root_path).ok().map(|p| p.to_path_buf())
}

/// Build a set of existing repo relative paths
fn build_existing_paths(repos: &[GitRepo], root_path: &Path) -> HashSet<PathBuf> {
    repos
        .iter()
        .filter_map(|repo| get_relative_path(repo.path(), root_path))
        .collect()
}

/// Add missing repos from cache to the repo list
fn add_missing_repos(repos: &mut Vec<GitRepo>, cached_repos: &[CachedRepo], existing_paths: &HashSet<PathBuf>, root_path: &Path) {
    for cached in cached_repos {
        if !existing_paths.contains(&cached.path) {
            let full_path = root_path.join(&cached.path);
            repos.push(GitRepo::new_missing(full_path, cached.remote.clone()));
        }
    }
}

/// Build updated cache from current repos and missing cached repos
fn build_updated_cache(repos: &[GitRepo], cached_repos: Vec<CachedRepo>, existing_paths: &HashSet<PathBuf>, root_path: &Path) -> Vec<CachedRepo> {
    repos
        .iter()
        .filter_map(|repo| {
            // Skip missing repos - they're already in the cache
            if repo.is_missing() {
                return None;
            }

            // Get relative path from root
            let relative_path = get_relative_path(repo.path(), root_path)?;

            Some(CachedRepo {
                path: relative_path,
                remote: repo.get_remote_url(),
            })
        })
        .chain(
            // Keep cached repos that are missing
            cached_repos.into_iter().filter(|c| !existing_paths.contains(&c.path))
        )
        .collect()
}

/// Merge discovered repos with cached repos and update the cache
fn merge_with_cache(repos: &mut Vec<GitRepo>, root_path: &Path) -> Result<()> {
    let cached_repos = load_repo_cache().unwrap_or_default();
    let mut existing_paths = build_existing_paths(repos, root_path);

    // Add missing repos from cache
    add_missing_repos(repos, &cached_repos, &existing_paths, root_path);

    // Mark all paths as existing for cache update
    for cached in &cached_repos {
        existing_paths.insert(cached.path.clone());
    }

    // Build and save updated cache
    let updated_cache = build_updated_cache(repos, cached_repos, &existing_paths, root_path);
    save_repo_cache(root_path, &updated_cache)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        return match command {
            Command::Set { setting } => match setting {
                SetCommand::Root { path } => handle_set_root(path),
                SetCommand::Update { enabled } => handle_set_update(enabled),
            },
        };
    }

    // Load settings
    let settings = Settings::load()?;

    // Determine scan path and configuration
    let scan_path = determine_scan_path(args.path, &settings)?;
    let mut repos = find_git_repos(&scan_path);
    let update_enabled = args.update || settings.update_by_default;

    // If scanning the root directory, merge with cache
    let is_root = if let Some(root_path) = &settings.root_path
        && &scan_path == root_path
    {
        merge_with_cache(&mut repos, root_path)?;
        true
    } else {
        false
    };

    // Run the TUI
    let root_for_app = if is_root {
        settings.root_path.clone()
    } else {
        None
    };
    let mut app = App::new_with_root(repos, &scan_path, !args.no_fetch, update_enabled, root_for_app);
    app.run().await?;

    // If a repository was selected, change to that directory
    if let Some(repo_path) = app.selected_repo {
        println!("{}", repo_path);
    }

    Ok(())
}
