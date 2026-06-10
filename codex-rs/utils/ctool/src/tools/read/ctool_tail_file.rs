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

pub const CTOOL_TAIL_FILE_TOOL_NAME: &str = "ctool_tail_file";

const DEFAULT_TAIL_LINES: usize = 200;
const MAX_TAIL_LINES: usize = 2000;
const DEFAULT_TAIL_BYTES: u64 = 64 * 1024;
const MAX_TAIL_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolTailFileInput {
    pub path: PathBuf,
    #[serde(default)]
    pub lines: Option<usize>,
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolTailFileOutput {
    pub path: String,
    pub byte_len: u64,
    pub total_lines: usize,
    pub returned_lines: usize,
    pub truncated: bool,
    pub content: String,
}

pub struct CToolTailFile;

impl CTool for CToolTailFile {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_TAIL_FILE_TOOL_NAME,
            description: "Read the tail of a UTF-8 text file inside CToolScopeBase.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolTailFileInput = serde_json::from_value(input)?;
        let output = tail_file(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn tail_file(
    ctx: &CToolContext,
    input: CToolTailFileInput,
) -> CToolResult<CToolTailFileOutput> {
    let line_limit = input.lines.unwrap_or(DEFAULT_TAIL_LINES);
    if line_limit == 0 || line_limit > MAX_TAIL_LINES {
        return Err(CToolError::InvalidInput(format!(
            "lines must be between 1 and {MAX_TAIL_LINES}"
        )));
    }

    let byte_limit = input.max_bytes.unwrap_or(DEFAULT_TAIL_BYTES);
    if byte_limit == 0 || byte_limit > MAX_TAIL_BYTES {
        return Err(CToolError::InvalidInput(format!(
            "max_bytes must be between 1 and {MAX_TAIL_BYTES}"
        )));
    }

    let path = gate::ensure_read_allowed(ctx, &input.path)?;
    let metadata = std::fs::metadata(&path)?;
    if !metadata.is_file() {
        return Err(CToolError::InvalidInput(format!(
            "path is not a file: {}",
            path.display()
        )));
    }

    let text = std::fs::read_to_string(&path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "file is not valid UTF-8 text: {} ({error})",
            path.display()
        ))
    })?;

    let all_lines: Vec<&str> = text.lines().collect();
    let total_lines = all_lines.len();
    let start = total_lines.saturating_sub(line_limit);
    let mut selected = all_lines[start..].join("\n");
    let line_truncated = start > 0;

    let byte_truncated = selected.len() as u64 > byte_limit;
    if byte_truncated {
        let keep_from = byte_start_for_tail(&selected, byte_limit as usize);
        selected = selected[keep_from..].to_string();
    }

    Ok(CToolTailFileOutput {
        path: path.display().to_string(),
        byte_len: metadata.len(),
        total_lines,
        returned_lines: selected.lines().count(),
        truncated: line_truncated || byte_truncated,
        content: selected,
    })
}

fn byte_start_for_tail(text: &str, max_bytes: usize) -> usize {
    let mut start = text.len().saturating_sub(max_bytes);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    start
}
