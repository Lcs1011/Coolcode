# ctool_command_request 产品方案

## 1. 产品定位

`ctool_command_request` 是 CoolCode / CoolTool 体系中的受控命令申请工具。

它不是 Codex 原生 Shell，也不是默认自由命令行能力。它属于 `ctool_` 工具体系，必须受到 SafeMode、系统配置、项目配置、风险分级、用户确认和日志审计的严格管制。

核心目标：

```text
在普通 CTool 文件工具无法完成任务时，
允许 Codex 申请执行一组完整命令；
但命令必须完整展示、按风险分级、必要时等待用户确认。
```

---

## 2. 三条铁律

### 2.1 能不用就不用

`ctool_command_request` 是保险工具，不是主力工具。

优先使用普通 CTool：

```text
ctool_read_file
ctool_rg_search
ctool_read_code_range
ctool_edit_replace
ctool_edit_insert
ctool_create_file
ctool_delete_file
ctool_move_file
...
```

只有普通 CTool 做不到时，才允许申请命令执行。

### 2.2 必须完整展示命令

所有要执行的命令必须逐条完整展示。

必须显示：

```text
CurrentDir
风险等级
命令列表
确认提示
```

禁止：

```text
执行一些检查命令
执行上面的命令
cargo check 等
省略参数
隐藏参数
模型自己补全但不展示
```

### 2.3 非绿色命令必须人工确认

Yellow 必须确认一次。

Red 必须确认两次。

Blocked 不允许进入确认执行流程。

模型不能替用户确认。

没有用户确认时，默认一律拒绝执行。

---

## 3. SafeMode 关系

`ctool_command_request` 绝对不能绕过 SafeMode。

执行链路：

```text
SafeMode ON
=> CoolReadWrite
=> CTool allowed
=> ctool_command_request
=> 加载 CoolSystemDir\command.toml 与 CoolDir\command.toml
=> 命令分类
=> 展示完整命令
=> 必要时等待用户确认
=> 执行或拒绝
=> 记录日志
=> 返回结果
```

它不是恢复 Codex 原生 Shell，而是一个受控 CTool。

---

## 4. 配置文件

配置分两层，沿用 Cool 配置体系：

```text
CoolSystemDir\command.toml   系统级，更高优先级
CoolDir\command.toml         Session 级，低一级
```

说明：

```text
CoolSystemDir = LauncherDir\.cool-system
CoolDir       = SessionRoot\.cool
```

系统配置权限更大。

系统配置可以强制把某些命令升为 Yellow / Red / Blocked，也可以禁止某些命令。

Session 配置可以添加自己的 Green / Yellow / Red 规则，但不能覆盖系统级更高风险规则。

---

## 5. 配置段设计

建议配置段名：

```toml
[ctool_command]
```

示例：

```toml
[ctool_command]
enabled = true

green_exact_commands = [
  "git status",
  "git diff"
]

green_prefixes = [
  "git add"
]

yellow_prefixes = [
  "cargo check",
  "cargo build",
  "cargo test",
  "cargo fmt",
  "git commit",
  "rg"
]

red_prefixes = [
  "del",
  "erase",
  "rmdir",
  "rd",
  "Remove-Item",
  "git reset --hard",
  "git clean -fd",
  "powershell",
  "cmd",
  "python",
  "node",
  "curl",
  "wget",
  "Invoke-WebRequest",
  "Invoke-RestMethod",
  "shutdown",
  "taskkill",
  "reg",
  "netsh",
  "git clone"
]

red_contains = [
  "http://",
  "https://",
  "ftp://",
  "download",
  ".exe",
  ".msi",
  ".dll",
  ".bat",
  ".cmd",
  ".ps1",
  ".sh",
  ".zip",
  ".rar",
  ".7z",
  ".tar",
  ".gz"
]

blocked_prefixes = [
  "python -m venv",
  "py -m venv",
  "pip install",
  "pip3 install",
  "python -m pip install",
  "py -m pip install",
  "conda install",
  "uv python install",
  "pyenv install"
]

blocked_contains = [
  "set PATH=",
  "$env:PATH",
  "PATH=",
  "Scripts\\activate",
  "activate.bat",
  "activate.ps1"
]
```

---

## 6. 风险等级

风险等级分四类：

```text
Green   = 用户白名单，免确认执行
Yellow  = 主力命令，确认一次
Red     = 高危 / 未知 / AI 主动升级，确认两次
Blocked = 硬拒绝命令，直接拒绝执行
```

风险等级顺序：

```text
Green < Yellow < Red < Blocked
```

Blocked 不进入确认流程，但请求本身仍然要完整展示并记录。

---

## 7. Green 规则

Green 不是“AI 判断安全”。

Green 是：

```text
用户明确写进配置的免审白名单。
```

默认可以没有 Green。

Green 命令可以免确认执行，但必须明显展示和记录日志。

建议支持两种 Green：

```text
green_exact_commands
完全相等才免确认。

green_prefixes
前缀匹配，谨慎使用。
```

例子：

```toml
green_exact_commands = [
  "git status",
  "git diff"
]
```

如果用户希望 `git add` 免审：

```toml
green_prefixes = [
  "git add"
]
```

注意：

```text
green_prefixes = ["git add"] 会放行 git add .
```

是否接受由用户决定，模型不能自己把命令加入 Green。

---

## 8. Yellow 规则

Yellow 是日常开发主力命令。

典型 Yellow：

```text
cargo check
cargo build
cargo test
cargo fmt
git commit
rg
```

Yellow 必须确认一次。


确认提示：

```text
Confirm? Type Y to run, N to reject:
```

只有首字母是 `Y` / `y` 才执行。

---

## 9. Red 规则

Red 是高危命令、未知命令、下载命令、打开网站命令、解释器命令、AI 主动升级命令。

典型 Red：

```text
del
erase
rmdir
rd

## 9.1 Blocked 规则

Blocked 是硬拒绝命令。

典型 Blocked：

```text
python -m venv
py -m venv
pip install
pip3 install
python -m pip install
py -m pip install
conda install
uv python install
pyenv install
修改 PATH
激活虚拟环境脚本
```

以下行为永远 Blocked：

```text
创建或安装新的 Python 运行环境
安装新的 Python 包依赖
修改 PATH 等进程环境路径
激活虚拟环境脚本
```

Blocked 请求必须展示，但不能进入执行。
Remove-Item
git reset --hard
git clean -fd
powershell
cmd
python
node
curl
wget
Invoke-WebRequest
Invoke-RestMethod
shutdown
taskkill
reg
netsh
git clone
```

以下行为永远 Red：

```text
下载内容
打开网站
启动浏览器
执行 shell / powershell / cmd
运行脚本解释器
下载外部仓库
删除文件/目录
修改系统配置
结束进程
关机
注册表操作
网络配置
```

Red 必须确认两次。

第一次：

```text
First confirm? Type Y:
```

第二次：

```text
Second confirm? Type Y:
```

Blocked 不允许确认执行。

---

## 10. 未知命令规则

未知命令默认 Red。

```text
没有命中 green_exact_commands
没有命中 green_prefixes
没有命中 yellow_prefixes
没有命中 red_prefixes
=> Red
```

这样可以避免模型用陌生命令绕过分类。

---

## 11. AI 风险升级规则

AI 可以提升命令危险级别，但不能降低危险级别。

```text
系统判断 Green，AI 可以升级为 Yellow / Red / Blocked。
系统判断 Yellow，AI 可以升级为 Red / Blocked。
系统判断 Red，AI 可以升级为 Blocked。
系统判断 Blocked，AI 不能降级。
```

最终风险：

```text
FinalRisk = max(SystemRisk, AIRiskUpgrade)
```

例子：

```text
cargo check 正常是 Yellow。
如果 AI 判断这次 cargo check 可能触发 build.rs 执行外部脚本，
AI 可以主动升级为 Red。
```

但模型不能这样做：

```text
系统判断 Red。
AI 说“我觉得安全”。
最终仍然 Red。
```

---

## 12. 批量命令规则

`ctool_command_request` 可以一次申请多条命令。

一组命令按顺序执行。

风险等级按最高风险计算：

```text
Green + Green = Green
Green + Yellow = Yellow
Yellow + Red = Red
Green + Red = Red
任意 + Blocked = Blocked
```

一组命令中只要有一个 Red，整组就是 Red，需要二次确认。

一组命令中只要有一个 Blocked，整组就是 Blocked，直接拒绝。

---


## 13. 命令展示格式

所有请求必须明显显示，方便回头查。

### 13.1 Green 请求

```text
==============================
🟢 COMMAND REQUEST: GREEN
CurrentDir: C:\CodexLab\codex\codex-rs
Auto Approved: green whitelist

[1] git status
[2] git diff
==============================
```

### 13.2 Yellow 请求

```text
==============================
🟡 COMMAND REQUEST: YELLOW
CurrentDir: C:\CodexLab\codex\codex-rs

[1] cargo check -p ctool
[2] cargo check -p codex-core

Confirm? Type Y to run, N to reject:
==============================
```

### 13.3 Red 请求

```text
==============================
🔴 COMMAND REQUEST: RED
CurrentDir: C:\CodexLab\codex\codex-rs

[1] powershell -Command "..."
[2] git reset --hard

First confirm? Type Y:
Second confirm? Type Y:
==============================
```

### 13.4 Blocked 请求

```text
==============================
🔴🔴🔴 COMMAND REQUEST: BLOCKED
CurrentDir: C:\CodexLab\codex\codex-rs
Blocked: hard policy

[1] python -m venv .venv
[2] pip install -r requirements.txt
==============================
```

---

## 14. 用户确认输入规则

确认输入采用首字母判断。

### 14.1 Yellow

```text
Y / y 开头 = 同意执行
N / n 开头 = 拒绝执行
其他任何输入 = 拒绝执行
空输入 = 拒绝执行
```

### 14.2 Red

第一次确认：

```text
Y / y 开头 = 进入第二次确认
其他任何输入 = 拒绝执行
空输入 = 拒绝执行
```

第二次确认：

```text
Y / y 开头 = 同意执行
其他任何输入 = 拒绝执行
空输入 = 拒绝执行
```

### 14.3 Blocked

```text
不进入确认流程
直接拒绝执行
仍然返回展示信息和拒绝原因
```

---

## 15. 拒绝时允许带反馈

这是本工具的重要体验设计。

用户可以输入：

```text
N 不要跑 cargo build，先只修改文件，然后让我外部 cargo check
```

处理方式：

```text
首字母 N => 拒绝执行
N 后面的内容 => 作为用户反馈返回给 AI
```

如果首字母无法识别，也按拒绝处理，并把完整输入作为反馈返回给 AI。

例子：

```text
算了，先别跑这个，换成 cargo check -p ctool
```

等价于：

```text
拒绝执行
反馈给 AI：算了，先别跑这个，换成 cargo check -p ctool
```

这样 AI 不会因为命令被拒绝而傻掉。

---

## 16. 执行规则

一组命令按顺序执行：

```text
[1] 成功，再执行 [2]
[2] 成功，再执行 [3]
中途失败，停止后续命令
```

每条命令记录：

```text
command
exit_code
stdout
stderr
start_time
end_time
duration
```

默认不允许后台静默运行。

---

## 17. 日志设计

建议日志目录：

```text
CoolDir\cache\command_request\YYYY-MM-DD\
```

示例：

```text
SessionRoot\.cool\cache\command_request\2026-06-09\
  request_log.md
  00000_yellow_cargo_check.md
  00001_green_git_status.md
  00002_red_powershell.md
  00003_blocked_python_venv.md
```

### 17.1 request_log.md

每次申请追加记录：

```markdown
# CTool Command Request Log

Date: 2026-06-09

## 00000

Time: 2026-06-09 15:30:12
Risk: Yellow
Approved: Yes
CurrentDir: C:\CodexLab\codex\codex-rs

Commands:

1. cargo check -p ctool
2. cargo check -p codex-core

Output:
00000_yellow_cargo_check.md

---


## 00001

Time: 2026-06-09 15:35:00
Risk: Red
Approved: No
CurrentDir: C:\CodexLab\codex\codex-rs

Commands:

1. powershell -Command "..."

User Feedback:
不要用 powershell，换成 cargo check。
```

### 17.2 单次结果文件

```markdown
# CTool Command Request Result

Risk: Yellow
Approved: Yes
CurrentDir: C:\CodexLab\codex\codex-rs
Time: 2026-06-09 15:30:12

## Commands

### 1. cargo check -p ctool

Exit Code:
0

Stdout:
...

Stderr:
...

### 2. cargo check -p codex-core

Exit Code:
101

Stdout:
...

Stderr:
...
```

日志目录建议加入 `protected_paths`，Codex 可以读，但不能改。


---

## 18. 输出返回给 Codex 的内容

不要把超长 stdout / stderr 全部塞回模型上下文。

工具返回给 Codex 的内容应控制长度：

```text
Command request completed.

Risk: Yellow
Approved: Yes

Output:
SessionRoot\.cool\cache\command_request\2026-06-09\00000_yellow_cargo_check.md

Summary:
cargo check -p ctool passed.
cargo check -p codex-core failed with 3 errors.

Suggested next step:
Read the output file and fix the first compile error.
```

Codex 需要详细日志时，再用 `ctool_read_file` / `ctool_read_code_range` 读结果文件。

---


## 19. 下载和打开网站规则

必须定成硬规则：

```text
所有下载内容相关命令 = Red
所有打开网站相关命令 = Red
```

如果同时命中 Blocked 规则，则按 Blocked 处理。

包括：

```text
curl
wget
Invoke-WebRequest
Invoke-RestMethod
Start-Process http://...
start http://...
explorer http://...
浏览器打开 URL
下载 zip / exe / msi / ps1 / bat / dll
git clone 外部仓库
```

`git clone` 也 Red，因为它会下载外部代码到本地，后续可能参与构建或执行。

---

## 20. 与 CToolScope 的关系

命令执行本身不能用普通文件 Scope 完全约束。

但至少要做到：

```text
CurrentDir 必须显示
默认执行目录是 CoolWorkspace
禁止静默切换目录
如果命令里包含 cd / pushd，至少 Red
如果命令目标明显超出 CurrentDir，至少 Red
```

未来可以进一步做：

```text
命令分析器识别文件路径
命令涉及文件路径时，调用 CToolScopeContext 判断
```

当前版本先完成风险分级、确认和执行链路，后续继续加强命令解析与 Scope 联动。

---


## 21. 推荐开发顺序

当前实现已经覆盖：

```text
配置加载
Green / Yellow / Red / Blocked 分类
Yellow 一次确认
Red 二次确认
Blocked 直接拒绝
真实执行命令
写 request_log.md
写单次结果文件
返回短摘要
```

后续加强方向：

```text
加强命令解析
补充更多 blocked_prefixes / blocked_contains
增强路径识别并接入 CToolScopeContext
优化日志摘要与拒绝原因文本
```

---

## 22. 最终采用版

本方案正式采用以下原则：

```text
ctool_command_request 是受 SafeMode 管制的命令申请工具。
普通 CTool 能完成时，不使用它。
所有命令必须完整展示。
Green 是用户白名单，免确认但必须展示和记录。
Yellow 是主力命令，确认一次。
Red 是高危、未知、下载、打开网站、AI 升级命令，确认两次。
Blocked 是硬拒绝命令，必须展示，但不能执行。
AI 可以升级风险，不能降级风险。
拒绝时允许用户带反馈，反馈返回给 AI。
一组命令按最高风险处理。
命令结果写入 CoolDir\cache\command_request\YYYY-MM-DD\。
工具只把结果文件路径和短摘要返回给 Codex。
Codex 通过普通 CTool 读取结果文件。
```
