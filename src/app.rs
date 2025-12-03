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
    pub fetch_animation_frame: usize,
}

impl App {
    /// Create a new App instance
    pub fn new(repos: Vec<GitRepo>, scan_path: &Path, fetch: bool, update: bool) -> Self {
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
            fetch_animation_frame: 0,
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
                    if !self.fetching_repos.is_empty() {
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
                    _ => {}
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
            },
        }
        Ok(())
    }

    /// Move to next item
    fn next(&mut self) {
        if self.repos.is_empty() {
            return;
        }

        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.repos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Move to previous item
    fn previous(&mut self) {
        if self.repos.is_empty() {
            return;
        }

        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.repos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}
