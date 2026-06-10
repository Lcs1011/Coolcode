use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::gate;
use crate::scope_context::can_write_path;
use crate::scope_context::is_protected_path;
use crate::tool::CTool;
use crate::tool::CToolSpec;

pub const CTOOL_ANNOTATE_MARKDOWN_TOOL_NAME: &str = "ctool_annotate_markdown";

const NORMAL_MARK_START: &str = "<mark style=\"background:#d3f8b6\">";
const IMPORTANT_MARK_START: &str = "<mark style=\"background:#ff4d4f\">";
const MARK_END: &str = "</mark>";
const UP_ARROW: &str = "↑";
const DOWN_ARROW: &str = "↓";
const PREVIEW_CONTEXT_LINES: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolAnnotateMarkdownInput {
    pub path: PathBuf,
    pub target_text: String,
    #[serde(default = "default_annotation_kind")]
    pub annotation_kind: CToolMarkdownAnnotationKind,
    #[serde(default)]
    pub annotation_direction: Option<CToolMarkdownAnnotationDirection>,
    #[serde(default)]
    pub occurrence: Option<usize>,
    #[serde(default = "default_allow_readonly")]
    pub allow_readonly: bool,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CToolMarkdownAnnotationKind {
    Normal,
    Important,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CToolMarkdownAnnotationDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolAnnotateMarkdownOutput {
    pub success: bool,
    pub dry_run: bool,
    pub readonly_exception_used: bool,
    pub annotation_kind: CToolMarkdownAnnotationKind,
    pub annotation_direction: Option<CToolMarkdownAnnotationDirection>,
    pub path: String,
    pub line_number: usize,
    pub occurrence: usize,
    pub before_preview: String,
    pub after_preview: String,
    pub note: String,
}

pub struct CToolAnnotateMarkdown;

impl CTool for CToolAnnotateMarkdown {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_ANNOTATE_MARKDOWN_TOOL_NAME,
            description: "Add a safe <mark> annotation to a Markdown file inside CToolScopeBase.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolAnnotateMarkdownInput = serde_json::from_value(input)?;
        let output = annotate_markdown(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}
#[cfg(test)]
#[path = "ctool_annotate_markdown_tests.rs"]
mod tests;


pub fn annotate_markdown(
    ctx: &CToolContext,
    input: CToolAnnotateMarkdownInput,
) -> CToolResult<CToolAnnotateMarkdownOutput> {
    if input.target_text.is_empty() {
        return Err(CToolError::InvalidInput(
            "target_text must not be empty".to_string(),
        ));
    }
    if input.occurrence == Some(0) {
        return Err(CToolError::InvalidInput(
            "occurrence must be greater than 0".to_string(),
        ));
    }

    let path = gate::ensure_read_allowed(ctx, &input.path)?;
    ensure_markdown_file(&path)?;

    let can_write = can_write_path(&ctx.scope_context, &path);
    let readonly_exception_used = !can_write && is_protected_path(&ctx.scope_context, &path);
    if !can_write && !(readonly_exception_used && input.allow_readonly) {
        return Err(CToolError::OutOfScope {
            path: path.display().to_string(),
            operation: "markdown annotation write",
        });
    }

    let text = std::fs::read_to_string(&path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "file is not valid UTF-8 Markdown: {} ({error})",
            path.display()
        ))
    })?;

    let protected_ranges = markdown_protected_ranges(&text)?;
    let matches = find_target_matches(&text, &input.target_text);
    if matches.is_empty() {
        return Err(CToolError::InvalidInput(
            "target_text was not found".to_string(),
        ));
    }

    let selected_occurrence = match input.occurrence {
        Some(occurrence) => occurrence,
        None if matches.len() == 1 => 1,
        None => {
            return Err(CToolError::InvalidInput(format!(
                "target_text matched {} times; specify occurrence",
                matches.len()
            )));
        }
    };

    let Some(&(start, end)) = matches.get(selected_occurrence - 1) else {
        return Err(CToolError::InvalidInput(format!(
            "occurrence {selected_occurrence} is greater than match count {}",
            matches.len()
        )));
    };

    ensure_not_in_protected_range(start, end, &protected_ranges)?;
    ensure_not_already_marked(&text, start, end)?;

    let mark_start = match input.annotation_kind {
        CToolMarkdownAnnotationKind::Normal => NORMAL_MARK_START,
        CToolMarkdownAnnotationKind::Important => IMPORTANT_MARK_START,
    };

    let direction_prefix = input
        .annotation_direction
        .map(annotation_direction_prefix)
        .unwrap_or_default();
    let mut annotated = String::with_capacity(
        text.len() + mark_start.len() + direction_prefix.len() + MARK_END.len(),
    );
    annotated.push_str(&text[..start]);
    annotated.push_str(mark_start);
    annotated.push_str(direction_prefix);
    annotated.push_str(&text[start..end]);
    annotated.push_str(MARK_END);
    annotated.push_str(&text[end..]);

    if !input.dry_run {
        std::fs::write(&path, &annotated)?;
    }

    let line_number = line_number_at_byte(&text, start);
    let before_preview = preview_around_line(&text, line_number, PREVIEW_CONTEXT_LINES);
    let after_preview = preview_around_line(&annotated, line_number, PREVIEW_CONTEXT_LINES);
    let note = match (readonly_exception_used, input.dry_run) {
        (true, true) => "Dry run only; ReadOnly scope exception would be used.".to_string(),
        (true, false) => {
            "ReadOnly scope exception used: only Markdown <mark> annotation was applied.".to_string()
        }
        (false, true) => "Dry run only; file was not modified.".to_string(),
        (false, false) => "Markdown annotation applied.".to_string(),
    };

    Ok(CToolAnnotateMarkdownOutput {
        success: true,
        dry_run: input.dry_run,
        readonly_exception_used,
        annotation_kind: input.annotation_kind,
        annotation_direction: input.annotation_direction,
        path: path.display().to_string(),
        line_number,
        occurrence: selected_occurrence,
        before_preview,
        after_preview,
        note,
    })
}

fn default_annotation_kind() -> CToolMarkdownAnnotationKind {
    CToolMarkdownAnnotationKind::Normal
}

fn default_allow_readonly() -> bool {
    true
}

fn annotation_direction_prefix(direction: CToolMarkdownAnnotationDirection) -> &'static str {
    match direction {
        CToolMarkdownAnnotationDirection::Up => UP_ARROW,
        CToolMarkdownAnnotationDirection::Down => DOWN_ARROW,
    }
}

fn ensure_markdown_file(path: &Path) -> CToolResult<()> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if extension == "md" || extension == "markdown" {
        Ok(())
    } else {
        Err(CToolError::InvalidInput(format!(
            "ctool_annotate_markdown only supports .md/.markdown files: {}",
            path.display()
        )))
    }
}

fn find_target_matches(text: &str, target: &str) -> Vec<(usize, usize)> {
    text.match_indices(target)
        .map(|(start, matched)| (start, start + matched.len()))
        .collect()
}

fn ensure_not_in_protected_range(
    start: usize,
    end: usize,
    protected_ranges: &[(usize, usize)],
) -> CToolResult<()> {
    if protected_ranges
        .iter()
        .any(|&(protected_start, protected_end)| start < protected_end && end > protected_start)
    {
        return Err(CToolError::InvalidInput(
            "target_text overlaps Markdown protected content".to_string(),
        ));
    }

    Ok(())
}

fn ensure_not_already_marked(text: &str, start: usize, end: usize) -> CToolResult<()> {
    let before = &text[..start];
    let after = &text[end..];
    let last_mark_start = before.rfind("<mark");
    let last_mark_end = before.rfind(MARK_END);

    let inside_open_mark = match (last_mark_start, last_mark_end) {
        (Some(mark_start), Some(mark_end)) => mark_start > mark_end,
        (Some(_), None) => true,
        (None, _) => false,
    };

    if inside_open_mark && after.contains(MARK_END) {
        return Err(CToolError::InvalidInput(
            "target_text is already inside a <mark> annotation".to_string(),
        ));
    }

    if before.ends_with(NORMAL_MARK_START)
        || before.ends_with(IMPORTANT_MARK_START)
        || after.starts_with(MARK_END)
    {
        return Err(CToolError::InvalidInput(
            "target_text is already wrapped by a <mark> annotation".to_string(),
        ));
    }

    Ok(())
}

fn markdown_protected_ranges(text: &str) -> CToolResult<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    add_yaml_front_matter_range(text, &mut ranges);
    add_fenced_code_ranges(text, &mut ranges)?;
    add_html_comment_ranges(text, &mut ranges)?;
    add_inline_code_ranges(text, &mut ranges);
    Ok(ranges)
}

fn add_yaml_front_matter_range(text: &str, ranges: &mut Vec<(usize, usize)>) {
    let mut offset = 0;
    let mut lines = text.split_inclusive('\n');
    let Some(first_line) = lines.next() else {
        return;
    };
    if first_line.trim_end() != "---" {
        return;
    }
    offset += first_line.len();

    for line in lines {
        let line_start = offset;
        offset += line.len();
        if line.trim_end() == "---" {
            ranges.push((0, line_start + line.len()));
            return;
        }
    }
}

fn add_fenced_code_ranges(text: &str, ranges: &mut Vec<(usize, usize)>) -> CToolResult<()> {
    let mut in_fence: Option<(&str, usize)> = None;
    let mut offset = 0;

    for line in text.split_inclusive('\n') {
        let line_start = offset;
        let line_end = line_start + line.len();
        let trimmed = line.trim_start();

        if let Some((marker, fence_start)) = in_fence {
            if trimmed.starts_with(marker) {
                ranges.push((fence_start, line_end));
                in_fence = None;
            }
        } else if trimmed.starts_with("```") {
            in_fence = Some(("```", line_start));
        } else if trimmed.starts_with("~~~") {
            in_fence = Some(("~~~", line_start));
        }

        offset = line_end;
    }

    if in_fence.is_some() {
        return Err(CToolError::InvalidInput(
            "unclosed Markdown fenced code block; refusing to annotate".to_string(),
        ));
    }

    Ok(())
}

fn add_html_comment_ranges(text: &str, ranges: &mut Vec<(usize, usize)>) -> CToolResult<()> {
    let mut search_from = 0;
    while let Some(relative_start) = text[search_from..].find("<!--") {
        let start = search_from + relative_start;
        let Some(relative_end) = text[start + 4..].find("-->") else {
            return Err(CToolError::InvalidInput(
                "unclosed HTML comment; refusing to annotate".to_string(),
            ));
        };
        let end = start + 4 + relative_end + 3;
        ranges.push((start, end));
        search_from = end;
    }

    Ok(())
}

fn add_inline_code_ranges(text: &str, ranges: &mut Vec<(usize, usize)>) {
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let line_without_newline = line.trim_end_matches(|ch| ch == '\r' || ch == '\n');
        let mut local = 0;
        while let Some(relative_start) = line_without_newline[local..].find('`') {
            let start = local + relative_start;
            let tick_count = count_backticks(&line_without_newline[start..]);
            let search_start = start + tick_count;
            let closing = "`".repeat(tick_count);
            let Some(relative_end) = line_without_newline[search_start..].find(&closing) else {
                break;
            };
            let end = search_start + relative_end + tick_count;
            ranges.push((offset + start, offset + end));
            local = end;
        }
        offset += line.len();
    }
}

fn count_backticks(text: &str) -> usize {
    text.chars().take_while(|ch| *ch == '`').count()
}

fn line_number_at_byte(text: &str, byte_index: usize) -> usize {
    text[..byte_index].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn preview_around_line(text: &str, line_number: usize, context: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = line_number.saturating_sub(context + 1);
    let end = (line_number + context).min(lines.len());
    let mut output = String::new();

    for index in start..end {
        output.push_str(&format!("{}: {}\n", index + 1, lines[index]));
    }

    output
}
