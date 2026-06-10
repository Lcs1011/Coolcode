# CTool 四层结构

## 第 1 层：SafeMode

SafeMode 采用 AboveAll 原则。

一旦 SafeMode 开启：

* 强制关闭所有 Codex 原生工具。
* 强制关闭 skills / plugins / connectors 等原生扩展系统。
* 只有 CTool 可以正常使用。
* 不允许通过非 CTool 通道读取、写入、执行、联网、安装或下载。
* 模型 API 和账号登录所必需的网络请求除外，其余网络请求必须被 SafeMode 审核或阻断。

SafeMode 必须在启动早期初始化，并作为全局最高安全开关。

---

## 第 2 层：PermissionProfile

增加 Cool 开头的 PermissionProfile。

当前核心模式：

```text
CoolReadWrite
```

Cool 开头的 PermissionProfile 只能使用 CTool 工具。

也就是说：

```text
CoolReadWrite = 只能通过 CTool 读写
```

不能直接使用 Codex 原生 shell、apply_patch、skills、plugins、connectors、原生文件搜索等工具。

CoolReadWrite 应作为 SafeMode 下的默认 PermissionProfile。

---

## 第 3 层：CToolScope

CToolScope 不是直接限制 PermissionProfile 的视野范围。

CToolScope 是直接限制 CTool 的视野范围。

由于 Cool 开头的 PermissionProfile 只能使用 CTool，所以只要 CToolScope 可靠，就可以实现对 CoolReadWrite 的绝对控制。

---

## CToolScopeBase

CToolScopeBase 决定 CToolScope 的基础视野。

枚举值：

```text
None
CoolWorkspace
SelectedOnly
TheEyeofProvidence
```

含义：

```text
None
= 默认无任何基础视野。

CoolWorkspace
= 使用 CoolWorkspace 作为基础视野。

SelectedOnly
= 不开放基础视野，只依赖显式配置。

TheEyeofProvidence
= 全局视野。暂时不启用。
```

CToolScopeBase 位于：

```text
SessionRoot\.cool\config.toml
```

如果没有设置，默认：

```text
CToolScopeBase = None
```

注意：

即使 CoolWorkspace 默认等于 SessionRoot，如果 CToolScopeBase 是 None，CTool 仍然没有基础视野。

只有设置：

```text
CToolScopeBase = CoolWorkspace
```

时，CTool 才把 CoolWorkspace 作为基础可见范围。

---

## SessionRoot

SessionRoot 指的是启动 Codex 时，CMD 当前所在的路径。

也就是 Codex 进程启动时绑定的会话根目录。

这个路径通常会显示在 Codex 界面最下方。

例如：

```text
SessionRoot = C:\CodexLab\codex
```

SessionRoot 下放置当前会话级 Cool 目录：

```text
SessionRoot\.cool
```

---

## LauncherDir

LauncherDir 指的是启动器所在目录。

我们目前暂时使用 BAT 启动 Codex，因此 LauncherDir 就是 `CodexRelease.bat` 所在的文件夹。

当前约定：

```text
LauncherDir = C:\Arsenal
```

LauncherDir 下放置系统级 Cool 目录：

```text
LauncherDir\.cool-system
```

也就是：

```text
C:\Arsenal\.cool-system
```

这样以后即使 BAT 从 `C:\Arsenal` 移动到别的位置，CoolSystemDir 仍然可以由 LauncherDir 推导出来，整体设计不依赖固定路径。

---

## CoolSystemDir

CoolSystemDir 是系统级 Cool 配置目录。

```text
CoolSystemDir = LauncherDir\.cool-system
```

里面存放：

```text
config.toml
scope.toml
command.toml
logs
state
```

系统级配置优先级高于 SessionRoot 级配置。

---

## CoolDir

CoolDir 是当前 SessionRoot 对应的 Cool 配置目录。

```text
CoolDir = SessionRoot\.cool
```

里面存放：

```text
config.toml
scope.toml
command.toml
cache
logs
state
```

CoolDir 对 Codex 模型不可见，也不能被普通 CTool 读写。

---

## CoolWorkspace

CoolWorkspace 是真正的项目工作区。

它位于：

```text
SessionRoot\.cool\config.toml
```

中设置。

如果没有设置：

```text
CoolWorkspace = SessionRoot
```

如果设置为相对路径：

```toml
[cool]
workspace = "codex-rs"
```

则：

```text
CoolWorkspace = SessionRoot\codex-rs
```

CTool 的默认工作范围和 `ctool_command_request` 的执行目录都应该基于 CoolWorkspace，而不是 SessionRoot。

---

## 启动时显示

启动 Codex 时，顶部或显眼位置应显示：

```text
SafeMode: ON
PermissionProfile: CoolReadWrite
CToolScopeBase: CoolWorkspace
CoolWorkspace: <path>
```

之后可以考虑在底部状态栏持续显示简短状态：

```text
Safe ON · CoolReadWrite · Scope:CoolWorkspace · WS:<path>
```

---

## 第 4 层：CTool

CTool 是 CoolReadWrite 唯一可用的工具系统。

制作 CTool 时必须考虑：

1. 工具是否只读、可写、可创建、可删除、可移动、可执行命令、可联网。
2. 每个工具都必须经过 CToolScope 权限判断。
3. 除 `CToolCommandRequest` 和未来明确批准的 `CToolTavilySearch` 外，普通 CTool 不允许联网、下载、编译可执行文件、运行可执行文件。
4. CToolCommandRequest 必须经过 CommandPolicy。
5. CTool 不能绕过 `.cool`、`.cool-system` 等硬保护目录。

---

## CTool 分类

### 读取工具

| 功能                  | Rust 类型名             | 文件名                        | 工具名                     |
| ------------------- | -------------------- | -------------------------- | ----------------------- |
| 查看目录结构              | `CToolListDirectory` | `ctool_list_directory.rs`  | `ctool_list_directory`  |
| 搜关键词 / 类型 / 函数 / 报错 | `CToolRgSearch`      | `ctool_rg_search.rs`       | `ctool_rg_search`       |
| 读取命中位置附近代码          | `CToolReadCodeRange` | `ctool_read_code_range.rs` | `ctool_read_code_range` |
| 读取完整小文件或配置文件        | `CToolReadFile`      | `ctool_read_file.rs`       | `ctool_read_file`       |

### 修改工具

| 功能            | Rust 类型名           | 文件名                     | 工具名                  |
| ------------- | ------------------ | ----------------------- | -------------------- |
| 单点精确替换        | `CToolEditReplace` | `ctool_edit_replace.rs` | `ctool_edit_replace` |
| 单点插入          | `CToolEditInsert`  | `ctool_edit_insert.rs`  | `ctool_edit_insert`  |
| 预览修改结果        | `CToolPreviewDiff` | `ctool_preview_diff.rs` | `ctool_preview_diff` |
| 多个替换 / 插入打包执行 | `CToolEditBatch`   | `ctool_edit_batch.rs`   | `ctool_edit_batch`   |

### 文件操作工具

| 功能          | Rust 类型名               | 文件名                         | 工具名                      |
| ----------- | ---------------------- | --------------------------- | ------------------------ |
| 创建文件        | `CToolCreateFile`      | `ctool_create_file.rs`      | `ctool_create_file`      |
| 删除文件        | `CToolDeleteFile`      | `ctool_delete_file.rs`      | `ctool_delete_file`      |
| 移动 / 重命名文件  | `CToolMoveFile`        | `ctool_move_file.rs`        | `ctool_move_file`        |
| 创建文件夹       | `CToolCreateDirectory` | `ctool_create_directory.rs` | `ctool_create_directory` |
| 删除文件夹       | `CToolDeleteDirectory` | `ctool_delete_directory.rs` | `ctool_delete_directory` |
| 移动 / 重命名文件夹 | `CToolMoveDirectory`   | `ctool_move_directory.rs`   | `ctool_move_directory`   |

### 特殊工具

| 功能          | Rust 类型名                | 文件名                          | 工具名                       |
| ----------- | ----------------------- | ---------------------------- | ------------------------- |
| Markdown 批注 | `CToolAnnotateMarkdown` | `ctool_annotate_markdown.rs` | `ctool_annotate_markdown` |
| 命令申请        | `CToolCommandRequest`   | `ctool_command_request.rs`   | `ctool_command_request`   |
| Tavily 搜索   | `CToolTavilySearch`     | `ctool_tavily_search.rs`     | `ctool_tavily_search`     |

---

## 实现阶段规划

### 第一阶段

创建并接入：

```text
CToolScopeBase
None
CoolWorkspace
SelectedOnly
TheEyeofProvidence
```

启动时显示：

```text
SafeMode
PermissionProfile
CToolScopeBase
CoolWorkspace
```

### 第二阶段

增加 PermissionProfile：

```text
CoolReadWrite
```

CoolReadWrite 默认不能调用 Codex 原生工具，只能使用 CTool。

### 第三阶段

将基础 CTool 一个一个注册给 CoolReadWrite。

### 第四阶段

迁移目录结构：

```text
旧：.coolconfig.toml
新：.cool\config.toml

旧：.coolcache
新：.cool\cache
```

### 第五阶段

接入系统级配置：

```text
LauncherDir\.cool-system\config.toml
LauncherDir\.cool-system\scope.toml
LauncherDir\.cool-system\command.toml
```

系统级规则优先于 SessionRoot 级规则。
