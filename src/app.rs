use crate::git_repo::GitRepo;
use color_eyre::Result;
use crossterm::{
    event::{self, poll, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use std::io;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Duration;

/// Application state
pub struct App {
    pub repos: Vec<GitRepo>,
    pub scan_path: String,
    pub table_state: TableState,
    should_quit: bool,
    needs_redraw: bool,
    rx: Receiver<crate::GitDataUpdate>,
}

impl App {
    /// Create a new App instance
    pub fn new(repos: Vec<GitRepo>, scan_path: &Path, rx: Receiver<crate::GitDataUpdate>) -> Self {
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

        Self {
            repos,
            scan_path: display_path,
            table_state,
            should_quit: false,
            needs_redraw: false,
            rx,
        }
    }

    /// Run the TUI application
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        let result = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    /// Main event loop
    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        while !self.should_quit {
            // Process any pending git data updates
            while let Ok(update) = self.rx.try_recv() {
                match update {
                    crate::GitDataUpdate::RemoteStatus(idx, status) => {
                        if let Some(repo) = self.repos.get_mut(idx) {
                            repo.set_remote_status(status);
                            self.needs_redraw = true;
                        }
                    }
                    crate::GitDataUpdate::Status(idx, status) => {
                        if let Some(repo) = self.repos.get_mut(idx) {
                            repo.set_status(status);
                            self.needs_redraw = true;
                        }
                    }
                }
            }

            terminal.draw(|f| f.render_widget(&mut *self, f.area()))?;
            self.needs_redraw = false;

            // Check for events with a timeout to allow periodic redraws
            if poll(Duration::from_millis(100))? {
                self.handle_events()?;
            }
        }
        Ok(())
    }

    /// Handle terminal events
    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    self.should_quit = true;
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
