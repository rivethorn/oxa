use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{AppState, AuthState, Panel};
use crate::github::{RepoWithActions, WorkflowSummary};

pub fn render(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Render header
    render_header(f, chunks[0], app);

    // Render main content
    render_main_content(f, chunks[1], app);

    // Render footer
    render_footer(f, chunks[2], app);

    // Render loading overlay if needed
    if app.loading {
        render_loading_overlay(f);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &AppState) {
    let auth_status = match &app.auth_state {
        AuthState::Authenticated { username } => Line::from(vec![
            Span::styled("🔒 ", Style::default().fg(Color::Green)),
            Span::styled("Authenticated as ", Style::default()),
            Span::styled(username, Style::default().fg(Color::Cyan)),
        ]),
        AuthState::Unauthenticated => Line::from(vec![
            Span::styled("🔓 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Not authenticated - Press 'L' to login",
                Style::default().fg(Color::Yellow),
            ),
        ]),
        AuthState::Authenticating => Line::from(vec![
            Span::styled("🔄 ", Style::default().fg(Color::Blue)),
            Span::styled("Authenticating...", Style::default().fg(Color::Blue)),
        ]),
        AuthState::Error { message } => Line::from(vec![
            Span::styled("❌ ", Style::default().fg(Color::Red)),
            Span::styled(message, Style::default().fg(Color::Red)),
        ]),
    };

    let header_block = Block::default()
        .borders(Borders::ALL)
        .title("OXA - GitHub Actions TUI");

    let header = Paragraph::new(auth_status)
        .block(header_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(header, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ])
        .split(area);

    render_repos_panel(f, chunks[0], app);
    render_actions_panel(f, chunks[1], app);
    render_details_panel(f, chunks[2], app);
}

fn render_repos_panel(f: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Repositories")
        .border_style(if app.current_panel == Panel::Repositories {
            Color::Yellow
        } else {
            Color::Gray
        });

    f.render_widget(block, area);

    if app.repos.is_empty() {
        let no_repos =
            Paragraph::new("No repositories found").style(Style::default().fg(Color::Gray));
        f.render_widget(no_repos, area.inner(Margin::new(1, 1)));
        return;
    }

    let items: Vec<ListItem> = app
        .repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let is_selected = Some(i) == app.selected_repo;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };

            let status_indicator = if repo.has_actions {
                let conclusion = repo
                    .last_run
                    .as_ref()
                    .and_then(|r| r.conclusion.as_deref())
                    .unwrap_or("○");
                format!("{} ({})", conclusion, "1")
            } else {
                "No Actions".to_string()
            };

            let content = Line::from(vec![
                Span::styled(&repo.repo.name, style),
                Span::raw(" "),
                Span::styled(status_indicator, Style::default().fg(Color::Gray)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let mut list_state = ListState::default();
    if let Some(selected) = app.selected_repo {
        list_state.select(Some(selected));
    }

    let list =
        List::new(items).highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_actions_panel(f: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Action Runs")
        .border_style(if app.current_panel == Panel::Actions {
            Color::Yellow
        } else {
            Color::Gray
        });

    f.render_widget(block, area);

    if app.repos.is_empty() || app.selected_repo.is_none() || app.actions.is_empty() {
        let no_actions =
            Paragraph::new("No repository selected").style(Style::default().fg(Color::Gray));
        f.render_widget(no_actions, area.inner(Margin::new(1, 1)));
        return;
    }

    let items: Vec<ListItem> = app
        .actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let is_selected = Some(i) == app.selected_action;
            let base_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };

            let conclusion_str = action.conclusion.as_deref().unwrap_or("unknown");
            let name_str = &action.name;
            let time_str = action.updated_at.format("%H:%M").to_string();

            let content = Line::from(vec![
                Span::raw(conclusion_str.to_owned()),
                Span::raw(" "),
                Span::styled(name_str.clone(), base_style),
                Span::raw(" "),
                Span::styled(time_str, Style::default().fg(Color::Gray)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let mut list_state = ListState::default();
    if let Some(selected) = app.selected_action {
        list_state.select(Some(selected));
    }

    let list =
        List::new(items).highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_details_panel(f: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .border_style(if app.current_panel == Panel::Details {
            Color::Yellow
        } else {
            Color::Gray
        });

    f.render_widget(block, area);

    let details_area = area.inner(Margin::new(1, 1));

    if app.selected_action.is_none() {
        let no_details =
            Paragraph::new("No action selected").style(Style::default().fg(Color::Gray));
        f.render_widget(no_details, details_area);
    } else if let Some(selected_index) = app.selected_action {
        if selected_index < app.actions.len() {
            let action = &app.actions[selected_index];

            let details = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                    Span::styled(&action.name, Style::default()),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                    Span::styled(&action.status, Style::default()),
                ]),
                Line::from(vec![
                    Span::styled("Created: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        action.created_at.format("%Y-%m-%d %H:%M").to_string(),
                        Style::default(),
                    ),
                ]),
            ];

            let details_paragraph = Paragraph::new(details)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::White));

            f.render_widget(details_paragraph, details_area);
        } else {
            let no_details =
                Paragraph::new("Invalid action selection").style(Style::default().fg(Color::Gray));
            f.render_widget(no_details, details_area);
        }
    } else {
        let no_details =
            Paragraph::new("Select a repository first").style(Style::default().fg(Color::Gray));
        f.render_widget(no_details, details_area);
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let help_text = match &app.auth_state {
        AuthState::Authenticated { .. } => {
            "[L]ogout [←→]Navigate Panels [↑↓]Navigate Items [Tab]Switch Panels [Q]uit"
        }
        _ => "[L]ogin [Q]uit",
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

fn render_loading_overlay(f: &mut Frame) {
    let area = f.area();

    let loading_text = vec![Line::from("Loading..."), Line::from("Please wait")];

    let paragraph = Paragraph::new(loading_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    let popup_area = centered_rect(60, 20, area);

    f.render_widget(Clear, area);
    f.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
