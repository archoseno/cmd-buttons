use crate::app_state::{AppState, ExecutionState};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const NORMAL_STYLE: Style = Style::new().fg(Color::White);
const FOCUSED_STYLE: Style = Style::new()
    .fg(Color::Yellow)
    .add_modifier(Modifier::BOLD);
const RUNNING_STYLE: Style = Style::new()
    .fg(Color::Green)
    .add_modifier(Modifier::BOLD);
const HEADER_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::White)
    .add_modifier(Modifier::BOLD);
const ERROR_STYLE: Style = Style::new().fg(Color::Red);
const INFO_STYLE: Style = Style::new().fg(Color::Cyan);

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, chunks[0]);

    if state.buttons.is_empty() {
        render_empty_state(frame, chunks[1], &state.errors);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        render_button_list(frame, main_chunks[0], state);
        render_execution_panel(frame, main_chunks[1], state);
    }

    render_footer(frame, chunks[2], state);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(Span::styled(
        " cmd-buttons ",
        HEADER_STYLE,
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

fn render_button_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    for (i, button) in state.buttons.iter().enumerate() {
        let is_focused = state.focused_index == Some(i);
        let is_running = matches!(&state.execution_state, ExecutionState::Running { .. })
            && state.focused_index == Some(i);

        let style = if is_running {
            RUNNING_STYLE
        } else if is_focused {
            FOCUSED_STYLE
        } else {
            NORMAL_STYLE
        };

        let index_str = format!("{:>3}", button.index);
        let line = Line::from(vec![
            Span::styled(format!("[{}] ", index_str), INFO_STYLE),
            Span::styled(&button.label, style),
        ]);
        lines.push(line);
    }

    let list = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Buttons "));
    frame.render_widget(list, area);
}

fn render_execution_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut content = Vec::new();

    match &state.execution_state {
        ExecutionState::Idle => {
            if let Some(button) = state.selected_button() {
                content.push(Line::from(Span::styled(
                    format!("Selected: {}", button.label),
                    INFO_STYLE,
                )));
                content.push(Line::from(""));
                content.push(Line::from(Span::styled(
                    format!("Command: {}", button.command),
                    NORMAL_STYLE,
                )));
                content.push(Line::from(""));
                content.push(Line::from(Span::styled(
                    "Press Enter to execute",
                    Style::new().fg(Color::DarkGray),
                )));
            }
        }
        ExecutionState::Running { .. } => {
            if let Some(button) = state.selected_button() {
                content.push(Line::from(Span::styled(
                    format!("Running: {}", button.label),
                    RUNNING_STYLE,
                )));
                content.push(Line::from(""));
                content.push(Line::from(Span::styled(
                    format!("Command: {}", button.command),
                    NORMAL_STYLE,
                )));
                content.push(Line::from(""));
                if let Some(elapsed) = state.get_elapsed() {
                    let secs = elapsed.as_secs();
                    let mins = secs / 60;
                    let secs = secs % 60;
                    content.push(Line::from(Span::styled(
                        format!("Time: {:02}:{:02}", mins, secs),
                        INFO_STYLE,
                    )));
                }
                content.push(Line::from(""));
                content.push(Line::from(Span::styled(
                    "Press Ctrl+C to cancel",
                    ERROR_STYLE,
                )));
            }
        }
        ExecutionState::Completed { exit_code, .. } => {
            if let Some(button) = state.selected_button() {
                let status = match exit_code {
                    Some(0) => Span::styled("Completed (OK)", Style::new().fg(Color::Green)),
                    Some(code) => Span::styled(
                        format!("Failed (exit code: {})", code),
                        ERROR_STYLE,
                    ),
                    None => Span::styled("Terminated", ERROR_STYLE),
                };
                content.push(Line::from(Span::styled(
                    format!("Result: {}", button.label),
                    INFO_STYLE,
                )));
                content.push(Line::from(""));
                content.push(Line::from(status));
            }
        }
    }

    content.push(Line::from(""));
    content.push(Line::from("--- Output ---"));
    content.push(Line::from(state.get_output()));

    let output = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Output "))
        .wrap(Wrap { trim: false });
    frame.render_widget(output, area);
}

fn render_empty_state(frame: &mut Frame, area: Rect, errors: &[crate::button_def::ButtonParseError]) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "No valid buttons found.",
        Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Checked directories:",
        Style::new().fg(Color::Cyan),
    )));
    lines.push(Line::from("  ./cmd-buttons (local)"));
    lines.push(Line::from("  ~/.config/cmd-buttons/buttons (XDG)"));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "To add buttons:",
        Style::new().fg(Color::Cyan),
    )));
    lines.push(Line::from("  1. Create a directory: mkdir -p ./cmd-buttons"));
    lines.push(Line::from("  2. Add .toml files with label and command"));
    lines.push(Line::from(""));

    if !errors.is_empty() {
        lines.push(Line::from(Span::styled(
            "Parse errors:",
            ERROR_STYLE,
        )));
        for error in errors {
            lines.push(Line::from(Span::styled(
                format!(
                    "  {}: {}",
                    error.file_path.file_name().unwrap_or_default().to_string_lossy(),
                    error.error
                ),
                ERROR_STYLE,
            )));
        }
    }

    let empty = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Welcome "))
        .wrap(Wrap { trim: false });
    frame.render_widget(empty, area);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let help_text = if state.buttons.is_empty() {
        "q: quit"
    } else if state.is_running() {
        "Ctrl+C: cancel | q: quit"
    } else {
        "↑/↓: navigate | Enter: execute | q: quit | d: diagnostics"
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        help_text,
        Style::new().fg(Color::DarkGray),
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}
