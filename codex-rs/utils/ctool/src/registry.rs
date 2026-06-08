use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::tool::CTool;
use crate::tool::CToolSpec;
use crate::tools::read::CTOOL_LIST_DIRECTORY_TOOL_NAME;
use crate::tools::read::CTOOL_READ_FILE_TOOL_NAME;
use crate::tools::read::CTOOL_RG_SEARCH_TOOL_NAME;
use crate::tools::read::CToolListDirectory;
use crate::tools::read::CToolReadFile;
use crate::tools::read::CToolRgSearch;

use crate::tools::edit::CTOOL_EDIT_BATCH_TOOL_NAME;
use crate::tools::edit::CTOOL_EDIT_INSERT_TOOL_NAME;
use crate::tools::edit::CTOOL_EDIT_REPLACE_TOOL_NAME;
use crate::tools::edit::CTOOL_PREVIEW_DIFF_TOOL_NAME;
use crate::tools::edit::CToolEditBatch;
use crate::tools::edit::CToolEditInsert;
use crate::tools::edit::CToolEditReplace;
use crate::tools::edit::CToolPreviewDiff;

use crate::tools::file_ops::CTOOL_CREATE_DIRECTORY_TOOL_NAME;
use crate::tools::file_ops::CTOOL_CREATE_FILE_TOOL_NAME;
use crate::tools::file_ops::CTOOL_DELETE_DIRECTORY_TOOL_NAME;
use crate::tools::file_ops::CTOOL_DELETE_FILE_TOOL_NAME;
use crate::tools::file_ops::CTOOL_MOVE_DIRECTORY_TOOL_NAME;
use crate::tools::file_ops::CTOOL_MOVE_FILE_TOOL_NAME;
use crate::tools::file_ops::CToolCreateDirectory;
use crate::tools::file_ops::CToolCreateFile;
use crate::tools::file_ops::CToolDeleteDirectory;
use crate::tools::file_ops::CToolDeleteFile;
use crate::tools::file_ops::CToolMoveDirectory;
use crate::tools::file_ops::CToolMoveFile;

pub fn available_specs() -> Vec<CToolSpec> {
    let list_directory = CToolListDirectory;
    let rg_search = CToolRgSearch;
    let read_file = CToolReadFile;

    let edit_replace = CToolEditReplace;
    let edit_insert = CToolEditInsert;
    let preview_diff = CToolPreviewDiff;
    let edit_batch = CToolEditBatch;

    let create_file = CToolCreateFile;
    let delete_file = CToolDeleteFile;
    let move_file = CToolMoveFile;

    let create_directory = CToolCreateDirectory;
    let delete_directory = CToolDeleteDirectory;
    let move_directory = CToolMoveDirectory;

    vec![
        list_directory.spec(),
        rg_search.spec(),
        read_file.spec(),
        edit_replace.spec(),
        edit_insert.spec(),
        preview_diff.spec(),
        edit_batch.spec(),
        create_file.spec(),
        delete_file.spec(),
        move_file.spec(),
        create_directory.spec(),
        delete_directory.spec(),
        move_directory.spec(),
    ]
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
        CTOOL_READ_FILE_TOOL_NAME => {
            let tool = CToolReadFile;
            tool.run_json(ctx, input)
        }
        CTOOL_EDIT_REPLACE_TOOL_NAME => {
            let tool = CToolEditReplace;
            tool.run_json(ctx, input)
        }
        CTOOL_EDIT_INSERT_TOOL_NAME => {
            let tool = CToolEditInsert;
            tool.run_json(ctx, input)
        }
        CTOOL_PREVIEW_DIFF_TOOL_NAME => {
            let tool = CToolPreviewDiff;
            tool.run_json(ctx, input)
        }
        CTOOL_EDIT_BATCH_TOOL_NAME => {
            let tool = CToolEditBatch;
            tool.run_json(ctx, input)
        }
        CTOOL_CREATE_FILE_TOOL_NAME => {
            let tool = CToolCreateFile;
            tool.run_json(ctx, input)
        }
        CTOOL_DELETE_FILE_TOOL_NAME => {
            let tool = CToolDeleteFile;
            tool.run_json(ctx, input)
        }
        CTOOL_MOVE_FILE_TOOL_NAME => {
            let tool = CToolMoveFile;
            tool.run_json(ctx, input)
        }
        CTOOL_CREATE_DIRECTORY_TOOL_NAME => {
            let tool = CToolCreateDirectory;
            tool.run_json(ctx, input)
        }
        CTOOL_DELETE_DIRECTORY_TOOL_NAME => {
            let tool = CToolDeleteDirectory;
            tool.run_json(ctx, input)
        }
        CTOOL_MOVE_DIRECTORY_TOOL_NAME => {
            let tool = CToolMoveDirectory;
            tool.run_json(ctx, input)
        }
        _ => Err(CToolError::ToolNotFound {
            name: name.to_string(),
        }),
    }
}
