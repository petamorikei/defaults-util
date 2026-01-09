use plist::Value as PlistValue;
use std::collections::HashMap;

/// 単一ドメインの設定データ
#[derive(Debug, Clone)]
pub struct DomainSettings {
    pub values: HashMap<String, PlistValue>,
}

/// 全ドメインのスナップショット
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub domains: HashMap<String, DomainSettings>,
}

impl Snapshot {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
        }
    }

    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}
