use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

mod app;
mod event;
mod git_repo;
mod ui;

use app::App;
use git_repo::find_git_repos;

/// CLI tool for managing git repositories
#[derive(Parser, Debug)]
#[command(name = "git-repos")]
#[command(about = "Scan and manage git repositories", long_about = None)]
struct Args {
    /// Path to scan for git repositories (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Skip automatic fetching of repositories with remotes
    #[arg(long)]
    no_fetch: bool,

    /// Update local branches with fast-forward merge after fetch
    #[arg(short, long)]
    update: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let scan_path = args.path.canonicalize()?;
    let repos = find_git_repos(&scan_path)?;

    let mut app = App::new(repos, &scan_path, !args.no_fetch, args.update);
    app.run().await?;

    // If a repository was selected, change to that directory
    if let Some(repo_path) = app.selected_repo {
        println!("{}", repo_path);
    }

    Ok(())
}
