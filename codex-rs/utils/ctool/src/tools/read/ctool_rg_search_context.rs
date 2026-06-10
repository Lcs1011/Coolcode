use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::tool::CTool;
use crate::tool::CToolSpec;
use crate::tools::read::search_support;

pub const CTOOL_RG_SEARCH_CONTEXT_TOOL_NAME: &str = "ctool_rg_search_context";

const MAX_CONTEXT_LINES: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolRgSearchContextInput {
    pub path: PathBuf,
    pub query: String,
    #[serde(default = "default_before")]
    pub before: usize,
    #[serde(default = "default_after")]
    pub after: usize,
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
pub struct CToolRgSearchContextLine {
    pub line_number: usize,
    pub line: String,
    pub is_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolRgSearchContextMatch {
    pub path: String,
    pub line_number: usize,
    pub lines: Vec<CToolRgSearchContextLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolRgSearchContextOutput {
    pub root: String,
    pub query: String,
    pub total_returned: usize,
    pub truncated: bool,
    pub matches: Vec<CToolRgSearchContextMatch>,
}

pub struct CToolRgSearchContext;

impl CTool for CToolRgSearchContext {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_RG_SEARCH_CONTEXT_TOOL_NAME,
            description: "Search literal text and return surrounding UTF-8 line context inside CToolScopeBase.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolRgSearchContextInput = serde_json::from_value(input)?;
        let output = rg_search_context(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn rg_search_context(
    ctx: &CToolContext,
    input: CToolRgSearchContextInput,
) -> CToolResult<CToolRgSearchContextOutput> {
    if input.query.is_empty() {
        return Err(CToolError::InvalidInput(
            "query must not be empty".to_string(),
        ));
    }
    if input.before > MAX_CONTEXT_LINES || input.after > MAX_CONTEXT_LINES {
        return Err(CToolError::InvalidInput(format!(
            "before and after cannot exceed {MAX_CONTEXT_LINES}"
        )));
    }
    search_support::validate_depth(input.max_depth)?;
    search_support::validate_result_limit(input.max_results)?;

    let root = search_support::resolve_search_root(ctx, &input.path)?;
    let needle = if input.case_sensitive {
        input.query.clone()
    } else {
        input.query.to_lowercase()
    };
    let mut matches = Vec::new();
    let mut truncated = false;

    search_support::visit_readable_text_files(
        ctx,
        &root,
        &root,
        0,
        input.max_depth,
        input.include_hidden,
        &mut |file_path| {
            let Some(text) = search_support::read_utf8_text_file(file_path)? else {
                return Ok(true);
            };
            let lines: Vec<&str> = text.lines().collect();
            for (index, line) in lines.iter().enumerate() {
                if matches.len() >= input.max_results {
                    truncated = true;
                    return Ok(false);
                }

                let haystack = if input.case_sensitive {
                    (*line).to_string()
                } else {
                    line.to_lowercase()
                };
                if !haystack.contains(&needle) {
                    continue;
                }

                let start = index.saturating_sub(input.before);
                let end = (index + input.after + 1).min(lines.len());
                let context_lines = lines[start..end]
                    .iter()
                    .enumerate()
                    .map(|(offset, context_line)| CToolRgSearchContextLine {
                        line_number: start + offset + 1,
                        line: search_support::truncate_line(context_line),
                        is_match: start + offset == index,
                    })
                    .collect();

                matches.push(CToolRgSearchContextMatch {
                    path: search_support::relative_display_path(&root, file_path),
                    line_number: index + 1,
                    lines: context_lines,
                });
            }

            Ok(true)
        },
    )?;

    Ok(CToolRgSearchContextOutput {
        root: root.display().to_string(),
        query: input.query,
        total_returned: matches.len(),
        truncated,
        matches,
    })
}

fn default_before() -> usize {
    2
}

fn default_after() -> usize {
    2
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
