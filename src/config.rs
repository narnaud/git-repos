use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// The default root directory to scan for git repositories
    pub root_path: Option<PathBuf>,

    /// Whether to enable fast-forward merge updates by default
    #[serde(default)]
    pub update_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRepo {
    /// Relative path from the root directory
    pub path: PathBuf,
    /// Remote URL (origin)
    pub remote: Option<String>,
}

impl Settings {
    /// Load settings from the config file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let settings: Settings = toml::from_str(&contents)?;

        Ok(settings)
    }

    /// Save settings to the config file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;

        Ok(())
    }

    /// Get the path to the config file
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;

        Ok(config_dir.join("git-repos").join("config.toml"))
    }

    /// Set the root path and save
    pub fn set_root_path(&mut self, path: PathBuf) -> Result<()> {
        // Remove the \\?\ prefix that Windows canonicalize adds
        let path_str = path.to_string_lossy();
        let cleaned_path = if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
            PathBuf::from(stripped)
        } else {
            path
        };

        self.root_path = Some(cleaned_path);
        self.save()
    }

    /// Set whether to update by default and save
    pub fn set_update(&mut self, enabled: bool) -> Result<()> {
        self.update_by_default = enabled;
        self.save()
    }
}

/// Save repository cache to YAML file
pub fn save_repo_cache(_root: &Path, repos: &[CachedRepo]) -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;

    let cache_path = config_dir.join("git-repos").join("repos.yaml");

    // Create parent directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml = serde_yaml::to_string(repos)?;
    fs::write(&cache_path, yaml)?;

    Ok(())
}

/// Load repository cache from YAML file
pub fn load_repo_cache() -> Result<Vec<CachedRepo>> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;

    let cache_path = config_dir.join("git-repos").join("repos.yaml");

    if !cache_path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(&cache_path)?;
    let repos: Vec<CachedRepo> = serde_yaml::from_str(&contents)?;

    Ok(repos)
}

/// Remove a repository from the cache by its relative path
pub fn remove_from_cache(relative_path: &Path) -> Result<()> {
    let mut cached_repos = load_repo_cache()?;
    
    // Remove the repo with matching path
    cached_repos.retain(|repo| repo.path != relative_path);
    
    // Save updated cache
    let config_dir = dirs::config_dir()
        .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;
    
    let cache_path = config_dir.join("git-repos").join("repos.yaml");
    
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let yaml = serde_yaml::to_string(&cached_repos)?;
    fs::write(&cache_path, yaml)?;
    
    Ok(())
}
