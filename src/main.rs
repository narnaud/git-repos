use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

mod app;
mod git_repo;
mod ui;

use app::App;
use git_repo::find_git_repos;

/// Message for async git data updates
pub enum GitDataUpdate {
    RemoteStatus(usize, String),
    Status(usize, String),
}

/// CLI tool for managing git repositories
#[derive(Parser, Debug)]
#[command(name = "git-repos")]
#[command(about = "Scan and manage git repositories", long_about = None)]
struct Args {
    /// Path to scan for git repositories (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let scan_path = args.path.canonicalize()?;

    let repos = find_git_repos(&scan_path)?;

    // Create channel for async updates
    let (tx, rx) = mpsc::channel();

    // Spawn background tasks to load git data
    for (idx, repo) in repos.iter().enumerate() {
        let path = repo.path().to_path_buf();
        let tx_remote = tx.clone();
        let tx_status = tx.clone();

        // Spawn task for remote status
        let path_clone = path.clone();
        thread::spawn(move || {
            let remote_status = git_repo::GitRepo::read_remote_status(&path_clone);
            let _ = tx_remote.send(GitDataUpdate::RemoteStatus(idx, remote_status));
        });

        // Spawn task for working tree status
        thread::spawn(move || {
            let status = git_repo::GitRepo::read_status(&path);
            let _ = tx_status.send(GitDataUpdate::Status(idx, status));
        });
    }
    drop(tx); // Close sender

    let mut app = App::new(repos, &scan_path, rx);
    app.run()?;

    Ok(())
}
