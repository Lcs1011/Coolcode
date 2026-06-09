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

pub const CTOOL_EDIT_INSERT_TOOL_NAME: &str = "ctool_edit_insert";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolEditInsertInput {
    pub path: PathBuf,

    /// 0 means insert at file beginning.
    /// 1 means insert after line 1.
    pub insert_after_line: usize,

    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolEditInsertOutput {
    pub path: String,
    pub inserted_after_line: usize,
    pub byte_len_before: usize,
    pub byte_len_after: usize,
}

pub struct CToolEditInsert;

impl CTool for CToolEditInsert {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_EDIT_INSERT_TOOL_NAME,
            description: "Insert text after a specific line in a UTF-8 file inside CToolBaseScope.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolEditInsertInput = serde_json::from_value(input)?;
        let output = edit_insert(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn edit_insert(
    ctx: &CToolContext,
    input: CToolEditInsertInput,
) -> CToolResult<CToolEditInsertOutput> {
    gate::ensure_write_allowed(ctx, &input.path)?;

    let before = std::fs::read_to_string(&input.path)?;
    let after = apply_insert_after_line_to_text(&before, input.insert_after_line, &input.content)?;
    std::fs::write(&input.path, &after)?;

    Ok(CToolEditInsertOutput {
        path: input.path.display().to_string(),
        inserted_after_line: input.insert_after_line,
        byte_len_before: before.len(),
        byte_len_after: after.len(),
    })
}

pub fn apply_insert_after_line_to_text(
    text: &str,
    insert_after_line: usize,
    content: &str,
) -> CToolResult<String> {
    if content.is_empty() {
        return Err(CToolError::InvalidInput(
            "content must not be empty".to_string(),
        ));
    }

    let total_lines = if text.is_empty() {
        0
    } else {
        text.lines().count()
    };

    if insert_after_line > total_lines {
        return Err(CToolError::InvalidInput(format!(
            "insert_after_line {insert_after_line} is greater than total line count {total_lines}"
        )));
    }

    let line_ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let insert_offset = byte_offset_after_line(text, insert_after_line)?;

    let mut insert_block = content.to_string();
    if !insert_block.ends_with('\n') {
        insert_block.push_str(line_ending);
    }

    let mut output = String::with_capacity(text.len() + insert_block.len());
    output.push_str(&text[..insert_offset]);
    output.push_str(&insert_block);
    output.push_str(&text[insert_offset..]);
    Ok(output)
}

fn byte_offset_after_line(text: &str, line: usize) -> CToolResult<usize> {
    if line == 0 {
        return Ok(0);
    }

    let mut current_line = 0;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            current_line += 1;
            if current_line == line {
                return Ok(index + ch.len_utf8());
            }
        }
    }

    if current_line + 1 == line {
        return Ok(text.len());
    }

    Err(CToolError::InvalidInput(format!(
        "line {line} was not found"
    )))
}
