use std::path::Path;
use std::path::PathBuf;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::scope::CToolBaseScope;

pub fn ensure_read_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    ensure_existing_path_allowed(ctx, path, "read")
}

pub fn ensure_write_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    ensure_existing_path_allowed(ctx, path, "write")
}

pub fn ensure_search_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    ensure_existing_path_allowed(ctx, path, "search")
}

pub fn ensure_create_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    ensure_new_path_parent_allowed(ctx, path, "create")
}

fn ensure_existing_path_allowed(
    ctx: &CToolContext,
    path: &Path,
    operation: &'static str,
) -> CToolResult<()> {
    match ctx.scope {
        CToolBaseScope::None => Err(CToolError::ScopeDenied {
            scope: ctx.scope,
            operation,
        }),
        CToolBaseScope::Workspace => {
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
        CToolBaseScope::SelectedOnly | CToolBaseScope::TheEyeofProvidence => {
            Err(CToolError::UnsupportedScope {
                scope: ctx.scope,
                operation,
            })
        }
    }
}

fn ensure_new_path_parent_allowed(
    ctx: &CToolContext,
    path: &Path,
    operation: &'static str,
) -> CToolResult<()> {
    let Some(parent) = path.parent() else {
        return Err(CToolError::InvalidInput(format!(
            "path has no parent directory: {}",
            path.display()
        )));
    };

    ensure_existing_path_allowed(ctx, parent, operation)
}

fn canonicalize_existing_path(path: &Path, operation: &'static str) -> CToolResult<PathBuf> {
    std::fs::canonicalize(path).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to canonicalize path for {operation}: {} ({error})",
            path.display()
        ))
    })
}
