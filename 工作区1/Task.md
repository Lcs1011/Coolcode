# 一、先搜索当前相关代码

先搜索这些关键词，确认所有相关入口：

```text
CTOOL_COMMAND_REQUEST_TOOL_NAME
CToolCommandRisk
CToolCommandApproval
CToolCommandConfig
classify_command
build_command_request_preview
render_command_request_banner
execute_approved_command_request
parse_yellow_confirmation_input
parse_red_first_confirmation_input
parse_red_second_confirmation_input
load_merged_cool_command_config
command_request_cache_dir
ctool_command_request
YELLOW and RED commands never execute
COOL_SYSTEM_CONFIG
COOL_SYSTEM_DIR
```

重点文件：

```text
codex-rs/utils/ctool/src/command_request.rs
codex-rs/utils/ctool/src/scope_config.rs
codex-rs/utils/ctool/src/scope_context.rs
codex-rs/utils/ctool/src/context.rs
codex-rs/utils/ctool/src/tools/command/ctool_command_request.rs
codex-rs/core/src/tools/handlers/ctool_adapter.rs
```

---

# 二、统一新目录标准

当前新标准：

```text
LauncherDir
= BAT 所在目录

CoolSystemDir
= LauncherDir\.cool-system

SessionRoot
= CMD 启动 Codex 时所在目录

CoolDir
= SessionRoot\.cool

CoolWorkspace
= 真正工作区
```

Command Request 配置文件必须改成：

```text
系统级：
CoolSystemDir\command.toml

Session 级：
CoolDir\command.toml
```

缓存和日志必须改成：

```text
CoolDir\cache\command_request\YYYY-MM-DD\
```

不要再写：

```text
.coolcache\command_request
```

---

# 三、修改 scope_config.rs

文件：

```text
codex-rs/utils/ctool/src/scope_config.rs
```

目标：

增加 command.toml 路径定位函数。

需要有：

```rust
pub const COMMAND_FILE_NAME: &str = "command.toml";
pub const COOL_SYSTEM_DIR_ENV: &str = "COOL_SYSTEM_DIR";
```

保留兼容：

```rust
pub const COOL_SYSTEM_CONFIG_ENV: &str = "COOL_SYSTEM_CONFIG";
```

新增函数：

```rust
pub fn locate_cool_command_path(session_root: impl AsRef<Path>) -> PathBuf {
    locate_cool_dir(session_root).join(COMMAND_FILE_NAME)
}
```

新增系统目录函数：

```rust
pub fn locate_cool_system_dir() -> Option<PathBuf> {
    let value = std::env::var(COOL_SYSTEM_DIR_ENV).ok()?;
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}
```

新增：

```rust
pub fn locate_cool_system_config_path() -> Option<PathBuf> {
    if let Some(system_dir) = locate_cool_system_dir() {
        return Some(system_dir.join(CONFIG_FILE_NAME));
    }

    locate_legacy_cool_system_config_path()
}
```

把旧的 `locate_cool_system_config_path()` 改名为：

```rust
pub fn locate_legacy_cool_system_config_path() -> Option<PathBuf>
```

新增：

```rust
pub fn locate_cool_system_scope_path() -> Option<PathBuf> {
    locate_cool_system_dir().map(|dir| dir.join(SCOPE_FILE_NAME))
}
```

新增：

```rust
pub fn locate_cool_system_command_path() -> Option<PathBuf> {
    locate_cool_system_dir().map(|dir| dir.join(COMMAND_FILE_NAME))
}
```

注意：

`COOL_SYSTEM_CONFIG` 只是旧兼容路径。新标准优先使用：

```text
COOL_SYSTEM_DIR
```

程序应从 `COOL_SYSTEM_DIR\config.toml`、`COOL_SYSTEM_DIR\scope.toml`、`COOL_SYSTEM_DIR\command.toml` 推导配置。

---

# 四、修改 CToolScopeContext

文件：

```text
codex-rs/utils/ctool/src/scope_context.rs
```

目标：

让 `CToolScopeContext` 保存 command 配置路径。

在结构体里新增：

```rust
pub session_command_path: PathBuf,
pub system_command_path: Option<PathBuf>,
```

在 `build_ctool_scope_context(...)` 中：

1. 计算：

```rust
let session_command_path = locate_cool_command_path(&session_root);
```

2. 从传入或系统定位中得到：

```rust
let system_command_path = locate_cool_system_command_path();
```

3. 填入结构体。

如果当前函数参数 `_system_config_path` 只表示旧系统 config 路径，不要把它当 command.toml 使用。

---

# 五、修改 CToolContext 初始化

文件：

```text
codex-rs/utils/ctool/src/context.rs
```

目标：

使用新系统目录定位逻辑。

当前 `workspace()` / `none()` 里如果只传了 `locate_cool_system_config_path()`，需要确认 scope 和 command 各自能从 `COOL_SYSTEM_DIR` 推导。

最终效果：

```text
COOL_SYSTEM_DIR=C:\Arsenal\.cool-system
```

时，程序能找到：

```text
C:\Arsenal\.cool-system\config.toml
C:\Arsenal\.cool-system\scope.toml
C:\Arsenal\.cool-system\command.toml
```

---

# 六、修改 command_request.rs：增加 Blocked

文件：

```text
codex-rs/utils/ctool/src/command_request.rs
```

## 6.1 风险等级

把：

```rust
pub enum CToolCommandRisk {
    Green,
    Yellow,
    Red,
}
```

改成：

```rust
pub enum CToolCommandRisk {
    Green,
    Yellow,
    Red,
    Blocked,
}
```

确保排序是：

```text
Green < Yellow < Red < Blocked
```

`Blocked` 必须最高。

## 6.2 icon / label

增加：

```rust
CToolCommandRisk::Blocked => "⛔"
```

label：

```rust
CToolCommandRisk::Blocked => "BLOCKED"
```

## 6.3 approval

给 `CToolCommandApproval` 增加：

```rust
Blocked
```

风险对应：

```rust
CToolCommandRisk::Blocked => CToolCommandApproval::Blocked
```

## 6.4 配置结构

给 `CToolCommandConfig` 增加：

```rust
pub blocked_prefixes: Vec<String>,
pub blocked_contains: Vec<String>,
```

Default 中加入 Python 环境安装硬封锁规则。

blocked_prefixes 至少包含：

```text
python -m venv
py -m venv
python3 -m venv
virtualenv
python -m virtualenv
py -m virtualenv
pip install
pip3 install
python -m pip install
py -m pip install
python3 -m pip install
pipx install
uv venv
uv python install
uv tool install
rye
hatch env
pipenv install
poetry install
poetry env
conda create
conda install
mamba create
mamba install
micromamba create
micromamba install
winget install python
choco install python
scoop install python
msiexec
python -m ensurepip
py -m ensurepip
```

blocked_contains 至少包含：

```text
python.org
install python
python installer
python-3.
python3.
venv
virtualenv
ensurepip
pip install
conda create
conda install
pyenv
pyenv-win
AppData\Local\Programs\Python
C:\Python
Program Files\Python
WindowsApps\python
setx PATH
set PATH
```

## 6.5 merge_command_configs

合并时加入：

```rust
blocked_prefixes
blocked_contains
```

系统级和 session 级都能添加 blocked 规则。

## 6.6 classify_command 优先级

分类优先级必须是：

```text
blocked_contains
blocked_prefixes
red_contains
red_prefixes
green_exact_commands
green_prefixes
yellow_prefixes
unknown => red
```

也就是说，只要命中 Blocked，后面 Green / Yellow / Red 都不能覆盖它。

## 6.7 AI 风险升级

`ai_risk_upgrade` 可以升级到 Red 或 Blocked。

规则：

```text
FinalRisk = max(SystemRisk, AIRiskUpgrade)
```

但不能降级。

如果系统判断 Blocked，最终必须仍然 Blocked。

## 6.8 banner

`render_command_request_banner` 增加 Blocked 展示。

Blocked 横幅示例：

```text
==============================
⛔ COMMAND REQUEST: BLOCKED
CurrentDir: C:\CodexLab\codex\codex-rs

[1] [BLOCKED] python -m venv .venv
    reason: matched blocked prefix rule: python -m venv

Blocked: this command is forbidden by Cool system policy.
No confirmation is allowed.
It will never be executed.
==============================
```

## 6.9 execute 防御

`execute_approved_command_request` 开头必须检查：

```rust
if preview.final_risk == CToolCommandRisk::Blocked {
    return Err(CToolError::InvalidInput(
        "blocked command request cannot be executed".to_string(),
    ));
}
```

即使外部错误传入 approval，也不能执行。

---

# 七、修改 ctool_command_request.rs：支持确认执行

文件：

```text
codex-rs/utils/ctool/src/tools/command/ctool_command_request.rs
```

当前工具只接收：

```rust
commands
ai_risk_upgrade
reason
```

需要增加字段：

```rust
pub approval_input: Option<String>,
```

语义：

```text
Green:
  不需要 approval_input，自动执行。

Yellow:
  没有 approval_input => 只预览，不执行。
  approval_input 首字母 Y/y => 执行。
  其他输入 => 拒绝，记录反馈，不执行。

Red:
  不允许单次 approval_input 直接执行。
  需要 red_first_confirmation 和 red_second_confirmation 两个字段。
```

更推荐字段：

```rust
pub yellow_confirmation: Option<String>,
pub red_first_confirmation: Option<String>,
pub red_second_confirmation: Option<String>,
```

执行规则：

```text
Green:
  自动执行。

Yellow:
  yellow_confirmation 是 Y/y 开头 => 执行。
  没有 yellow_confirmation => 预览等待确认。
  其他输入 => 拒绝，不执行，反馈给 AI。

Red:
  red_first_confirmation 是 Y/y 开头，并且 red_second_confirmation 是 Y/y 开头 => 执行。
  否则拒绝或等待确认。
```

Blocked：

```text
永远不执行。
不接受确认。
display_text 必须说明 blocked。
```

注意：

如果 Athena 判断 TUI 里已经有更适合的 PendingCommandRequest 机制，可以改为 TUI pending state。  
但如果没有，就先用上面的字段方式完成第一版。

## 输出字段

输出里增加：

```rust
pub blocked: bool,
pub rejected: bool,
pub user_feedback: Option<String>,
```

`approval_required` 对应：

```text
none_green_auto_approved
confirm_once
confirm_twice
blocked
```

## display_text

必须始终包含完整 banner。

必须明确显示：

```text
executed: true/false
blocked: true/false
rejected: true/false
result_file
log_file
note
```

---

# 八、迁移缓存目录

文件：

```text
codex-rs/utils/ctool/src/command_request.rs
```

把：

```text
current_dir\.coolcache\command_request\YYYY-MM-DD
```

改成：

```text
SessionRoot\.cool\cache\command_request\YYYY-MM-DD
```

注意：

`execute_approved_command_request` 当前只收 `current_dir` 和 `preview`，不够用了。

建议改成：

```rust
pub fn execute_approved_command_request(
    current_dir: impl AsRef<Path>,
    cache_root: impl AsRef<Path>,
    preview: &CToolCommandRequestPreview,
) -> CToolResult<CToolCommandExecutionReport>
```

或者传入：

```rust
ctx: &CToolContext
```

推荐更简单：

在 `ctool_command_request.rs` 中计算：

```rust
let cache_root = ctx.scope_context.session_root
    .join(".cool")
    .join("cache")
    .join("command_request");
```

然后传给 core 函数。

结果目录：

```text
.cool\cache\command_request\YYYY-MM-DD
```

---

# 九、命令执行 CurrentDir

命令执行目录必须是：

```text
CoolWorkspace
```

不是 SessionRoot。

也就是说：

```rust
ctx.scope_context.cool_workspace
```

作为 `CurrentDir` 和 `Command::current_dir(...)`。

这一点必须保持。

---

# 十、修改 ctool_adapter.rs 工具说明

文件：

```text
codex-rs/core/src/tools/handlers/ctool_adapter.rs
```

把 `ctool_command_request` 说明更新：

必须说明：

```text
It classifies every command as GREEN / YELLOW / RED / BLOCKED.
GREEN may auto-execute only by whitelist.
YELLOW executes only after user confirmation.
RED executes only after two user confirmations.
BLOCKED never executes and cannot be confirmed.
After every call, paste output.display_text verbatim to the user.
Do not summarize it.
```

删除旧句：

```text
YELLOW and RED commands never execute in this version.
```

增加 Python 禁止说明：

```text
Never request Python installation, Python venv creation, pip install, conda environment creation, uv python install, pyenv, or Windows PATH changes for Python. These are BLOCKED.
```

---