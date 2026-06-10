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
use crate::tools::file_ops::ensure_safe_text_file_extension;

pub const CTOOL_MOVE_FILE_TOOL_NAME: &str = "ctool_move_file";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolMoveFileInput {
    pub from: PathBuf,
    pub to: PathBuf,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolMoveFileOutput {
    pub from: String,
    pub to: String,
    pub overwritten: bool,
    pub moved: bool,
}

pub struct CToolMoveFile;

impl CTool for CToolMoveFile {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_MOVE_FILE_TOOL_NAME,
            description: "Move or rename one file inside CToolScopeBase. Directories are never moved.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolMoveFileInput = serde_json::from_value(input)?;
        let output = move_file(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn move_file(
    ctx: &CToolContext,
    input: CToolMoveFileInput,
) -> CToolResult<CToolMoveFileOutput> {
    let (from, to) = gate::ensure_move_allowed(ctx, &input.from, &input.to)?;
    ensure_safe_text_file_extension(&to, "move_file")?;

    let from_metadata = std::fs::metadata(&from)?;
    if !from_metadata.is_file() {
        return Err(CToolError::InvalidInput(format!(
            "move_file only moves files, not directories: {}",
            from.display()
        )));
    }

    let target_exists = to.exists();

    if target_exists {
        let to = gate::ensure_write_allowed(ctx, &to)?;

        let to_metadata = std::fs::metadata(&to)?;
        if !to_metadata.is_file() {
            return Err(CToolError::InvalidInput(format!(
                "target exists but is not a file: {}",
                to.display()
            )));
        }

        if !input.overwrite {
            return Err(CToolError::InvalidInput(format!(
                "target file already exists: {}",
                to.display()
            )));
        }

        std::fs::remove_file(&to)?;
    }

    std::fs::rename(&from, &to)?;

    Ok(CToolMoveFileOutput {
        from: from.display().to_string(),
        to: to.display().to_string(),
        overwritten: target_exists,
        moved: true,
    })
}
