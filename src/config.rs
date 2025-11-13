use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::{AppSettings, Language, RemoteTarget, sample_remote_targets};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Default)]
struct PersistedState {
    #[serde(default = "default_language_code")]
    language: String,
    #[serde(default = "default_true")]
    auto_connect: bool,
    #[serde(default = "default_true")]
    watch_local_changes: bool,
    #[serde(default = "default_true")]
    confirm_destructive: bool,
    #[serde(default)]
    limit_bandwidth: bool,
    #[serde(default = "default_bandwidth")]
    bandwidth_mbps: u32,
    #[serde(default)]
    remote_targets: Vec<RemoteTarget>,
}

#[derive(Serialize, Deserialize, Default)]
struct LegacySettings {
    #[serde(default = "default_language_code")]
    language: String,
}

fn default_language_code() -> String {
    "en".to_string()
}

fn default_true() -> bool {
    true
}

fn default_bandwidth() -> u32 {
    200
}

pub fn load_state() -> (AppSettings, Vec<RemoteTarget>) {
    let mut settings = AppSettings::default();
    settings.language = detect_system_language();
    let mut remote_targets = sample_remote_targets();

    if let Some(path) = config_path() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(serialized) = serde_json::from_str::<PersistedState>(&contents) {
                settings.language = language_from_code(&serialized.language);
                settings.auto_connect = serialized.auto_connect;
                settings.watch_local_changes = serialized.watch_local_changes;
                settings.confirm_destructive = serialized.confirm_destructive;
                settings.limit_bandwidth = serialized.limit_bandwidth;
                settings.bandwidth_mbps = serialized.bandwidth_mbps;

                if !serialized.remote_targets.is_empty() {
                    remote_targets = serialized.remote_targets;
                }

                return (settings, remote_targets);
            } else if let Ok(legacy) = serde_json::from_str::<LegacySettings>(&contents) {
                settings.language = language_from_code(&legacy.language);
                return (settings, remote_targets);
            }
        }
    }

    (settings, remote_targets)
}

pub fn save_state(settings: &AppSettings, remote_targets: &[RemoteTarget]) {
    if let Some(path) = config_path() {
        let data = PersistedState {
            language: language_to_code(settings.language).to_string(),
            auto_connect: settings.auto_connect,
            watch_local_changes: settings.watch_local_changes,
            confirm_destructive: settings.confirm_destructive,
            limit_bandwidth: settings.limit_bandwidth,
            bandwidth_mbps: settings.bandwidth_mbps,
            remote_targets: remote_targets.to_vec(),
        };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(contents) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(path, contents);
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("SFTP-SYNC").join(CONFIG_FILE_NAME))
}

fn language_from_code(code: &str) -> Language {
    match code {
        "zh-Hans" | "zh_CN" | "zh-cn" | "zh_hans" | "zh" => Language::SimplifiedChinese,
        "zh-Hant" | "zh_TW" | "zh-tw" | "zh_hant" => Language::TraditionalChinese,
        _ => Language::English,
    }
}

fn language_to_code(language: Language) -> &'static str {
    match language {
        Language::English => "en",
        Language::SimplifiedChinese => "zh-Hans",
        Language::TraditionalChinese => "zh-Hant",
    }
}

fn detect_system_language() -> Language {
    sys_locale::get_locale()
        .as_deref()
        .map(language_from_code)
        .unwrap_or(Language::English)
}
