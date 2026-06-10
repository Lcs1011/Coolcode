use std::collections::BTreeSet;
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

pub const CTOOL_EXTRACT_LINES_MATCHING_TOOL_NAME: &str = "ctool_extract_lines_matching";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolExtractLinesMatchingInput {
    pub path: PathBuf,
    pub query: String,
    #[serde(default)]
    pub is_regex: bool,
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub sort: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default)]
    pub include_hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolExtractedLine {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolExtractLinesMatchingOutput {
    pub root: String,
    pub query: String,
    pub is_regex: bool,
    pub total_returned: usize,
    pub truncated: bool,
    pub lines: Vec<CToolExtractedLine>,
}

pub struct CToolExtractLinesMatching;

impl CTool for CToolExtractLinesMatching {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_EXTRACT_LINES_MATCHING_TOOL_NAME,
            description: "Extract matching UTF-8 text lines inside CToolScopeBase, with optional unique/sort cleanup.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolExtractLinesMatchingInput = serde_json::from_value(input)?;
        let output = extract_lines_matching(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn extract_lines_matching(
    ctx: &CToolContext,
    input: CToolExtractLinesMatchingInput,
) -> CToolResult<CToolExtractLinesMatchingOutput> {
    if input.query.is_empty() {
        return Err(CToolError::InvalidInput(
            "query must not be empty".to_string(),
        ));
    }
    search_support::validate_depth(input.max_depth)?;
    search_support::validate_result_limit(input.max_results)?;

    let matcher = Matcher::new(&input.query, input.is_regex, input.case_sensitive)?;
    let root = search_support::resolve_search_root(ctx, &input.path)?;
    let mut lines = Vec::new();
    let mut seen = BTreeSet::new();
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
            let display_path = search_support::relative_display_path(&root, file_path);

            for (index, line) in text.lines().enumerate() {
                if !matcher.is_match(line) {
                    continue;
                }

                let line = search_support::truncate_line(line);
                if input.unique && !seen.insert(line.clone()) {
                    continue;
                }

                if lines.len() >= input.max_results {
                    truncated = true;
                    return Ok(false);
                }

                lines.push(CToolExtractedLine {
                    path: display_path.clone(),
                    line_number: index + 1,
                    line,
                });
            }

            Ok(true)
        },
    )?;

    if input.sort {
        lines.sort_by(|left, right| {
            left.line
                .cmp(&right.line)
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.line_number.cmp(&right.line_number))
        });
    }

    Ok(CToolExtractLinesMatchingOutput {
        root: root.display().to_string(),
        query: input.query,
        is_regex: input.is_regex,
        total_returned: lines.len(),
        truncated,
        lines,
    })
}

struct Matcher {
    literal: Option<String>,
    regex: Option<Regex>,
    case_sensitive: bool,
}

impl Matcher {
    fn new(query: &str, is_regex: bool, case_sensitive: bool) -> CToolResult<Self> {
        if is_regex {
            let regex = RegexBuilder::new(query)
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|error| CToolError::InvalidInput(format!("invalid regex pattern: {error}")))?;
            Ok(Self {
                literal: None,
                regex: Some(regex),
                case_sensitive,
            })
        } else {
            let literal = if case_sensitive {
                query.to_string()
            } else {
                query.to_lowercase()
            };
            Ok(Self {
                literal: Some(literal),
                regex: None,
                case_sensitive,
            })
        }
    }

    fn is_match(&self, line: &str) -> bool {
        if let Some(regex) = &self.regex {
            return regex.is_match(line);
        }

        let Some(literal) = &self.literal else {
            return false;
        };
        if self.case_sensitive {
            line.contains(literal)
        } else {
            line.to_lowercase().contains(literal)
        }
    }
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
