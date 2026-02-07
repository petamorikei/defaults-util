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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults::types::DomainSettings;

    // Helper to create a Snapshot with given domains
    fn make_snapshot(domains: Vec<(&str, Vec<(&str, Value)>)>) -> Snapshot {
        let mut snap = Snapshot::new();
        for (domain, kvs) in domains {
            let mut settings = DomainSettings {
                values: HashMap::new(),
            };
            for (k, v) in kvs {
                settings.values.insert(k.to_string(), v);
            }
            snap.domains.insert(domain.to_string(), settings);
        }
        snap
    }

    // --- detect_diff tests ---

    #[test]
    fn test_detect_diff_added() {
        let before = make_snapshot(vec![]);
        let after = make_snapshot(vec![("com.test", vec![("key1", Value::Boolean(true))])]);

        let result = detect_diff(&before, &after);
        assert_eq!(result.total_changes, 1);
        assert_eq!(result.domain_diffs.len(), 1);
        match &result.domain_diffs[0].changes[0] {
            Change::Added { domain, key, value } => {
                assert_eq!(domain, "com.test");
                assert_eq!(key, "key1");
                assert!(matches!(value, Value::Boolean(true)));
            }
            _ => panic!("Expected Added change"),
        }
    }

    #[test]
    fn test_detect_diff_removed() {
        let before = make_snapshot(vec![(
            "com.test",
            vec![("key1", Value::String("old".to_string()))],
        )]);
        let after = make_snapshot(vec![]);

        let result = detect_diff(&before, &after);
        assert_eq!(result.total_changes, 1);
        match &result.domain_diffs[0].changes[0] {
            Change::Removed {
                domain,
                key,
                old_value,
            } => {
                assert_eq!(domain, "com.test");
                assert_eq!(key, "key1");
                assert!(matches!(old_value, Value::String(s) if s == "old"));
            }
            _ => panic!("Expected Removed change"),
        }
    }

    #[test]
    fn test_detect_diff_modified() {
        let before = make_snapshot(vec![("com.test", vec![("key1", Value::Integer(1.into()))])]);
        let after = make_snapshot(vec![("com.test", vec![("key1", Value::Integer(2.into()))])]);

        let result = detect_diff(&before, &after);
        assert_eq!(result.total_changes, 1);
        match &result.domain_diffs[0].changes[0] {
            Change::Modified {
                domain,
                key,
                old_value,
                new_value,
            } => {
                assert_eq!(domain, "com.test");
                assert_eq!(key, "key1");
                assert!(matches!(old_value, Value::Integer(i) if i.as_signed() == Some(1)));
                assert!(matches!(new_value, Value::Integer(i) if i.as_signed() == Some(2)));
            }
            _ => panic!("Expected Modified change"),
        }
    }

    #[test]
    fn test_detect_diff_no_change() {
        let before = make_snapshot(vec![("com.test", vec![("key1", Value::Boolean(true))])]);
        let after = make_snapshot(vec![("com.test", vec![("key1", Value::Boolean(true))])]);

        let result = detect_diff(&before, &after);
        assert_eq!(result.total_changes, 0);
        assert!(result.domain_diffs.is_empty());
    }

    // --- values_equal tests ---

    #[test]
    fn test_values_equal_bool() {
        assert!(values_equal(&Value::Boolean(true), &Value::Boolean(true)));
        assert!(!values_equal(&Value::Boolean(true), &Value::Boolean(false)));
    }

    #[test]
    fn test_values_equal_int() {
        assert!(values_equal(
            &Value::Integer(42.into()),
            &Value::Integer(42.into())
        ));
        assert!(!values_equal(
            &Value::Integer(1.into()),
            &Value::Integer(2.into())
        ));
    }

    #[test]
    fn test_values_equal_real() {
        assert!(values_equal(&Value::Real(3.14), &Value::Real(3.14)));
        assert!(!values_equal(&Value::Real(1.0), &Value::Real(2.0)));
    }

    #[test]
    fn test_values_equal_string() {
        assert!(values_equal(
            &Value::String("abc".to_string()),
            &Value::String("abc".to_string())
        ));
        assert!(!values_equal(
            &Value::String("abc".to_string()),
            &Value::String("xyz".to_string())
        ));
    }

    #[test]
    fn test_values_equal_array() {
        let a = Value::Array(vec![Value::Boolean(true), Value::Integer(1.into())]);
        let b = Value::Array(vec![Value::Boolean(true), Value::Integer(1.into())]);
        let c = Value::Array(vec![Value::Boolean(false)]);
        assert!(values_equal(&a, &b));
        assert!(!values_equal(&a, &c));
    }

    #[test]
    fn test_values_equal_dict() {
        let mut da = plist::Dictionary::new();
        da.insert("k".to_string(), Value::Integer(1.into()));
        let mut db = plist::Dictionary::new();
        db.insert("k".to_string(), Value::Integer(1.into()));
        let mut dc = plist::Dictionary::new();
        dc.insert("k".to_string(), Value::Integer(2.into()));

        assert!(values_equal(
            &Value::Dictionary(da.clone()),
            &Value::Dictionary(db)
        ));
        assert!(!values_equal(
            &Value::Dictionary(da),
            &Value::Dictionary(dc)
        ));
    }

    #[test]
    fn test_values_equal_different_types() {
        assert!(!values_equal(
            &Value::Boolean(true),
            &Value::Integer(1.into())
        ));
        assert!(!values_equal(
            &Value::String("1".to_string()),
            &Value::Integer(1.into())
        ));
    }
}
