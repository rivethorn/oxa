use anyhow::Result;

#[derive(Debug, Clone)]
pub enum AuthState {
    Unauthenticated,
    Authenticating,
    Authenticated { username: String },
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Repositories,
    Actions,
    Details,
}

pub struct AppState {
    pub auth_state: AuthState,
    pub repos: Vec<crate::github::RepoWithActions>,
    pub selected_repo: Option<usize>,
    pub current_panel: Panel,
    pub loading: bool,
    pub actions: Vec<crate::github::WorkflowSummary>,
    pub selected_action: Option<usize>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            auth_state: AuthState::Unauthenticated,
            repos: Vec::new(),
            selected_repo: None,
            current_panel: Panel::Repositories,
            loading: false,
            actions: Vec::new(),
            selected_action: None,
        }
    }

    pub fn move_selection(&mut self, direction: i32) {
        if !self.repos.is_empty() {
            let current = self.selected_repo.unwrap_or(0);
            let new_pos = (current as i32 + direction).max(0) as usize % self.repos.len();
            self.selected_repo = Some(new_pos);
        }
    }

    pub fn switch_panel(&mut self, direction: i32) {
        let panels = [Panel::Repositories, Panel::Actions, Panel::Details];
        let current_index = panels
            .iter()
            .position(|&p| p == self.current_panel)
            .unwrap_or(0);
        let new_index = (current_index as i32 + direction).rem_euclid(panels.len() as i32) as usize;
        self.current_panel = panels[new_index];
    }
}
