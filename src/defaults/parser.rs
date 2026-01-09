use plist::Value;
use std::collections::HashMap;

use super::types::DomainSettings;
use crate::error::Result;

/// Parse plist data into DomainSettings
pub fn parse_domain_plist(_domain: &str, data: &[u8]) -> Result<DomainSettings> {
    let value: Value = plist::from_bytes(data)?;

    let values = match value {
        Value::Dictionary(dict) => {
            let mut map = HashMap::new();
            for (key, val) in dict {
                map.insert(key, val);
            }
            map
        }
        _ => HashMap::new(),
    };

    Ok(DomainSettings { values })
}
