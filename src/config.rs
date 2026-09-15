use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

use crate::app::state::LayoutMode;

impl LayoutMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Columns => "columns",
            Self::Split => "split",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_refresh_minutes")]
    pub refresh_minutes: u64,
    #[serde(default = "default_recent_days")]
    pub recent_days: i64,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub hide_read_feeds: bool,
    #[serde(default = "default_fetch_timeout_seconds")]
    pub fetch_timeout_seconds: u64,
}

fn default_layout() -> String {
    "columns".to_string()
}

fn default_refresh_minutes() -> u64 {
    30
}

fn default_recent_days() -> i64 {
    30
}

fn default_language() -> String {
    "en".to_string()
}

fn default_theme() -> String {
    "terminal".to_string()
}

fn default_fetch_timeout_seconds() -> u64 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: default_language(),
            refresh_minutes: default_refresh_minutes(),
            recent_days: default_recent_days(),
            layout: default_layout(),
            theme: default_theme(),
            hide_read_feeds: false,
            fetch_timeout_seconds: default_fetch_timeout_seconds(),
        }
    }
}

fn config_dir() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
    Some(base.join("rivulet"))
}

fn save_config_field(key: &str, value: &str) {
    let Some(dir) = config_dir() else { return };
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("config.toml");
    let contents = std::fs::read_to_string(&path).unwrap_or_default();
    let new_line = format!("{key} = {value}");
    let mut found = false;
    let new_contents: String = contents
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix(key)
                && rest.starts_with(|c: char| c == '=' || c.is_whitespace())
                && !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_')
            {
                found = true;
                new_line.as_str()
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let new_contents = if found {
        if contents.ends_with('\n') {
            format!("{new_contents}\n")
        } else {
            new_contents
        }
    } else {
        format!("{contents}{new_line}\n")
    };
    if let Err(e) = std::fs::write(&path, new_contents) {
        log::warn!("Failed to save {key} to {}: {e}", path.display());
    }
}

impl Config {
    pub fn fetch_timeout(&self) -> Duration {
        let seconds = if self.fetch_timeout_seconds == 0 {
            default_fetch_timeout_seconds()
        } else {
            self.fetch_timeout_seconds
        };
        Duration::from_secs(seconds)
    }

    pub fn layout_mode(&self) -> LayoutMode {
        match self.layout.as_str() {
            "split" => LayoutMode::Split,
            _ => LayoutMode::Columns,
        }
    }

    pub fn save_layout(mode: LayoutMode) {
        let value = mode.as_str();
        save_config_field("layout", &format!("\"{value}\""));
    }

    pub fn save_hide_read_feeds(value: bool) {
        save_config_field("hide_read_feeds", &value.to_string());
    }

    pub fn load() -> Self {
        let Some(dir) = config_dir() else {
            return Self::default();
        };
        let path = dir.join("config.toml");
        if let Ok(contents) = std::fs::read_to_string(&path) {
            match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!("Failed to parse {}: {}", path.display(), e);
                    Self::default()
                }
            }
        } else {
            // Create default config on first run
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(
                &path,
                "language = \"en\"\nrefresh_minutes = 30\nrecent_days = 30\n",
            );
            Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let config = Config::default();
        assert_eq!(config.language, "en");
        assert_eq!(config.refresh_minutes, 30);
        assert_eq!(config.recent_days, 30);
    }

    #[test]
    fn parse_full_config() {
        let toml = r#"
            language = "ca"
            refresh_minutes = 15
            recent_days = 7
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.language, "ca");
        assert_eq!(config.refresh_minutes, 15);
        assert_eq!(config.recent_days, 7);
    }

    #[test]
    fn parse_partial_config_uses_defaults() {
        let toml = r#"language = "ca""#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.language, "ca");
        assert_eq!(config.refresh_minutes, 30);
        assert_eq!(config.recent_days, 30);
    }

    #[test]
    fn parse_empty_config_uses_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.language, "en");
        assert_eq!(config.refresh_minutes, 30);
    }

    #[test]
    fn fetch_timeout_defaults_to_thirty_seconds() {
        let config = Config::default();
        assert_eq!(config.fetch_timeout_seconds, 30);
        assert_eq!(config.fetch_timeout(), Duration::from_secs(30));
    }

    #[test]
    fn parse_fetch_timeout_seconds() {
        let config: Config = toml::from_str("fetch_timeout_seconds = 45").unwrap();
        assert_eq!(config.fetch_timeout(), Duration::from_secs(45));
    }

    #[test]
    fn fetch_timeout_zero_falls_back_to_default() {
        let config: Config = toml::from_str("fetch_timeout_seconds = 0").unwrap();
        assert_eq!(config.fetch_timeout(), Duration::from_secs(30));
    }
}
