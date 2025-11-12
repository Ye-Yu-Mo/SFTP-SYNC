use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::{AppSettings, Language};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Default)]
struct PersistedSettings {
    #[serde(default = "default_language_code")]
    language: String,
}

fn default_language_code() -> String {
    "en".to_string()
}

pub fn load_settings() -> AppSettings {
    let mut settings = AppSettings::default();

    if let Some(path) = config_path() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(serialized) = serde_json::from_str::<PersistedSettings>(&contents) {
                settings.language = language_from_code(&serialized.language);
                return settings;
            }
        }
    }

    settings.language = detect_system_language();
    settings
}

pub fn save_language(language: Language) {
    if let Some(path) = config_path() {
        let data = PersistedSettings {
            language: language_to_code(language).to_string(),
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
