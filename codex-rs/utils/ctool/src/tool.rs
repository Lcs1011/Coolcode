use serde::Serialize;
use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolResult;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolSpec {
    pub name: &'static str,
    pub description: &'static str,
}

pub trait CTool {
    fn spec(&self) -> CToolSpec;

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value>;
}
