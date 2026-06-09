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

pub const CTOOL_DELETE_DIRECTORY_TOOL_NAME: &str = "ctool_delete_directory";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolDeleteDirectoryInput {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolDeleteDirectoryOutput {
    pub path: String,
    pub deleted: bool,
}

pub struct CToolDeleteDirectory;

impl CTool for CToolDeleteDirectory {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_DELETE_DIRECTORY_TOOL_NAME,
            description: "Delete one empty directory inside CToolBaseScope. Recursive deletion is never allowed.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolDeleteDirectoryInput = serde_json::from_value(input)?;
        let output = delete_directory(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn delete_directory(
    ctx: &CToolContext,
    input: CToolDeleteDirectoryInput,
) -> CToolResult<CToolDeleteDirectoryOutput> {
    gate::ensure_write_allowed(ctx, &input.path)?;

    let metadata = std::fs::metadata(&input.path)?;
    if !metadata.is_dir() {
        return Err(CToolError::InvalidInput(format!(
            "delete_directory only deletes directories: {}",
            input.path.display()
        )));
    }

    ensure_not_workspace_root(ctx, &input.path)?;

    if std::fs::read_dir(&input.path)?.next().is_some() {
        return Err(CToolError::InvalidInput(format!(
            "delete_directory only deletes empty directories: {}",
            input.path.display()
        )));
    }

    std::fs::remove_dir(&input.path)?;

    Ok(CToolDeleteDirectoryOutput {
        path: input.path.display().to_string(),
        deleted: true,
    })
}

fn ensure_not_workspace_root(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    let path = std::fs::canonicalize(path)?;

    for root in &ctx.workspace_roots {
        if let Ok(root) = std::fs::canonicalize(root) {
            if path == root {
                return Err(CToolError::InvalidInput(format!(
                    "refusing to delete workspace root: {}",
                    path.display()
                )));
            }
        }
    }

    Ok(())
}
