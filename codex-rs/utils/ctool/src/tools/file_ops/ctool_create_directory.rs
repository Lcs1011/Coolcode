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

pub const CTOOL_CREATE_DIRECTORY_TOOL_NAME: &str = "ctool_create_directory";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolCreateDirectoryInput {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolCreateDirectoryOutput {
    pub path: String,
    pub created: bool,
}

pub struct CToolCreateDirectory;

impl CTool for CToolCreateDirectory {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_CREATE_DIRECTORY_TOOL_NAME,
            description: "Create one directory inside CToolScope. Parent directory must already exist.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolCreateDirectoryInput = serde_json::from_value(input)?;
        let output = create_directory(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn create_directory(
    ctx: &CToolContext,
    input: CToolCreateDirectoryInput,
) -> CToolResult<CToolCreateDirectoryOutput> {
    gate::ensure_create_allowed(ctx, &input.path)?;

    if input.path.exists() {
        return Err(CToolError::InvalidInput(format!(
            "target already exists: {}",
            input.path.display()
        )));
    }

    std::fs::create_dir(&input.path)?;

    Ok(CToolCreateDirectoryOutput {
        path: input.path.display().to_string(),
        created: true,
    })
}
