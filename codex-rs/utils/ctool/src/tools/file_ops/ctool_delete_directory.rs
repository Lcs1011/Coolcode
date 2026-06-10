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
            description: "Delete one empty directory inside CToolScopeBase. Recursive deletion is never allowed.",
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
    let path = gate::ensure_delete_allowed(ctx, &input.path)?;

    let metadata = std::fs::metadata(&path)?;
    if !metadata.is_dir() {
        return Err(CToolError::InvalidInput(format!(
            "delete_directory only deletes directories: {}",
            path.display()
        )));
    }

    ensure_not_cool_workspace(ctx, &path)?;

    if std::fs::read_dir(&path)?.next().is_some() {
        return Err(CToolError::InvalidInput(format!(
            "delete_directory only deletes empty directories: {}",
            path.display()
        )));
    }

    std::fs::remove_dir(&path)?;

    Ok(CToolDeleteDirectoryOutput {
        path: path.display().to_string(),
        deleted: true,
    })
}

fn ensure_not_cool_workspace(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    let path = std::fs::canonicalize(path)?;
    let cool_workspace = std::fs::canonicalize(&ctx.scope_context.cool_workspace)?;

    if path == cool_workspace {
        return Err(CToolError::InvalidInput(format!(
            "refusing to delete CoolWorkspace root: {}",
            path.display()
        )));
    }

    Ok(())
}
