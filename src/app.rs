use std::time::Instant;

use ratatui::widgets::ListState;

use crate::defaults::{Snapshot, capture_snapshot};
use crate::diff::{Change, DiffResult, detect_diff};

/// Application screen state
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Initial screen (waiting for first snapshot)
    Initial,
    /// Capturing first snapshot
    LoadingFirst,
    /// Capturing second snapshot
    LoadingSecond,
    /// First snapshot captured (waiting for changes)
    WaitingForChanges,
    /// Diff view screen
    DiffView,
    /// Error display
    Error(String),
}

/// Currently focused UI element
#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Domain,
    Diff,
}

/// Status message type
#[derive(Debug, Clone, PartialEq)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
}

/// Status message
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

    /// Check if message is still valid (within 3 seconds)
    pub fn is_valid(&self) -> bool {
        self.created_at.elapsed().as_secs() < 3
    }
}

/// Application state
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
    pub domain_list_state: ListState,
    pub diff_list_state: ListState,
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
            domain_list_state: ListState::default(),
            diff_list_state: ListState::default(),
        }
    }

    /// Set status message
    pub fn set_status(&mut self, status: StatusMessage) {
        self.status = Some(status);
    }

    /// Get valid status message
    pub fn get_status(&self) -> Option<&StatusMessage> {
        self.status.as_ref().filter(|s| s.is_valid())
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.screen = Screen::Initial;
        self.focus = Focus::Domain;
        self.snapshot_before = None;
        self.snapshot_after = None;
        self.diff_result = None;
        self.selected_domain_index = 0;
        self.selected_diff_index = 0;
        self.domain_list_state.select(None);
        self.diff_list_state.select(None);
        self.status = Some(StatusMessage::info("Reset complete"));
    }

    /// Start first snapshot capture (transition to loading screen)
    pub fn start_first_snapshot(&mut self) {
        self.screen = Screen::LoadingFirst;
        self.status = Some(StatusMessage::info(
            "Capturing defaults... This may take a few seconds",
        ));
    }

    /// Start second snapshot capture (transition to loading screen)
    pub fn start_second_snapshot(&mut self) {
        self.screen = Screen::LoadingSecond;
        self.status = Some(StatusMessage::info(
            "Capturing defaults and detecting changes...",
        ));
    }

    /// Execute snapshot capture (called from main loop)
    pub fn execute_capture(&mut self) {
        match self.screen {
            Screen::LoadingFirst => self.capture_first_snapshot(),
            Screen::LoadingSecond => self.capture_second_snapshot(),
            _ => {}
        }
    }

    /// Capture first snapshot
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

    /// Capture second snapshot and detect diff
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

    /// Detect changes between snapshots
    fn detect_changes(&mut self) {
        if let (Some(before), Some(after)) = (&self.snapshot_before, &self.snapshot_after) {
            let diff = detect_diff(before, after);
            let total = diff.total_changes;

            self.diff_result = Some(diff);
            self.screen = Screen::DiffView;
            self.domain_list_state.select(Some(0));
            self.diff_list_state.select(Some(0));

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

    /// Get currently selected change
    pub fn selected_change(&self) -> Option<&Change> {
        self.diff_result
            .as_ref()
            .and_then(|diff| diff.domain_diffs.get(self.selected_domain_index))
            .and_then(|domain_diff| domain_diff.changes.get(self.selected_diff_index))
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.screen == Screen::DiffView {
            match self.focus {
                Focus::Domain => {
                    if self.selected_domain_index > 0 {
                        self.selected_domain_index -= 1;
                        self.selected_diff_index = 0;
                        self.domain_list_state
                            .select(Some(self.selected_domain_index));
                        self.diff_list_state.select(Some(0));
                    }
                }
                Focus::Diff => {
                    if self.selected_diff_index > 0 {
                        self.selected_diff_index -= 1;
                        self.diff_list_state.select(Some(self.selected_diff_index));
                    }
                }
            }
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.screen == Screen::DiffView
            && let Some(diff) = &self.diff_result
        {
            match self.focus {
                Focus::Domain => {
                    if self.selected_domain_index < diff.domain_diffs.len().saturating_sub(1) {
                        self.selected_domain_index += 1;
                        self.selected_diff_index = 0;
                        self.domain_list_state
                            .select(Some(self.selected_domain_index));
                        self.diff_list_state.select(Some(0));
                    }
                }
                Focus::Diff => {
                    if let Some(domain_diff) = diff.domain_diffs.get(self.selected_domain_index)
                        && self.selected_diff_index < domain_diff.changes.len().saturating_sub(1)
                    {
                        self.selected_diff_index += 1;
                        self.diff_list_state.select(Some(self.selected_diff_index));
                    }
                }
            }
        }
    }

    /// Toggle focus between panes
    pub fn toggle_focus(&mut self) {
        if self.screen == Screen::DiffView {
            self.focus = match self.focus {
                Focus::Domain => Focus::Diff,
                Focus::Diff => Focus::Domain,
            };
        }
    }

    /// Check if currently in loading state
    pub fn is_loading(&self) -> bool {
        matches!(self.screen, Screen::LoadingFirst | Screen::LoadingSecond)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
