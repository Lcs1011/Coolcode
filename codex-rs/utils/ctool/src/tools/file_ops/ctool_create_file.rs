use std::path::Path;
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
            description: "Create a safe UTF-8 text/source file inside CToolScope.",
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
    gate::ensure_create_allowed(ctx, &input.path)?;
    ensure_safe_create_extension(&input.path)?;

    let byte_len = input.content.len();
    if byte_len > MAX_CREATE_FILE_BYTES {
        return Err(CToolError::InvalidInput(format!(
            "content is too large; max bytes: {MAX_CREATE_FILE_BYTES}"
        )));
    }

    let existed = input.path.exists();

    if existed && !input.overwrite {
        return Err(CToolError::InvalidInput(format!(
            "file already exists: {}",
            input.path.display()
        )));
    }

    if existed {
        gate::ensure_write_allowed(ctx, &input.path)?;
        let metadata = std::fs::metadata(&input.path)?;
        if !metadata.is_file() {
            return Err(CToolError::InvalidInput(format!(
                "target exists but is not a file: {}",
                input.path.display()
            )));
        }
    }

    std::fs::write(&input.path, input.content)?;

    Ok(CToolCreateFileOutput {
        path: input.path.display().to_string(),
        byte_len,
        overwritten: existed,
    })
}

fn ensure_safe_create_extension(path: &Path) -> CToolResult<()> {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(CToolError::InvalidInput(format!(
            "invalid file name: {}",
            path.display()
        )));
    };

    if file_name == ".gitignore" {
        return Ok(());
    }

    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return Err(CToolError::InvalidInput(format!(
            "file extension is required for create_file: {}",
            path.display()
        )));
    };

    let extension = extension.to_ascii_lowercase();
    let allowed = matches!(
        extension.as_str(),
        "rs" | "toml"
            | "md"
            | "txt"
            | "json"
            | "jsonl"
            | "yaml"
            | "yml"
            | "css"
            | "html"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "java"
            | "go"
            | "py"
            | "lua"
            | "ini"
            | "cfg"
    );

    if allowed {
        Ok(())
    } else {
        Err(CToolError::InvalidInput(format!(
            "create_file does not allow this extension: .{extension}"
        )))
    }
}
