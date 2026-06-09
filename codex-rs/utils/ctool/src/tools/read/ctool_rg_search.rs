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

pub const CTOOL_RG_SEARCH_TOOL_NAME: &str = "ctool_rg_search";

const MAX_RG_SEARCH_DEPTH: usize = 8;
const MAX_RG_SEARCH_RESULTS: usize = 500;
const MAX_RG_SEARCH_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_LINE_PREVIEW_CHARS: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolRgSearchInput {
    pub path: PathBuf,
    pub query: String,
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default)]
    pub include_hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolRgSearchMatch {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolRgSearchOutput {
    pub root: String,
    pub query: String,
    pub total_returned: usize,
    pub truncated: bool,
    pub matches: Vec<CToolRgSearchMatch>,
}

pub struct CToolRgSearch;

impl CTool for CToolRgSearch {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_RG_SEARCH_TOOL_NAME,
            description: "Search UTF-8 text files inside CToolBaseScope using a literal query.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolRgSearchInput = serde_json::from_value(input)?;
        let output = rg_search(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn rg_search(
    ctx: &CToolContext,
    input: CToolRgSearchInput,
) -> CToolResult<CToolRgSearchOutput> {
    if input.query.is_empty() {
        return Err(CToolError::InvalidInput(
            "query must not be empty".to_string(),
        ));
    }

    if input.max_depth > MAX_RG_SEARCH_DEPTH {
        return Err(CToolError::InvalidInput(format!(
            "max_depth cannot exceed {MAX_RG_SEARCH_DEPTH}"
        )));
    }

    if input.max_results == 0 || input.max_results > MAX_RG_SEARCH_RESULTS {
        return Err(CToolError::InvalidInput(format!(
            "max_results must be between 1 and {MAX_RG_SEARCH_RESULTS}"
        )));
    }

    let root = gate::ensure_search_allowed(ctx, &input.path)?;
    let needle = if input.case_sensitive {
        input.query.clone()
    } else {
        input.query.to_lowercase()
    };

    let mut matches = Vec::new();
    let mut truncated = false;

    search_path(
        ctx,
        &root,
        &root,
        &input.query,
        &needle,
        input.case_sensitive,
        0,
        input.max_depth,
        input.max_results,
        input.include_hidden,
        &mut matches,
        &mut truncated,
    )?;

    Ok(CToolRgSearchOutput {
        root: root.display().to_string(),
        query: input.query,
        total_returned: matches.len(),
        truncated,
        matches,
    })
}

#[allow(clippy::too_many_arguments)]
fn search_path(
    ctx: &CToolContext,
    root: &Path,
    current: &Path,
    original_query: &str,
    needle: &str,
    case_sensitive: bool,
    depth: usize,
    max_depth: usize,
    max_results: usize,
    include_hidden: bool,
    matches: &mut Vec<CToolRgSearchMatch>,
    truncated: &mut bool,
) -> CToolResult<()> {
    if matches.len() >= max_results {
        *truncated = true;
        return Ok(());
    }

    gate::ensure_search_allowed(ctx, current)?;

    let metadata = fs::metadata(current)?;
    if metadata.is_file() {
        search_file(
            ctx,
            root,
            current,
            original_query,
            needle,
            case_sensitive,
            max_results,
            matches,
            truncated,
        )?;
        return Ok(());
    }

    if !metadata.is_dir() {
        return Ok(());
    }

    if depth >= max_depth {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(current)? {
        entries.push(entry?);
    }
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        if matches.len() >= max_results {
            *truncated = true;
            return Ok(());
        }

        let file_name = entry.file_name();
        if !include_hidden && file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let path = entry.path();
        search_path(
            ctx,
            root,
            &path,
            original_query,
            needle,
            case_sensitive,
            depth + 1,
            max_depth,
            max_results,
            include_hidden,
            matches,
            truncated,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn search_file(
    ctx: &CToolContext,
    root: &Path,
    file_path: &Path,
    _original_query: &str,
    needle: &str,
    case_sensitive: bool,
    max_results: usize,
    matches: &mut Vec<CToolRgSearchMatch>,
    truncated: &mut bool,
) -> CToolResult<()> {
    let file_path = gate::ensure_read_allowed(ctx, file_path)?;

    let metadata = fs::metadata(&file_path)?;
    if metadata.len() > MAX_RG_SEARCH_FILE_BYTES {
        return Ok(());
    }

    let Ok(text) = fs::read_to_string(&file_path) else {
        return Ok(());
    };

    for (index, line) in text.lines().enumerate() {
        if matches.len() >= max_results {
            *truncated = true;
            return Ok(());
        }

        let haystack = if case_sensitive {
            line.to_string()
        } else {
            line.to_lowercase()
        };

        if haystack.contains(needle) {
            let preview = truncate_line(line, MAX_LINE_PREVIEW_CHARS);
            let display_path = file_path
                .strip_prefix(root)
                .unwrap_or(file_path.as_path())
                .display()
                .to_string();

            matches.push(CToolRgSearchMatch {
                path: display_path,
                line_number: index + 1,
                line: preview,
            });
        }
    }

    Ok(())
}

fn truncate_line(line: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for ch in line.chars().take(max_chars) {
        output.push(ch);
    }
    if line.chars().count() > max_chars {
        output.push_str("...");
    }
    output
}

fn default_case_sensitive() -> bool {
    false
}

fn default_max_depth() -> usize {
    6
}

fn default_max_results() -> usize {
    100
}
