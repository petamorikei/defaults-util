use std::process::Command;

use super::parser::parse_domain_plist;
use super::types::Snapshot;
use crate::error::{AppError, Result};

/// Get list of all domains
pub fn list_domains() -> Result<Vec<String>> {
    let output = Command::new("defaults").arg("domains").output()?;

    if !output.status.success() {
        return Err(AppError::DefaultsCommand(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let domains_str = String::from_utf8(output.stdout)?;
    let domains: Vec<String> = domains_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(domains)
}

/// Export domain settings as XML plist
pub fn export_domain(domain: &str) -> Result<Vec<u8>> {
    let output = Command::new("defaults")
        .args(["export", domain, "-"])
        .output()?;

    if !output.status.success() {
        return Err(AppError::DefaultsCommand(format!(
            "Failed to export domain '{}': {}",
            domain,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(output.stdout)
}

/// Capture snapshot of all domain settings
pub fn capture_snapshot() -> Result<Snapshot> {
    let domains = list_domains()?;
    let mut snapshot = Snapshot::new();

    for domain in domains {
        match export_domain(&domain) {
            Ok(plist_data) => {
                if let Ok(settings) = parse_domain_plist(&domain, &plist_data) {
                    snapshot.domains.insert(domain.clone(), settings);
                }
            }
            Err(_) => {
                // Skip domains that cannot be read
                continue;
            }
        }
    }

    Ok(snapshot)
}
