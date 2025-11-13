use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const KNOWN_HOSTS_FILE: &str = "known_hosts.json";

#[derive(Default, Serialize, Deserialize)]
struct KnownHosts {
    entries: HashMap<String, String>,
}

fn storage_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("SFTP-SYNC").join(KNOWN_HOSTS_FILE))
}

fn load_hosts() -> KnownHosts {
    if let Some(path) = storage_path() {
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(hosts) = serde_json::from_slice::<KnownHosts>(&bytes) {
                return hosts;
            }
        }
    }
    KnownHosts::default()
}

fn persist(hosts: &KnownHosts) -> Result<()> {
    if let Some(path) = storage_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("failed to create known_hosts directory")?;
        }
        let data =
            serde_json::to_vec_pretty(hosts).context("failed to serialize known hosts store")?;
        fs::write(path, data).context("failed to write known hosts store")?;
    }
    Ok(())
}

pub enum HostCheck {
    Match,
    New,
    Mismatch { expected: String, got: String },
}

pub fn verify_host(host: &str, fingerprint: &str) -> Result<HostCheck> {
    let mut hosts = load_hosts();
    match hosts.entries.get(host) {
        Some(stored) if stored == fingerprint => Ok(HostCheck::Match),
        Some(stored) => Ok(HostCheck::Mismatch {
            expected: stored.clone(),
            got: fingerprint.to_string(),
        }),
        None => {
            hosts
                .entries
                .insert(host.to_string(), fingerprint.to_string());
            persist(&hosts)?;
            Ok(HostCheck::New)
        }
    }
}

pub fn fingerprint_from_raw(key: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(key);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
