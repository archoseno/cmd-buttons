use crate::button_def::Button;
use crate::button_def::ButtonParseError;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    Idle,
    Running {
        start_time: Instant,
        output: String,
    },
    Completed {
        exit_code: Option<i32>,
        output: String,
    },
}

pub struct AppState {
    pub buttons: Vec<Button>,
    pub errors: Vec<ButtonParseError>,
    pub focused_index: Option<usize>,
    pub scroll_offset: usize,
    pub execution_state: ExecutionState,
    pub quit: bool,
}

impl AppState {
    pub fn new(buttons: Vec<Button>, errors: Vec<ButtonParseError>) -> Self {
        let focused_index = if buttons.is_empty() { None } else { Some(0) };
        Self {
            buttons,
            errors,
            focused_index,
            scroll_offset: 0,
            execution_state: ExecutionState::Idle,
            quit: false,
        }
    }

    pub fn move_focus_up(&mut self) {
        if let Some(idx) = self.focused_index {
            if idx > 0 {
                self.focused_index = Some(idx - 1);
            }
        }
    }

    pub fn move_focus_down(&mut self) {
        if let Some(idx) = self.focused_index {
            if idx + 1 < self.buttons.len() {
                self.focused_index = Some(idx + 1);
            }
        }
    }

    pub fn selected_button(&self) -> Option<&Button> {
        self.focused_index
            .and_then(|idx| self.buttons.get(idx))
    }

    pub fn is_running(&self) -> bool {
        matches!(self.execution_state, ExecutionState::Running { .. })
    }

    pub fn get_elapsed(&self) -> Option<std::time::Duration> {
        match &self.execution_state {
            ExecutionState::Running { start_time, .. } => {
                Some(start_time.elapsed())
            }
            _ => None,
        }
    }

    pub fn get_output(&self) -> &str {
        match &self.execution_state {
            ExecutionState::Idle => "",
            ExecutionState::Running { output, .. } => output,
            ExecutionState::Completed { output, .. } => output,
        }
    }
}
