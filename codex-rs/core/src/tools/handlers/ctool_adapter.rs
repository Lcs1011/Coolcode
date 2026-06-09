use std::collections::BTreeMap;

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_tools::AdditionalProperties;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde_json::Value;
use serde_json::json;

pub(crate) struct CToolHandler {
    name: String,
    description: String,
}

impl CToolHandler {
    pub(crate) fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for CToolHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(self.name.clone())
    }

    fn spec(&self) -> ToolSpec {
        create_ctool_tool_spec(&self.name, &self.description)
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let arguments = match invocation.payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(format!(
                    "{} received unsupported payload",
                    self.name
                )));
            }
        };

        let input: Value = serde_json::from_str(&arguments).map_err(|error| {
            FunctionCallError::RespondToModel(format!(
                "failed to parse {} arguments: {error}",
                self.name
            ))
        })?;

        #[allow(deprecated)]
        let workspace_root = invocation.turn.cwd.as_path().to_path_buf();

        let ctx = ctool::CToolContext::workspace(vec![workspace_root]);

        let output = ctool::registry::run_tool(&self.name, &ctx, input).map_err(|error| {
            FunctionCallError::RespondToModel(format!("{} failed: {error}", self.name))
        })?;

        let text = serde_json::to_string_pretty(&output).unwrap_or_else(|error| {
            json!({
                "error": format!("failed to serialize {} output: {error}", self.name)
            })
            .to_string()
        });

        Ok(boxed_tool_output(FunctionToolOutput::from_text(
            text,
            Some(true),
        )))
    }
}

impl CoreToolRuntime for CToolHandler {}

fn create_ctool_tool_spec(name: &str, base_description: &str) -> ToolSpec {
    let description = ctool_description(name, base_description);

    ToolSpec::Function(ResponsesApiTool {
        name: name.to_string(),
        description,
        strict: false,
        defer_loading: None,
        parameters: generic_object_schema(),
        output_schema: None,
    })
}

fn generic_object_schema() -> JsonSchema {
    JsonSchema::object(
        BTreeMap::new(),
        None,
        Some(AdditionalProperties::from(true)),
    )
}

fn ctool_description(name: &str, base_description: &str) -> String {
    let usage = match name {
        "ctool_list_directory" => {
            r#"Input JSON:
{
  "path": ".",
  "max_depth": 1,
  "max_entries": 200,
  "include_hidden": false
}

Use this to inspect project structure inside CToolScope."#
        }
        "ctool_rg_search" => {
            r#"Input JSON:
{
  "path": ".",
  "query": "search text",
  "case_sensitive": false,
  "max_depth": 6,
  "max_results": 100,
  "include_hidden": false
}

Use this to search UTF-8 text files by literal substring inside CToolScope."#
        }
        "ctool_read_code_range" => {
            r#"Input JSON:
{
  "path": "relative/or/absolute/file/path.rs",
  "start_line": 1,
  "end_line": 80
}

Use this to read a specific inclusive line range from a UTF-8 text file."#
        }
        "ctool_read_file" => {
            r#"Input JSON:
{
  "path": "relative/or/absolute/file/path.toml",
  "max_bytes": 262144
}

Use this to read one small UTF-8 text/config/source file."#
        }
        "ctool_edit_replace" => {
            r#"Input JSON:
{
  "path": "relative/or/absolute/file/path.rs",
  "old_string": "exact text to replace",
  "new_string": "replacement text"
}

Replaces exactly one occurrence. Fails if old_string is missing or appears more than once."#
        }
        "ctool_edit_insert" => {
            r#"Input JSON:
{
  "path": "relative/or/absolute/file/path.rs",
  "insert_after_line": 10,
  "content": "text to insert\n"
}

insert_after_line = 0 inserts at file beginning. Otherwise inserts after that 1-based line."#
        }
        "ctool_preview_diff" => {
            r#"Input JSON:
{
  "path": "relative/or/absolute/file/path.rs",
  "operations": [
    {
      "operation": "replace",
      "old_string": "exact text",
      "new_string": "replacement"
    },
    {
      "operation": "insert",
      "insert_after_line": 10,
      "content": "text to insert\n"
    }
  ]
}

Previews replace/insert operations without writing the file."#
        }
        "ctool_edit_batch" => {
            r#"Input JSON:
{
  "operations": [
    {
      "operation": "replace",
      "path": "file.rs",
      "old_string": "exact text",
      "new_string": "replacement"
    },
    {
      "operation": "insert",
      "path": "file.rs",
      "insert_after_line": 10,
      "content": "text to insert\n"
    }
  ]
}

Applies multiple replace/insert operations. Use preview_diff first when practical."#
        }
        "ctool_create_file" => {
            r#"Input JSON:
{
  "path": "new_file.rs",
  "content": "file content",
  "overwrite": false
}

Creates a safe UTF-8 text/source/config file. overwrite=false fails if the file exists."#
        }
        "ctool_delete_file" => {
            r#"Input JSON:
{
  "path": "file_to_delete.rs",
  "expected_content": null
}

Deletes one file. Directories are never deleted. expected_content may be provided for extra safety."#
        }
        "ctool_move_file" => {
            r#"Input JSON:
{
  "from": "old_file.rs",
  "to": "new_file.rs",
  "overwrite": false
}

Moves or renames one file. Directories are never moved by this tool."#
        }
        "ctool_create_directory" => {
            r#"Input JSON:
{
  "path": "new_directory"
}

Creates one directory. Parent directory must already exist."#
        }
        "ctool_delete_directory" => {
            r#"Input JSON:
{
  "path": "empty_directory"
}

Deletes one empty directory. Recursive deletion is never allowed."#
        }
        "ctool_move_directory" => {
            r#"Input JSON:
{
  "from": "old_directory",
  "to": "new_directory"
}

Moves or renames one directory. Overwrite is never allowed."#
        }
        _ => "Input JSON must be an object accepted by the CTool runtime.",
    };

    format!(
        "{base_description}\n\n\
        This is a CTool. It is the only allowed tool family in CoolReadWrite mode. \
        Codex native shell/apply_patch/read/write/search tools are disabled.\n\n\
        {usage}"
    )
}
