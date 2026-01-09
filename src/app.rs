use std::time::Instant;

use crate::defaults::{Snapshot, capture_snapshot};
use crate::diff::{Change, DiffResult, detect_diff};

/// アプリケーションの画面状態
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// 初期画面（スナップショット取得待ち）
    Initial,
    /// スナップショット取得中（1回目）
    LoadingFirst,
    /// スナップショット取得中（2回目）
    LoadingSecond,
    /// スナップショット1取得済み（変更待ち）
    WaitingForChanges,
    /// 差分表示画面
    DiffView,
    /// エラー表示
    Error(String),
}

/// 選択されているUI要素
#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Domain,
    Diff,
}

/// ステータスメッセージの種類
#[derive(Debug, Clone, PartialEq)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
}

/// ステータスメッセージ
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub kind: StatusKind,
    pub created_at: Instant,
}

impl StatusMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: StatusKind::Info,
            created_at: Instant::now(),
        }
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: StatusKind::Success,
            created_at: Instant::now(),
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: StatusKind::Warning,
            created_at: Instant::now(),
        }
    }

    /// メッセージが有効かどうか（3秒以内）
    pub fn is_valid(&self) -> bool {
        self.created_at.elapsed().as_secs() < 3
    }
}

/// アプリケーション全体の状態
pub struct App {
    pub screen: Screen,
    pub focus: Focus,
    pub snapshot_before: Option<Snapshot>,
    pub snapshot_after: Option<Snapshot>,
    pub diff_result: Option<DiffResult>,
    pub selected_domain_index: usize,
    pub selected_diff_index: usize,
    pub should_quit: bool,
    pub status: Option<StatusMessage>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Initial,
            focus: Focus::Domain,
            snapshot_before: None,
            snapshot_after: None,
            diff_result: None,
            selected_domain_index: 0,
            selected_diff_index: 0,
            should_quit: false,
            status: None,
        }
    }

    /// ステータスメッセージを設定
    pub fn set_status(&mut self, status: StatusMessage) {
        self.status = Some(status);
    }

    /// 有効なステータスメッセージを取得
    pub fn get_status(&self) -> Option<&StatusMessage> {
        self.status.as_ref().filter(|s| s.is_valid())
    }

    /// 最初の状態にリセット
    pub fn reset(&mut self) {
        self.screen = Screen::Initial;
        self.focus = Focus::Domain;
        self.snapshot_before = None;
        self.snapshot_after = None;
        self.diff_result = None;
        self.selected_domain_index = 0;
        self.selected_diff_index = 0;
        self.status = Some(StatusMessage::info("Reset complete"));
    }

    /// 1回目のスナップショット取得を開始（Loading画面に遷移）
    pub fn start_first_snapshot(&mut self) {
        self.screen = Screen::LoadingFirst;
        self.status = Some(StatusMessage::info(
            "Capturing defaults... This may take a few seconds",
        ));
    }

    /// 2回目のスナップショット取得を開始（Loading画面に遷移）
    pub fn start_second_snapshot(&mut self) {
        self.screen = Screen::LoadingSecond;
        self.status = Some(StatusMessage::info(
            "Capturing defaults and detecting changes...",
        ));
    }

    /// 実際にスナップショットを取得（メインループから呼ばれる）
    pub fn execute_capture(&mut self) {
        match self.screen {
            Screen::LoadingFirst => self.capture_first_snapshot(),
            Screen::LoadingSecond => self.capture_second_snapshot(),
            _ => {}
        }
    }

    /// 最初のスナップショットを取得
    fn capture_first_snapshot(&mut self) {
        match capture_snapshot() {
            Ok(snapshot) => {
                let count = snapshot.domain_count();
                self.snapshot_before = Some(snapshot);
                self.screen = Screen::WaitingForChanges;
                self.status = Some(StatusMessage::success(format!(
                    "✓ Captured {} domains successfully",
                    count
                )));
            }
            Err(e) => {
                self.screen = Screen::Error(format!("Failed to capture snapshot: {}", e));
            }
        }
    }

    /// 2番目のスナップショットを取得して差分を検出
    fn capture_second_snapshot(&mut self) {
        match capture_snapshot() {
            Ok(snapshot) => {
                self.snapshot_after = Some(snapshot);
                self.detect_changes();
            }
            Err(e) => {
                self.screen = Screen::Error(format!("Failed to capture snapshot: {}", e));
            }
        }
    }

    /// 差分を検出
    fn detect_changes(&mut self) {
        if let (Some(before), Some(after)) = (&self.snapshot_before, &self.snapshot_after) {
            let diff = detect_diff(before, after);
            let total = diff.total_changes;

            self.diff_result = Some(diff);
            self.screen = Screen::DiffView;

            if total == 0 {
                self.status = Some(StatusMessage::warning("No changes detected"));
            } else {
                self.status = Some(StatusMessage::success(format!(
                    "✓ Found {} change{}",
                    total,
                    if total == 1 { "" } else { "s" }
                )));
            }
        }
    }

    /// 現在選択中の差分を取得
    pub fn selected_change(&self) -> Option<&Change> {
        self.diff_result
            .as_ref()
            .and_then(|diff| diff.domain_diffs.get(self.selected_domain_index))
            .and_then(|domain_diff| domain_diff.changes.get(self.selected_diff_index))
    }

    /// 上に移動
    pub fn move_up(&mut self) {
        if self.screen == Screen::DiffView {
            match self.focus {
                Focus::Domain => {
                    if self.selected_domain_index > 0 {
                        self.selected_domain_index -= 1;
                        self.selected_diff_index = 0;
                    }
                }
                Focus::Diff => {
                    if self.selected_diff_index > 0 {
                        self.selected_diff_index -= 1;
                    }
                }
            }
        }
    }

    /// 下に移動
    pub fn move_down(&mut self) {
        if self.screen == Screen::DiffView
            && let Some(diff) = &self.diff_result
        {
            match self.focus {
                Focus::Domain => {
                    if self.selected_domain_index < diff.domain_diffs.len().saturating_sub(1) {
                        self.selected_domain_index += 1;
                        self.selected_diff_index = 0;
                    }
                }
                Focus::Diff => {
                    if let Some(domain_diff) = diff.domain_diffs.get(self.selected_domain_index)
                        && self.selected_diff_index < domain_diff.changes.len().saturating_sub(1)
                    {
                        self.selected_diff_index += 1;
                    }
                }
            }
        }
    }

    /// フォーカス切り替え
    pub fn toggle_focus(&mut self) {
        if self.screen == Screen::DiffView {
            self.focus = match self.focus {
                Focus::Domain => Focus::Diff,
                Focus::Diff => Focus::Domain,
            };
        }
    }

    /// Loading状態かどうか
    pub fn is_loading(&self) -> bool {
        matches!(self.screen, Screen::LoadingFirst | Screen::LoadingSecond)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
