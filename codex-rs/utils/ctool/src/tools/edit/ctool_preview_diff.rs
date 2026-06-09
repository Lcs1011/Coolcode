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
use crate::tools::edit::ctool_edit_insert::apply_insert_after_line_to_text;
use crate::tools::edit::ctool_edit_replace::apply_exact_replace_to_text;

pub const CTOOL_PREVIEW_DIFF_TOOL_NAME: &str = "ctool_preview_diff";

const MAX_PREVIEW_DIFF_OPERATIONS: usize = 50;
const MAX_PREVIEW_DIFF_LINES: usize = 400;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolPreviewDiffInput {
    pub path: PathBuf,
    pub operations: Vec<CToolPreviewDiffOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum CToolPreviewDiffOperation {
    Replace {
        old_string: String,
        new_string: String,
    },
    Insert {
        insert_after_line: usize,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolPreviewDiffOutput {
    pub path: String,
    pub operation_count: usize,
    pub changed: bool,
    pub diff: String,
}

pub struct CToolPreviewDiff;

impl CTool for CToolPreviewDiff {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_PREVIEW_DIFF_TOOL_NAME,
            description: "Preview replace/insert edits for one UTF-8 file inside CToolBaseScope without writing.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolPreviewDiffInput = serde_json::from_value(input)?;
        let output = preview_diff(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn preview_diff(
    ctx: &CToolContext,
    input: CToolPreviewDiffInput,
) -> CToolResult<CToolPreviewDiffOutput> {
    if input.operations.is_empty() {
        return Err(CToolError::InvalidInput(
            "operations must not be empty".to_string(),
        ));
    }

    if input.operations.len() > MAX_PREVIEW_DIFF_OPERATIONS {
        return Err(CToolError::InvalidInput(format!(
            "preview_diff can preview at most {MAX_PREVIEW_DIFF_OPERATIONS} operations"
        )));
    }

    let path = gate::ensure_read_allowed(ctx, &input.path)?;

    let before = std::fs::read_to_string(&path)?;
    let mut after = before.clone();

    for operation in &input.operations {
        after = match operation {
            CToolPreviewDiffOperation::Replace {
                old_string,
                new_string,
            } => apply_exact_replace_to_text(&after, old_string, new_string)?,
            CToolPreviewDiffOperation::Insert {
                insert_after_line,
                content,
            } => apply_insert_after_line_to_text(&after, *insert_after_line, content)?,
        };
    }

    let changed = before != after;
    let diff = make_simple_diff(&before, &after);

    Ok(CToolPreviewDiffOutput {
        path: path.display().to_string(),
        operation_count: input.operations.len(),
        changed,
        diff,
    })
}

fn make_simple_diff(before: &str, after: &str) -> String {
    if before == after {
        return "No changes.".to_string();
    }

    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    let max_len = before_lines.len().max(after_lines.len());

    let mut output = String::new();
    output.push_str("--- before\n");
    output.push_str("+++ after\n");

    let mut changed_lines = 0;

    for index in 0..max_len {
        let old = before_lines.get(index).copied();
        let new = after_lines.get(index).copied();

        if old == new {
            continue;
        }

        changed_lines += 1;
        if changed_lines > MAX_PREVIEW_DIFF_LINES {
            output.push_str("... diff truncated ...\n");
            break;
        }

        output.push_str(&format!("@@ line {} @@\n", index + 1));

        if let Some(old) = old {
            output.push_str("- ");
            output.push_str(old);
            output.push('\n');
        }

        if let Some(new) = new {
            output.push_str("+ ");
            output.push_str(new);
            output.push('\n');
        }
    }

    output
}
