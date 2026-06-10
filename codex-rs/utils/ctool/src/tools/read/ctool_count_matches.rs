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

pub const CTOOL_COUNT_MATCHES_TOOL_NAME: &str = "ctool_count_matches";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolCountMatchesInput {
    pub path: PathBuf,
    pub query: String,
    #[serde(default)]
    pub is_regex: bool,
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default)]
    pub include_hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolCountMatchesOutput {
    pub root: String,
    pub query: String,
    pub is_regex: bool,
    pub file_count: usize,
    pub matching_file_count: usize,
    pub line_match_count: usize,
}

pub struct CToolCountMatches;

impl CTool for CToolCountMatches {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_COUNT_MATCHES_TOOL_NAME,
            description: "Count matching UTF-8 text lines inside CToolScopeBase without returning every line.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolCountMatchesInput = serde_json::from_value(input)?;
        let output = count_matches(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn count_matches(
    ctx: &CToolContext,
    input: CToolCountMatchesInput,
) -> CToolResult<CToolCountMatchesOutput> {
    if input.query.is_empty() {
        return Err(CToolError::InvalidInput(
            "query must not be empty".to_string(),
        ));
    }
    search_support::validate_depth(input.max_depth)?;

    let matcher = Matcher::new(&input.query, input.is_regex, input.case_sensitive)?;
    let root = search_support::resolve_search_root(ctx, &input.path)?;
    let mut file_count = 0;
    let mut matching_file_count = 0;
    let mut line_match_count = 0;

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
            file_count += 1;
            let mut file_matched = false;

            for line in text.lines() {
                if matcher.is_match(line) {
                    line_match_count += 1;
                    file_matched = true;
                }
            }

            if file_matched {
                matching_file_count += 1;
            }

            Ok(true)
        },
    )?;

    Ok(CToolCountMatchesOutput {
        root: root.display().to_string(),
        query: input.query,
        is_regex: input.is_regex,
        file_count,
        matching_file_count,
        line_match_count,
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
