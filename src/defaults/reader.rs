use std::process::Command;

use super::parser::parse_domain_plist;
use super::types::Snapshot;
use crate::error::{AppError, Result};

/// 全ドメインの一覧を取得
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

/// 特定ドメインの設定をXML plist形式で取得
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

/// 全ドメインの設定を取得してスナップショットを作成
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
                // 読み取りできないドメインはスキップ
                continue;
            }
        }
    }

    Ok(snapshot)
}
