use std::path::PathBuf;

use crate::scope::CToolScope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CToolContext {
    pub scope: CToolScope,
    pub workspace_roots: Vec<PathBuf>,
    pub selected_paths: Vec<PathBuf>,
}

impl CToolContext {
    pub fn new(
        scope: CToolScope,
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
            scope: CToolScope::Workspace,
            workspace_roots,
            selected_paths: Vec::new(),
        }
    }

    pub fn none() -> Self {
        Self {
            scope: CToolScope::None,
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
