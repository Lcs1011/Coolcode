use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::error::CToolError;
use crate::error::CToolResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CToolCommandRisk {
    Green,
    Yellow,
    Red,
    Blocked,
}

impl CToolCommandRisk {
    pub fn icon(self) -> &'static str {
        match self {
            CToolCommandRisk::Green => "🟢",
            CToolCommandRisk::Yellow => "🟡",
            CToolCommandRisk::Red => "🔴",
            CToolCommandRisk::Blocked => "⛔",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CToolCommandRisk::Green => "GREEN",
            CToolCommandRisk::Yellow => "YELLOW",
            CToolCommandRisk::Red => "RED",
            CToolCommandRisk::Blocked => "BLOCKED",
        }
    }

    pub fn approval(self) -> CToolCommandApproval {
        match self {
            CToolCommandRisk::Green => CToolCommandApproval::AutoApprovedGreen,
            CToolCommandRisk::Yellow => CToolCommandApproval::ConfirmOnce,
            CToolCommandRisk::Red => CToolCommandApproval::ConfirmTwice,
            CToolCommandRisk::Blocked => CToolCommandApproval::Blocked,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CToolCommandApproval {
    AutoApprovedGreen,
    ConfirmOnce,
    ConfirmTwice,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CToolCommandConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub green_exact_commands: Vec<String>,
    #[serde(default)]
    pub green_prefixes: Vec<String>,
    #[serde(default)]
    pub yellow_prefixes: Vec<String>,
    #[serde(default)]
    pub red_prefixes: Vec<String>,
    #[serde(default)]
    pub red_contains: Vec<String>,
    #[serde(default)]
    pub blocked_prefixes: Vec<String>,
    #[serde(default)]
    pub blocked_contains: Vec<String>,
}

impl Default for CToolCommandConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            green_exact_commands: Vec::new(),
            green_prefixes: Vec::new(),
            yellow_prefixes: vec![
                "cargo check".to_string(),
                "cargo build".to_string(),
                "cargo test".to_string(),
                "cargo fmt".to_string(),
                "git status".to_string(),
                "git diff".to_string(),
                "git add".to_string(),
                "git commit".to_string(),
                "rg".to_string(),
            ],
            red_prefixes: vec![
                "del".to_string(),
                "erase".to_string(),
                "rmdir".to_string(),
                "rd".to_string(),
                "remove-item".to_string(),
                "git reset --hard".to_string(),
                "git clean -fd".to_string(),
                "git clone".to_string(),
                "powershell".to_string(),
                "pwsh".to_string(),
                "cmd".to_string(),
                "python".to_string(),
                "node".to_string(),
                "curl".to_string(),
                "wget".to_string(),
                "invoke-webrequest".to_string(),
                "invoke-restmethod".to_string(),
                "shutdown".to_string(),
                "taskkill".to_string(),
                "reg".to_string(),
                "netsh".to_string(),
                "start".to_string(),
                "start-process".to_string(),
                "explorer".to_string(),
            ],
            red_contains: vec![
                "http://".to_string(),
                "https://".to_string(),
                "ftp://".to_string(),
                "download".to_string(),
                ".exe".to_string(),
                ".msi".to_string(),
                ".dll".to_string(),
                ".bat".to_string(),
                ".cmd".to_string(),
                ".ps1".to_string(),
                ".sh".to_string(),
                ".zip".to_string(),
                ".rar".to_string(),
                ".7z".to_string(),
                ".tar".to_string(),
                ".gz".to_string(),
                "&&".to_string(),
                "||".to_string(),
                ">".to_string(),
                ">>".to_string(),
                "|".to_string(),
            ],
            blocked_prefixes: vec![
                "python -m venv".to_string(),
                "py -m venv".to_string(),
                "python3 -m venv".to_string(),
                "virtualenv".to_string(),
                "python -m virtualenv".to_string(),
                "py -m virtualenv".to_string(),
                "pip install".to_string(),
                "pip3 install".to_string(),
                "python -m pip install".to_string(),
                "py -m pip install".to_string(),
                "python3 -m pip install".to_string(),
                "pipx install".to_string(),
                "uv venv".to_string(),
                "uv python install".to_string(),
                "uv tool install".to_string(),
                "rye".to_string(),
                "hatch env".to_string(),
                "pipenv install".to_string(),
                "poetry install".to_string(),
                "poetry env".to_string(),
                "conda create".to_string(),
                "conda install".to_string(),
                "mamba create".to_string(),
                "mamba install".to_string(),
                "micromamba create".to_string(),
                "micromamba install".to_string(),
                "winget install python".to_string(),
                "choco install python".to_string(),
                "scoop install python".to_string(),
                "msiexec".to_string(),
                "python -m ensurepip".to_string(),
                "py -m ensurepip".to_string(),
            ],
            blocked_contains: vec![
                "python.org".to_string(),
                "install python".to_string(),
                "python installer".to_string(),
                "python-3.".to_string(),
                "python3.".to_string(),
                "venv".to_string(),
                "virtualenv".to_string(),
                "ensurepip".to_string(),
                "pip install".to_string(),
                "conda create".to_string(),
                "conda install".to_string(),
                "pyenv".to_string(),
                "pyenv-win".to_string(),
                "appdata\\local\\programs\\python".to_string(),
                "c:\\python".to_string(),
                "program files\\python".to_string(),
                "windowsapps\\python".to_string(),
                "setx path".to_string(),
                "set path".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CToolCommandClassification {
    pub command: String,
    pub risk: CToolCommandRisk,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CToolCommandRequestPreview {
    pub current_dir: String,
    pub commands: Vec<CToolCommandClassification>,
    pub system_risk: CToolCommandRisk,
    pub ai_risk_upgrade: Option<CToolCommandRisk>,
    pub final_risk: CToolCommandRisk,
    pub approval: CToolCommandApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolCommandExecutionReport {
    pub executed: bool,
    pub all_success: bool,
    pub result_file: String,
    pub log_file: String,
    pub commands: Vec<CToolCommandExecutionItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CToolCommandExecutionItem {
    pub command: String,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub stdout_preview: String,
    pub stderr_preview: String,
}

fn default_true() -> bool {
    true
}

pub fn default_command_config() -> CToolCommandConfig {
    CToolCommandConfig::default()
}

pub fn merge_command_configs(
    session_config: CToolCommandConfig,
    system_config: CToolCommandConfig,
) -> CToolCommandConfig {
    let mut merged = CToolCommandConfig {
        enabled: session_config.enabled && system_config.enabled,
        green_exact_commands: Vec::new(),
        green_prefixes: Vec::new(),
        yellow_prefixes: Vec::new(),
        red_prefixes: Vec::new(),
        red_contains: Vec::new(),
        blocked_prefixes: Vec::new(),
        blocked_contains: Vec::new(),
    };

    append_unique_strings(
        &mut merged.green_exact_commands,
        system_config.green_exact_commands,
    );
    append_unique_strings(
        &mut merged.green_exact_commands,
        session_config.green_exact_commands,
    );
    append_unique_strings(&mut merged.green_prefixes, system_config.green_prefixes);
    append_unique_strings(&mut merged.green_prefixes, session_config.green_prefixes);
    append_unique_strings(&mut merged.yellow_prefixes, system_config.yellow_prefixes);
    append_unique_strings(&mut merged.yellow_prefixes, session_config.yellow_prefixes);
    append_unique_strings(&mut merged.red_prefixes, system_config.red_prefixes);
    append_unique_strings(&mut merged.red_prefixes, session_config.red_prefixes);
    append_unique_strings(&mut merged.red_contains, system_config.red_contains);
    append_unique_strings(&mut merged.red_contains, session_config.red_contains);
    append_unique_strings(&mut merged.blocked_prefixes, system_config.blocked_prefixes);
    append_unique_strings(
        &mut merged.blocked_prefixes,
        session_config.blocked_prefixes,
    );
    append_unique_strings(&mut merged.blocked_contains, system_config.blocked_contains);
    append_unique_strings(
        &mut merged.blocked_contains,
        session_config.blocked_contains,
    );

    merged
}

pub fn classify_command(
    command: impl AsRef<str>,
    config: &CToolCommandConfig,
) -> CToolCommandClassification {
    let raw_command = command.as_ref().trim().to_string();
    let normalized_command = normalize_command_for_match(&raw_command);

    if raw_command.is_empty() {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Red,
            reason: "empty command".to_string(),
        };
    }

    if let Some(rule) = first_contains_match(&normalized_command, &config.blocked_contains) {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Blocked,
            reason: format!("matched blocked contains rule: {rule}"),
        };
    }
    if let Some(rule) = first_prefix_match(&normalized_command, &config.blocked_prefixes) {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Blocked,
            reason: format!("matched blocked prefix rule: {rule}"),
        };
    }
    if let Some(rule) = first_contains_match(&normalized_command, &config.red_contains) {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Red,
            reason: format!("matched red contains rule: {rule}"),
        };
    }
    if let Some(rule) = first_prefix_match(&normalized_command, &config.red_prefixes) {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Red,
            reason: format!("matched red prefix rule: {rule}"),
        };
    }
    if let Some(rule) = first_exact_match(&normalized_command, &config.green_exact_commands) {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Green,
            reason: format!("matched green exact rule: {rule}"),
        };
    }
    if let Some(rule) = first_prefix_match(&normalized_command, &config.green_prefixes) {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Green,
            reason: format!("matched green prefix rule: {rule}"),
        };
    }
    if let Some(rule) = first_prefix_match(&normalized_command, &config.yellow_prefixes) {
        return CToolCommandClassification {
            command: raw_command,
            risk: CToolCommandRisk::Yellow,
            reason: format!("matched yellow prefix rule: {rule}"),
        };
    }

    CToolCommandClassification {
        command: raw_command,
        risk: CToolCommandRisk::Red,
        reason: "unknown command defaults to red".to_string(),
    }
}

pub fn build_command_request_preview(
    current_dir: impl AsRef<Path>,
    commands: Vec<String>,
    config: &CToolCommandConfig,
    ai_risk_upgrade: Option<CToolCommandRisk>,
) -> CToolResult<CToolCommandRequestPreview> {
    if !config.enabled {
        return Err(CToolError::InvalidInput(
            "ctool_command_request is disabled by config".to_string(),
        ));
    }
    if commands.is_empty() {
        return Err(CToolError::InvalidInput(
            "ctool_command_request requires at least one command".to_string(),
        ));
    }

    let classified_commands = commands
        .into_iter()
        .map(|command| classify_command(command, config))
        .collect::<Vec<_>>();

    let system_risk = classified_commands
        .iter()
        .map(|item| item.risk)
        .max()
        .unwrap_or(CToolCommandRisk::Red);
    let final_risk = match ai_risk_upgrade {
        Some(ai_risk) => std::cmp::max(system_risk, ai_risk),
        None => system_risk,
    };

    Ok(CToolCommandRequestPreview {
        current_dir: current_dir.as_ref().display().to_string(),
        commands: classified_commands,
        system_risk,
        ai_risk_upgrade,
        final_risk,
        approval: final_risk.approval(),
    })
}

pub fn render_command_request_banner(preview: &CToolCommandRequestPreview) -> String {
    let mut text = String::new();
    text.push_str("==============================\n");
    text.push_str(preview.final_risk.icon());
    text.push_str(" COMMAND REQUEST: ");
    text.push_str(preview.final_risk.label());
    text.push('\n');
    text.push_str("CurrentDir: ");
    text.push_str(&preview.current_dir);
    text.push('\n');
    if let Some(ai_risk) = preview.ai_risk_upgrade {
        text.push_str("SystemRisk: ");
        text.push_str(preview.system_risk.label());
        text.push('\n');
        text.push_str("AI Risk Upgrade: ");
        text.push_str(ai_risk.label());
        text.push('\n');
    }
    if preview.final_risk == CToolCommandRisk::Green {
        text.push_str("Auto Approved: green whitelist\n");
    }
    text.push('\n');

    for (index, command) in preview.commands.iter().enumerate() {
        text.push_str(&format!(
            "[{}] [{}] {}\n",
            index + 1,
            command.risk.label(),
            command.command
        ));
        text.push_str("    reason: ");
        text.push_str(&command.reason);
        text.push('\n');
    }

    text.push('\n');
    match preview.approval {
        CToolCommandApproval::AutoApprovedGreen => {
            text.push_str("No confirmation required.\n");
        }
        CToolCommandApproval::ConfirmOnce => {
            text.push_str("Confirm once: provide a Y/y confirmation to execute.\n");
        }
        CToolCommandApproval::ConfirmTwice => {
            text.push_str("Confirm twice: provide two separate Y/y confirmations to execute.\n");
        }
        CToolCommandApproval::Blocked => {
            text.push_str("Blocked: this command is forbidden by Cool system policy.\n");
            text.push_str("No confirmation is allowed.\n");
            text.push_str("It will never be executed.\n");
        }
    }
    text.push_str("==============================");
    text
}

pub fn execute_approved_command_request(
    current_dir: impl AsRef<Path>,
    cache_root: impl AsRef<Path>,
    preview: &CToolCommandRequestPreview,
) -> CToolResult<CToolCommandExecutionReport> {
    if preview.final_risk == CToolCommandRisk::Blocked {
        return Err(CToolError::InvalidInput(
            "blocked command request cannot be executed".to_string(),
        ));
    }

    let current_dir = current_dir.as_ref();
    let cache_dir = command_request_cache_dir(cache_root.as_ref());
    std::fs::create_dir_all(&cache_dir)?;

    let index = next_command_request_index(&cache_dir)?;
    let result_file_name = format!(
        "{index:05}_{}_command_request.md",
        preview.final_risk.label().to_ascii_lowercase()
    );
    let result_path = cache_dir.join(result_file_name);
    let log_path = cache_dir.join("request_log.md");

    let started_at = Local::now();
    let mut result_text = String::new();
    result_text.push_str("# CTool Command Request Result\n\n");
    result_text.push_str(&format!(
        "Time: {}\n",
        started_at.format("%Y-%m-%d %H:%M:%S")
    ));
    result_text.push_str(&format!("Risk: {}\n", preview.final_risk.label()));
    result_text.push_str("Approved: Yes\n");
    result_text.push_str(&format!("CurrentDir: {}\n\n", current_dir.display()));
    result_text.push_str("## Commands\n\n");

    let mut items = Vec::new();
    let mut all_success = true;

    for (index, command) in preview.commands.iter().enumerate() {
        let command_started_at = Local::now();
        result_text.push_str(&format!("### {}. `{}`\n\n", index + 1, command.command));
        result_text.push_str(&format!(
            "StartedAt: {}\n\n",
            command_started_at.format("%Y-%m-%d %H:%M:%S")
        ));

        let output = run_shell_command(current_dir, &command.command)?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        let success = output.status.success();

        result_text.push_str(&format!("ExitCode: {:?}\n", exit_code));
        result_text.push_str(&format!("Success: {success}\n\n"));
        result_text.push_str("#### Stdout\n\n```text\n");
        result_text.push_str(&stdout);
        result_text.push_str("\n```\n\n");
        result_text.push_str("#### Stderr\n\n```text\n");
        result_text.push_str(&stderr);
        result_text.push_str("\n```\n\n");

        items.push(CToolCommandExecutionItem {
            command: command.command.clone(),
            exit_code,
            success,
            stdout_preview: truncate_for_preview(&stdout, 4000),
            stderr_preview: truncate_for_preview(&stderr, 4000),
        });

        if !success {
            all_success = false;
            result_text.push_str("Stopped: command failed, later commands were not executed.\n\n");
            break;
        }
    }

    std::fs::write(&result_path, result_text)?;
    append_command_request_log(&log_path, preview, &result_path, all_success)?;

    Ok(CToolCommandExecutionReport {
        executed: true,
        all_success,
        result_file: result_path.display().to_string(),
        log_file: log_path.display().to_string(),
        commands: items,
    })
}

fn command_request_cache_dir(cache_root: &Path) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d").to_string();
    cache_root.join(date)
}

fn next_command_request_index(cache_dir: &Path) -> CToolResult<u64> {
    let mut max_index: Option<u64> = None;

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
        max_index = Some(max_index.map_or(index, |old| old.max(index)));
    }

    Ok(max_index.map_or(0, |index| index + 1))
}

fn append_command_request_log(
    log_path: &Path,
    preview: &CToolCommandRequestPreview,
    result_path: &Path,
    all_success: bool,
) -> CToolResult<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    writeln!(file, "## {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(file)?;
    writeln!(file, "Risk: {}", preview.final_risk.label())?;
    writeln!(file, "Approved: Yes")?;
    writeln!(file, "AllSuccess: {all_success}")?;
    writeln!(file, "CurrentDir: {}", preview.current_dir)?;
    writeln!(file)?;
    writeln!(file, "Commands:")?;
    for (index, command) in preview.commands.iter().enumerate() {
        writeln!(file, "{}. {}", index + 1, command.command)?;
    }
    writeln!(file)?;
    writeln!(file, "Output:")?;
    writeln!(file, "{}", result_path.display())?;
    writeln!(file)?;
    writeln!(file, "---")?;
    writeln!(file)?;

    Ok(())
}

fn run_shell_command(current_dir: &Path, command: &str) -> CToolResult<std::process::Output> {
    #[cfg(windows)]
    {
        Ok(Command::new("cmd")
            .arg("/C")
            .arg(command)
            .current_dir(current_dir)
            .output()?)
    }

    #[cfg(not(windows))]
    {
        Ok(Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(current_dir)
            .output()?)
    }
}

fn truncate_for_preview(text: &str, max_chars: usize) -> String {
    let mut result = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        result.push_str("\n...[truncated]");
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CToolCommandUserDecision {
    Approved,
    Rejected { feedback: Option<String> },
    NeedsSecondRedConfirmation,
}

pub fn parse_yellow_confirmation_input(input: &str) -> CToolCommandUserDecision {
    parse_single_confirmation_input(input)
}

pub fn parse_red_first_confirmation_input(input: &str) -> CToolCommandUserDecision {
    let trimmed = input.trim();
    if starts_with_yes(trimmed) {
        CToolCommandUserDecision::NeedsSecondRedConfirmation
    } else {
        CToolCommandUserDecision::Rejected {
            feedback: feedback_from_reject_input(trimmed),
        }
    }
}

pub fn parse_red_second_confirmation_input(input: &str) -> CToolCommandUserDecision {
    let trimmed = input.trim();
    if starts_with_yes(trimmed) {
        CToolCommandUserDecision::Approved
    } else {
        CToolCommandUserDecision::Rejected {
            feedback: feedback_from_reject_input(trimmed),
        }
    }
}

fn parse_single_confirmation_input(input: &str) -> CToolCommandUserDecision {
    let trimmed = input.trim();
    if starts_with_yes(trimmed) {
        CToolCommandUserDecision::Approved
    } else {
        CToolCommandUserDecision::Rejected {
            feedback: feedback_from_reject_input(trimmed),
        }
    }
}

fn starts_with_yes(input: &str) -> bool {
    input
        .chars()
        .next()
        .is_some_and(|first_char| first_char == 'Y' || first_char == 'y')
}

fn feedback_from_reject_input(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }
    let mut chars = input.chars();
    let Some(first_char) = chars.next() else {
        return None;
    };
    let rest = chars.as_str().trim();

    if first_char == 'N' || first_char == 'n' {
        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    } else {
        Some(input.to_string())
    }
}

fn append_unique_strings(target: &mut Vec<String>, source: Vec<String>) {
    for item in source {
        if !target.iter().any(|existing| existing == &item) {
            target.push(item);
        }
    }
}

pub fn normalize_command_for_match(command: &str) -> String {
    command
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn first_exact_match<'a>(command: &str, rules: &'a [String]) -> Option<&'a str> {
    rules.iter().find_map(|rule| {
        let normalized_rule = normalize_command_for_match(rule);
        if command == normalized_rule {
            Some(rule.as_str())
        } else {
            None
        }
    })
}

fn first_prefix_match<'a>(command: &str, rules: &'a [String]) -> Option<&'a str> {
    rules.iter().find_map(|rule| {
        let normalized_rule = normalize_command_for_match(rule);
        if command == normalized_rule || command.starts_with(&(normalized_rule + " ")) {
            Some(rule.as_str())
        } else {
            None
        }
    })
}

fn first_contains_match<'a>(command: &str, rules: &'a [String]) -> Option<&'a str> {
    rules.iter().find_map(|rule| {
        let normalized_rule = normalize_command_for_match(rule);
        if !normalized_rule.is_empty() && command.contains(&normalized_rule) {
            Some(rule.as_str())
        } else {
            None
        }
    })
}
