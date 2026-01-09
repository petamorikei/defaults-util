use plist::Value;
use std::collections::HashMap;

use crate::defaults::Snapshot;

use super::types::{Change, DiffResult, DomainDiff};

/// Detect diff between two snapshots
pub fn detect_diff(before: &Snapshot, after: &Snapshot) -> DiffResult {
    let mut domain_diffs = Vec::new();
    let mut total_changes = 0;

    // Check domains that exist in after
    for (domain, after_settings) in &after.domains {
        let mut changes = Vec::new();

        match before.domains.get(domain) {
            Some(before_settings) => {
                // Detect changes in existing domain
                changes.extend(detect_domain_changes(
                    domain,
                    &before_settings.values,
                    &after_settings.values,
                ));
            }
            None => {
                // New domain (all keys are added)
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

    // Domains that only exist in before (deleted domains)
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

    // Sort by domain name
    domain_diffs.sort_by(|a, b| a.domain.cmp(&b.domain));

    DiffResult {
        domain_diffs,
        total_changes,
    }
}

/// Detect key changes within a domain
fn detect_domain_changes(
    domain: &str,
    before: &HashMap<String, Value>,
    after: &HashMap<String, Value>,
) -> Vec<Change> {
    let mut changes = Vec::new();

    // Check keys that exist in after
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

    // Keys that only exist in before (deleted)
    for (key, before_value) in before {
        if !after.contains_key(key) {
            changes.push(Change::Removed {
                domain: domain.to_string(),
                key: key.clone(),
                old_value: before_value.clone(),
            });
        }
    }

    // Sort by key name
    changes.sort_by(|a, b| a.key().cmp(b.key()));

    changes
}

/// Compare plist::Value recursively
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
