use crate::git_repo::GitRepo;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Row, Table, TableState},
};
use std::io;
use std::path::Path;

/// Application state
pub struct App {
    repos: Vec<GitRepo>,
    scan_path: String,
    table_state: TableState,
    should_quit: bool,
}

impl App {
    /// Create a new App instance
    pub fn new(repos: Vec<GitRepo>, scan_path: &Path) -> Self {
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
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
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

    /// Render the UI
    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Min(1),    // Main table
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

        self.render_table(f, chunks[0]);
        self.render_status_bar(f, chunks[1]);
    }

    /// Render the repository table
    fn render_table(&mut self, f: &mut Frame, area: Rect) {
        let header = Row::new(vec!["Repository", "Branch"])
            .style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = self
            .repos
            .iter()
            .map(|repo| Row::new(vec![repo.display_short(), repo.branch()]))
            .collect();

        let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title("") // Add a small padding on the left
                    .title(
                        format!("Git Repositories - {}", self.scan_path)
                            .bold()
                            .light_blue(),
                    )
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::White))
                    .style(Style::default()),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    /// Render the status bar
    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let repo_count = if self.repos.len() == 1 {
            "Found 1 repository".to_string()
        } else {
            format!("Found {} repositories", self.repos.len())
        };

        let status_text = Line::from(vec![Span::styled(
            format!("{} | Navigate: ↑/↓ or j/k | Quit: q or Ctrl-C", repo_count),
            Style::default().fg(Color::Cyan),
        )]);

        f.render_widget(status_text, area);
    }
}
