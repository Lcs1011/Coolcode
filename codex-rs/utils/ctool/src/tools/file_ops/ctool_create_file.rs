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

pub const CTOOL_CREATE_FILE_TOOL_NAME: &str = "ctool_create_file";

const MAX_CREATE_FILE_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolCreateFileInput {
    pub path: PathBuf,
    pub content: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolCreateFileOutput {
    pub path: String,
    pub byte_len: usize,
    pub overwritten: bool,
}

pub struct CToolCreateFile;

impl CTool for CToolCreateFile {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_CREATE_FILE_TOOL_NAME,
            description: "Create a safe UTF-8 text/source file inside CToolScopeBase.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolCreateFileInput = serde_json::from_value(input)?;
        let output = create_file(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn create_file(
    ctx: &CToolContext,
    input: CToolCreateFileInput,
) -> CToolResult<CToolCreateFileOutput> {
    let path = gate::ensure_create_allowed(ctx, &input.path)?;
    ensure_safe_text_file_extension(&path, "create_file")?;

    let byte_len = input.content.len();
    if byte_len > MAX_CREATE_FILE_BYTES {
        return Err(CToolError::InvalidInput(format!(
            "content is too large; max bytes: {MAX_CREATE_FILE_BYTES}"
        )));
    }

    let existed = path.exists();

    if existed && !input.overwrite {
        return Err(CToolError::InvalidInput(format!(
            "file already exists: {}",
            path.display()
        )));
    }

    if existed {
        let path = gate::ensure_write_allowed(ctx, &path)?;
        let metadata = std::fs::metadata(&path)?;
        if !metadata.is_file() {
            return Err(CToolError::InvalidInput(format!(
                "target exists but is not a file: {}",
                path.display()
            )));
        }
    }

    std::fs::write(&path, input.content)?;

    Ok(CToolCreateFileOutput {
        path: path.display().to_string(),
        byte_len,
        overwritten: existed,
    })
}
