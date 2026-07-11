use omniget_core::models::settings::AppSettings;

fn parse_app_settings(json: &str) -> Result<AppSettings, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let settings = value.get("app_settings").unwrap_or(&value);
    serde_json::from_value(settings.clone())
}

pub fn load_app_settings() -> AppSettings {
    let path = match dirs::data_dir() {
        Some(d) => d.join("wtf.tonho.omniget").join("settings.json"),
        None => {
            tracing::warn!("[settings] could not determine data directory, using defaults");
            return AppSettings::default();
        }
    };

    match std::fs::read_to_string(&path) {
        Ok(json) => match parse_app_settings(&json) {
            Ok(settings) => {
                tracing::info!("[settings] loaded from {}", path.display());
                settings
            }
            Err(e) => {
                tracing::warn!("[settings] failed to parse {}: {}, using defaults", path.display(), e);
                AppSettings::default()
            }
        },
        Err(_) => {
            tracing::info!("[settings] no settings file at {}, using defaults", path.display());
            AppSettings::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tauri_store_wrapper() {
        let mut expected = AppSettings::default();
        expected.download.download_subtitles = true;
        expected.download.continuous_lecture_numbers = true;
        let wrapped = serde_json::json!({ "app_settings": expected });

        let parsed = parse_app_settings(&wrapped.to_string()).unwrap();

        assert!(parsed.download.download_subtitles);
        assert!(parsed.download.continuous_lecture_numbers);
    }

    #[test]
    fn still_parses_legacy_unwrapped_settings() {
        let expected = AppSettings::default();
        let parsed = parse_app_settings(&serde_json::to_string(&expected).unwrap()).unwrap();

        assert_eq!(parsed.download.video_quality, expected.download.video_quality);
    }
}
