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

pub const CTOOL_READ_CODE_RANGE_TOOL_NAME: &str = "ctool_read_code_range";

const MAX_READ_CODE_RANGE_LINES: usize = 400;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolReadCodeRangeInput {
    pub path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolReadCodeRangeOutput {
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub total_lines: usize,
    pub content: String,
}

pub struct CToolReadCodeRange;

impl CTool for CToolReadCodeRange {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_READ_CODE_RANGE_TOOL_NAME,
            description: "Read a specific inclusive line range from a UTF-8 text file inside CToolBaseScope.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolReadCodeRangeInput = serde_json::from_value(input)?;
        let output = read_code_range(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn read_code_range(
    ctx: &CToolContext,
    input: CToolReadCodeRangeInput,
) -> CToolResult<CToolReadCodeRangeOutput> {
    if input.start_line == 0 {
        return Err(CToolError::InvalidInput(
            "start_line must be greater than 0".to_string(),
        ));
    }

    if input.end_line < input.start_line {
        return Err(CToolError::InvalidInput(
            "end_line must be greater than or equal to start_line".to_string(),
        ));
    }

    let requested_lines = input.end_line - input.start_line + 1;
    if requested_lines > MAX_READ_CODE_RANGE_LINES {
        return Err(CToolError::InvalidInput(format!(
            "read_code_range can read at most {MAX_READ_CODE_RANGE_LINES} lines at a time"
        )));
    }

    gate::ensure_read_allowed(ctx, &input.path)?;

    let text = std::fs::read_to_string(&input.path)?;
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len();

    if input.start_line > total_lines {
        return Err(CToolError::InvalidInput(format!(
            "start_line {} is greater than total line count {}",
            input.start_line, total_lines
        )));
    }

    let effective_end_line = input.end_line.min(total_lines);
    let content = lines[(input.start_line - 1)..effective_end_line].join("\n");

    Ok(CToolReadCodeRangeOutput {
        path: input.path.display().to_string(),
        start_line: input.start_line,
        end_line: effective_end_line,
        total_lines,
        content,
    })
}
