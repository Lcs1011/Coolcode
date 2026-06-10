use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::gate;

pub(crate) const MAX_TEXT_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub(crate) const MAX_SEARCH_DEPTH: usize = 8;
pub(crate) const MAX_SEARCH_RESULTS: usize = 500;
pub(crate) const MAX_LINE_CHARS: usize = 1000;

pub(crate) fn validate_depth(max_depth: usize) -> CToolResult<()> {
    if max_depth > MAX_SEARCH_DEPTH {
        return Err(CToolError::InvalidInput(format!(
            "max_depth cannot exceed {MAX_SEARCH_DEPTH}"
        )));
    }

    Ok(())
}

pub(crate) fn validate_result_limit(max_results: usize) -> CToolResult<()> {
    if max_results == 0 || max_results > MAX_SEARCH_RESULTS {
        return Err(CToolError::InvalidInput(format!(
            "max_results must be between 1 and {MAX_SEARCH_RESULTS}"
        )));
    }

    Ok(())
}

pub(crate) fn read_utf8_text_file(path: &Path) -> CToolResult<Option<String>> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_TEXT_FILE_BYTES {
        return Ok(None);
    }

    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn truncate_line(line: &str) -> String {
    let mut output = String::new();
    for ch in line.chars().take(MAX_LINE_CHARS) {
        output.push(ch);
    }
    if line.chars().count() > MAX_LINE_CHARS {
        output.push_str("...");
    }
    output
}

pub(crate) fn relative_display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(crate) fn visit_readable_text_files<F>(
    ctx: &CToolContext,
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    include_hidden: bool,
    visitor: &mut F,
) -> CToolResult<bool>
where
    F: FnMut(&Path) -> CToolResult<bool>,
{
    gate::ensure_search_allowed(ctx, current)?;

    let metadata = fs::metadata(current)?;
    if metadata.is_file() {
        let path = gate::ensure_read_allowed(ctx, current)?;
        return visitor(&path);
    }

    if !metadata.is_dir() || depth >= max_depth {
        return Ok(true);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(current)? {
        entries.push(entry?);
    }
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let file_name = entry.file_name();
        if !include_hidden && file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let path = match gate::ensure_search_allowed(ctx, &entry.path()) {
            Ok(path) => path,
            Err(CToolError::OutOfScope { .. }) => continue,
            Err(error) => return Err(error),
        };

        if !visit_readable_text_files(
            ctx,
            root,
            &path,
            depth + 1,
            max_depth,
            include_hidden,
            visitor,
        )? {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) fn resolve_search_root(ctx: &CToolContext, path: &Path) -> CToolResult<PathBuf> {
    gate::ensure_search_allowed(ctx, path)
}
