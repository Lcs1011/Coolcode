use std::path::Path;

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
            if ctx
                .workspace_roots
                .iter()
                .any(|root| path.starts_with(root))
            {
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
