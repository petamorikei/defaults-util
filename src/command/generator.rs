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
        Value::Dictionary(_) => {
            // Write nested dictionary as NeXTSTEP plist format
            let plist_str = format_as_plist(value);
            format!("defaults write \"{}\" \"{}\" '{}'", domain, key, plist_str)
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

/// Format as NeXTSTEP plist
fn format_as_plist(value: &Value) -> String {
    match value {
        Value::Boolean(b) => { if *b { "1" } else { "0" } }.to_string(),
        Value::Integer(i) => i.as_signed().unwrap_or(0).to_string(),
        Value::Real(f) => f.to_string(),
        Value::String(s) => format!("\"{}\"", escape_string(s)),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_as_plist).collect();
            format!("({})", elements.join(", "))
        }
        Value::Dictionary(dict) => {
            let pairs: Vec<String> = dict
                .iter()
                .map(|(k, v)| format!("\"{}\" = {}", k, format_as_plist(v)))
                .collect();
            format!("{{{}}}", pairs.join("; "))
        }
        _ => String::new(),
    }
}

/// Escape string for shell
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
}
