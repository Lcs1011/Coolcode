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

pub const CTOOL_EDIT_REPLACE_TOOL_NAME: &str = "ctool_edit_replace";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolEditReplaceInput {
    pub path: PathBuf,
    pub old_string: String,
    pub new_string: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolEditReplaceOutput {
    pub path: String,
    pub replaced: usize,
    pub byte_len_before: usize,
    pub byte_len_after: usize,
}

pub struct CToolEditReplace;

impl CTool for CToolEditReplace {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_EDIT_REPLACE_TOOL_NAME,
            description: "Replace one exact text occurrence in a UTF-8 file inside CToolScopeBase.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolEditReplaceInput = serde_json::from_value(input)?;
        let output = edit_replace(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn edit_replace(
    ctx: &CToolContext,
    input: CToolEditReplaceInput,
) -> CToolResult<CToolEditReplaceOutput> {
    gate::ensure_read_allowed(ctx, &input.path)?;
    let path = gate::ensure_write_allowed(ctx, &input.path)?;

    let before = std::fs::read_to_string(&path)?;
    let after = apply_exact_replace_to_text(&before, &input.old_string, &input.new_string)?;
    std::fs::write(&path, &after)?;

    Ok(CToolEditReplaceOutput {
        path: path.display().to_string(),
        replaced: 1,
        byte_len_before: before.len(),
        byte_len_after: after.len(),
    })
}

pub fn apply_exact_replace_to_text(
    text: &str,
    old_string: &str,
    new_string: &str,
) -> CToolResult<String> {
    if old_string.is_empty() {
        return Err(CToolError::InvalidInput(
            "old_string must not be empty".to_string(),
        ));
    }

    let count = text.matches(old_string).count();
    if count == 0 {
        return Err(CToolError::InvalidInput(
            "old_string was not found".to_string(),
        ));
    }

    if count > 1 {
        return Err(CToolError::InvalidInput(format!(
            "old_string matched {count} times; edit_replace requires exactly one match"
        )));
    }

    Ok(text.replacen(old_string, new_string, 1))
}
