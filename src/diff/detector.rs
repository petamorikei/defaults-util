use plist::Value;
use std::collections::HashMap;

use crate::defaults::Snapshot;

use super::types::{Change, DiffResult, DomainDiff};

/// 2つのスナップショット間の差分を検出
pub fn detect_diff(before: &Snapshot, after: &Snapshot) -> DiffResult {
    let mut domain_diffs = Vec::new();
    let mut total_changes = 0;

    // afterに存在するドメインをチェック
    for (domain, after_settings) in &after.domains {
        let mut changes = Vec::new();

        match before.domains.get(domain) {
            Some(before_settings) => {
                // 既存ドメインの差分検出
                changes.extend(detect_domain_changes(
                    domain,
                    &before_settings.values,
                    &after_settings.values,
                ));
            }
            None => {
                // 新規ドメイン（全キーが追加）
                for (key, value) in &after_settings.values {
                    changes.push(Change::Added {
                        domain: domain.clone(),
                        key: key.clone(),
                        value: value.clone(),
                    });
                }
            }
        }

        if !changes.is_empty() {
            total_changes += changes.len();
            domain_diffs.push(DomainDiff {
                domain: domain.clone(),
                changes,
            });
        }
    }

    // beforeにのみ存在するドメイン（削除されたドメイン）
    for (domain, before_settings) in &before.domains {
        if !after.domains.contains_key(domain) {
            let changes: Vec<Change> = before_settings
                .values
                .iter()
                .map(|(key, value)| Change::Removed {
                    domain: domain.clone(),
                    key: key.clone(),
                    old_value: value.clone(),
                })
                .collect();

            total_changes += changes.len();
            domain_diffs.push(DomainDiff {
                domain: domain.clone(),
                changes,
            });
        }
    }

    // ドメイン名でソート
    domain_diffs.sort_by(|a, b| a.domain.cmp(&b.domain));

    DiffResult {
        domain_diffs,
        total_changes,
    }
}

/// 同一ドメイン内のキー変更を検出
fn detect_domain_changes(
    domain: &str,
    before: &HashMap<String, Value>,
    after: &HashMap<String, Value>,
) -> Vec<Change> {
    let mut changes = Vec::new();

    // afterに存在するキーをチェック
    for (key, after_value) in after {
        match before.get(key) {
            Some(before_value) => {
                if !values_equal(before_value, after_value) {
                    changes.push(Change::Modified {
                        domain: domain.to_string(),
                        key: key.clone(),
                        old_value: before_value.clone(),
                        new_value: after_value.clone(),
                    });
                }
            }
            None => {
                changes.push(Change::Added {
                    domain: domain.to_string(),
                    key: key.clone(),
                    value: after_value.clone(),
                });
            }
        }
    }

    // beforeにのみ存在するキー（削除）
    for (key, before_value) in before {
        if !after.contains_key(key) {
            changes.push(Change::Removed {
                domain: domain.to_string(),
                key: key.clone(),
                old_value: before_value.clone(),
            });
        }
    }

    // キー名でソート
    changes.sort_by(|a, b| a.key().cmp(b.key()));

    changes
}

/// plist::Valueの比較（再帰的）
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Real(a), Value::Real(b)) => (a - b).abs() < f64::EPSILON,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Data(a), Value::Data(b)) => a == b,
        (Value::Date(a), Value::Date(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Dictionary(a), Value::Dictionary(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|bv| values_equal(v, bv)))
        }
        (Value::Uid(a), Value::Uid(b)) => a == b,
        _ => false,
    }
}
