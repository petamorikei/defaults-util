mod app;
mod command;
mod defaults;
mod diff;
mod error;
mod ui;

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;
use ui::{handle_input, render};

fn main() -> anyhow::Result<()> {
    // ターミナルの初期化
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // アプリケーション実行
    let result = run_app(&mut terminal);

    // ターミナルの復元
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let mut app = App::new();

    loop {
        // 画面を描画
        terminal.draw(|f| render(f, &app))?;

        // Loading状態の場合は、画面描画後にキャプチャを実行
        if app.is_loading() {
            app.execute_capture();
            continue;
        }

        // ユーザー入力を処理
        if handle_input(&mut app)? {
            break;
        }
    }

    Ok(())
}
