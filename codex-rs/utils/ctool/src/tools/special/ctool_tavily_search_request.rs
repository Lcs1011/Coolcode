use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use chrono::Local;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::command_request::CToolCommandRisk;
use crate::command_request::CToolCommandUserDecision;
use crate::command_request::parse_red_first_confirmation_input;
use crate::command_request::parse_red_second_confirmation_input;
use crate::command_request::parse_yellow_confirmation_input;
use crate::context::CToolContext;
use crate::error::CToolError;
use crate::error::CToolResult;
use crate::scope_config::COOL_DIR_NAME;
use crate::tool::CTool;
use crate::tool::CToolSpec;

pub const CTOOL_TAVILY_SEARCH_REQUEST_TOOL_NAME: &str = "ctool_tavily_search_request";

const TAVILY_SEARCH_URL: &str = "https://api.tavily.com/search";
const TAVILY_EXTRACT_URL: &str = "https://api.tavily.com/extract";
const DEFAULT_MAX_SEARCH_RESULTS: usize = 8;
const DEFAULT_MAX_EXTRACT_CHARS: usize = 12_000;
const DEFAULT_MAX_ZOOM_CHARS: usize = 6_000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CToolTavilySearchRequestInput {
    pub action: CToolTavilyAction,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub source_file: Option<PathBuf>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub file_name_hint: Option<String>,
    #[serde(default)]
    pub yellow_confirmation: Option<String>,
    #[serde(default)]
    pub red_first_confirmation: Option<String>,
    #[serde(default)]
    pub red_second_confirmation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CToolTavilyAction {
    Search,
    Extract,
    Zoom,
    Research,
    SearchWithImages,
    ExtractWithImages,
}

impl CToolTavilyAction {
    fn label(self) -> &'static str {
        match self {
            CToolTavilyAction::Search => "search",
            CToolTavilyAction::Extract => "extract",
            CToolTavilyAction::Zoom => "zoom",
            CToolTavilyAction::Research => "research",
            CToolTavilyAction::SearchWithImages => "search_with_images",
            CToolTavilyAction::ExtractWithImages => "extract_with_images",
        }
    }

    fn requires_tavily(self) -> bool {
        matches!(
            self,
            CToolTavilyAction::Search
                | CToolTavilyAction::Extract
                | CToolTavilyAction::Research
                | CToolTavilyAction::SearchWithImages
                | CToolTavilyAction::ExtractWithImages
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolTavilySearchRequestOutput {
    pub will_execute: bool,
    pub executed: bool,
    pub blocked: bool,
    pub rejected: bool,
    pub current_dir: String,
    pub action: String,
    pub final_risk: String,
    pub output_file: String,
    pub log_file: String,
    pub summary: String,
    pub user_feedback: Option<String>,
    pub display_text: String,
    pub banner: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TavilyConfigToml {
    #[serde(default)]
    ctool_tavily_search: TavilySearchConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TavilySearchConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_provider")]
    provider: String,
    #[serde(default)]
    tavily_api_key: Option<String>,
    #[serde(default = "default_true")]
    allow_text_search: bool,
    #[serde(default = "default_true")]
    allow_extract: bool,
    #[serde(default = "default_true")]
    allow_zoom: bool,
    #[serde(default = "default_true")]
    allow_research: bool,
    #[serde(default)]
    allow_image_search: bool,
    #[serde(default = "default_max_search_results")]
    max_search_results: usize,
    #[serde(default = "default_max_extract_chars")]
    max_extract_chars: usize,
    #[serde(default = "default_max_zoom_chars")]
    max_zoom_chars: usize,
    #[serde(default = "default_sensitive_keywords")]
    sensitive_keywords: Vec<String>,
    #[serde(default = "default_blocked_keywords")]
    blocked_keywords: Vec<String>,
}

impl Default for TavilySearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: default_provider(),
            tavily_api_key: None,
            allow_text_search: true,
            allow_extract: true,
            allow_zoom: true,
            allow_research: true,
            allow_image_search: false,
            max_search_results: DEFAULT_MAX_SEARCH_RESULTS,
            max_extract_chars: DEFAULT_MAX_EXTRACT_CHARS,
            max_zoom_chars: DEFAULT_MAX_ZOOM_CHARS,
            sensitive_keywords: default_sensitive_keywords(),
            blocked_keywords: default_blocked_keywords(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TavilyRequestPlan {
    risk: CToolCommandRisk,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TavilyWriteTarget {
    cache_dir: PathBuf,
    output_path: PathBuf,
    log_path: PathBuf,
    file_name: String,
}

pub struct CToolTavilySearchRequest;

impl CTool for CToolTavilySearchRequest {
    fn spec(&self) -> CToolSpec {
        CToolSpec {
            name: CTOOL_TAVILY_SEARCH_REQUEST_TOOL_NAME,
            description: "Run a controlled Tavily search request and write results under CoolDir cache.",
        }
    }

    fn run_json(&self, ctx: &CToolContext, input: Value) -> CToolResult<Value> {
        let input: CToolTavilySearchRequestInput = serde_json::from_value(input)?;
        let output = run_tavily_search_request(ctx, input)?;
        Ok(serde_json::to_value(output)?)
    }
}

#[cfg(test)]
#[path = "ctool_tavily_search_request_tests.rs"]
mod tests;

pub fn run_tavily_search_request(
    ctx: &CToolContext,
    input: CToolTavilySearchRequestInput,
) -> CToolResult<CToolTavilySearchRequestOutput> {
    let system_config = load_system_tavily_config(ctx)?;
    let misplaced_token_path = find_session_tavily_token_path(ctx)?;
    let session_config = load_session_tavily_config(ctx)?;
    let config = merge_tavily_configs(
        system_config.ctool_tavily_search,
        session_config.ctool_tavily_search,
    );
    let mut plan = classify_tavily_request(&input, &config);
    if let Some(path) = misplaced_token_path {
        plan = TavilyRequestPlan {
            risk: CToolCommandRisk::Blocked,
            reason: format!(
                "tavily_api_key is only allowed in CoolSystemDir\\config.toml, not {}",
                path.display()
            ),
        };
    }
    let target = build_write_target(ctx, input.action, &input)?;
    let banner = render_tavily_banner(ctx, &input, &plan, &target);

    let mut executed = false;
    let mut rejected = false;
    let mut user_feedback = None;
    let mut note = match plan.risk {
        CToolCommandRisk::Green => "Green Tavily request will auto-execute.".to_string(),
        CToolCommandRisk::Yellow => {
            "Yellow Tavily request is waiting for one user confirmation.".to_string()
        }
        CToolCommandRisk::Red => {
            "Red Tavily request is waiting for two user confirmations.".to_string()
        }
        CToolCommandRisk::Blocked => {
            "Blocked Tavily request cannot be confirmed or executed.".to_string()
        }
    };

    let approved = match plan.risk {
        CToolCommandRisk::Green => true,
        CToolCommandRisk::Yellow => match input.yellow_confirmation.as_deref() {
            Some(value) => match parse_yellow_confirmation_input(value) {
                CToolCommandUserDecision::Approved => true,
                CToolCommandUserDecision::Rejected { feedback } => {
                    rejected = true;
                    user_feedback = feedback;
                    note = "Yellow Tavily request was rejected by user confirmation input."
                        .to_string();
                    false
                }
                CToolCommandUserDecision::NeedsSecondRedConfirmation => false,
            },
            None => false,
        },
        CToolCommandRisk::Red => match input.red_first_confirmation.as_deref() {
            Some(first) => match parse_red_first_confirmation_input(first) {
                CToolCommandUserDecision::Rejected { feedback } => {
                    rejected = true;
                    user_feedback = feedback;
                    note = "Red Tavily request was rejected at first confirmation.".to_string();
                    false
                }
                CToolCommandUserDecision::NeedsSecondRedConfirmation => {
                    match input.red_second_confirmation.as_deref() {
                        Some(second) => match parse_red_second_confirmation_input(second) {
                            CToolCommandUserDecision::Approved => true,
                            CToolCommandUserDecision::Rejected { feedback } => {
                                rejected = true;
                                user_feedback = feedback;
                                note = "Red Tavily request was rejected at second confirmation."
                                    .to_string();
                                false
                            }
                            CToolCommandUserDecision::NeedsSecondRedConfirmation => false,
                        },
                        None => {
                            note = "Red Tavily request passed first confirmation and is waiting for second confirmation.".to_string();
                            false
                        }
                    }
                }
                CToolCommandUserDecision::Approved => false,
            },
            None => false,
        },
        CToolCommandRisk::Blocked => false,
    };

    let blocked = plan.risk == CToolCommandRisk::Blocked;
    let summary = if approved {
        let api_key = if input.action.requires_tavily() {
            Some(require_tavily_api_key(&config)?)
        } else {
            None
        };
        let markdown = execute_tavily_action(&input, &config, api_key.as_deref(), ctx)?;
        std::fs::create_dir_all(&target.cache_dir)?;
        std::fs::write(&target.output_path, &markdown)?;
        append_tavily_log(
            &target,
            ctx,
            &input,
            &plan,
            "Yes",
            /*status*/ None,
            user_feedback.as_deref(),
        )?;
        executed = true;
        note = "Tavily request executed and written to cache.".to_string();
        short_summary_from_markdown(&markdown)
    } else {
        std::fs::create_dir_all(&target.cache_dir)?;
        let status = if blocked { "Blocked" } else { "Rejected" };
        let markdown = render_unexecuted_markdown(ctx, &input, &plan, status, &note, user_feedback.as_deref());
        std::fs::write(&target.output_path, &markdown)?;
        append_tavily_log(
            &target,
            ctx,
            &input,
            &plan,
            "No",
            Some(status),
            user_feedback.as_deref(),
        )?;
        short_summary_from_markdown(&markdown)
    };

    let display_text = render_display_text(
        &banner,
        executed,
        blocked,
        rejected,
        &target.output_path,
        &target.log_path,
        &summary,
        &note,
        user_feedback.as_deref(),
    );

    Ok(CToolTavilySearchRequestOutput {
        will_execute: approved,
        executed,
        blocked,
        rejected,
        current_dir: ctx.scope_context.cool_workspace.display().to_string(),
        action: input.action.label().to_string(),
        final_risk: plan.risk.label().to_string(),
        output_file: target.output_path.display().to_string(),
        log_file: target.log_path.display().to_string(),
        summary,
        user_feedback,
        display_text,
        banner,
        note,
    })
}

fn load_system_tavily_config(ctx: &CToolContext) -> CToolResult<TavilyConfigToml> {
    let Some(path) = ctx.scope_context.system_config_path.as_deref() else {
        return Ok(default_tavily_config_toml());
    };
    if !path.exists() {
        return Ok(default_tavily_config_toml());
    }
    let text = std::fs::read_to_string(path)?;
    toml::from_str(&text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to parse Tavily system config: {} ({error})",
            path.display()
        ))
    })
}

fn load_session_tavily_config(ctx: &CToolContext) -> CToolResult<TavilyConfigToml> {
    let path = &ctx.scope_context.session_config_path;
    if !path.exists() {
        return Ok(default_tavily_config_toml());
    }
    let text = std::fs::read_to_string(path)?;
    toml::from_str(&text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to parse Tavily session config: {} ({error})",
            path.display()
        ))
    })
}

fn default_tavily_config_toml() -> TavilyConfigToml {
    TavilyConfigToml {
        ctool_tavily_search: TavilySearchConfig::default(),
    }
}

fn merge_tavily_configs(
    system: TavilySearchConfig,
    session: TavilySearchConfig,
) -> TavilySearchConfig {
    TavilySearchConfig {
        enabled: system.enabled && session.enabled,
        provider: system.provider,
        tavily_api_key: system.tavily_api_key,
        allow_text_search: system.allow_text_search && session.allow_text_search,
        allow_extract: system.allow_extract && session.allow_extract,
        allow_zoom: system.allow_zoom && session.allow_zoom,
        allow_research: system.allow_research && session.allow_research,
        allow_image_search: system.allow_image_search && session.allow_image_search,
        max_search_results: system.max_search_results.min(session.max_search_results),
        max_extract_chars: system.max_extract_chars.min(session.max_extract_chars),
        max_zoom_chars: system.max_zoom_chars.min(session.max_zoom_chars),
        sensitive_keywords: merge_strings(system.sensitive_keywords, session.sensitive_keywords),
        blocked_keywords: merge_strings(system.blocked_keywords, session.blocked_keywords),
    }
}

fn merge_strings(mut system: Vec<String>, session: Vec<String>) -> Vec<String> {
    for item in session {
        if !system.iter().any(|existing| existing == &item) {
            system.push(item);
        }
    }
    system
}

fn find_session_tavily_token_path(ctx: &CToolContext) -> CToolResult<Option<PathBuf>> {
    for path in [
        &ctx.scope_context.session_config_path,
        &ctx.scope_context.user_config_path,
        &ctx.scope_context.session_command_path,
    ] {
        if !path.exists() {
            continue;
        }
        let text = std::fs::read_to_string(path)?;
        if text.to_ascii_lowercase().contains("tavily_api_key") {
            return Ok(Some(path.clone()));
        }
    }
    Ok(None)
}

fn require_tavily_api_key(config: &TavilySearchConfig) -> CToolResult<&str> {
    let Some(api_key) = config.tavily_api_key.as_deref() else {
        return Err(CToolError::InvalidInput(
            "missing tavily_api_key in CoolSystemDir\\config.toml".to_string(),
        ));
    };
    if api_key.trim().is_empty() {
        return Err(CToolError::InvalidInput(
            "empty tavily_api_key in CoolSystemDir\\config.toml".to_string(),
        ));
    }
    Ok(api_key)
}

fn classify_tavily_request(
    input: &CToolTavilySearchRequestInput,
    config: &TavilySearchConfig,
) -> TavilyRequestPlan {
    if !config.enabled {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Blocked,
            reason: "ctool_tavily_search_request is disabled by config".to_string(),
        };
    }
    if config.provider.to_ascii_lowercase() != "tavily" {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Blocked,
            reason: format!("unsupported Tavily search provider: {}", config.provider),
        };
    }
    if action_disabled(input.action, config) {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Blocked,
            reason: format!("action is disabled by config: {}", input.action.label()),
        };
    }

    let text = request_text_for_risk(input);
    let normalized = text.to_ascii_lowercase();

    if input.action.requires_tavily() && config.tavily_api_key.as_deref().unwrap_or_default().is_empty() {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Blocked,
            reason: "missing Tavily token in system config".to_string(),
        };
    }
    if let Some(keyword) = first_keyword_match(&normalized, &config.blocked_keywords) {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Blocked,
            reason: format!("matched blocked keyword: {keyword}"),
        };
    }
    if looks_like_download_request(&normalized) || looks_like_browser_request(&normalized) {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Blocked,
            reason: "request asks for download, browser, or executable web behavior".to_string(),
        };
    }
    if matches!(
        input.action,
        CToolTavilyAction::SearchWithImages | CToolTavilyAction::ExtractWithImages
    ) {
        if let Some(format) = blocked_image_format(&normalized) {
            return TavilyRequestPlan {
                risk: CToolCommandRisk::Blocked,
                reason: format!("image search requested blocked image format: {format}"),
            };
        }
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Red,
            reason: "image search requires red confirmation".to_string(),
        };
    }
    if let Some(keyword) = first_keyword_match(&normalized, &config.sensitive_keywords) {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Red,
            reason: format!("matched sensitive keyword: {keyword}"),
        };
    }
    if input.action == CToolTavilyAction::Research {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Yellow,
            reason: "research may combine multiple external sources".to_string(),
        };
    }
    if input.action == CToolTavilyAction::Extract {
        return TavilyRequestPlan {
            risk: CToolCommandRisk::Yellow,
            reason: "extract fetches external page content".to_string(),
        };
    }

    TavilyRequestPlan {
        risk: CToolCommandRisk::Green,
        reason: "ordinary public text search allowed by config".to_string(),
    }
}

fn action_disabled(action: CToolTavilyAction, config: &TavilySearchConfig) -> bool {
    match action {
        CToolTavilyAction::Search => !config.allow_text_search,
        CToolTavilyAction::Extract => !config.allow_extract,
        CToolTavilyAction::Zoom => !config.allow_zoom,
        CToolTavilyAction::Research => !config.allow_research,
        CToolTavilyAction::SearchWithImages | CToolTavilyAction::ExtractWithImages => {
            !config.allow_image_search
        }
    }
}

fn execute_tavily_action(
    input: &CToolTavilySearchRequestInput,
    config: &TavilySearchConfig,
    api_key: Option<&str>,
    ctx: &CToolContext,
) -> CToolResult<String> {
    match input.action {
        CToolTavilyAction::Search | CToolTavilyAction::SearchWithImages => {
            tavily_search_markdown(input, config, api_key.unwrap_or_default(), ctx, false)
        }
        CToolTavilyAction::Research => {
            tavily_search_markdown(input, config, api_key.unwrap_or_default(), ctx, true)
        }
        CToolTavilyAction::Extract | CToolTavilyAction::ExtractWithImages => {
            tavily_extract_markdown(input, config, api_key.unwrap_or_default(), ctx)
        }
        CToolTavilyAction::Zoom => zoom_markdown(input, config, ctx),
    }
}

fn tavily_search_markdown(
    input: &CToolTavilySearchRequestInput,
    config: &TavilySearchConfig,
    api_key: &str,
    ctx: &CToolContext,
    advanced: bool,
) -> CToolResult<String> {
    let query = required_field(input.query.as_deref(), "query")?;
    let include_images = input.action == CToolTavilyAction::SearchWithImages;
    let body = json!({
        "api_key": api_key,
        "query": query,
        "max_results": config.max_search_results,
        "include_answer": true,
        "include_images": include_images,
        "search_depth": if advanced { "advanced" } else { "basic" },
    });
    let response = post_tavily_json(TAVILY_SEARCH_URL, &body)?;
    let answer = response
        .get("answer")
        .and_then(Value::as_str)
        .unwrap_or("No Tavily answer returned.");

    let mut markdown = String::new();
    markdown.push_str("# Tavily Search Result\n\n");
    markdown.push_str("Provider: Tavily\n");
    markdown.push_str(&format!("Kind: {}\n", if advanced { "Research" } else { "Search" }));
    markdown.push_str(&format!("Time: {}\n", timestamp()));
    markdown.push_str(&format!("CurrentDir: {}\n\n", ctx.scope_context.cool_workspace.display()));
    markdown.push_str("## Query\n\n");
    markdown.push_str(query);
    markdown.push_str("\n\n## Short Summary\n\n");
    markdown.push_str(answer);
    markdown.push_str("\n\n## Results\n\n");

    if let Some(results) = response.get("results").and_then(Value::as_array) {
        for (index, result) in results.iter().enumerate() {
            let title = result.get("title").and_then(Value::as_str).unwrap_or("Untitled");
            let url = result.get("url").and_then(Value::as_str).unwrap_or("");
            let content = result.get("content").and_then(Value::as_str).unwrap_or("");
            markdown.push_str(&format!("### {}. {}\n\n", index + 1, title));
            markdown.push_str("URL:\n");
            markdown.push_str(url);
            markdown.push_str("\n\nSummary:\n");
            markdown.push_str(content);
            markdown.push_str("\n\n");
        }
    }
    append_tavily_images_section(&mut markdown, &response);

    Ok(markdown)
}

fn tavily_extract_markdown(
    input: &CToolTavilySearchRequestInput,
    config: &TavilySearchConfig,
    api_key: &str,
    ctx: &CToolContext,
) -> CToolResult<String> {
    let url = required_field(input.url.as_deref(), "url")?;
    ensure_http_url(url)?;
    let include_images = input.action == CToolTavilyAction::ExtractWithImages;
    let body = json!({
        "api_key": api_key,
        "urls": [url],
        "extract_depth": "basic",
        "include_images": include_images,
    });
    let response = post_tavily_json(TAVILY_EXTRACT_URL, &body)?;
    let content = extract_tavily_content(&response);
    let content = truncate_chars(&content, config.max_extract_chars);

    let mut markdown = String::new();
    markdown.push_str("# Tavily Extract Result\n\n");
    markdown.push_str("Provider: Tavily\n");
    markdown.push_str("Kind: Extract\n");
    markdown.push_str(&format!("Time: {}\n", timestamp()));
    markdown.push_str(&format!("CurrentDir: {}\n\n", ctx.scope_context.cool_workspace.display()));
    markdown.push_str("## URL\n\n");
    markdown.push_str(url);
    markdown.push_str("\n\n## Short Summary\n\n");
    markdown.push_str(first_non_empty_line(&content).unwrap_or("Extract completed."));
    markdown.push_str("\n\n## Content\n\n");
    markdown.push_str(&content);
    markdown.push('\n');
    append_tavily_images_section(&mut markdown, &response);
    Ok(markdown)
}

fn zoom_markdown(
    input: &CToolTavilySearchRequestInput,
    config: &TavilySearchConfig,
    ctx: &CToolContext,
) -> CToolResult<String> {
    let source_file = input.source_file.as_ref().ok_or_else(|| {
        CToolError::InvalidInput("zoom requires source_file".to_string())
    })?;
    let target = required_field(input.target.as_deref(), "target")?;
    let source_path = resolve_cache_source_path(ctx, source_file);
    ensure_inside_web_cache(ctx, &source_path)?;
    let text = std::fs::read_to_string(&source_path)?;
    let excerpt = zoom_excerpt(&text, target, config.max_zoom_chars);

    let mut markdown = String::new();
    markdown.push_str("# Tavily Zoom Result\n\n");
    markdown.push_str("Provider: Tavily\n");
    markdown.push_str("Kind: Zoom\n");
    markdown.push_str(&format!("Time: {}\n", timestamp()));
    markdown.push_str(&format!("Source: {}\n", source_path.display()));
    markdown.push_str(&format!("Target: {target}\n\n"));
    markdown.push_str("## Short Summary\n\n");
    markdown.push_str("Local zoom excerpt generated from cached Tavily Markdown.\n\n");
    markdown.push_str("## Content\n\n");
    markdown.push_str(&excerpt);
    markdown.push('\n');
    Ok(markdown)
}

fn post_tavily_json(url: &str, body: &Value) -> CToolResult<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| CToolError::InvalidInput(format!("failed to build Tavily HTTP client: {error}")))?;
    let response = client
        .post(url)
        .json(body)
        .send()
        .map_err(|error| CToolError::InvalidInput(format!("Tavily request failed: {error}")))?;
    let status = response.status();
    let text = response
        .text()
        .map_err(|error| CToolError::InvalidInput(format!("failed to read Tavily response: {error}")))?;
    if !status.is_success() {
        return Err(CToolError::InvalidInput(format!(
            "Tavily request returned HTTP {status}; response body is not echoed to avoid leaking secrets"
        )));
    }
    serde_json::from_str(&text)
        .map_err(|error| CToolError::InvalidInput(format!("invalid Tavily JSON response: {error}")))
}

fn append_tavily_images_section(markdown: &mut String, response: &Value) {
    let mut image_urls = Vec::new();
    collect_tavily_images(response, &mut image_urls);
    if image_urls.is_empty() {
        return;
    }

    image_urls.sort_unstable();
    image_urls.dedup();
    markdown.push_str("\n## Images\n\n");
    for (index, url) in image_urls.iter().enumerate() {
        markdown.push_str(&format!("{}. {}\n", index + 1, url));
    }
}

fn collect_tavily_images<'a>(value: &'a Value, image_urls: &mut Vec<&'a str>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_tavily_images(item, image_urls);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                if key == "images" || key == "image" {
                    collect_tavily_image_value(item, image_urls);
                } else {
                    collect_tavily_images(item, image_urls);
                }
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn collect_tavily_image_value<'a>(value: &'a Value, image_urls: &mut Vec<&'a str>) {
    match value {
        Value::String(text) => {
            if looks_like_http_image_url(text) {
                image_urls.push(text);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_tavily_image_value(item, image_urls);
            }
        }
        Value::Object(map) => {
            if let Some(url) = map.get("url").and_then(Value::as_str)
                && looks_like_http_image_url(url)
            {
                image_urls.push(url);
            }
            if let Some(url) = map.get("image_url").and_then(Value::as_str)
                && looks_like_http_image_url(url)
            {
                image_urls.push(url);
            }
            for item in map.values() {
                collect_tavily_image_value(item, image_urls);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn looks_like_http_image_url(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    (lower.starts_with("https://") || lower.starts_with("http://"))
        && [".png", ".jpg", ".jpeg", ".webp"]
            .iter()
            .any(|suffix| lower.contains(suffix))
}

fn blocked_image_format(text: &str) -> Option<&'static str> {
    [".svg", ".gif", ".bmp", ".ico", ".tif", ".tiff", ".heic", ".avif"]
        .into_iter()
        .find(|format| text.contains(format))
}

fn extract_tavily_content(response: &Value) -> String {
    if let Some(results) = response.get("results").and_then(Value::as_array) {
        return results
            .iter()
            .filter_map(|item| {
                item.get("raw_content")
                    .or_else(|| item.get("content"))
                    .and_then(Value::as_str)
            })
            .collect::<Vec<_>>()
            .join("\n\n");
    }
    response.to_string()
}

fn build_write_target(
    ctx: &CToolContext,
    action: CToolTavilyAction,
    input: &CToolTavilySearchRequestInput,
) -> CToolResult<TavilyWriteTarget> {
    let cache_dir = ctx
        .scope_context
        .session_root
        .join(COOL_DIR_NAME)
        .join("cache")
        .join("web_search")
        .join(Local::now().format("%Y-%m-%d").to_string());
    let index = next_index(&cache_dir)?;
    let slug = slug_for_input(input);
    let file_name = format!("{index:05}_{}_{}.md", action.label(), slug);
    let output_path = cache_dir.join(&file_name);
    let log_path = cache_dir.join("request_log.md");
    Ok(TavilyWriteTarget {
        cache_dir,
        output_path,
        log_path,
        file_name,
    })
}

fn next_index(cache_dir: &Path) -> CToolResult<u64> {
    let mut max_index = None;
    if !cache_dir.exists() {
        return Ok(0);
    }
    for entry in std::fs::read_dir(cache_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let Some(prefix) = file_name.split('_').next() else {
            continue;
        };
        let Ok(index) = prefix.parse::<u64>() else {
            continue;
        };
        max_index = Some(max_index.map_or(index, |old: u64| old.max(index)));
    }
    Ok(max_index.map_or(0, |index| index + 1))
}

fn render_tavily_banner(
    ctx: &CToolContext,
    input: &CToolTavilySearchRequestInput,
    plan: &TavilyRequestPlan,
    target: &TavilyWriteTarget,
) -> String {
    let mut text = String::new();
    text.push_str("==============================\n");
    text.push_str(plan.risk.icon());
    text.push_str(" TAVILY SEARCH REQUEST: ");
    text.push_str(plan.risk.label());
    text.push('\n');
    text.push_str("Provider: Tavily\n");
    text.push_str("Action: ");
    text.push_str(input.action.label());
    text.push('\n');
    text.push_str("CurrentDir: ");
    text.push_str(&ctx.scope_context.cool_workspace.display().to_string());
    text.push('\n');
    if plan.risk == CToolCommandRisk::Blocked {
        text.push_str("Blocked: hard policy\n");
    }
    text.push('\n');
    if let Some(query) = input.query.as_deref() {
        text.push_str("Query:\n");
        text.push_str(query);
        text.push_str("\n\n");
    }
    if let Some(url) = input.url.as_deref() {
        text.push_str("URL:\n");
        text.push_str(url);
        text.push_str("\n\n");
    }
    if let Some(source_file) = input.source_file.as_deref() {
        text.push_str("Source:\n");
        text.push_str(&source_file.display().to_string());
        text.push_str("\n\n");
    }
    text.push_str("Reason:\n");
    text.push_str(&plan.reason);
    text.push_str("\n\nWill write:\n");
    text.push_str(&target.output_path.display().to_string());
    text.push_str("\n\n");
    match plan.risk {
        CToolCommandRisk::Green => text.push_str("Auto Approved: text search allowed by config\n"),
        CToolCommandRisk::Yellow => text.push_str("Confirm? Type Y to run, N to reject:\n"),
        CToolCommandRisk::Red => {
            text.push_str("First confirm? Type Y:\n");
            text.push_str("Second confirm? Type Y:\n");
        }
        CToolCommandRisk::Blocked => text.push_str("No confirmation is allowed.\n"),
    }
    text.push_str("==============================");
    text
}

fn append_tavily_log(
    target: &TavilyWriteTarget,
    ctx: &CToolContext,
    input: &CToolTavilySearchRequestInput,
    plan: &TavilyRequestPlan,
    approved: &str,
    status: Option<&str>,
    user_feedback: Option<&str>,
) -> CToolResult<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&target.log_path)?;
    writeln!(file, "## {}", timestamp())?;
    writeln!(file)?;
    writeln!(file, "Provider: Tavily")?;
    writeln!(file, "Tool: {CTOOL_TAVILY_SEARCH_REQUEST_TOOL_NAME}")?;
    writeln!(file, "Action: {}", input.action.label())?;
    writeln!(file, "Risk: {}", plan.risk.label())?;
    writeln!(file, "Approved: {approved}")?;
    if let Some(status) = status {
        writeln!(file, "Status: {status}")?;
    }
    writeln!(file, "CurrentDir: {}", ctx.scope_context.cool_workspace.display())?;
    writeln!(file)?;
    if let Some(query) = input.query.as_deref() {
        writeln!(file, "Query:")?;
        writeln!(file, "{query}")?;
        writeln!(file)?;
    }
    if let Some(url) = input.url.as_deref() {
        writeln!(file, "URL:")?;
        writeln!(file, "{url}")?;
        writeln!(file)?;
    }
    writeln!(file, "Reason:")?;
    writeln!(file, "{}", plan.reason)?;
    if let Some(user_feedback) = user_feedback {
        writeln!(file)?;
        writeln!(file, "User Feedback:")?;
        writeln!(file, "{user_feedback}")?;
    }
    writeln!(file)?;
    writeln!(file, "Output:")?;
    writeln!(file, "{}", target.file_name)?;
    writeln!(file)?;
    writeln!(file, "---")?;
    writeln!(file)?;
    Ok(())
}

fn render_unexecuted_markdown(
    ctx: &CToolContext,
    input: &CToolTavilySearchRequestInput,
    plan: &TavilyRequestPlan,
    status: &str,
    note: &str,
    user_feedback: Option<&str>,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Tavily Search Request Result\n\n");
    markdown.push_str("Provider: Tavily\n");
    markdown.push_str(&format!("Kind: {status}\n"));
    markdown.push_str(&format!("Time: {}\n", timestamp()));
    markdown.push_str(&format!("Risk: {}\n", plan.risk.label()));
    markdown.push_str("Approved: No\n");
    markdown.push_str(&format!("Status: {status}\n"));
    markdown.push_str(&format!("CurrentDir: {}\n\n", ctx.scope_context.cool_workspace.display()));
    if let Some(query) = input.query.as_deref() {
        markdown.push_str("## Query\n\n");
        markdown.push_str(query);
        markdown.push_str("\n\n");
    }
    if let Some(url) = input.url.as_deref() {
        markdown.push_str("## URL\n\n");
        markdown.push_str(url);
        markdown.push_str("\n\n");
    }
    markdown.push_str("## Reason\n\n");
    markdown.push_str(&plan.reason);
    markdown.push_str("\n\n## Note\n\n");
    markdown.push_str(note);
    markdown.push_str("\n\n");
    if let Some(user_feedback) = user_feedback {
        markdown.push_str("## User Feedback\n\n");
        markdown.push_str(user_feedback);
        markdown.push_str("\n\n");
    }
    markdown.push_str("## Network\n\nNo Tavily request was sent.\n");
    markdown
}

fn render_display_text(
    banner: &str,
    executed: bool,
    blocked: bool,
    rejected: bool,
    output_file: &Path,
    log_file: &Path,
    summary: &str,
    note: &str,
    user_feedback: Option<&str>,
) -> String {
    let mut text = String::new();
    text.push_str(banner);
    text.push_str("\n\nTAVILY SEARCH REQUEST RESULT\n");
    text.push_str("==============================\n");
    text.push_str(&format!("executed: {executed}\n"));
    text.push_str(&format!("blocked: {blocked}\n"));
    text.push_str(&format!("rejected: {rejected}\n"));
    text.push_str("output_file: ");
    text.push_str(&output_file.display().to_string());
    text.push('\n');
    text.push_str("log_file: ");
    text.push_str(&log_file.display().to_string());
    text.push('\n');
    text.push_str("summary: ");
    text.push_str(summary);
    text.push('\n');
    if let Some(user_feedback) = user_feedback {
        text.push_str("user_feedback: ");
        text.push_str(user_feedback);
        text.push('\n');
    }
    text.push_str("note: ");
    text.push_str(note);
    text.push('\n');
    text.push_str("==============================");
    text
}

fn request_text_for_risk(input: &CToolTavilySearchRequestInput) -> String {
    [
        input.query.as_deref(),
        input.url.as_deref(),
        input.target.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
}

fn slug_for_input(input: &CToolTavilySearchRequestInput) -> String {
    let source = input
        .file_name_hint
        .as_deref()
        .or(input.query.as_deref())
        .or(input.url.as_deref())
        .or(input.target.as_deref())
        .unwrap_or("tavily_request");
    slugify(source)
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
        if slug.len() >= 80 {
            break;
        }
    }
    let slug = slug.trim_matches('_').to_string();
    if slug.is_empty() {
        "tavily_request".to_string()
    } else {
        slug
    }
}

fn required_field<'a>(value: Option<&'a str>, name: &str) -> CToolResult<&'a str> {
    let Some(value) = value else {
        return Err(CToolError::InvalidInput(format!("{name} is required")));
    };
    if value.trim().is_empty() {
        return Err(CToolError::InvalidInput(format!("{name} must not be empty")));
    }
    Ok(value)
}

fn ensure_http_url(url: &str) -> CToolResult<()> {
    if url.starts_with("https://") || url.starts_with("http://") {
        Ok(())
    } else {
        Err(CToolError::InvalidInput(format!(
            "Tavily extract URL must be http/https: {url}"
        )))
    }
}

fn resolve_cache_source_path(ctx: &CToolContext, source_file: &Path) -> PathBuf {
    if source_file.is_absolute() {
        source_file.to_path_buf()
    } else {
        ctx.scope_context.cool_workspace.join(source_file)
    }
}

fn ensure_inside_web_cache(ctx: &CToolContext, path: &Path) -> CToolResult<()> {
    let cache_root = ctx
        .scope_context
        .session_root
        .join(COOL_DIR_NAME)
        .join("cache")
        .join("web_search");
    let normalized_path = path.to_string_lossy().to_ascii_lowercase();
    let normalized_cache = cache_root.to_string_lossy().to_ascii_lowercase();
    if normalized_path.starts_with(&normalized_cache) {
        Ok(())
    } else {
        Err(CToolError::OutOfScope {
            path: path.display().to_string(),
            operation: "tavily zoom source read",
        })
    }
}

fn zoom_excerpt(text: &str, target: &str, max_chars: usize) -> String {
    let lower_text = text.to_ascii_lowercase();
    let lower_target = target.to_ascii_lowercase();
    let Some(byte_index) = lower_text.find(&lower_target) else {
        return truncate_chars(text, max_chars);
    };
    let start = text[..byte_index]
        .char_indices()
        .rev()
        .nth(1000)
        .map_or(0, |(index, _)| index);
    truncate_chars(&text[start..], max_chars)
}

fn first_keyword_match<'a>(text: &str, keywords: &'a [String]) -> Option<&'a str> {
    keywords.iter().find_map(|keyword| {
        let normalized = keyword.to_ascii_lowercase();
        if !normalized.is_empty() && text.contains(&normalized) {
            Some(keyword.as_str())
        } else {
            None
        }
    })
}

fn looks_like_download_request(text: &str) -> bool {
    [
        ".exe", ".msi", ".dll", ".bat", ".cmd", ".ps1", ".sh", ".zip", ".rar",
        ".7z", ".tar", ".gz", "download", "git clone", "curl ", "wget ",
        "invoke-webrequest",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_browser_request(text: &str) -> bool {
    ["open browser", "start http", "explorer http", "execute javascript"]
        .iter()
        .any(|needle| text.contains(needle))
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let mut result = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        result.push_str("\n...[truncated]");
    }
    result
}

fn short_summary_from_markdown(markdown: &str) -> String {
    first_non_empty_line_after_heading(markdown, "## Short Summary")
        .unwrap_or_else(|| "Tavily request completed.".to_string())
}

fn first_non_empty_line_after_heading(markdown: &str, heading: &str) -> Option<String> {
    let mut in_section = false;
    for line in markdown.lines() {
        if line.trim() == heading {
            in_section = true;
            continue;
        }
        if in_section && !line.trim().is_empty() {
            return Some(line.trim().to_string());
        }
    }
    None
}

fn first_non_empty_line(text: &str) -> Option<&str> {
    text.lines().find(|line| !line.trim().is_empty())
}

fn timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn default_true() -> bool {
    true
}

fn default_provider() -> String {
    "tavily".to_string()
}

fn default_max_search_results() -> usize {
    DEFAULT_MAX_SEARCH_RESULTS
}

fn default_max_extract_chars() -> usize {
    DEFAULT_MAX_EXTRACT_CHARS
}

fn default_max_zoom_chars() -> usize {
    DEFAULT_MAX_ZOOM_CHARS
}

fn default_sensitive_keywords() -> Vec<String> {
    [
        "token",
        "api key",
        "apikey",
        "password",
        "secret",
        "bearer",
        "sk-",
        "tvly-",
        "private key",
        "ssh key",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn default_blocked_keywords() -> Vec<String> {
    [
        "leak token",
        "steal api key",
        "dump password",
        "bypass login",
        "exploit private key",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
