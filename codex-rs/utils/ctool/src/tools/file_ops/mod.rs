use std::path::Path;

use crate::error::CToolError;
use crate::error::CToolResult;

pub mod ctool_create_directory;
pub mod ctool_create_file;
pub mod ctool_delete_directory;
pub mod ctool_delete_file;
pub mod ctool_move_directory;
pub mod ctool_move_file;

pub use ctool_create_directory::CTOOL_CREATE_DIRECTORY_TOOL_NAME;
pub use ctool_create_directory::CToolCreateDirectory;
pub use ctool_create_directory::CToolCreateDirectoryInput;
pub use ctool_create_directory::CToolCreateDirectoryOutput;
pub use ctool_create_file::CTOOL_CREATE_FILE_TOOL_NAME;
pub use ctool_create_file::CToolCreateFile;
pub use ctool_create_file::CToolCreateFileInput;
pub use ctool_create_file::CToolCreateFileOutput;
pub use ctool_delete_directory::CTOOL_DELETE_DIRECTORY_TOOL_NAME;
pub use ctool_delete_directory::CToolDeleteDirectory;
pub use ctool_delete_directory::CToolDeleteDirectoryInput;
pub use ctool_delete_directory::CToolDeleteDirectoryOutput;
pub use ctool_delete_file::CTOOL_DELETE_FILE_TOOL_NAME;
pub use ctool_delete_file::CToolDeleteFile;
pub use ctool_delete_file::CToolDeleteFileInput;
pub use ctool_delete_file::CToolDeleteFileOutput;
pub use ctool_move_directory::CTOOL_MOVE_DIRECTORY_TOOL_NAME;
pub use ctool_move_directory::CToolMoveDirectory;
pub use ctool_move_directory::CToolMoveDirectoryInput;
pub use ctool_move_directory::CToolMoveDirectoryOutput;
pub use ctool_move_file::CTOOL_MOVE_FILE_TOOL_NAME;
pub use ctool_move_file::CToolMoveFile;
pub use ctool_move_file::CToolMoveFileInput;
pub use ctool_move_file::CToolMoveFileOutput;

pub(crate) fn ensure_safe_text_file_extension(path: &Path, operation: &str) -> CToolResult<()> {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(CToolError::InvalidInput(format!(
            "invalid file name: {}",
            path.display()
        )));
    };

    if file_name == ".gitignore" {
        return Ok(());
    }

    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return Err(CToolError::InvalidInput(format!(
            "file extension is required for {operation}: {}",
            path.display()
        )));
    };

    let extension = extension.to_ascii_lowercase();
    let allowed = matches!(
        extension.as_str(),
        "rs" | "toml"
            | "md"
            | "txt"
            | "json"
            | "jsonl"
            | "yaml"
            | "yml"
            | "css"
            | "html"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "java"
            | "go"
            | "py"
            | "lua"
            | "ini"
            | "cfg"
    );

    if allowed {
        Ok(())
    } else {
        Err(CToolError::InvalidInput(format!(
            "{operation} does not allow this extension: .{extension}"
        )))
    }
}
