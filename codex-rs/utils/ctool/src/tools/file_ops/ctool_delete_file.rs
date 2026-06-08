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

pub const CTOOL_DELETE_FILE_TOOL_NAME: &str = "ctool_delete_file";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolDeleteFileInput {
    pub path: PathBuf,
    #[serde(default)]
    pub expected_content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolDeleteFileOutput {
    pub path: String,
    pub byte_len_before: u64,
    pub deleted: bool,
}

pub struct CToolDeleteFile;

impl CTool for CToolDeleteFile {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_DELETE_FILE_TOOL_NAME,
            description: "Delete one file inside CToolScope. Directories are never deleted.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolDeleteFileInput = serde_json::from_value(input)?;
        let output = delete_file(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn delete_file(
    ctx: &CToolContext,
    input: CToolDeleteFileInput,
) -> CToolResult<CToolDeleteFileOutput> {
    gate::ensure_write_allowed(ctx, &input.path)?;

    let metadata = std::fs::metadata(&input.path)?;
    if !metadata.is_file() {
        return Err(CToolError::InvalidInput(format!(
            "delete_file only deletes files, not directories: {}",
            input.path.display()
        )));
    }

    if let Some(expected_content) = input.expected_content {
        let actual_content = std::fs::read_to_string(&input.path)?;
        if actual_content != expected_content {
            return Err(CToolError::InvalidInput(format!(
                "expected_content did not match actual file content: {}",
                input.path.display()
            )));
        }
    }

    let byte_len_before = metadata.len();
    std::fs::remove_file(&input.path)?;

    Ok(CToolDeleteFileOutput {
        path: input.path.display().to_string(),
        byte_len_before,
        deleted: true,
    })
}
