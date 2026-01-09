use plist::Value as PlistValue;

/// 単一の変更を表す
#[derive(Debug, Clone)]
pub enum Change {
    /// キーが追加された
    Added {
        domain: String,
        key: String,
        value: PlistValue,
    },
    /// キーが削除された
    Removed {
        domain: String,
        key: String,
        old_value: PlistValue,
    },
    /// 値が変更された
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

/// ドメイン単位の差分
#[derive(Debug, Clone)]
pub struct DomainDiff {
    pub domain: String,
    pub changes: Vec<Change>,
}

/// 全体の差分結果
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub domain_diffs: Vec<DomainDiff>,
    pub total_changes: usize,
}
