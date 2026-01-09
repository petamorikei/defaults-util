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
            // 終了
            KeyCode::Char('q') => {
                app.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }

            // リセット
            KeyCode::Char('r') => {
                app.reset();
            }

            // Enter: スナップショット取得
            KeyCode::Enter => {
                handle_enter(app);
            }

            // 移動
            KeyCode::Up | KeyCode::Char('k') => {
                app.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.move_down();
            }

            // フォーカス切り替え
            KeyCode::Tab => {
                app.toggle_focus();
            }

            // コピー（Changesペインにフォーカス時のみ）
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
    // DiffViewでChangesペインにフォーカスしている時のみコピー可能
    if app.screen == Screen::DiffView
        && app.focus == Focus::Diff
        && let Some(change) = app.selected_change()
    {
        let cmd = generate_command(change);

        // macOSのpbcopyを使用してクリップボードにコピー
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
