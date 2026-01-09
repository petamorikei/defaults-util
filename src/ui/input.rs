use std::io;
use std::process::Command;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::app::{App, Focus, Screen, StatusMessage};
use crate::command::generator::generate_command;

pub fn handle_input(app: &mut App) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(100))?
        && let Event::Key(key) = event::read()?
    {
        match key.code {
            // Quit
            KeyCode::Char('q') => {
                app.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }

            // Reset
            KeyCode::Char('r') => {
                app.reset();
            }

            // Enter: Capture snapshot
            KeyCode::Enter => {
                handle_enter(app);
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                app.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.move_down();
            }

            // Toggle focus
            KeyCode::Tab => {
                app.toggle_focus();
            }

            // Copy (only when focused on Changes pane)
            KeyCode::Char('y') => {
                handle_copy(app);
            }

            _ => {}
        }
    }
    Ok(app.should_quit)
}

fn handle_enter(app: &mut App) {
    match app.screen {
        Screen::Initial => {
            app.start_first_snapshot();
        }
        Screen::WaitingForChanges => {
            app.start_second_snapshot();
        }
        Screen::Error(_) => {
            app.reset();
        }
        _ => {}
    }
}

fn handle_copy(app: &mut App) {
    // Copy only when focused on Changes pane in DiffView
    if app.screen == Screen::DiffView
        && app.focus == Focus::Diff
        && let Some(change) = app.selected_change()
    {
        let cmd = generate_command(change);

        // Use macOS pbcopy to copy to clipboard
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                let _ = stdin.write_all(cmd.as_bytes());
            }
            if child.wait().is_ok() {
                app.set_status(StatusMessage::success("✓ Command copied to clipboard"));
            }
        }
    }
}
