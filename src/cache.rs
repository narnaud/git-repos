use crate::config::{load_repo_cache, save_repo_cache, CachedRepo};
use crate::git_repo::GitRepo;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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

/// Merge discovered repos with cached repos by adding missing repos
fn merge_with_cache(repos: &mut Vec<GitRepo>, root_path: &Path, cached_repos: &[CachedRepo]) {
    let existing_paths = build_existing_paths(repos, root_path);
    add_missing_repos(repos, cached_repos, &existing_paths, root_path);
}

/// Build cache from all repos, sorted alphabetically
fn build_cache_from_repos(repos: &[GitRepo], root_path: &Path) -> Vec<CachedRepo> {
    let mut cache: Vec<CachedRepo> = repos
        .iter()
        .filter_map(|repo| {
            let relative_path = get_relative_path(repo.path(), root_path)?;
            Some(CachedRepo {
                path: relative_path,
                remote: repo.get_remote_url(),
            })
        })
        .collect();

    // Sort alphabetically by path
    cache.sort_by(|a, b| a.path.cmp(&b.path));
    cache
}

/// Load repositories, merging with cache if scanning root directory
pub fn load_repos_with_cache(scan_path: &Path, root_path: Option<&Path>) -> (Vec<GitRepo>, bool) {
    let mut repos = crate::git_repo::find_git_repos(scan_path);
    let is_root = if let Some(root) = root_path
        && scan_path == root
    {
        let cached = load_repo_cache().unwrap_or_default();
        merge_with_cache(&mut repos, root, &cached);
        true
    } else {
        false
    };
    (repos, is_root)
}

/// Save cache from repositories to disk
pub fn save_repos_to_cache(repos: &[GitRepo], root_path: &Path) -> color_eyre::Result<()> {
    let cache = build_cache_from_repos(repos, root_path);
    save_repo_cache(root_path, &cache)
}
