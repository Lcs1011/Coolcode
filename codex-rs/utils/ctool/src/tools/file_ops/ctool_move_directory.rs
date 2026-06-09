use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::gate;
use crate::tool::CTool;
use crate::tool::CToolSpec;

pub const CTOOL_MOVE_DIRECTORY_TOOL_NAME: &str = "ctool_move_directory";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolMoveDirectoryInput {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolMoveDirectoryOutput {
    pub from: String,
    pub to: String,
    pub moved: bool,
}

pub struct CToolMoveDirectory;

impl CTool for CToolMoveDirectory {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_MOVE_DIRECTORY_TOOL_NAME,
            description: "Move or rename one directory inside CToolBaseScope. Overwrite is never allowed.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolMoveDirectoryInput = serde_json::from_value(input)?;
        let output = move_directory(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn move_directory(
    ctx: &CToolContext,
    input: CToolMoveDirectoryInput,
) -> CToolResult<CToolMoveDirectoryOutput> {
    let (from, to) = gate::ensure_move_allowed(ctx, &input.from, &input.to)?;

    let from_metadata = std::fs::metadata(&from)?;
    if !from_metadata.is_dir() {
        return Err(CToolError::InvalidInput(format!(
            "move_directory only moves directories: {}",
            from.display()
        )));
    }

    ensure_not_current_dir(ctx, &from)?;
    ensure_target_not_inside_source(&from, &to)?;

    if to.exists() {
        return Err(CToolError::InvalidInput(format!(
            "target already exists; move_directory never overwrites: {}",
            to.display()
        )));
    }

    std::fs::rename(&from, &to)?;

    Ok(CToolMoveDirectoryOutput {
        from: from.display().to_string(),
        to: to.display().to_string(),
        moved: true,
    })
}

fn ensure_not_current_dir(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    let path = std::fs::canonicalize(path)?;
    let current_dir = std::fs::canonicalize(&ctx.scope_context.current_dir)?;

    if path == current_dir {
        return Err(CToolError::InvalidInput(format!(
            "refusing to move current dir: {}",
            path.display()
        )));
    }

    Ok(())
}

fn ensure_target_not_inside_source(from: &Path, to: &Path) -> CToolResult<()> {
    let from = std::fs::canonicalize(from)?;

    let Some(to_parent) = to.parent() else {
        return Err(CToolError::InvalidInput(format!(
            "target path has no parent directory: {}",
            to.display()
        )));
    };

    let to_parent = std::fs::canonicalize(to_parent)?;

    if to_parent.starts_with(&from) {
        return Err(CToolError::InvalidInput(format!(
            "refusing to move directory into itself: {} -> {}",
            from.display(),
            to.display()
        )));
    }

    Ok(())
}
