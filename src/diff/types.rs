use plist::Value as PlistValue;

/// Represents a single change
#[derive(Debug, Clone)]
pub enum Change {
    /// Key was added
    Added {
        domain: String,
        key: String,
        value: PlistValue,
    },
    /// Key was removed
    Removed {
        domain: String,
        key: String,
        old_value: PlistValue,
    },
    /// Value was modified
    Modified {
        domain: String,
        key: String,
        old_value: PlistValue,
        new_value: PlistValue,
    },
}

impl Change {
    pub fn key(&self) -> &str {
        match self {
            Change::Added { key, .. } => key,
            Change::Removed { key, .. } => key,
            Change::Modified { key, .. } => key,
        }
    }
}

/// Diff for a single domain
#[derive(Debug, Clone)]
pub struct DomainDiff {
    pub domain: String,
    pub changes: Vec<Change>,
}

/// Overall diff result
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub domain_diffs: Vec<DomainDiff>,
    pub total_changes: usize,
}
