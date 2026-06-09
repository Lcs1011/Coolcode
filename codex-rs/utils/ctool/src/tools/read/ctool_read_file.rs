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

pub const CTOOL_READ_FILE_TOOL_NAME: &str = "ctool_read_file";

const MAX_READ_FILE_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolReadFileInput {
    pub path: PathBuf,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolReadFileOutput {
    pub path: String,
    pub byte_len: u64,
    pub truncated: bool,
    pub content: String,
}

pub struct CToolReadFile;

impl CTool for CToolReadFile {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_READ_FILE_TOOL_NAME,
            description: "Read a small UTF-8 text file inside CToolBaseScope.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolReadFileInput = serde_json::from_value(input)?;
        let output = read_file(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn read_file(
    ctx: &CToolContext,
    input: CToolReadFileInput,
) -> CToolResult<CToolReadFileOutput> {
    if input.max_bytes == 0 || input.max_bytes > MAX_READ_FILE_BYTES {
        return Err(CToolError::InvalidInput(format!(
            "max_bytes must be between 1 and {MAX_READ_FILE_BYTES}"
        )));
    }

    gate::ensure_read_allowed(ctx, &input.path)?;

    let metadata = std::fs::metadata(&input.path)?;
    if !metadata.is_file() {
        return Err(CToolError::InvalidInput(format!(
            "path is not a file: {}",
            input.path.display()
        )));
    }

    let bytes = std::fs::read(&input.path)?;
    let byte_len = bytes.len() as u64;
    let truncated = byte_len > input.max_bytes;
    let usable_bytes = if truncated {
        &bytes[..input.max_bytes as usize]
    } else {
        &bytes
    };

    let content = String::from_utf8(usable_bytes.to_vec()).map_err(|error| {
        CToolError::InvalidInput(format!(
            "file is not valid UTF-8 text: {} ({error})",
            input.path.display()
        ))
    })?;

    Ok(CToolReadFileOutput {
        path: input.path.display().to_string(),
        byte_len,
        truncated,
        content,
    })
}

fn default_max_bytes() -> u64 {
    MAX_READ_FILE_BYTES
}
