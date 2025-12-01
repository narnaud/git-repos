use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

mod git_repo;
mod ui;

use git_repo::find_git_repos;
use ui::App;

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

    let mut app = App::new(repos, &scan_path);
    app.run()?;

    Ok(())
}
