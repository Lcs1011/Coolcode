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
            description: "Move or rename one file inside CToolScope. Directories are never moved.",
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
    gate::ensure_write_allowed(ctx, &input.from)?;
    gate::ensure_create_allowed(ctx, &input.to)?;

    let from_metadata = std::fs::metadata(&input.from)?;
    if !from_metadata.is_file() {
        return Err(CToolError::InvalidInput(format!(
            "move_file only moves files, not directories: {}",
            input.from.display()
        )));
    }

    let target_exists = input.to.exists();

    if target_exists {
        gate::ensure_write_allowed(ctx, &input.to)?;

        let to_metadata = std::fs::metadata(&input.to)?;
        if !to_metadata.is_file() {
            return Err(CToolError::InvalidInput(format!(
                "target exists but is not a file: {}",
                input.to.display()
            )));
        }

        if !input.overwrite {
            return Err(CToolError::InvalidInput(format!(
                "target file already exists: {}",
                input.to.display()
            )));
        }

        std::fs::remove_file(&input.to)?;
    }

    std::fs::rename(&input.from, &input.to)?;

    Ok(CToolMoveFileOutput {
        from: input.from.display().to_string(),
        to: input.to.display().to_string(),
        overwritten: target_exists,
        moved: true,
    })
}
