use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::path::PathBuf;

mod app;
mod cache;
mod config;
mod event;
mod git_repo;
mod ui;

use app::App;
use cache::{load_repos_with_cache, save_repos_to_cache};
use config::Settings;

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

    /// Write selected repository path to this file on exit (for shell integration)
    #[arg(long, value_name = "PATH")]
    cwd_file: Option<PathBuf>,
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

    // Determine scan path and load repositories
    let scan_path = determine_scan_path(args.path, &settings)?;
    let (repos, is_root) = load_repos_with_cache(&scan_path, settings.root_path.as_deref());
    let update_enabled = args.update || settings.update_by_default;

    // Run the TUI
    let root_for_app = is_root.then(|| settings.root_path.clone()).flatten();
    let mut app = App::new_with_root(repos, &scan_path, !args.no_fetch, update_enabled, root_for_app);
    app.run().await?;

    // Save cache if we were scanning root directory
    if is_root
        && let Some(root_path) = &settings.root_path
    {
        save_repos_to_cache(app.repos(), root_path)?;
    }


    // If a repository was selected and --cwd-file is set, write to the file
    if let (Some(repo_path), Some(cwd_file)) = (app.selected_repo, args.cwd_file) {
        // Remove Windows UNC prefix if present
        let cleaned = if repo_path.starts_with(r"\\?\") {
            &repo_path[4..]
        } else {
            &repo_path
        };
        std::fs::write(cwd_file, cleaned)?;
    }

    Ok(())
}
