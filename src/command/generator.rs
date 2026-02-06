use plist::Value;

use crate::diff::Change;

/// Generate defaults command from a change
pub fn generate_command(change: &Change) -> String {
    match change {
        Change::Added { domain, key, value } => generate_write_command(domain, key, value),
        Change::Modified {
            domain,
            key,
            new_value,
            ..
        } => generate_write_command(domain, key, new_value),
        Change::Removed { domain, key, .. } => {
            format!(
                "defaults delete \"{}\" \"{}\"",
                escape_string(domain),
                escape_string(key)
            )
        }
    }
}

/// Generate defaults write command
fn generate_write_command(domain: &str, key: &str, value: &Value) -> String {
    let domain = escape_string(domain);
    let key = escape_string(key);
    match value {
        Value::Boolean(b) => {
            format!(
                "defaults write \"{}\" \"{}\" -bool {}",
                domain,
                key,
                if *b { "true" } else { "false" }
            )
        }
        Value::Integer(i) => {
            format!(
                "defaults write \"{}\" \"{}\" -int {}",
                domain,
                key,
                i.as_signed().unwrap_or(0)
            )
        }
        Value::Real(f) => {
            format!("defaults write \"{}\" \"{}\" -float {}", domain, key, f)
        }
        Value::String(s) => {
            format!(
                "defaults write \"{}\" \"{}\" -string \"{}\"",
                domain,
                key,
                escape_string(s)
            )
        }
        Value::Data(d) => {
            let hex: String = d.iter().map(|b| format!("{:02x}", b)).collect();
            format!("defaults write \"{}\" \"{}\" -data {}", domain, key, hex)
        }
        Value::Array(arr) => {
            let elements = format_array_elements(arr);
            format!(
                "defaults write \"{}\" \"{}\" -array {}",
                domain, key, elements
            )
        }
        Value::Dictionary(dict) => {
            if has_nested_structure(dict) {
                format!(
                    "# Nested dictionary not supported by defaults command: {} {}",
                    domain, key
                )
            } else {
                let pairs = format_dict_pairs(dict);
                format!("defaults write \"{}\" \"{}\" -dict {}", domain, key, pairs)
            }
        }
        Value::Date(d) => {
            format!(
                "defaults write \"{}\" \"{}\" -date \"{}\"",
                domain,
                key,
                d.to_xml_format()
            )
        }
        Value::Uid(u) => {
            format!(
                "defaults write \"{}\" \"{}\" -int {} # UID type stored as integer",
                domain, key, u.get()
            )
        }
        _ => format!("# Unsupported type for key: {}", key),
    }
}

/// Format array elements as command arguments
fn format_array_elements(arr: &[Value]) -> String {
    arr.iter()
        .filter_map(|v| match v {
            Value::String(s) => Some(format!("-string \"{}\"", escape_string(s))),
            Value::Integer(i) => Some(format!("-int {}", i.as_signed().unwrap_or(0))),
            Value::Real(f) => Some(format!("-float {}", f)),
            Value::Boolean(b) => {
                Some(format!("-bool {}", if *b { "true" } else { "false" }))
            }
            _ => None, // Skip complex types
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if dictionary contains nested structures
fn has_nested_structure(dict: &plist::Dictionary) -> bool {
    dict.values()
        .any(|v| matches!(v, Value::Dictionary(_) | Value::Array(_)))
}

/// Format dictionary as -dict arguments
fn format_dict_pairs(dict: &plist::Dictionary) -> String {
    dict.iter()
        .filter_map(|(k, v)| format_dict_value(k, v))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a single dictionary key-value pair
fn format_dict_value(key: &str, value: &Value) -> Option<String> {
    match value {
        Value::Boolean(b) => Some(format!(
            "\"{}\" -bool {}",
            escape_string(key),
            if *b { "true" } else { "false" }
        )),
        Value::Integer(i) => Some(format!(
            "\"{}\" -int {}",
            escape_string(key),
            i.as_signed().unwrap_or(0)
        )),
        Value::Real(f) => Some(format!("\"{}\" -float {}", escape_string(key), f)),
        Value::String(s) => Some(format!(
            "\"{}\" -string \"{}\"",
            escape_string(key),
            escape_string(s)
        )),
        Value::Data(d) => {
            let hex: String = d.iter().map(|b| format!("{:02x}", b)).collect();
            Some(format!("\"{}\" -data {}", escape_string(key), hex))
        }
        _ => None,
    }
}

/// Escape string for shell
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::Change;
    use plist::Value;

    // --- escape_string tests ---

    #[test]
    fn test_escape_backslash() {
        assert_eq!(escape_string(r"a\b"), r"a\\b");
    }

    #[test]
    fn test_escape_double_quote() {
        assert_eq!(escape_string(r#"say "hello""#), r#"say \"hello\""#);
    }

    #[test]
    fn test_escape_dollar() {
        assert_eq!(escape_string("$HOME"), "\\$HOME");
    }

    #[test]
    fn test_escape_backtick() {
        assert_eq!(escape_string("`cmd`"), "\\`cmd\\`");
    }

    #[test]
    fn test_escape_combined() {
        assert_eq!(escape_string(r#"\$"`"#), r#"\\\$\"\`"#);
    }

    // --- generate_command tests ---

    #[test]
    fn test_generate_command_added_bool() {
        let change = Change::Added {
            domain: "com.example".to_string(),
            key: "enabled".to_string(),
            value: Value::Boolean(true),
        };
        assert_eq!(
            generate_command(&change),
            r#"defaults write "com.example" "enabled" -bool true"#
        );
    }

    #[test]
    fn test_generate_command_added_string() {
        let change = Change::Added {
            domain: "com.example".to_string(),
            key: "name".to_string(),
            value: Value::String("hello".to_string()),
        };
        assert_eq!(
            generate_command(&change),
            r#"defaults write "com.example" "name" -string "hello""#
        );
    }

    #[test]
    fn test_generate_command_added_int() {
        let change = Change::Added {
            domain: "com.example".to_string(),
            key: "count".to_string(),
            value: Value::Integer(42.into()),
        };
        assert_eq!(
            generate_command(&change),
            r#"defaults write "com.example" "count" -int 42"#
        );
    }

    #[test]
    fn test_generate_command_modified() {
        let change = Change::Modified {
            domain: "com.example".to_string(),
            key: "flag".to_string(),
            old_value: Value::Boolean(false),
            new_value: Value::Boolean(true),
        };
        assert_eq!(
            generate_command(&change),
            r#"defaults write "com.example" "flag" -bool true"#
        );
    }

    #[test]
    fn test_generate_command_removed() {
        let change = Change::Removed {
            domain: "com.example".to_string(),
            key: "old_key".to_string(),
            old_value: Value::Boolean(false),
        };
        assert_eq!(
            generate_command(&change),
            r#"defaults delete "com.example" "old_key""#
        );
    }

    // --- format_array_elements tests ---

    #[test]
    fn test_format_array_string() {
        let arr = vec![Value::String("hello".to_string())];
        assert_eq!(format_array_elements(&arr), r#"-string "hello""#);
    }

    #[test]
    fn test_format_array_int() {
        let arr = vec![Value::Integer(10.into())];
        assert_eq!(format_array_elements(&arr), "-int 10");
    }

    #[test]
    fn test_format_array_float() {
        let arr = vec![Value::Real(3.14)];
        assert_eq!(format_array_elements(&arr), "-float 3.14");
    }

    #[test]
    fn test_format_array_bool() {
        let arr = vec![Value::Boolean(false)];
        assert_eq!(format_array_elements(&arr), "-bool false");
    }

    #[test]
    fn test_format_array_mixed() {
        let arr = vec![
            Value::String("a".to_string()),
            Value::Integer(1.into()),
        ];
        assert_eq!(format_array_elements(&arr), r#"-string "a" -int 1"#);
    }

    // --- format_dict_pairs tests ---

    #[test]
    fn test_format_dict_pairs_basic() {
        let mut dict = plist::Dictionary::new();
        dict.insert("key1".to_string(), Value::Boolean(true));
        let result = format_dict_pairs(&dict);
        assert_eq!(result, r#""key1" -bool true"#);
    }

    #[test]
    fn test_format_dict_pairs_string_value() {
        let mut dict = plist::Dictionary::new();
        dict.insert("name".to_string(), Value::String("val".to_string()));
        let result = format_dict_pairs(&dict);
        assert_eq!(result, r#""name" -string "val""#);
    }

    #[test]
    fn test_format_dict_pairs_int_value() {
        let mut dict = plist::Dictionary::new();
        dict.insert("num".to_string(), Value::Integer(7.into()));
        let result = format_dict_pairs(&dict);
        assert_eq!(result, r#""num" -int 7"#);
    }

    // --- has_nested_structure tests ---

    #[test]
    fn test_has_nested_structure_false() {
        let mut dict = plist::Dictionary::new();
        dict.insert("a".to_string(), Value::Boolean(true));
        dict.insert("b".to_string(), Value::Integer(1.into()));
        assert!(!has_nested_structure(&dict));
    }

    #[test]
    fn test_has_nested_structure_with_array() {
        let mut dict = plist::Dictionary::new();
        dict.insert("arr".to_string(), Value::Array(vec![]));
        assert!(has_nested_structure(&dict));
    }

    #[test]
    fn test_has_nested_structure_with_dict() {
        let mut dict = plist::Dictionary::new();
        dict.insert(
            "nested".to_string(),
            Value::Dictionary(plist::Dictionary::new()),
        );
        assert!(has_nested_structure(&dict));
    }

    #[test]
    fn test_has_nested_structure_empty() {
        let dict = plist::Dictionary::new();
        assert!(!has_nested_structure(&dict));
    }
}
