use std::fs;
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

pub const CTOOL_LIST_DIRECTORY_TOOL_NAME: &str = "ctool_list_directory";

const MAX_LIST_DIRECTORY_DEPTH: usize = 4;
const MAX_LIST_DIRECTORY_ENTRIES: usize = 1000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolListDirectoryInput {
    pub path: PathBuf,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default)]
    pub include_hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolListDirectoryItem {
    pub path: String,
    pub kind: String,
    pub byte_len: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolListDirectoryOutput {
    pub root: String,
    pub total_returned: usize,
    pub truncated: bool,
    pub items: Vec<CToolListDirectoryItem>,
}

pub struct CToolListDirectory;

impl CTool for CToolListDirectory {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_LIST_DIRECTORY_TOOL_NAME,
            description: "List directory entries inside CToolScopeBase.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolListDirectoryInput = serde_json::from_value(input)?;
        let output = list_directory(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn list_directory(
    ctx: &CToolContext,
    input: CToolListDirectoryInput,
) -> CToolResult<CToolListDirectoryOutput> {
    if input.max_depth > MAX_LIST_DIRECTORY_DEPTH {
        return Err(CToolError::InvalidInput(format!(
            "max_depth cannot exceed {MAX_LIST_DIRECTORY_DEPTH}"
        )));
    }

    if input.max_entries == 0 || input.max_entries > MAX_LIST_DIRECTORY_ENTRIES {
        return Err(CToolError::InvalidInput(format!(
            "max_entries must be between 1 and {MAX_LIST_DIRECTORY_ENTRIES}"
        )));
    }

    let root = gate::ensure_search_allowed(ctx, &input.path)?;
    let mut items = Vec::new();
    let mut truncated = false;

    collect_directory_items(
        ctx,
        &root,
        &root,
        0,
        input.max_depth,
        input.max_entries,
        input.include_hidden,
        &mut items,
        &mut truncated,
    )?;

    Ok(CToolListDirectoryOutput {
        root: root.display().to_string(),
        total_returned: items.len(),
        truncated,
        items,
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_directory_items(
    ctx: &CToolContext,
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_entries: usize,
    include_hidden: bool,
    items: &mut Vec<CToolListDirectoryItem>,
    truncated: &mut bool,
) -> CToolResult<()> {
    if items.len() >= max_entries {
        *truncated = true;
        return Ok(());
    }

    gate::ensure_search_allowed(ctx, current)?;

    let mut entries = Vec::new();
    for entry in fs::read_dir(current)? {
        entries.push(entry?);
    }
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        if items.len() >= max_entries {
            *truncated = true;
            return Ok(());
        }

        let file_name = entry.file_name();
        if !include_hidden && file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let path = match gate::ensure_read_allowed(ctx, &entry.path()) {
            Ok(path) => path,
            Err(CToolError::OutOfScope { .. }) => continue,
            Err(error) => return Err(error),
        };

        let file_type = entry.file_type()?;
        let kind = if file_type.is_dir() {
            "directory"
        } else if file_type.is_file() {
            "file"
        } else if file_type.is_symlink() {
            "symlink"
        } else {
            "other"
        };

        let byte_len = if file_type.is_file() {
            entry.metadata().ok().map(|metadata| metadata.len())
        } else {
            None
        };

        let display_path = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .display()
            .to_string();

        items.push(CToolListDirectoryItem {
            path: display_path,
            kind: kind.to_string(),
            byte_len,
        });

        if file_type.is_dir() && depth < max_depth {
            collect_directory_items(
                ctx,
                root,
                &path,
                depth + 1,
                max_depth,
                max_entries,
                include_hidden,
                items,
                truncated,
            )?;
        }
    }

    Ok(())
}

fn default_max_depth() -> usize {
    1
}

fn default_max_entries() -> usize {
    200
}
