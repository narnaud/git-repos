use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::path::PathBuf;

mod app;
mod config;
mod event;
mod git_repo;
mod ui;

use app::App;
use config::Settings;
use git_repo::find_git_repos;

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
    let repos = find_git_repos(&scan_path);
    let update_enabled = args.update || settings.update_by_default;

    // Run the TUI
    let mut app = App::new(repos, &scan_path, !args.no_fetch, update_enabled);
    app.run().await?;

    // If a repository was selected, change to that directory
    if let Some(repo_path) = app.selected_repo {
        println!("{}", repo_path);
    }

    Ok(())
}
