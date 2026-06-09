use std::path::Path;
use std::path::PathBuf;

use crate::context::CToolContext;
use crate::error::CToolResult;
use crate::scope_context::ensure_create_allowed_by_scope;
use crate::scope_context::ensure_delete_allowed_by_scope;
use crate::scope_context::ensure_move_allowed_by_scope;
use crate::scope_context::ensure_read_allowed_by_scope;
use crate::scope_context::ensure_search_allowed_by_scope;
use crate::scope_context::ensure_write_allowed_by_scope;

pub fn ensure_read_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<PathBuf> {
    ensure_read_allowed_by_scope(&ctx.scope_context, path)
}

pub fn ensure_search_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<PathBuf> {
    ensure_search_allowed_by_scope(&ctx.scope_context, path)
}

pub fn ensure_write_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<PathBuf> {
    ensure_write_allowed_by_scope(&ctx.scope_context, path)
}

pub fn ensure_create_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<PathBuf> {
    ensure_create_allowed_by_scope(&ctx.scope_context, path)
}

pub fn ensure_delete_allowed(ctx: &CToolContext, path: &Path) -> CToolResult<PathBuf> {
    ensure_delete_allowed_by_scope(&ctx.scope_context, path)
}

pub fn ensure_move_allowed(
    ctx: &CToolContext,
    from: &Path,
    to: &Path,
) -> CToolResult<(PathBuf, PathBuf)> {
    ensure_move_allowed_by_scope(&ctx.scope_context, from, to)
}
