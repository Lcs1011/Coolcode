任务：寻找并关闭 `.codex` 自动生成 / 自动加载逻辑。

背景：

当前项目根目录是：

`C:\CodexLab\codex`

当前 Codex 启动后，项目根目录下存在一个：

`.codex`

它内部结构大致是：

`.codex\environments\environment.toml`  
`.codex\environments\setup.py`  
`.codex\skills\...\SKILL.md`  
`.codex\skills\...\agents\openai.yaml`  
`.codex\skills\...\scripts\*.py`

这些文件不是我们 CoolCode 主线需要的功能。我们不希望 Codex 在启动时自动生成、自动加载、自动执行 `.codex` 里的任何内容。

核心目标：

找到源码中是否存在以下逻辑：

1. 启动时检查 LaunchDir 下是否存在 `.codex`。

2. 如果 `.codex` 不存在，就自动创建 `.codex`。

3. 自动写入 `.codex\environments` 或 `.codex\skills`。

4. 自动加载 `.codex\skills\**\SKILL.md`。

5. 自动读取 `.codex\skills\**\agents\openai.yaml`。

6. 自动执行 `.codex\environments\setup.py` 或 `.codex\skills\**\scripts\*.py`。

7. 任何与 `/skills`、skills registry、skills bootstrap、environment setup 有关的启动流程。

第一阶段：只分析，不修改。

请全仓库搜索以下关键词：

`.codex`  
`SKILL.md`  
`openai.yaml`  
`setup.py`  
`skills`  
`environments`  
`babysit-pr`  
`codex-issue-digest`  
`collect_issue_digest`  
`gh_pr_watch`  
`include_dir`  
`include_str`  
`include_bytes`  
`create_dir_all`  
`std::fs::write`  
`write_all`  
`copy`  
`copy_dir`  
`bootstrap`  
`skill`  
`Skill`  
`skills_dir`  
`environment.toml`

重点检查这些目录：

`codex-rs\cli`  
`codex-rs\tui`  
`codex-rs\core`  
`codex-rs\app-server`  
`codex-rs\codex-cli`  
`codex-rs\codex-login`  
`codex-rs\utils`

允许使用的方式：

优先使用 CTool 的代码搜索/读取工具。  
如果必须申请命令，只允许申请 `rg` 搜索命令。  
不要申请 `python`、`powershell`、`cmd`、`curl`、`wget`、`git clone`。  
不要执行 `.codex` 里的任何 `.py` 文件。  
不要运行 `.codex` 里的任何脚本。  
不要删除、移动、重命名 `.codex`。

第一阶段输出要求：

请输出：

1. 所有相关命中的文件路径和行号。

2. 判断是否存在 `.codex` 自动生成逻辑。

3. 判断是否存在 `.codex\skills` 自动加载逻辑。

4. 判断是否存在 `.py` 自动执行逻辑。

5. 如果没有找到，也要明确说明“没有在当前源码中找到明确入口”。

6. 如果找到入口，请先说明准备如何关闭，不要立刻修改。

第二阶段：在我确认后再修改。

如果找到明确入口，修改目标是：

在 SafeMode ON 且 PermissionProfile 为 CoolReadWrite 时：

1. 不生成 `.codex`。

2. 不加载 LaunchDir 下的 `.codex\skills`。

3. 不读取 `.codex\skills\**\agents\openai.yaml`。

4. 不执行 `.codex\environments\setup.py`。

5. 不执行 `.codex\skills\**\scripts\*.py`。

6. 如果用户输入 `/skills`，应显示 disabled by CoolReadWrite 或类似说明，而不是加载 `.codex`。

修改要求：

只修改必要的 Rust 源码。  
不要修改 `.codex` 目录内容。  
不要删除 `.codex`。  
不要新增联网能力。  
不要新增 Python 执行能力。  
不要改 CTool 读写工具。  
不要改 ctool_command_request 的风险规则。  
不要破坏 SafeMode 默认开启逻辑。

最终修改后，请告诉我：

1. 修改了哪些文件。

2. 每个文件为什么要改。

3. 哪个函数是入口。

4. 新逻辑如何保证 `.codex` 不会生成/加载/执行。

5. 外部需要运行哪些验证命令。
