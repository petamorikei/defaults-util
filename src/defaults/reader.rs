use std::process::Command;
use std::time::{Duration, Instant};

use super::parser::parse_domain_plist;
use super::types::Snapshot;
use anyhow::{Result, bail};

/// Run a command with a timeout, killing the child process if it exceeds the limit.
fn run_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<std::process::Output> {
    let mut child = cmd.spawn()?;
    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(_) => return Ok(child.wait_with_output()?),
            None if start.elapsed() > timeout => {
                let _ = child.kill();
                bail!("Command timed out after {:?}", timeout);
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// Get list of all domains
pub fn list_domains() -> Result<Vec<String>> {
    let output = run_with_timeout(
        Command::new("defaults").arg("domains"),
        Duration::from_secs(10),
    )?;

    if !output.status.success() {
        bail!(
            "Failed to execute defaults command: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let domains_str = String::from_utf8_lossy(&output.stdout).into_owned();
    let domains: Vec<String> = domains_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(domains)
}

/// Export domain settings as XML plist
pub fn export_domain(domain: &str) -> Result<Vec<u8>> {
    let output = run_with_timeout(
        Command::new("defaults").args(["export", domain, "-"]),
        Duration::from_secs(5),
    )?;

    if !output.status.success() {
        bail!(
            "Failed to export domain '{}': {}",
            domain,
            String::from_utf8_lossy(&output.stderr)
        );
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
