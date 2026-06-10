use serde_json::Value;

use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::tool::CTool;
use crate::tool::CToolSpec;
use crate::tools::command::CTOOL_COMMAND_REQUEST_TOOL_NAME;
use crate::tools::command::CToolCommandRequest;
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
use crate::tools::read::CTOOL_COUNT_MATCHES_TOOL_NAME;
use crate::tools::read::CTOOL_EXTRACT_LINES_MATCHING_TOOL_NAME;
use crate::tools::read::CTOOL_LIST_DIRECTORY_TOOL_NAME;
use crate::tools::read::CTOOL_READ_CODE_RANGE_TOOL_NAME;
use crate::tools::read::CTOOL_READ_FILE_TOOL_NAME;
use crate::tools::read::CTOOL_REGEX_SEARCH_TOOL_NAME;
use crate::tools::read::CTOOL_RG_SEARCH_CONTEXT_TOOL_NAME;
use crate::tools::read::CTOOL_RG_SEARCH_TOOL_NAME;
use crate::tools::read::CTOOL_TAIL_FILE_TOOL_NAME;
use crate::tools::read::CToolCountMatches;
use crate::tools::read::CToolExtractLinesMatching;
use crate::tools::read::CToolListDirectory;
use crate::tools::read::CToolReadCodeRange;
use crate::tools::read::CToolReadFile;
use crate::tools::read::CToolRegexSearch;
use crate::tools::read::CToolRgSearch;
use crate::tools::read::CToolRgSearchContext;
use crate::tools::read::CToolTailFile;
use crate::tools::special::CTOOL_ANNOTATE_MARKDOWN_TOOL_NAME;
use crate::tools::special::CToolAnnotateMarkdown;
use crate::tools::special::CTOOL_TAVILY_SEARCH_REQUEST_TOOL_NAME;
use crate::tools::special::CToolTavilySearchRequest;

pub fn available_specs() -> Vec<CToolSpec> {
    let command_request = CToolCommandRequest;

    let list_directory = CToolListDirectory;
    let rg_search = CToolRgSearch;
    let read_code_range = CToolReadCodeRange;
    let read_file = CToolReadFile;
    let tail_file = CToolTailFile;
    let rg_search_context = CToolRgSearchContext;
    let regex_search = CToolRegexSearch;
    let count_matches = CToolCountMatches;
    let extract_lines_matching = CToolExtractLinesMatching;

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
    let tavily_search_request = CToolTavilySearchRequest;

    let annotate_markdown = CToolAnnotateMarkdown;
    vec![
        command_request.spec(),
        list_directory.spec(),
        rg_search.spec(),
        read_code_range.spec(),
        read_file.spec(),
        tail_file.spec(),
        rg_search_context.spec(),
        regex_search.spec(),
        count_matches.spec(),
        extract_lines_matching.spec(),
        edit_replace.spec(),
        edit_insert.spec(),
        preview_diff.spec(),
        edit_batch.spec(),
        annotate_markdown.spec(),
        create_file.spec(),
        tavily_search_request.spec(),
        delete_file.spec(),
        move_file.spec(),
        create_directory.spec(),
        delete_directory.spec(),
        move_directory.spec(),
    ]
}

pub fn run_tool(name: &str, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
    match name {
        CTOOL_COMMAND_REQUEST_TOOL_NAME => {
            let tool = CToolCommandRequest;
            tool.run_json(ctx, input)
        }
        CTOOL_LIST_DIRECTORY_TOOL_NAME => {
            let tool = CToolListDirectory;
            tool.run_json(ctx, input)
        }
        CTOOL_RG_SEARCH_TOOL_NAME => {
            let tool = CToolRgSearch;
            tool.run_json(ctx, input)
        }
        CTOOL_READ_CODE_RANGE_TOOL_NAME => {
            let tool = CToolReadCodeRange;
            tool.run_json(ctx, input)
        }
        CTOOL_READ_FILE_TOOL_NAME => {
            let tool = CToolReadFile;
            tool.run_json(ctx, input)
        }
        CTOOL_TAIL_FILE_TOOL_NAME => {
            let tool = CToolTailFile;
            tool.run_json(ctx, input)
        }
        CTOOL_RG_SEARCH_CONTEXT_TOOL_NAME => {
            let tool = CToolRgSearchContext;
            tool.run_json(ctx, input)
        }
        CTOOL_REGEX_SEARCH_TOOL_NAME => {
            let tool = CToolRegexSearch;
            tool.run_json(ctx, input)
        }
        CTOOL_COUNT_MATCHES_TOOL_NAME => {
            let tool = CToolCountMatches;
            tool.run_json(ctx, input)
        }
        CTOOL_EXTRACT_LINES_MATCHING_TOOL_NAME => {
            let tool = CToolExtractLinesMatching;
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
        CTOOL_ANNOTATE_MARKDOWN_TOOL_NAME => {
            let tool = CToolAnnotateMarkdown;
            tool.run_json(ctx, input)
        }
        CTOOL_TAVILY_SEARCH_REQUEST_TOOL_NAME => {
            let tool = CToolTavilySearchRequest;
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
