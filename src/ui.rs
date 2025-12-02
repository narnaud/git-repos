use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Widget},
};

/// Widget implementation for App
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Min(1),    // Main table
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        self.render_table(chunks[0], buf);
        self.render_status_bar(chunks[1], buf);
    }
}

impl App {
    /// Render the repository table
    fn render_table(&mut self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec!["Repository", "Branch", "Remote Status", "Status"])
            .style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = self
            .repos
            .iter()
            .map(|repo| {
                let remote_status = repo.remote_status();
                let remote_color = match remote_status {
                    "local-only" => Color::Gray,
                    "up-to-date" => Color::Green,
                    "no-tracking" => Color::Yellow,
                    _ if remote_status.contains('↑') || remote_status.contains('↓') => Color::Cyan,
                    _ => Color::White,
                };

                let status = repo.status();
                let status_color = match status {
                    "clean" => Color::Green,
                    "unknown" => Color::Gray,
                    _ => Color::Yellow,
                };

                Row::new(vec![
                    Cell::from(repo.display_short()),
                    Cell::from(repo.branch()),
                    Cell::from(remote_status).fg(remote_color),
                    Cell::from(status).fg(status_color),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
        ];

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

        StatefulWidget::render(table, area, buf, &mut self.table_state);
    }

    /// Render the status bar
    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        let repo_count = if self.repos.len() == 1 {
            "Found 1 repository".to_string()
        } else {
            format!("Found {} repositories", self.repos.len())
        };

        let status_text = Line::from(vec![Span::styled(
            format!("{} | Navigate: ↑/↓ or j/k | Quit: q or Ctrl-C", repo_count),
            Style::default().fg(Color::Cyan),
        )]);

        status_text.render(area, buf);
    }
}

