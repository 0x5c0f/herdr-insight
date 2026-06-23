use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct TimelineConfig {
    #[serde(default)]
    pub(crate) columns: ColumnConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ColumnConfig {
    #[serde(default = "default_true")]
    pub(crate) time: bool,
    #[serde(default = "default_true")]
    pub(crate) state: bool,
    #[serde(default = "default_true")]
    pub(crate) agent: bool,
    #[serde(default = "default_true")]
    pub(crate) pane: bool,
    #[serde(default = "default_true")]
    pub(crate) status: bool,
    #[serde(default = "default_true")]
    pub(crate) duration: bool,
    #[serde(default = "default_false")]
    pub(crate) session: bool,
    #[serde(default = "default_false")]
    pub(crate) output: bool,
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

impl Default for ColumnConfig {
    fn default() -> Self {
        Self {
            time: true,
            state: true,
            agent: true,
            pane: true,
            status: true,
            duration: true,
            session: false,
            output: false,
        }
    }
}

impl TimelineConfig {
    pub(crate) fn load() -> Self {
        let config_dir = std::env::var("HERDR_PLUGIN_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_default();
        let path = config_dir.join("config.toml");
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }
}
