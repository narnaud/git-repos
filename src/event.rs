use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Message for async git data updates
pub enum GitDataUpdate {
    RemoteStatus(usize, String),
    Status(usize, String),
    FetchProgress(usize),
    FetchComplete(usize),
    CloneProgress(usize),
    CloneComplete(usize),
}

/// Terminal event types
pub enum TerminalEvent {
    Key(KeyCode, KeyModifiers),
    GitUpdate(GitDataUpdate),
}

/// Event handler with channel for git updates
pub struct EventHandler {
    terminal_events: EventStream,
    git_rx: mpsc::UnboundedReceiver<GitDataUpdate>,
    git_tx: mpsc::UnboundedSender<GitDataUpdate>,
}

impl EventHandler {
    /// Create a new event handler and spawn git data loading tasks
    pub fn new<F>(repo_count: usize, get_path: F, fetch_repos: bool, update_local: bool) -> Self
    where
        F: Fn(usize) -> PathBuf + Send + 'static,
    {
        let (tx, git_rx) = mpsc::unbounded_channel();

        // Spawn background tasks to load git data
        for idx in 0..repo_count {
            let path = get_path(idx);
            let tx_clone = tx.clone();
            let should_fetch = fetch_repos;
            let should_update = update_local;

            tokio::spawn(async move {
                // Load both remote status and working tree status
                let remote_status = tokio::task::spawn_blocking({
                    let path = path.clone();
                    move || crate::git_repo::GitRepo::read_remote_status(&path)
                })
                .await
                .unwrap_or_else(|_| "error".to_string());

                let status = tokio::task::spawn_blocking({
                    let path = path.clone();
                    move || crate::git_repo::GitRepo::read_status(&path)
                })
                .await
                .unwrap_or_else(|_| "error".to_string());

                let _ = tx_clone.send(GitDataUpdate::RemoteStatus(idx, remote_status.clone()));
                let _ = tx_clone.send(GitDataUpdate::Status(idx, status));

                // If fetch is enabled and repo has remote, fetch it
                if should_fetch && remote_status != "local-only" && remote_status != "error" {
                    let _ = tx_clone.send(GitDataUpdate::FetchProgress(idx));

                    let fetch_result = tokio::task::spawn_blocking({
                        let path = path.clone();
                        move || crate::git_repo::GitRepo::fetch(&path, should_update)
                    })
                    .await;

                    if fetch_result.is_ok() {
                        // Re-read remote status after fetch
                        let new_remote_status = tokio::task::spawn_blocking(move || {
                            crate::git_repo::GitRepo::read_remote_status(&path)
                        })
                        .await
                        .unwrap_or_else(|_| "error".to_string());

                        let _ = tx_clone.send(GitDataUpdate::RemoteStatus(idx, new_remote_status));
                    }

                    let _ = tx_clone.send(GitDataUpdate::FetchComplete(idx));
                }
            });
        }
        let tx_clone = tx.clone();

        Self {
            terminal_events: EventStream::new(),
            git_rx,
            git_tx: tx_clone,
        }
    }

    /// Get a clone of the git update sender
    pub fn git_tx(&self) -> mpsc::UnboundedSender<GitDataUpdate> {
        self.git_tx.clone()
    }

    /// Get next event (terminal or git update)
    pub async fn next(&mut self) -> Result<Option<TerminalEvent>> {
        tokio::select! {
            // Check for git updates
            Some(update) = self.git_rx.recv() => {
                Ok(Some(TerminalEvent::GitUpdate(update)))
            }
            // Check for terminal events
            Some(event) = self.terminal_events.next().fuse() => {
                match event? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        Ok(Some(TerminalEvent::Key(key.code, key.modifiers)))
                    }
                    _ => Ok(None)
                }
            }
            else => Ok(None)
        }
    }
}
