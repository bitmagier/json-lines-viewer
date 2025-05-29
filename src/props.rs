use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Props {
    pub fields_order: Vec<String>,
    pub fields_suppressed: Vec<String>,
}

impl Props {
    pub fn config_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|e| e.join("json-lines-viewer.toml"))
    }

    pub fn init() -> anyhow::Result<Props> {
        let Some(f) = &Self::config_file_path().filter(|f| f.exists()) else {
            return Ok(Props::default());
        };

        let props = fs::read_to_string(f).with_context(|| format!("failed to read config file {f:?}"))?;
        let props = toml::from_str::<Props>(&props).context("failed to parse config file as toml")?;

        Ok(props)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let f = Self::config_file_path().context("Config dir not found")?;
        let toml = toml::to_string_pretty(self)?;

        std::fs::write(&f, toml)?;

        Ok(())
    }
}
