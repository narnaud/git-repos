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

        let filtered_indices = self.filtered_repos();
        let rows: Vec<Row> = self
            .repos
            .iter()
            .enumerate()
            .filter(|(idx, _)| filtered_indices.contains(idx))
            .map(|(_, repo)| {
                // If repo is missing, render everything in gray
                if repo.is_missing() {
                    return Row::new(vec![
                        Cell::from(repo.display_short()).fg(Color::DarkGray),
                        Cell::from("").fg(Color::DarkGray),
                        Cell::from("missing").fg(Color::DarkGray),
                        Cell::from("").fg(Color::DarkGray),
                    ]);
                }

                let remote_status = repo.remote_status();
                let (remote_text, remote_color) = match remote_status {
                    "loading..." => (format!("⟳ {}", remote_status), Color::DarkGray),
                    "local-only" => (remote_status.to_string(), Color::Red),
                    "up-to-date" => (remote_status.to_string(), Color::Green),
                    "no-tracking" => (remote_status.to_string(), Color::Yellow),
                    _ if remote_status.contains('↑') || remote_status.contains('↓') => {
                        (remote_status.to_string(), Color::Cyan)
                    }
                    _ => (remote_status.to_string(), Color::White),
                };

                let status = repo.status();
                let (status_text, status_color) = match status {
                    "loading..." => (format!("⟳ {}", status), Color::DarkGray),
                    "clean" => (status.to_string(), Color::Green),
                    "unknown" => (status.to_string(), Color::DarkGray),
                    _ => (status.to_string(), Color::Yellow),
                };

                Row::new(vec![
                    Cell::from(repo.display_short()),
                    Cell::from(repo.branch()),
                    Cell::from(remote_text).fg(remote_color),
                    Cell::from(status_text).fg(status_color),
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
                    .title(
                        format!("Git Repositories - {}", self.scan_path)
                            .bold()
                            .light_blue(),
                    )
                    .title_bottom(
                        Line::from(vec![
                            if self.filter_mode == crate::app::FilterMode::All {
                                Span::styled("All", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
                            } else {
                                Span::styled("All", Style::default().fg(Color::White))
                            },
                            Span::raw(" - "),
                            if self.filter_mode == crate::app::FilterMode::NeedsAttention {
                                Span::styled("Needs Attention", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
                            } else {
                                Span::styled("Needs Attention", Style::default().fg(Color::White))
                            },
                            Span::raw(" - "),
                            if self.filter_mode == crate::app::FilterMode::Behind {
                                Span::styled("Behind", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
                            } else {
                                Span::styled("Behind", Style::default().fg(Color::White))
                            },
                            Span::raw(" - "),
                            if self.filter_mode == crate::app::FilterMode::Modified {
                                Span::styled("Modified", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
                            } else {
                                Span::styled("Modified", Style::default().fg(Color::White))
                            },
                        ]).right_aligned()
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
        // In search mode, show only the search prompt
        if self.is_search_mode() {
            let search_text = Line::from(vec![
                Span::styled("Search: ", Style::default().fg(Color::Yellow)),
                Span::styled(self.search_query(), Style::default().fg(Color::White)),
            ]);
            search_text.render(area, buf);
            return;
        }

        let filtered_count = self.filtered_repos().len();
        let total_count = self.repos.len();

        let repo_count = if filtered_count == total_count {
            if total_count == 1 {
                "Found 1 repository".to_string()
            } else {
                format!("Found {} repositories", total_count)
            }
        } else {
            format!("Showing {} of {} repositories", filtered_count, total_count)
        };

        let status_text = if !self.search_query().is_empty() {
            // Show search at the bottom left when a search filter is active
            let search_display = format!("Search: {} (press / to edit)", self.search_query());

            if !self.fetching_repos.is_empty() || !self.cloning_repos.is_empty() {
                let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let spinner = spinner_chars[self.fetch_animation_frame % spinner_chars.len()];

                let mut progress_parts = Vec::new();

                if !self.fetching_repos.is_empty() {
                    let fetch_text = if self.fetching_repos.len() == 1 {
                        format!("{} Fetching 1 repo", spinner)
                    } else {
                        format!("{} Fetching {} repos", spinner, self.fetching_repos.len())
                    };
                    progress_parts.push(fetch_text);
                }

                if !self.cloning_repos.is_empty() {
                    let clone_text = if self.cloning_repos.len() == 1 {
                        format!("{} Cloning 1 repo", spinner)
                    } else {
                        format!("{} Cloning {} repos", spinner, self.cloning_repos.len())
                    };
                    progress_parts.push(clone_text);
                }

                let progress_text = progress_parts.join(", ");

                Line::from(vec![
                    Span::styled(search_display, Style::default().fg(Color::Yellow)),
                    Span::raw(" | "),
                    Span::styled(repo_count, Style::default().fg(Color::Cyan)),
                    Span::raw(" | "),
                    Span::styled(progress_text, Style::default().fg(Color::Yellow)),
                    Span::styled(" | Navigate: ↑/↓ or j/k | Mode: [/] | Clone: c | Drop: d | Quit: q or Ctrl-C", Style::default().fg(Color::DarkGray)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(search_display, Style::default().fg(Color::Yellow)),
                    Span::raw(" | "),
                    Span::styled(repo_count, Style::default().fg(Color::Cyan)),
                    Span::styled(" | Navigate: ↑/↓ or j/k | Mode: [/] | Clone: c | Drop: d | Quit: q or Ctrl-C", Style::default().fg(Color::DarkGray)),
                ])
            }
        } else if !self.fetching_repos.is_empty() || !self.cloning_repos.is_empty() {
            // Show fetch/clone progress with animation
            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner = spinner_chars[self.fetch_animation_frame % spinner_chars.len()];

            let mut progress_parts = Vec::new();

            if !self.fetching_repos.is_empty() {
                let fetch_text = if self.fetching_repos.len() == 1 {
                    format!("{} Fetching 1 repo", spinner)
                } else {
                    format!("{} Fetching {} repos", spinner, self.fetching_repos.len())
                };
                progress_parts.push(fetch_text);
            }

            if !self.cloning_repos.is_empty() {
                let clone_text = if self.cloning_repos.len() == 1 {
                    format!("{} Cloning 1 repo", spinner)
                } else {
                    format!("{} Cloning {} repos", spinner, self.cloning_repos.len())
                };
                progress_parts.push(clone_text);
            }

            let progress_text = progress_parts.join(", ");

            Line::from(vec![
                Span::styled(repo_count, Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled(progress_text, Style::default().fg(Color::Yellow)),
                Span::styled(" | Navigate: ↑/↓ or j/k | Mode: [/] | Search: / | Clone: c | Drop: d | Quit: q or Ctrl-C", Style::default().fg(Color::DarkGray)),
            ])
        } else {
            Line::from(vec![
                Span::styled(repo_count, Style::default().fg(Color::Cyan)),
                Span::styled(" | Navigate: ↑/↓ or j/k | Mode: [/] | Search: / | Clone: c | Drop: d | Quit: q or Ctrl-C", Style::default().fg(Color::DarkGray)),
            ])
        };

        status_text.render(area, buf);
    }
}

