use std::collections::BTreeMap;
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

pub const CTOOL_EDIT_BATCH_TOOL_NAME: &str = "ctool_edit_batch";

const MAX_EDIT_BATCH_OPERATIONS: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolEditBatchInput {
    pub operations: Vec<CToolEditBatchOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum CToolEditBatchOperation {
    Replace {
        path: PathBuf,
        old_string: String,
        new_string: String,
    },
    Insert {
        path: PathBuf,
        insert_after_line: usize,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolEditBatchOutput {
    pub operation_count: usize,
    pub files_touched: Vec<String>,
}

pub struct CToolEditBatch;

impl CTool for CToolEditBatch {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_EDIT_BATCH_TOOL_NAME,
            description: "Apply multiple exact replace/insert edits inside CToolBaseScope.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolEditBatchInput = serde_json::from_value(input)?;
        let output = edit_batch(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn edit_batch(
    ctx: &CToolContext,
    input: CToolEditBatchInput,
) -> CToolResult<CToolEditBatchOutput> {
    if input.operations.is_empty() {
        return Err(CToolError::InvalidInput(
            "operations must not be empty".to_string(),
        ));
    }

    if input.operations.len() > MAX_EDIT_BATCH_OPERATIONS {
        return Err(CToolError::InvalidInput(format!(
            "edit_batch can apply at most {MAX_EDIT_BATCH_OPERATIONS} operations"
        )));
    }

    let operation_count = input.operations.len();
    let mut text_by_path: BTreeMap<PathBuf, String> = BTreeMap::new();

    for operation in input.operations {
        match operation {
            CToolEditBatchOperation::Replace {
                path,
                old_string,
                new_string,
            } => {
                gate::ensure_write_allowed(ctx, &path)?;

                if !text_by_path.contains_key(&path) {
                    text_by_path.insert(path.clone(), std::fs::read_to_string(&path)?);
                }

                let text = text_by_path
                    .get_mut(&path)
                    .expect("path was inserted before edit");

                *text = apply_exact_replace_to_text(text, &old_string, &new_string)?;
            }
            CToolEditBatchOperation::Insert {
                path,
                insert_after_line,
                content,
            } => {
                gate::ensure_write_allowed(ctx, &path)?;

                if !text_by_path.contains_key(&path) {
                    text_by_path.insert(path.clone(), std::fs::read_to_string(&path)?);
                }

                let text = text_by_path
                    .get_mut(&path)
                    .expect("path was inserted before edit");

                *text = apply_insert_after_line_to_text(text, insert_after_line, &content)?;
            }
        }
    }

    let mut files_touched = Vec::new();

    for (path, text) in &text_by_path {
        std::fs::write(path, text)?;
        files_touched.push(path.display().to_string());
    }

    Ok(CToolEditBatchOutput {
        operation_count,
        files_touched,
    })
}
