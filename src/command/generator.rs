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
            format!("defaults delete \"{}\" \"{}\"", domain, key)
        }
    }
}

/// Generate defaults write command
fn generate_write_command(domain: &str, key: &str, value: &Value) -> String {
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
            // Treat UID as integer
            format!("defaults write \"{}\" \"{}\" -int {}", domain, key, u.get())
        }
        _ => format!("# Unsupported type for key: {}", key),
    }
}

/// Format array elements as command arguments
fn format_array_elements(arr: &[Value]) -> String {
    arr.iter()
        .filter_map(|v| match v {
            Value::String(s) => Some(format!("\"{}\"", escape_string(s))),
            Value::Integer(i) => Some(i.as_signed().unwrap_or(0).to_string()),
            Value::Real(f) => Some(f.to_string()),
            Value::Boolean(b) => Some(if *b { "1" } else { "0" }.to_string()),
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
}
