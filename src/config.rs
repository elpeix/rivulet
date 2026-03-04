use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "en".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: default_language(),
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

impl Config {
    pub fn load() -> Self {
        let Some(dir) = config_dir() else {
            return Self::default();
        };
        let path = dir.join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => {
                // Create default config on first run
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(&path, "language = \"en\"\n");
                Self::default()
            }
        }
    }
}
