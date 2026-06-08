use std::path::Path;
use std::path::PathBuf;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::scope::CToolScope;

pub fn ensure_read_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    ensure_path_allowed(ctx, path, "read")
}

pub fn ensure_write_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    ensure_path_allowed(ctx, path, "write")
}

pub fn ensure_search_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    ensure_path_allowed(ctx, path, "search")
}

fn ensure_path_allowed(
    ctx: &CToolContext,
    path: &Path,
    operation: &'static str,
) -> CToolResult<()> {
    match ctx.scope {
        CToolScope::None => Err(CToolError::ScopeDenied {
            scope: ctx.scope,
            operation,
        }),
        CToolScope::Workspace => {
            let path = canonicalize_existing_path(path, operation)?;

            let is_allowed = ctx
                .workspace_roots
                .iter()
                .filter_map(|root| canonicalize_existing_path(root, operation).ok())
                .any(|root| path.starts_with(root));

            if is_allowed {
                Ok(())
            } else {
                Err(CToolError::OutOfScope {
                    path: path.display().to_string(),
                    operation,
                })
            }
        }
        CToolScope::SelectedOnly | CToolScope::TheEyeofProvidence => {
            Err(CToolError::UnsupportedScope {
                scope: ctx.scope,
                operation,
            })
        }
    }
}

fn canonicalize_existing_path(path: &Path, operation: &'static str) -> CToolResult<PathBuf> {
    std::fs::canonicalize(path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to canonicalize path for {operation}: {} ({error})",
            path.display()
        ))
    })
}
