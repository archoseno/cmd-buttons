mod app_state;
mod button_def;
mod config;
mod discovery;
mod logging;
mod runner;
mod ui;

use app_state::{AppState, ExecutionState};
use clap::Parser;
use config::{ensure_dirs, load_config, resolve_buttons_dir, Paths};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use discovery::{scan_buttons, write_index};
use logging::save_session_log;
use runner::{send_signal, RunningProcess};
use std::io::{self, stdout};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "cmd-buttons")]
#[command(about = "A TUI application for running commands from button files")]
struct Cli {
    #[arg(short = 's', long = "save-session")]
    save_session: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let paths = Paths::new().expect("Failed to determine XDG paths");
    ensure_dirs(&paths)?;

    let config = load_config(&paths);
    let buttons_dir = resolve_buttons_dir(&paths, &config);

    let (buttons, errors) = match buttons_dir {
        Some(dir) => scan_buttons(&dir),
        None => (Vec::new(), Vec::new()),
    };

    write_index(&paths.index_file, &buttons).ok();

    let mut state = AppState::new(buttons, errors);

    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut running_process: Option<RunningProcess> = None;
    let mut show_diagnostics = false;

    let shell = config.shell.unwrap_or_else(|| "bash".to_string());

    loop {
        if let Some(proc) = running_process.as_mut() {
            match proc.try_wait() {
                Ok(Some(status)) => {
                    let exit_code = status.code();
                    let output = proc.get_output();

                    if cli.save_session {
                        if let Some(button) = state.selected_button() {
                            let file_name = button
                                .file_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy();
                            save_session_log(
                                &paths.sessions_dir,
                                &button.label,
                                &file_name,
                                &button.command,
                                &output,
                                exit_code,
                            )
                            .ok();
                        }
                    }

                    state.execution_state = ExecutionState::Completed {
                        exit_code,
                        output,
                    };
                    running_process = None;
                }
                Ok(None) => {
                    let output = proc.get_output();
                    state.execution_state = ExecutionState::Running {
                        start_time: match &state.execution_state {
                            ExecutionState::Running { start_time, .. } => *start_time,
                            _ => std::time::Instant::now(),
                        },
                        output,
                    };
                }
                Err(_) => {
                    state.execution_state = ExecutionState::Completed {
                        exit_code: None,
                        output: proc.get_output(),
                    };
                    running_process = None;
                }
            }
        }

        terminal.draw(|f| ui::render(f, &state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    } => {
                        state.quit = true;
                    }
                    KeyEvent {
                        code: KeyCode::Up,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('k'),
                        ..
                    } => {
                        if !state.is_running() {
                            state.move_focus_up();
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('j'),
                        ..
                    } => {
                        if !state.is_running() {
                            state.move_focus_down();
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        if !state.is_running() {
                            if let Some(button) = state.selected_button() {
                                match RunningProcess::start(&button.command, &shell) {
                                    Ok(proc) => {
                                        state.execution_state = ExecutionState::Running {
                                            start_time: std::time::Instant::now(),
                                            output: String::new(),
                                        };
                                        running_process = Some(proc);
                                    }
                                    Err(e) => {
                                        state.execution_state = ExecutionState::Completed {
                                            exit_code: Some(1),
                                            output: format!("Failed to start: {}", e),
                                        };
                                    }
                                }
                            }
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        if state.is_running() {
                            if let Some(proc) = running_process.as_mut() {
                                let pid = proc.id();
                                send_signal(pid, nix::sys::signal::Signal::SIGINT).ok();
                                std::thread::sleep(Duration::from_millis(500));
                                if proc.try_wait().ok().flatten().is_none() {
                                    proc.kill().ok();
                                }
                            }
                        } else {
                            state.quit = true;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char('d'),
                        ..
                    } => {
                        show_diagnostics = !show_diagnostics;
                    }
                    _ => {}
                }
            }
        }

        if state.quit {
            break;
        }
    }

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
