use anyhow::{Context, Result};
use keyring::Entry;

use crate::model::TargetId;

const SERVICE_NAME: &str = "SFTP-SYNC";

pub enum SecretSlot {
    Password(TargetId),
    KeyPassphrase(TargetId),
}

impl SecretSlot {
    fn storage_key(&self) -> String {
        match self {
            SecretSlot::Password(id) => format!("target-{id}-password"),
            SecretSlot::KeyPassphrase(id) => format!("target-{id}-passphrase"),
        }
    }
}

fn entry_for(slot: &SecretSlot) -> Result<Entry> {
    Entry::new(SERVICE_NAME, &slot.storage_key()).context("failed to open keyring entry")
}

pub fn store(slot: SecretSlot, secret: &str) -> Result<()> {
    if secret.is_empty() {
        return Ok(());
    }
    let entry = entry_for(&slot)?;
    entry
        .set_password(secret)
        .context("failed to set keyring secret")
}

pub fn load(slot: SecretSlot) -> Result<Option<String>> {
    let entry = entry_for(&slot)?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err).context("failed to load keyring secret"),
    }
}

pub fn delete(slot: SecretSlot) -> Result<()> {
    let entry = entry_for(&slot)?;
    match entry.delete_password() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err).context("failed to delete keyring secret"),
    }
}
