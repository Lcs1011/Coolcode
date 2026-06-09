任务：修复 ctool_command_request 的显示问题。

背景：
现在 ctool_command_request 已经可以把 git status 识别为 GREEN，并且可以自动执行、写入 .coolcache\command_request\YYYY-MM-DD\。
但是用户界面里没有稳定显示大横幅。模型只是总结了一句：
“已通过 ctool_command_request 申请并执行 git status。”
这不符合产品设计。

产品要求：
即使是 GREEN 命令，不需要用户确认，也必须非常醒目地显示 COMMAND REQUEST 大横幅。
ctool_command_request 调用后，模型必须能直接拿到一个完整的 display_text，并原样贴给用户。

本次只修显示，不改变安全逻辑：

1. 不要让 Yellow / Red 执行。
2. 不要改 SafeMode。
3. 不要改 Scope。
4. 不要改命令分类规则。
5. 不要新增联网功能。
6. 不要恢复原生 shell。
7. 只改 ctool_command_request 的输出展示和工具说明。

需要修改的文件：

1. utils/ctool/src/tools/command/ctool_command_request.rs
2. core/src/tools/handlers/ctool_adapter.rs

第一步：修改 ctool_command_request.rs

打开：
utils/ctool/src/tools/command/ctool_command_request.rs

找到结构体：

pub struct CToolCommandRequestOutput {
    pub will_execute: bool;
    pub executed: bool;
    pub all_success: Option<bool>;
    pub result_file: Option<String>;
    pub log_file: Option<String>;
    ...
    pub banner: String;
    pub note: String;
}

实际代码里是逗号，不是分号。请按实际代码修改。

在 output 结构体里新增字段：

pub display_text: String,

建议放在 banner 前面：

pub commands: Vec<CToolCommandRequestCommandOutput>,

pub display_text: String,
pub banner: String,
pub note: String,

第二步：在 preview_command_request 里生成 display_text

在 preview_command_request(...) 函数中，现在已经有：

let banner = render_command_request_banner(&preview);

后面有：

let mut executed = false;
let mut all_success = None;
let mut result_file = None;
let mut log_file = None;
let mut note = ...

然后 Green 会执行，并填充 result_file / log_file。

请在构造 Ok(CToolCommandRequestOutput { ... }) 之前，增加：

let display_text = render_command_request_display_text(
    &banner,
    executed,
    all_success,
    result_file.as_deref(),
    log_file.as_deref(),
    &note,
);

注意：
result_file / log_file 后面还要放进输出结构体，所以这里必须用 as_deref()，不要 move 掉 Option<String>。

第三步：增加 helper 函数

在 ctool_command_request.rs 末尾，approval_label(...) 后面，添加：

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

第四步：把 display_text 放入输出

在 Ok(CToolCommandRequestOutput { ... }) 里加入：

display_text,

位置建议：

commands,

display_text,
banner,
note,

第五步：更新 CToolCommandRequest::spec() 的 description

当前 description 可能还说：
“but does not execute commands.”

请改成准确描述：

"Preview a controlled command execution request. It classifies command risk and renders the required approval banner. GREEN commands may auto-execute only when allowed by the user's whitelist; YELLOW and RED commands are not executed yet."

第六步：修改 core/src/tools/handlers/ctool_adapter.rs

打开：
core/src/tools/handlers/ctool_adapter.rs

找到 ctool_command_request 的说明分支。

把里面不准确的句子改掉。

重点要加入这几条硬规则：

1. After calling this tool, always paste output.display_text verbatim to the user.
2. Do not summarize display_text.
3. Do not omit the COMMAND REQUEST banner.
4. GREEN commands may auto-execute only when allowed by the user's whitelist.
5. YELLOW and RED commands never execute in this version.
6. If result_file or log_file exists, show them as part of output.display_text.

建议把 ctool_command_request 的说明整体调整为类似：

Input JSON:
{
  "commands": [
    "cargo check -p ctool",
    "cargo check -p codex-core"
  ],
  "ai_risk_upgrade": null,
  "reason": "Need compile errors after code changes."
}

Use this only when normal CTool file tools cannot complete the task.

This is a controlled command request tool.
It classifies every command as GREEN / YELLOW / RED, computes the highest batch risk,
renders a very visible COMMAND REQUEST banner, and reports whether execution happened.

Important display rule:
After every call, paste output.display_text verbatim to the user.
Do not summarize it.
Do not omit the COMMAND REQUEST banner.

Rules:

- Prefer normal CTool read/edit/file tools.
- List every command completely.
- Unknown commands are RED.
- Downloads and opening websites are RED.
- AI may upgrade risk, but cannot downgrade risk.
- GREEN commands may auto-execute only when allowed by the user's whitelist.
- YELLOW and RED commands never execute in this version.

第七步：检查

完成修改后，请不要运行任意 shell 命令。
你当前没有实际命令执行权限。

你只需要告诉用户：

1. 修改了哪些文件。
2. display_text 字段已经加入。
3. ctool_command_request 的工具说明已经要求原样贴出 display_text。
4. 建议用户在外部 CMD 执行以下检查：

cd /d C:\CodexLab\codex\codex-rs
cargo fmt
cargo test -p ctool
cargo check -p ctool
cargo check -p codex-core
cargo build --release --bin codex

第八步：不要做的事

不要实现 Yellow / Red 确认执行。
不要实现 TUI PendingCommandRequest。
不要改 .coolconfig.toml。
不要改 .coolsystemconfig.toml。
不要改 SafeMode。
不要改 CToolScope。
不要改其他工具。
