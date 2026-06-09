use std::env;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::error::CToolError;
use crate::error::CToolResult;

pub const USER_CONFIG_FILE_NAME: &str = ".coolconfig.toml";
pub const SYSTEM_CONFIG_FILE_NAME: &str = ".coolsystemconfig.toml";
pub const COOL_SYSTEM_CONFIG_ENV: &str = "COOL_SYSTEM_CONFIG";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct CToolScopeConfig {
    #[serde(default)]
    pub visible_paths: Vec<PathBuf>,

    #[serde(default)]
    pub hide_paths: Vec<PathBuf>,

    #[serde(default)]
    pub protected_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
struct CoolConfigToml {
    #[serde(default)]
    ctool_scope: CToolScopeConfig,
}

pub fn empty_scope_config() -> CToolScopeConfig {
    CToolScopeConfig::default()
}

pub fn locate_cool_config_path(current_dir: impl AsRef<Path>) -> PathBuf {
    current_dir.as_ref().join(USER_CONFIG_FILE_NAME)
}

pub fn locate_cool_system_config_path() -> Option<PathBuf> {
    let value = env::var(COOL_SYSTEM_CONFIG_ENV).ok()?;
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

pub fn load_optional_cool_config(path: &Path) -> CToolResult<CToolScopeConfig> {
    if !path.exists() {
        return Ok(empty_scope_config());
    }

    let text = std::fs::read_to_string(path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to read Cool config file: {} ({error})",
            path.display()
        ))
    })?;

    parse_cool_config_toml(&text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to parse Cool config file: {} ({error})",
            path.display()
        ))
    })
}

pub fn parse_cool_config_toml(text: &str) -> CToolResult<CToolScopeConfig> {
    let file: CoolConfigToml = toml::from_str(text)
        .map_err(|error| CToolError::InvalidInput(format!("invalid Cool TOML config: {error}")))?;

    Ok(file.ctool_scope)
}
