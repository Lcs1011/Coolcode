use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::tool::CTool;
use crate::tool::CToolSpec;
use crate::tools::read::CTOOL_LIST_DIRECTORY_TOOL_NAME;
use crate::tools::read::CTOOL_RG_SEARCH_TOOL_NAME;
use crate::tools::read::CToolListDirectory;
use crate::tools::read::CToolRgSearch;

pub fn available_specs() -> Vec<CToolSpec> {
    let list_directory = CToolListDirectory;
    let rg_search = CToolRgSearch;

    vec![list_directory.spec(), rg_search.spec()]
}

pub fn run_tool(name: &str, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
    match name {
        CTOOL_LIST_DIRECTORY_TOOL_NAME => {
            let tool = CToolListDirectory;
            tool.run_json(ctx, input)
        }
        CTOOL_RG_SEARCH_TOOL_NAME => {
            let tool = CToolRgSearch;
            tool.run_json(ctx, input)
        }
        _ => Err(CToolError::ToolNotFound {
            name: name.to_string(),
        }),
    }
}
