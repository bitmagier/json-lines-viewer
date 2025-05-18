use anyhow::anyhow;
use serde::{Deserialize, Serialize};
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
        match &Self::config_file_path() {
            Some(f) if f.exists() => Ok(toml::from_str(
                &std::fs::read_to_string(f).map_err(|e| anyhow!("'{}' - while reading file {}", e, f.to_string_lossy()))?,
            )?),
            _ => Ok(Props::default()),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        match Self::config_file_path() {
            None => Err(anyhow!("Config dir not found")),
            Some(f) => {
                std::fs::write(&f, toml::to_string_pretty(self)?)?;
                Ok(())
            }
        }
    }
}
