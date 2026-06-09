use std::path::Path;
use std::path::PathBuf;

use crate::error::CToolResult;
use crate::scope::CToolBaseScope;
use crate::scope_config::locate_cool_system_config_path;
use crate::scope_context::CToolScopeContext;
use crate::scope_context::build_ctool_scope_context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CToolContext {
    pub scope_context: CToolScopeContext,
}

impl CToolContext {
    pub fn new(scope_context: CToolScopeContext) -> Self {
        Self { scope_context }
    }

    pub fn from_current_dir(
        current_dir: impl AsRef<Path>,
        base_scope: CToolBaseScope,
        system_config_path: Option<PathBuf>,
    ) -> CToolResult<Self> {
        let scope_context = build_ctool_scope_context(current_dir, base_scope, system_config_path)?;

        Ok(Self::new(scope_context))
    }

    pub fn workspace(current_dir: impl AsRef<Path>) -> CToolResult<Self> {
        Self::from_current_dir(
            current_dir,
            CToolBaseScope::Workspace,
            locate_cool_system_config_path(),
        )
    }

    pub fn none(current_dir: impl AsRef<Path>) -> CToolResult<Self> {
        Self::from_current_dir(
            current_dir,
            CToolBaseScope::None,
            locate_cool_system_config_path(),
        )
    }
}
