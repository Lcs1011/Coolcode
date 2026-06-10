use std::env;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::command_request::CToolCommandConfig;
use crate::command_request::default_command_config;
use crate::command_request::merge_command_configs;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::scope::CToolScopeBase;

pub const COOL_DIR_NAME: &str = ".cool";
pub const COOL_SYSTEM_DIR_NAME: &str = ".cool-system";
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const SCOPE_FILE_NAME: &str = "scope.toml";
pub const COMMAND_FILE_NAME: &str = "command.toml";
pub const COOL_SYSTEM_DIR_ENV: &str = "COOL_SYSTEM_DIR";
pub const COOL_SYSTEM_CONFIG_ENV: &str = "COOL_SYSTEM_CONFIG";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CToolSessionConfig {
    pub scope_base: Option<CToolScopeBase>,
    pub cool_workspace: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct CToolScopeConfig {
    #[serde(default)]
    pub files: CToolScopeRuleSet,
    #[serde(default)]
    pub folders: CToolScopeRuleSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct CToolScopeRuleSet {
    #[serde(default)]
    pub readwrite: Vec<PathBuf>,
    #[serde(default)]
    pub readonly: Vec<PathBuf>,
    #[serde(default)]
    pub hide: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
struct CoolSessionConfigToml {
    #[serde(default)]
    ctool_scope_base: Option<String>,
    #[serde(default)]
    cool_workspace: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct CoolCommandConfigToml {
    #[serde(default = "default_command_config")]
    ctool_command: CToolCommandConfig,
}

impl Default for CoolCommandConfigToml {
    fn default() -> Self {
        Self {
            ctool_command: default_command_config(),
        }
    }
}

pub fn empty_session_config() -> CToolSessionConfig {
    CToolSessionConfig::default()
}

pub fn empty_scope_config() -> CToolScopeConfig {
    CToolScopeConfig::default()
}

pub fn locate_cool_dir(session_root: impl AsRef<Path>) -> PathBuf {
    session_root.as_ref().join(COOL_DIR_NAME)
}

pub fn locate_cool_config_path(session_root: impl AsRef<Path>) -> PathBuf {
    locate_cool_dir(session_root).join(CONFIG_FILE_NAME)
}

pub fn locate_cool_scope_path(session_root: impl AsRef<Path>) -> PathBuf {
    locate_cool_dir(session_root).join(SCOPE_FILE_NAME)
}

pub fn locate_cool_command_path(session_root: impl AsRef<Path>) -> PathBuf {
    locate_cool_dir(session_root).join(COMMAND_FILE_NAME)
}

pub fn locate_cool_system_dir() -> Option<PathBuf> {
    let value = env::var(COOL_SYSTEM_DIR_ENV).ok()?;
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

pub fn locate_legacy_cool_system_config_path() -> Option<PathBuf> {
    let value = env::var(COOL_SYSTEM_CONFIG_ENV).ok()?;
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

pub fn locate_cool_system_config_path() -> Option<PathBuf> {
    if let Some(system_dir) = locate_cool_system_dir() {
        return Some(system_dir.join(CONFIG_FILE_NAME));
    }

    locate_legacy_cool_system_config_path()
}

pub fn locate_cool_system_scope_path() -> Option<PathBuf> {
    locate_cool_system_dir().map(|dir| dir.join(SCOPE_FILE_NAME))
}

pub fn locate_cool_system_command_path() -> Option<PathBuf> {
    locate_cool_system_dir().map(|dir| dir.join(COMMAND_FILE_NAME))
}

pub fn load_optional_cool_session_config(path: &Path) -> CToolResult<CToolSessionConfig> {
    if !path.exists() {
        return Ok(empty_session_config());
    }

    let text = std::fs::read_to_string(path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to read Cool session config file: {} ({error})",
            path.display()
        ))
    })?;

    parse_cool_session_config_toml(&text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to parse Cool session config file: {} ({error})",
            path.display()
        ))
    })
}

pub fn parse_cool_session_config_toml(text: &str) -> CToolResult<CToolSessionConfig> {
    let file: CoolSessionConfigToml = toml::from_str(text).map_err(|error| {
        CToolError::InvalidInput(format!("invalid Cool session TOML config: {error}"))
    })?;

    let scope_base = match file.ctool_scope_base.as_deref() {
        Some(value) => Some(parse_scope_base(value)?),
        None => None,
    };

    Ok(CToolSessionConfig {
        scope_base,
        cool_workspace: file.cool_workspace,
    })
}

pub fn load_optional_cool_config(path: &Path) -> CToolResult<CToolScopeConfig> {
    if !path.exists() {
        return Ok(empty_scope_config());
    }

    let text = std::fs::read_to_string(path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to read Cool scope file: {} ({error})",
            path.display()
        ))
    })?;

    parse_cool_config_toml(&text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to parse Cool scope file: {} ({error})",
            path.display()
        ))
    })
}

pub fn parse_cool_config_toml(text: &str) -> CToolResult<CToolScopeConfig> {
    toml::from_str(text)
        .map_err(|error| CToolError::InvalidInput(format!("invalid Cool scope TOML: {error}")))
}

fn parse_scope_base(value: &str) -> CToolResult<CToolScopeBase> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Ok(CToolScopeBase::None),
        "workspace" | "coolworkspace" | "cool_workspace" | "cool-workspace" => {
            Ok(CToolScopeBase::CoolWorkspace)
        }
        "selectedonly" | "selected_only" | "selected-only" => Ok(CToolScopeBase::SelectedOnly),
        "theeyeofprovidence" | "the_eye_of_providence" | "the-eye-of-providence" => {
            Ok(CToolScopeBase::TheEyeofProvidence)
        }
        _ => Err(CToolError::InvalidInput(format!(
            "unsupported CToolScopeBase: {value}"
        ))),
    }
}

pub fn empty_command_config() -> CToolCommandConfig {
    default_command_config()
}

pub fn load_optional_cool_command_config(path: &Path) -> CToolResult<CToolCommandConfig> {
    if !path.exists() {
        return Ok(empty_command_config());
    }

    let text = std::fs::read_to_string(path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to read Cool command config file: {} ({error})",
            path.display()
        ))
    })?;

    parse_cool_command_config_toml(&text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to parse Cool command config file: {} ({error})",
            path.display()
        ))
    })
}

pub fn parse_cool_command_config_toml(text: &str) -> CToolResult<CToolCommandConfig> {
    let file: CoolCommandConfigToml = toml::from_str(text).map_err(|error| {
        CToolError::InvalidInput(format!("invalid Cool command TOML config: {error}"))
    })?;

    Ok(file.ctool_command)
}

pub fn load_merged_cool_command_config(
    session_command_path: &Path,
    system_command_path: Option<&Path>,
) -> CToolResult<CToolCommandConfig> {
    let session_config = load_optional_cool_command_config(session_command_path)?;
    let system_config = match system_command_path {
        Some(path) => load_optional_cool_command_config(path)?,
        None => empty_command_config(),
    };

    Ok(merge_command_configs(session_config, system_config))
}
