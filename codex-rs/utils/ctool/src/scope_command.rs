use std::fmt::Write as _;
use std::path::Path;
use std::path::PathBuf;

use crate::error::CToolError;
use crate::error::CToolResult;
use crate::scope::CToolScopeBase;
use crate::scope_config::CToolScopeConfig;
use crate::scope_config::load_optional_cool_config;
use crate::scope_context::CToolScopeContext;
use crate::scope_context::normalize_scope_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CToolScopeCommand {
    Show,
    UpdateRule {
        target: CToolScopeTarget,
        rule: CToolScopeRule,
        action: CToolScopeAction,
        path: PathBuf,
    },
    SetBaseScope(CToolScopeBase),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CToolScopeTarget {
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CToolScopeRule {
    Readwrite,
    Readonly,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CToolScopeAction {
    Add,
    Remove,
}

pub fn parse_ctool_scope_command(input: &str) -> CToolResult<CToolScopeCommand> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.is_empty() {
        return Err(CToolError::InvalidInput(
            "empty CToolScope command".to_string(),
        ));
    }

    let command_name = tokens[0].trim_start_matches('/').to_ascii_lowercase();

    if command_name != "cs" && command_name != "ctoolscope" {
        return Err(CToolError::InvalidInput(format!(
            "not a CToolScope command: {}",
            tokens[0]
        )));
    }

    if tokens.len() == 1 {
        return Ok(CToolScopeCommand::Show);
    }

    match tokens[1].to_ascii_lowercase().as_str() {
        "show" => {
            ensure_token_len(&tokens, 2)?;
            Ok(CToolScopeCommand::Show)
        }
        "base" => {
            ensure_token_len(&tokens, 3)?;
            let base_scope = parse_base_scope(tokens[2])?;
            Ok(CToolScopeCommand::SetBaseScope(base_scope))
        }
        _ => parse_scope_rule_command(&tokens),
    }
}

fn parse_scope_rule_command(tokens: &[&str]) -> CToolResult<CToolScopeCommand> {
    let mut index = 1;
    let target = if tokens
        .get(index)
        .is_some_and(|token| token.eq_ignore_ascii_case("f"))
    {
        index += 1;
        CToolScopeTarget::File
    } else {
        CToolScopeTarget::Folder
    };

    let rule = match tokens.get(index).map(|token| token.to_ascii_lowercase()) {
        Some(token) if token == "ro" => {
            index += 1;
            CToolScopeRule::Readonly
        }
        Some(token) if token == "hide" => {
            index += 1;
            CToolScopeRule::Hide
        }
        _ => CToolScopeRule::Readwrite,
    };

    let action = if tokens.get(index).is_some_and(|token| *token == "-") {
        index += 1;
        CToolScopeAction::Remove
    } else {
        CToolScopeAction::Add
    };

    let path = parse_path_from_tokens(tokens, index)?;

    Ok(CToolScopeCommand::UpdateRule {
        target,
        rule,
        action,
        path,
    })
}

pub fn handle_ctool_scope_command(
    command: CToolScopeCommand,
    ctx: &mut CToolScopeContext,
) -> CToolResult<String> {
    match command {
        CToolScopeCommand::Show => Ok(show_ctool_scope(ctx)),
        CToolScopeCommand::SetBaseScope(base_scope) => {
            ctx.base_scope = base_scope;
            save_session_base_scope(ctx, base_scope)?;
            Ok(format!(
                "CToolScopeBase set to {} for current session.",
                ctx.base_scope
            ))
        }
        CToolScopeCommand::UpdateRule {
            target,
            rule,
            action,
            path,
        } => {
            update_user_config(ctx, |config| {
                update_scope_rule(config, target, rule, action, path.clone())
            })?;
            Ok(format!(
                "{} {} {} path: {}",
                action.past_tense(),
                target.label(),
                rule.label(),
                path.display()
            ))
        }
    }
}

pub fn show_ctool_scope(ctx: &CToolScopeContext) -> String {
    let mut output = String::new();

    let _ = writeln!(output, "CToolScopeBase: {}", ctx.base_scope);
    let _ = writeln!(output, "SessionRoot: {}", ctx.session_root.display());
    let _ = writeln!(output, "CoolWorkspace: {}", ctx.cool_workspace.display());
    let _ = writeln!(
        output,
        "SessionConfig: {}",
        ctx.session_config_path.display()
    );
    let _ = writeln!(output, "SessionScope: {}", ctx.user_config_path.display());

    match &ctx.system_config_path {
        Some(path) => {
            let _ = writeln!(output, "SystemConfig: {}", path.display());
        }
        None => {
            let _ = writeln!(output, "SystemConfig: <none>");
        }
    }

    let _ = writeln!(output);
    let _ = writeln!(output, "[Session files]");
    write_path_list(&mut output, "readwrite", &ctx.user_config.files.readwrite);
    write_path_list(&mut output, "readonly", &ctx.user_config.files.readonly);
    write_path_list(&mut output, "hide", &ctx.user_config.files.hide);

    let _ = writeln!(output);
    let _ = writeln!(output, "[Session folders]");
    write_path_list(&mut output, "readwrite", &ctx.user_config.folders.readwrite);
    write_path_list(&mut output, "readonly", &ctx.user_config.folders.readonly);
    write_path_list(&mut output, "hide", &ctx.user_config.folders.hide);

    let _ = writeln!(output);
    let _ = writeln!(output, "[System files]");
    write_path_list(&mut output, "readwrite", &ctx.system_config.files.readwrite);
    write_path_list(&mut output, "readonly", &ctx.system_config.files.readonly);
    write_path_list(&mut output, "hide", &ctx.system_config.files.hide);

    let _ = writeln!(output);
    let _ = writeln!(output, "[System folders]");
    write_path_list(
        &mut output,
        "readwrite",
        &ctx.system_config.folders.readwrite,
    );
    write_path_list(&mut output, "readonly", &ctx.system_config.folders.readonly);
    write_path_list(&mut output, "hide", &ctx.system_config.folders.hide);

    output
}

impl CToolScopeTarget {
    fn label(self) -> &'static str {
        match self {
            CToolScopeTarget::File => "file",
            CToolScopeTarget::Folder => "folder",
        }
    }
}

impl CToolScopeRule {
    fn label(self) -> &'static str {
        match self {
            CToolScopeRule::Readwrite => "readwrite",
            CToolScopeRule::Readonly => "readonly",
            CToolScopeRule::Hide => "hide",
        }
    }
}

impl CToolScopeAction {
    fn past_tense(self) -> &'static str {
        match self {
            CToolScopeAction::Add => "Added",
            CToolScopeAction::Remove => "Removed",
        }
    }
}

pub fn add_visible_path(config: &mut CToolScopeConfig, path: PathBuf) -> bool {
    update_scope_rule(
        config,
        CToolScopeTarget::Folder,
        CToolScopeRule::Readwrite,
        CToolScopeAction::Add,
        path,
    )
}

pub fn remove_visible_path(config: &mut CToolScopeConfig, path: &Path) -> bool {
    update_scope_rule(
        config,
        CToolScopeTarget::Folder,
        CToolScopeRule::Readwrite,
        CToolScopeAction::Remove,
        path.to_path_buf(),
    )
}

pub fn add_hide_path(config: &mut CToolScopeConfig, path: PathBuf) -> bool {
    update_scope_rule(
        config,
        CToolScopeTarget::Folder,
        CToolScopeRule::Hide,
        CToolScopeAction::Add,
        path,
    )
}

pub fn remove_hide_path(config: &mut CToolScopeConfig, path: &Path) -> bool {
    update_scope_rule(
        config,
        CToolScopeTarget::Folder,
        CToolScopeRule::Hide,
        CToolScopeAction::Remove,
        path.to_path_buf(),
    )
}

pub fn add_protected_path(config: &mut CToolScopeConfig, path: PathBuf) -> bool {
    update_scope_rule(
        config,
        CToolScopeTarget::Folder,
        CToolScopeRule::Readonly,
        CToolScopeAction::Add,
        path,
    )
}

pub fn remove_protected_path(config: &mut CToolScopeConfig, path: &Path) -> bool {
    update_scope_rule(
        config,
        CToolScopeTarget::Folder,
        CToolScopeRule::Readonly,
        CToolScopeAction::Remove,
        path.to_path_buf(),
    )
}

fn update_scope_rule(
    config: &mut CToolScopeConfig,
    target: CToolScopeTarget,
    rule: CToolScopeRule,
    action: CToolScopeAction,
    path: PathBuf,
) -> bool {
    let paths = match (target, rule) {
        (CToolScopeTarget::File, CToolScopeRule::Readwrite) => &mut config.files.readwrite,
        (CToolScopeTarget::File, CToolScopeRule::Readonly) => &mut config.files.readonly,
        (CToolScopeTarget::File, CToolScopeRule::Hide) => &mut config.files.hide,
        (CToolScopeTarget::Folder, CToolScopeRule::Readwrite) => &mut config.folders.readwrite,
        (CToolScopeTarget::Folder, CToolScopeRule::Readonly) => &mut config.folders.readonly,
        (CToolScopeTarget::Folder, CToolScopeRule::Hide) => &mut config.folders.hide,
    };

    match action {
        CToolScopeAction::Add => add_unique_path(paths, path),
        CToolScopeAction::Remove => remove_path(paths, &path),
    }
}

pub fn save_current_dir_cool_config(path: &Path, config: &CToolScopeConfig) -> CToolResult<()> {
    let text = toml::to_string_pretty(config).map_err(|error| {
        CToolError::InvalidInput(format!("failed to serialize Cool scope TOML: {error}"))
    })?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            CToolError::InvalidInput(format!(
                "failed to create Cool config directory: {} ({error})",
                parent.display()
            ))
        })?;
    }

    std::fs::write(path, text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to save Cool config file: {} ({error})",
            path.display()
        ))
    })
}

fn save_session_base_scope(ctx: &CToolScopeContext, base_scope: CToolScopeBase) -> CToolResult<()> {
    let path = &ctx.session_config_path;
    let mut value = if path.exists() {
        let text = std::fs::read_to_string(path).map_err(|error| {
            CToolError::InvalidInput(format!(
                "failed to read Cool session config file: {} ({error})",
                path.display()
            ))
        })?;
        toml::from_str::<toml::Value>(&text).map_err(|error| {
            CToolError::InvalidInput(format!(
                "failed to parse Cool session config file: {} ({error})",
                path.display()
            ))
        })?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let Some(table) = value.as_table_mut() else {
        return Err(CToolError::InvalidInput(format!(
            "Cool session config must be a TOML table: {}",
            path.display()
        )));
    };

    table.insert(
        "ctool_scope_base".to_string(),
        toml::Value::String(base_scope.as_str().to_string()),
    );

    let text = toml::to_string_pretty(&value).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to serialize Cool session config TOML: {error}"
        ))
    })?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            CToolError::InvalidInput(format!(
                "failed to create Cool config directory: {} ({error})",
                parent.display()
            ))
        })?;
    }

    std::fs::write(path, text).map_err(|error| {
        CToolError::InvalidInput(format!(
            "failed to save Cool session config file: {} ({error})",
            path.display()
        ))
    })
}

fn update_user_config(
    ctx: &mut CToolScopeContext,
    update: impl FnOnce(&mut CToolScopeConfig) -> bool,
) -> CToolResult<()> {
    let mut raw_config = load_optional_cool_config(&ctx.user_config_path)?;
    let _changed = update(&mut raw_config);

    save_current_dir_cool_config(&ctx.user_config_path, &raw_config)?;

    ctx.user_config = normalize_scope_config(raw_config, &ctx.cool_workspace);

    Ok(())
}

fn parse_base_scope(value: &str) -> CToolResult<CToolScopeBase> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Ok(CToolScopeBase::None),
        "workspace" | "coolworkspace" | "cool_workspace" | "cool-workspace" => {
            Ok(CToolScopeBase::CoolWorkspace)
        }
        "selectedonly" | "selected_only" | "selected-only" => Ok(CToolScopeBase::SelectedOnly),
        "theeyeofprovidence" | "the_eye_of_providence" | "the-eye-of-providence" => {
            Ok(CToolScopeBase::TheEyeofProvidence)
        }
        _ => Err(CToolError::InvalidInput(format!(
            "unsupported CToolScopeBase: {value}"
        ))),
    }
}

fn parse_path_from_tokens(tokens: &[&str], start_index: usize) -> CToolResult<PathBuf> {
    if start_index >= tokens.len() {
        return Err(CToolError::InvalidInput(
            "missing path argument".to_string(),
        ));
    }

    let path = tokens[start_index..].join(" ");
    let path = path.trim();

    if path.is_empty() {
        return Err(CToolError::InvalidInput(
            "missing path argument".to_string(),
        ));
    }

    Ok(PathBuf::from(path))
}

fn ensure_token_len(tokens: &[&str], expected_len: usize) -> CToolResult<()> {
    if tokens.len() == expected_len {
        Ok(())
    } else {
        Err(CToolError::InvalidInput(format!(
            "unexpected argument count: expected {}, got {}",
            expected_len,
            tokens.len()
        )))
    }
}

fn add_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) -> bool {
    if paths.iter().any(|existing| existing == &path) {
        false
    } else {
        paths.push(path);
        true
    }
}

fn remove_path(paths: &mut Vec<PathBuf>, path: &Path) -> bool {
    let old_len = paths.len();
    paths.retain(|existing| existing != path);
    paths.len() != old_len
}

fn write_path_list(output: &mut String, label: &str, paths: &[PathBuf]) {
    let _ = writeln!(output, "{label}:");

    if paths.is_empty() {
        let _ = writeln!(output, "  []");
        return;
    }

    for path in paths {
        let _ = writeln!(output, "  - {}", path.display());
    }
}
