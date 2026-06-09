use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::command_request::CToolCommandApproval;
use crate::command_request::CToolCommandRisk;
use crate::command_request::build_command_request_preview;
use crate::command_request::execute_approved_command_request;
use crate::command_request::render_command_request_banner;
use crate::context::CToolContext;
use crate::error::CToolResult;
use crate::scope_config::load_merged_cool_command_config;
use crate::tool::CTool;
use crate::tool::CToolSpec;

pub const CTOOL_COMMAND_REQUEST_TOOL_NAME: &str = "ctool_command_request";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolCommandRequestInput {
    pub commands: Vec<String>,

    #[serde(default)]
    pub ai_risk_upgrade: Option<CToolCommandRisk>,

    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolCommandRequestCommandOutput {
    pub command: String,
    pub risk: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolCommandRequestOutput {
    pub will_execute: bool,
    pub executed: bool,
    pub all_success: Option<bool>,
    pub result_file: Option<String>,
    pub log_file: Option<String>,

    pub current_dir: String,
    pub command_count: usize,

    pub system_risk: String,
    pub ai_risk_upgrade: Option<String>,
    pub final_risk: String,
    pub approval_required: String,

    pub request_reason: Option<String>,
    pub commands: Vec<CToolCommandRequestCommandOutput>,

    pub display_text: String,
    pub banner: String,
    pub note: String,
}

pub struct CToolCommandRequest;

impl CTool for CToolCommandRequest {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_COMMAND_REQUEST_TOOL_NAME,
            description: "Preview a controlled command execution request. It classifies command risk and renders the required approval banner. GREEN commands may auto-execute only when allowed by the user's whitelist; YELLOW and RED commands are not executed yet.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolCommandRequestInput = serde_json::from_value(input)?;
        let output = preview_command_request(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

pub fn preview_command_request(
    ctx: &CToolContext,
    input: CToolCommandRequestInput,
) -> CToolResult<CToolCommandRequestOutput> {
    let request_reason = input.reason.clone();

    let command_config = load_merged_cool_command_config(
        &ctx.scope_context.user_config_path,
        ctx.scope_context.system_config_path.as_deref(),
    )?;

    let preview = build_command_request_preview(
        &ctx.scope_context.current_dir,
        input.commands,
        &command_config,
        input.ai_risk_upgrade,
    )?;

    let banner = render_command_request_banner(&preview);

    let mut executed = false;
    let mut all_success = None;
    let mut result_file = None;
    let mut log_file = None;
    let mut note =
        "Preview only. Yellow and Red command requests are not executed yet.".to_string();

    if preview.approval == CToolCommandApproval::AutoApprovedGreen {
        let report = execute_approved_command_request(&ctx.scope_context.current_dir, &preview)?;

        executed = true;
        all_success = Some(report.all_success);
        result_file = Some(report.result_file);
        log_file = Some(report.log_file);
        note = "Green command request auto-executed by user whitelist.".to_string();
    }

    let commands = preview
        .commands
        .iter()
        .map(|command| CToolCommandRequestCommandOutput {
            command: command.command.clone(),
            risk: command.risk.label().to_string(),
            reason: command.reason.clone(),
        })
        .collect::<Vec<_>>();

    let display_text = render_command_request_display_text(
        &banner,
        executed,
        all_success,
        result_file.as_deref(),
        log_file.as_deref(),
        &note,
    );

    Ok(CToolCommandRequestOutput {
        will_execute: executed,
        executed,
        all_success,
        result_file,
        log_file,

        current_dir: preview.current_dir.clone(),
        command_count: commands.len(),

        system_risk: preview.system_risk.label().to_string(),
        ai_risk_upgrade: preview.ai_risk_upgrade.map(|risk| risk.label().to_string()),
        final_risk: preview.final_risk.label().to_string(),
        approval_required: approval_label(preview.approval).to_string(),

        request_reason,
        commands,

        display_text,
        banner,
        note,
    })
}

fn approval_label(approval: CToolCommandApproval) -> &'static str {
    match approval {
        CToolCommandApproval::AutoApprovedGreen => "none_green_auto_approved",
        CToolCommandApproval::ConfirmOnce => "confirm_once",
        CToolCommandApproval::ConfirmTwice => "confirm_twice",
    }
}

fn render_command_request_display_text(
    banner: &str,
    executed: bool,
    all_success: Option<bool>,
    result_file: Option<&str>,
    log_file: Option<&str>,
    note: &str,
) -> String {
    let mut text = String::new();

    text.push_str(banner);
    text.push_str("\n\n");
    text.push_str("COMMAND REQUEST RESULT\n");
    text.push_str("==============================\n");
    text.push_str(&format!("executed: {executed}\n"));

    if let Some(all_success) = all_success {
        text.push_str(&format!("all_success: {all_success}\n"));
    }

    if let Some(result_file) = result_file {
        text.push_str("result_file: ");
        text.push_str(result_file);
        text.push('\n');
    }

    if let Some(log_file) = log_file {
        text.push_str("log_file: ");
        text.push_str(log_file);
        text.push('\n');
    }

    text.push_str("note: ");
    text.push_str(note);
    text.push('\n');
    text.push_str("==============================");

    text
}
