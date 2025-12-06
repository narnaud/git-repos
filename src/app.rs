use crate::event::{EventHandler, GitDataUpdate, TerminalEvent};
use crate::git_repo::GitRepo;
use color_eyre::Result;
use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use std::io;
use std::path::Path;

/// Filter mode for displaying repositories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    NeedsAttention,
    Modified,
    Behind,
}

impl FilterMode {
    /// Get the next filter mode in the cycle
    pub fn next(&self) -> Self {
        match self {
            FilterMode::All => FilterMode::NeedsAttention,
            FilterMode::NeedsAttention => FilterMode::Behind,
            FilterMode::Behind => FilterMode::Modified,
            FilterMode::Modified => FilterMode::All,
        }
    }

    /// Get the previous filter mode in the cycle
    pub fn previous(&self) -> Self {
        match self {
            FilterMode::All => FilterMode::Modified,
            FilterMode::Modified => FilterMode::Behind,
            FilterMode::Behind => FilterMode::NeedsAttention,
            FilterMode::NeedsAttention => FilterMode::All,
        }
    }

    /// Get display name for the filter mode
    pub fn display_name(&self) -> &str {
        match self {
            FilterMode::All => "All",
            FilterMode::NeedsAttention => "Needs Attention",
            FilterMode::Modified => "Modified",
            FilterMode::Behind => "Behind",
        }
    }
}

/// Application state
pub struct App {
    pub repos: Vec<GitRepo>,
    pub scan_path: String,
    pub table_state: TableState,
    should_quit: bool,
    needs_redraw: bool,
    event_handler: EventHandler,
    pub selected_repo: Option<String>,
    pub fetching_repos: Vec<usize>,
    pub cloning_repos: Vec<usize>,
    pub deleting_repos: Vec<usize>,
    pub fetch_animation_frame: usize,
    pub filter_mode: FilterMode,
    search_query: String,
    search_mode: bool,
    root_path: Option<std::path::PathBuf>,
}

impl App {
    /// Create a new App instance
    pub fn new(repos: Vec<GitRepo>, scan_path: &Path, fetch: bool, update: bool) -> Self {
        Self::new_with_root(repos, scan_path, fetch, update, None)
    }

    /// Create a new App instance with optional root path
    pub fn new_with_root(
        mut repos: Vec<GitRepo>,
        scan_path: &Path,
        fetch: bool,
        update: bool,
        root_path: Option<std::path::PathBuf>,
    ) -> Self {
        // Sort repositories: existing first (by name), then missing (by name)
        repos.sort_by(|a, b| {
            match (a.is_missing(), b.is_missing()) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => {
                    let a_name = a.display_short().to_lowercase();
                    let b_name = b.display_short().to_lowercase();
                    a_name.cmp(&b_name)
                }
            }
        });

        let mut table_state = TableState::default();
        if !repos.is_empty() {
            table_state.select(Some(0));
        }

        // Convert to normal path display (strip \\?\ prefix on Windows)
        let path_str = scan_path.display().to_string();
        let display_path = if cfg!(windows) && path_str.starts_with(r"\\?\") {
            path_str
                .strip_prefix(r"\\?\")
                .unwrap_or(&path_str)
                .to_string()
        } else {
            path_str
        };

        // Create event handler and spawn git data loading tasks
        let repos_clone = repos.clone();
        let event_handler = EventHandler::new(
            repos.len(),
            move |idx| repos_clone[idx].path().to_path_buf(),
            fetch,
            update,
        );

        Self {
            repos,
            scan_path: display_path,
            table_state,
            should_quit: false,
            needs_redraw: false,
            event_handler,
            selected_repo: None,
            fetching_repos: Vec::new(),
            cloning_repos: Vec::new(),
            deleting_repos: Vec::new(),
            fetch_animation_frame: 0,
            filter_mode: FilterMode::All,
            search_query: String::new(),
            search_mode: false,
            root_path,
        }
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        let result = self.run_loop(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    /// Get the repositories (for saving cache)
    pub fn repos(&self) -> &[GitRepo] {
        &self.repos
    }

    /// Main event loop
    async fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        // Create a timer for animation updates
        let mut animation_interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

        loop {
            terminal.draw(|f| f.render_widget(&mut *self, f.area()))?;
            self.needs_redraw = false;

            if self.should_quit {
                break;
            }

            // Wait for next event or animation tick
            tokio::select! {
                result = self.event_handler.next() => {
                    if let Some(event) = result? {
                        self.handle_event(event)?;
                    }
                }
                _ = animation_interval.tick() => {
                    if !self.fetching_repos.is_empty() || !self.cloning_repos.is_empty() || !self.deleting_repos.is_empty() {
                        self.fetch_animation_frame = (self.fetch_animation_frame + 1) % 10;
                        self.needs_redraw = true;
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle terminal events
    fn handle_event(&mut self, event: TerminalEvent) -> Result<()> {
        match event {
            TerminalEvent::Key(code, modifiers) => {
                if self.search_mode {
                    match code {
                        KeyCode::Esc => {
                            self.search_mode = false;
                            self.search_query.clear();
                            self.table_state.select(Some(0));
                            self.needs_redraw = true;
                        }
                        KeyCode::Enter => {
                            self.search_mode = false;
                            self.needs_redraw = true;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.table_state.select(Some(0));
                            self.needs_redraw = true;
                        }
                        KeyCode::Char(c) => {
                            self.search_query.push(c);
                            self.table_state.select(Some(0));
                            self.needs_redraw = true;
                        }
                        _ => {}
                    }
                } else {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            self.should_quit = true;
                        }
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.should_quit = true;
                        }
                        KeyCode::Enter => {
                            if let Some(selected) = self.table_state.selected()
                                && let Some(repo) = self.repos.get(selected)
                            {
                                self.selected_repo = Some(repo.path().display().to_string());
                                self.should_quit = true;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.previous();
                        }
                        KeyCode::Char('[') => {
                            self.filter_mode = self.filter_mode.previous();
                            self.table_state.select(Some(0));
                            self.needs_redraw = true;
                        }
                        KeyCode::Char(']') => {
                            self.filter_mode = self.filter_mode.next();
                            self.table_state.select(Some(0));
                            self.needs_redraw = true;
                        }
                        KeyCode::Char('/') => {
                            self.search_mode = true;
                            self.search_query.clear();
                            self.needs_redraw = true;
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            self.handle_drop_repo();
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            self.handle_clone_repo();
                        }
                        _ => {}
                    }
                }
            }
            TerminalEvent::GitUpdate(update) => match update {
                GitDataUpdate::RemoteStatus(idx, status) => {
                    if let Some(repo) = self.repos.get_mut(idx) {
                        repo.set_remote_status(status);
                        self.needs_redraw = true;
                    }
                }
                GitDataUpdate::Status(idx, status) => {
                    if let Some(repo) = self.repos.get_mut(idx) {
                        repo.set_status(status);
                        self.needs_redraw = true;
                    }
                }
                GitDataUpdate::FetchProgress(idx) => {
                    if !self.fetching_repos.contains(&idx) {
                        self.fetching_repos.push(idx);
                        self.needs_redraw = true;
                    }
                }
                GitDataUpdate::FetchComplete(idx) => {
                    self.fetching_repos.retain(|&i| i != idx);
                    self.fetch_animation_frame = (self.fetch_animation_frame + 1) % 10;
                    self.needs_redraw = true;
                }
                GitDataUpdate::CloneProgress(idx) => {
                    if !self.cloning_repos.contains(&idx) {
                        self.cloning_repos.push(idx);
                        self.needs_redraw = true;
                    }
                }
                GitDataUpdate::CloneComplete(idx) => {
                    self.cloning_repos.retain(|&i| i != idx);

                    // Refresh the repository by recreating it as a normal repo
                    if let Some(repo) = self.repos.get(idx) {
                        let path = repo.path().to_path_buf();

                        // Only refresh if the clone was successful (directory exists)
                        if path.exists() {
                            let new_repo = GitRepo::new(path.clone());
                            self.repos[idx] = new_repo;

                            // Sort repositories: existing first (by name), then missing (by name)
                            self.repos.sort_by(|a, b| {
                                match (a.is_missing(), b.is_missing()) {
                                    (false, true) => std::cmp::Ordering::Less,
                                    (true, false) => std::cmp::Ordering::Greater,
                                    _ => {
                                        let a_name = a.display_short().to_lowercase();
                                        let b_name = b.display_short().to_lowercase();
                                        a_name.cmp(&b_name)
                                    }
                                }
                            });

                            // Find the new index of the cloned repo after sorting
                            let new_idx = self.repos.iter()
                                .enumerate()
                                .find(|(_, r)| r.path() == path)
                                .map(|(idx, _)| idx);

                            if let Some(new_idx) = new_idx {
                                self.table_state.select(Some(new_idx));

                                // Spawn async task to load git data with the NEW index
                                let tx = self.event_handler.git_tx();
                                let path_clone = path.clone();
                                tokio::spawn(async move {
                                    let remote_status = tokio::task::spawn_blocking({
                                        let path = path_clone.clone();
                                        move || GitRepo::read_remote_status(&path)
                                    })
                                    .await
                                    .unwrap_or_else(|_| "error".to_string());

                                    let status = tokio::task::spawn_blocking({
                                        let path = path_clone.clone();
                                        move || GitRepo::read_status(&path)
                                    })
                                    .await
                                    .unwrap_or_else(|_| "error".to_string());

                                    let _ = tx.send(GitDataUpdate::RemoteStatus(new_idx, remote_status));
                                    let _ = tx.send(GitDataUpdate::Status(new_idx, status));
                                });
                            }
                        }
                    }

                    self.needs_redraw = true;
                }
                GitDataUpdate::DeleteProgress(idx) => {
                    if !self.deleting_repos.contains(&idx) {
                        self.deleting_repos.push(idx);
                        self.needs_redraw = true;
                    }
                }
                GitDataUpdate::DeleteComplete(idx, remote_url) => {
                    self.deleting_repos.retain(|&i| i != idx);

                    // Mark the repository as missing, preserving its remote URL
                    if let Some(repo) = self.repos.get_mut(idx) {
                        let repo_path = repo.path().to_path_buf();
                        repo.set_missing(remote_url);

                        // Sort repositories: existing first (by name), then missing (by name)
                        self.repos.sort_by(|a, b| {
                            match (a.is_missing(), b.is_missing()) {
                                (false, true) => std::cmp::Ordering::Less,
                                (true, false) => std::cmp::Ordering::Greater,
                                _ => {
                                    let a_name = a.display_short().to_lowercase();
                                    let b_name = b.display_short().to_lowercase();
                                    a_name.cmp(&b_name)
                                }
                            }
                        });

                        // Find the new index of the selected repo after sorting
                        let new_idx = self.repos.iter()
                            .enumerate()
                            .find(|(_, r)| r.path() == repo_path)
                            .map(|(idx, _)| idx);

                        if let Some(idx) = new_idx {
                            self.table_state.select(Some(idx));
                        }
                    }

                    self.needs_redraw = true;
                }
            },
        }
        Ok(())
    }

    /// Get filtered list of repository indices based on current filter mode
    pub fn filtered_repos(&self) -> Vec<usize> {
        self.repos
            .iter()
            .enumerate()
            .filter(|(_, repo)| {
                // Apply search filter
                if !self.search_query.is_empty() {
                    let query_lower = self.search_query.to_lowercase();
                    let name_match = repo.name()
                        .map(|n| n.to_lowercase().contains(&query_lower))
                        .unwrap_or(false);
                    let parent_match = repo.parent_name()
                        .map(|p| p.to_lowercase().contains(&query_lower))
                        .unwrap_or(false);

                    if !name_match && !parent_match {
                        return false;
                    }
                }

                // Apply filter mode
                match self.filter_mode {
                    FilterMode::All => true,
                    FilterMode::NeedsAttention => {
                        // Show repos that are behind, modified, or have no tracking
                        let remote = repo.remote_status();
                        let status = repo.status();
                        (remote.contains('↓') || remote == "no-tracking")
                            || (status != "clean" && status != "loading...")
                    }
                    FilterMode::Modified => {
                        let status = repo.status();
                        status != "clean" && status != "loading..."
                    }
                    FilterMode::Behind => {
                        let remote = repo.remote_status();
                        remote.contains('↓')
                    }
                }
            })
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Check if search mode is active
    pub fn is_search_mode(&self) -> bool {
        self.search_mode
    }

    /// Get current search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Move to next item
    fn next(&mut self) {
        let filtered = self.filtered_repos();
        if filtered.is_empty() {
            return;
        }

        let current_selected = self.table_state.selected().unwrap_or(0);
        let current_pos = filtered.iter().position(|&idx| idx == current_selected);

        let next_pos = match current_pos {
            Some(pos) if pos >= filtered.len() - 1 => 0,
            Some(pos) => pos + 1,
            None => 0,
        };

        self.table_state.select(Some(filtered[next_pos]));
    }

    /// Move to previous item
    fn previous(&mut self) {
        let filtered = self.filtered_repos();
        if filtered.is_empty() {
            return;
        }

        let current_selected = self.table_state.selected().unwrap_or(0);
        let current_pos = filtered.iter().position(|&idx| idx == current_selected);

        let prev_pos = match current_pos {
            Some(0) | None => filtered.len() - 1,
            Some(pos) => pos - 1,
        };

        self.table_state.select(Some(filtered[prev_pos]));
    }

    /// Handle dropping a repository
    fn handle_drop_repo(&mut self) {
        let Some(selected) = self.table_state.selected() else {
            return;
        };

        let Some(repo) = self.repos.get(selected) else {
            return;
        };

        let is_missing = repo.is_missing();
        let repo_path = repo.path().to_path_buf();

        if is_missing {
            // Missing repo: remove from cache
            if let Some(root_path) = &self.root_path {
                let repo_path_str = repo_path.to_str().unwrap_or("");
                let cleaned_path = if let Some(stripped) = repo_path_str.strip_prefix(r"\\?\") {
                    std::path::PathBuf::from(stripped)
                } else {
                    repo_path.clone()
                };

                if let Ok(relative_path) = cleaned_path.strip_prefix(root_path)
                    && crate::config::remove_from_cache(relative_path).is_ok()
                {
                    // Remove from repos list
                    self.repos.remove(selected);

                    // Adjust selection
                    if !self.repos.is_empty() {
                        let new_selected = if selected >= self.repos.len() {
                            self.repos.len() - 1
                        } else {
                            selected
                        };
                        self.table_state.select(Some(new_selected));
                    } else {
                        self.table_state.select(None);
                    }

                    self.needs_redraw = true;
                }
            }
        } else {
            // Normal repo: delete directory asynchronously and mark as missing
            // Get remote URL before deletion
            let remote_url = repo.get_remote_url();

            self.deleting_repos.push(selected);
            self.needs_redraw = true;

            let tx = self.event_handler.git_tx();
            let idx = selected;

            tokio::spawn(async move {
                // Send delete progress
                let _ = tx.send(GitDataUpdate::DeleteProgress(idx));

                // Perform deletion
                let delete_result = tokio::task::spawn_blocking(move || {
                    std::fs::remove_dir_all(&repo_path)
                }).await;

                // Send delete complete with remote URL
                let _ = tx.send(GitDataUpdate::DeleteComplete(idx, remote_url));

                drop(delete_result); // Ignore result
            });
        }
    }

    /// Handle cloning a missing repository
    fn handle_clone_repo(&mut self) {
        let Some(selected) = self.table_state.selected() else {
            return;
        };

        let Some(repo) = self.repos.get(selected) else {
            return;
        };

        // Only clone missing repositories
        if !repo.is_missing() {
            return;
        }

        // Mark as cloning
        self.cloning_repos.push(selected);
        self.needs_redraw = true;

        // Clone the repository in background
        let repo_clone = repo.clone();
        let tx = self.event_handler.git_tx();
        let idx = selected;

        tokio::spawn(async move {
            // Send clone progress
            let _ = tx.send(GitDataUpdate::CloneProgress(idx));

            // Perform clone
            let clone_result = tokio::task::spawn_blocking(move || {
                repo_clone.clone_repository()
            }).await;

            // Send clone complete
            let _ = tx.send(GitDataUpdate::CloneComplete(idx));

            // If successful, the UI will be updated through CloneComplete handler
            if clone_result.is_ok() {
                // Repository will be refreshed when user selects it again or on next scan
            }
        });
    }
}
