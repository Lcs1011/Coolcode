use std::path::PathBuf;

use crate::scope::CToolBaseScope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CToolContext {
    pub scope: CToolBaseScope,
    pub workspace_roots: Vec<PathBuf>,
    pub selected_paths: Vec<PathBuf>,
}

impl CToolContext {
    pub fn new(
        scope: CToolBaseScope,
        workspace_roots: Vec<PathBuf>,
        selected_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            scope,
            workspace_roots,
            selected_paths,
        }
    }

    pub fn workspace(workspace_roots: Vec<PathBuf>) -> Self {
        Self {
            scope: CToolBaseScope::Workspace,
            workspace_roots,
            selected_paths: Vec::new(),
        }
    }

    pub fn none() -> Self {
        Self {
            scope: CToolBaseScope::None,
            workspace_roots: Vec::new(),
            selected_paths: Vec::new(),
        }
    }
}

impl Default for CToolContext {
    fn default() -> Self {
        Self::none()
    }
}
