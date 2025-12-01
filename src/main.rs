use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

mod git_repo;
use git_repo::find_git_repos;

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

    println!("Scanning for git repositories in: {}", scan_path.display());
    println!();

    let repos = find_git_repos(&scan_path)?;

    if repos.is_empty() {
        println!("No git repositories found.");
    } else {
        println!("Found {} git repository(ies):", repos.len());
        for repo in repos {
            println!("  {}", repo.display_short());
        }
    }

    Ok(())
}
