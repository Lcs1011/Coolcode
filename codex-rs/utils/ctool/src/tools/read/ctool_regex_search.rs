use std::path::PathBuf;

use regex::Regex;
use regex::RegexBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::tool::CTool;
use crate::tool::CToolSpec;
use crate::tools::read::search_support;

pub const CTOOL_REGEX_SEARCH_TOOL_NAME: &str = "ctool_regex_search";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolRegexSearchInput {
    pub path: PathBuf,
    pub pattern: String,
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
pub struct CToolRegexSearchMatch {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolRegexSearchOutput {
    pub root: String,
    pub pattern: String,
    pub total_returned: usize,
    pub truncated: bool,
    pub matches: Vec<CToolRegexSearchMatch>,
}

pub struct CToolRegexSearch;

impl CTool for CToolRegexSearch {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_REGEX_SEARCH_TOOL_NAME,
            description: "Search UTF-8 text files with a Rust regex inside CToolScopeBase.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolRegexSearchInput = serde_json::from_value(input)?;
        let output = regex_search(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn regex_search(
    ctx: &CToolContext,
    input: CToolRegexSearchInput,
) -> CToolResult<CToolRegexSearchOutput> {
    if input.pattern.is_empty() {
        return Err(CToolError::InvalidInput(
            "pattern must not be empty".to_string(),
        ));
    }
    search_support::validate_depth(input.max_depth)?;
    search_support::validate_result_limit(input.max_results)?;

    let regex = build_regex(&input.pattern, input.case_sensitive)?;
    let root = search_support::resolve_search_root(ctx, &input.path)?;
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
            for (index, line) in text.lines().enumerate() {
                if matches.len() >= input.max_results {
                    truncated = true;
                    return Ok(false);
                }

                if regex.is_match(line) {
                    matches.push(CToolRegexSearchMatch {
                        path: search_support::relative_display_path(&root, file_path),
                        line_number: index + 1,
                        line: search_support::truncate_line(line),
                    });
                }
            }

            Ok(true)
        },
    )?;

    Ok(CToolRegexSearchOutput {
        root: root.display().to_string(),
        pattern: input.pattern,
        total_returned: matches.len(),
        truncated,
        matches,
    })
}

fn build_regex(pattern: &str, case_sensitive: bool) -> CToolResult<Regex> {
    RegexBuilder::new(pattern)
        .case_insensitive(!case_sensitive)
        .build()
        .map_err(|error| CToolError::InvalidInput(format!("invalid regex pattern: {error}")))
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
