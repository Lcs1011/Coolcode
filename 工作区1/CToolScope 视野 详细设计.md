


# 基础视野


基础视野 由  CToolScopeBase 枚举变量 决定

CToolScopeBase 的默认值位于
仅位于  SessionRoot   的 .cool 中 config.toml


如果没有 找到默认值 则默认为 none

 None 为基础视野
 即使 CoolWorkspace 默认等于 SessionRoot，
 CTool 也仍然没有视野；
 只有 CToolScopeBase = CoolWorkspace 时，才使用 CoolWorkspace 路径作为基础视野

# 视野设置

CToolScopeBase枚举 位于 SessionRoot  .cool 中的  config.toml  设置
CoolWorkspace  路径 也位于 SessionRoot  .cool 中的  config.toml  设置



## 视野详细设置
LauncherDir  .cool-system  中 scope.toml
和 SessionRoot  .cool 中 scope.toml


其中有 以下分段 分别的决定 System级别 的 和SessionRoot 级别 的视野设置。

```
[files]
readwrite = []
readonly = []
hide = []


[folders]
readwrite = []
readonly = []
hide = [
  ".codex",
  ".cool"
]
```





## 视野设置权限

### 原则概述

系统 大于 Session

文件大于文件夹 

注：“文件规则优先于文件夹规则”是否符合预期。这意味着可以通过文件级规则对隐藏文件夹里的单个文件做例外放行
但是SessionRoot 级别 无法越过 System级别

禁止大于开放  （hide > readonly > readwrite）

都大于 CToolScopeBase


### 详细判定路径：
System 和 SessionRoot 表示两个权限级别

System.filehide>System.fileReadOnly>System.filereadwrite >
System.folderhide>System.folderReadOnly>System.folderreadwrite >
SessionRoot.filehide>SessionRoot.fileReadOnly>SessionRoot.filereadwrite >
SessionRoot.folderhide>SessionRoot.folderReadOnly>SessionRoot.folderreadwrite >
CToolScopeBase






#### 加载顺序

1. main 初始化 SafeMode
2. 解析 PermissionProfile
3. 解析 CToolScopeBase
4. 获取 SessionRoot，也就是当前 CMD 调用 codex 的文件夹

5. 加载 LauncherDir\.cool-system\config.toml //暂时位于启动 bat相同文件夹  找不到所有内容按默认空算

	- 找不到：按空配置  
	- 格式错误：CTool 不启用，报错  
6. 加载 SessionRoot\.cool\config.toml
	- 找不到：按空配置  
	- 格式错误：CTool 不启用，报错  
7. 构造 CToolScopeContext  
8. 构造 CToolContext  
9. 注册 CTool




#### 视野操作 相关命令

/cs  为 CToolScope 简写
f = file
不带 f = folder

可以操作配置的命令，都是操作 session 级别的。

```
//向Session设置 读写 只读 隐藏 添加 文件 路径
  /cs f <path>
  /cs f ro <path>
  /cs f hide <path>
  
//从Session设置 读写 只读 隐藏 移除 文件 路径
  /cs f - <path>
  /cs f ro - <path>
  /cs f hide - <path>

//向Session设置 读写 只读 隐藏 添加 文件夹 路径
  /cs <path>
  /cs ro <path>

  /cs hide <path>
  
//从Session设置 读写 只读 隐藏 移除 文件夹 路径
  /cs - <path>
  /cs ro - <path>
  /cs hide - <path>
  
//设置基础视野枚举
  /cs base none
  /cs base coolworkspace
  
//显示视野配置相关信息: 枚举、Session设置、系统视野设置
/cs show

```



# 视野相关工具函数

## locate_cool_dir

参数：
- `session_root`：SessionRoot 路径。

返回值：
- `PathBuf`：`SessionRoot\.cool` 的完整路径。

功能：
- 定位当前 session 的 `.cool` 目录。
- 只负责拼出路径，不负责判断目录是否存在。

## locate_cool_config_path

参数：
- `session_root`：SessionRoot 路径。

返回值：
- `PathBuf`：`SessionRoot\.cool\config.toml` 的完整路径。

功能：
- 定位 session 级别的基础配置文件。
- 这里读取 `CToolScopeBase` 和 `CoolWorkspace`。

## locate_cool_scope_path

参数：
- `session_root`：SessionRoot 路径。

返回值：
- `PathBuf`：`SessionRoot\.cool\scope.toml` 的完整路径。

功能：
- 定位 session 级别的视野详细配置文件。
- 这里读取 `[files]` 和 `[folders]` 下面的 `readwrite`、`readonly`、`hide` 列表。

## load_optional_cool_session_config

参数：
- `path`：session 配置文件路径，通常是 `SessionRoot\.cool\config.toml`。

返回值：
- `Result<Option<CToolSessionConfig>>`

功能：
- 尝试读取 session 基础配置。
- 文件不存在时返回 `Ok(None)`。
- 文件存在但格式错误时返回错误。

## parse_cool_session_config_toml

参数：
- `content`：`config.toml` 的文本内容。

返回值：
- `Result<CToolSessionConfig>`

功能：
- 解析 session 基础配置。
- 负责把 `ctool_scope_base` 和 `cool_workspace` 转成程序内部结构。

## load_optional_cool_config

参数：
- `path`：视野详细配置文件路径，通常是 `SessionRoot\.cool\scope.toml`。

返回值：
- `Result<Option<CToolScopeConfig>>`

功能：
- 尝试读取视野详细配置。
- 文件不存在时返回 `Ok(None)`。
- 文件存在但格式错误时返回错误。

## parse_cool_config_toml

参数：
- `content`：`scope.toml` 的文本内容。

返回值：
- `Result<CToolScopeConfig>`

功能：
- 解析视野详细配置。
- 负责读取 `files.readwrite`、`files.readonly`、`files.hide`、`folders.readwrite`、`folders.readonly`、`folders.hide`。

## build_ctool_scope_context

参数：
- `current_dir`：启动 Codex 时的当前目录，也就是 SessionRoot。
- `fallback_base_scope`：没有配置时使用的基础视野默认值。

返回值：
- `Result<CToolScopeContext>`

功能：
- 构造 CTool 运行时使用的完整视野上下文。
- 负责加载 `SessionRoot\.cool\config.toml` 和 `SessionRoot\.cool\scope.toml`。
- 负责确定 `SessionRoot`、`CoolWorkspace`、`CToolScopeBase`、session 级别视野规则。

## normalize_scope_config

参数：
- `config`：原始视野配置。
- `base_dir`：相对路径的解析基准，当前为 `CoolWorkspace`。

返回值：
- `Result<CToolScopeConfig>`

功能：
- 把配置中的相对路径统一转换成标准路径。
- 让后续权限判定不需要重复处理路径格式问题。

## resolve_user_path

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：用户输入的路径。

返回值：
- `PathBuf`

功能：
- 将用户输入路径转换成可判定路径。
- 相对路径按 `CoolWorkspace` 解析。
- 绝对路径保持绝对路径。

## canonicalize_existing_path

参数：
- `path`：需要标准化的路径。

返回值：
- `Result<PathBuf>`

功能：
- 标准化一个已经存在的路径。
- 通常用于读、写、删除、移动已有文件或目录之前。

## canonicalize_parent_for_new_path

参数：
- `path`：即将创建的新路径。

返回值：
- `Result<PathBuf>`

功能：
- 标准化新路径的父目录。
- 用于创建文件或目录前判断父目录是否允许创建。

## path_matches_rule

参数：
- `path`：待判断路径。
- `rule`：文件夹规则路径。

返回值：
- `bool`

功能：
- 判断一个路径是否命中文件夹规则。
- 精确等于规则路径，或者位于规则路径内部，都算命中。

## matches_any_path

参数：
- `path`：待判断路径。
- `rules`：文件夹规则列表。

返回值：
- `bool`

功能：
- 判断路径是否命中任意一个文件夹规则。

## matches_any_exact_path

参数：
- `path`：待判断路径。
- `rules`：文件规则列表。

返回值：
- `bool`

功能：
- 判断路径是否精确命中任意一个文件规则。
- 文件规则只做精确匹配，不把父子路径当成命中。

## is_visible_by_base_scope

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待判断路径。

返回值：
- `bool`

功能：
- 判断路径是否被 `CToolScopeBase` 提供基础可见性。
- `None` 默认不可见。
- `CoolWorkspace` 表示 `CoolWorkspace` 内可见。
- `SelectedOnly` 表示只依赖显式规则。
- `TheEyeofProvidence` 表示基础视野放开。

## can_read_path

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待读取路径。

返回值：
- `bool`

功能：
- 判断 CTool 是否可以读取该路径。
- 按详细判定路径处理 hide、readonly、readwrite 和基础视野。

## can_search_path

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待搜索路径。

返回值：
- `bool`

功能：
- 判断 CTool 是否可以在该路径执行搜索。
- 当前语义通常与读取权限一致。

## can_write_path

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待写入路径。

返回值：
- `bool`

功能：
- 判断 CTool 是否可以写入该路径。
- `readonly` 允许读但不允许写。
- `hide` 禁止读写。

## can_create_path

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待创建路径。

返回值：
- `bool`

功能：
- 判断 CTool 是否可以创建该路径。
- 主要根据目标父目录的视野权限判定。

## is_hard_protected_config_path

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待判断路径。

返回值：
- `bool`

功能：
- 判断路径是否属于硬保护配置区。
- 当前包括 `SessionRoot\.cool` 及其内部内容。
- 硬保护路径不允许普通 CTool 读写。

## is_protected_path

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待判断路径。

返回值：
- `bool`

功能：
- 判断路径是否属于受保护路径。
- 主要用于删除、移动等高风险操作前的统一保护判断。

## ensure_read_allowed_by_scope

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待读取路径。

返回值：
- `Result<()>`

功能：
- 读取前的权限门禁函数。
- 不允许时返回错误，允许时返回 `Ok(())`。

## ensure_search_allowed_by_scope

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待搜索路径。

返回值：
- `Result<()>`

功能：
- 搜索前的权限门禁函数。
- 不允许时返回错误，允许时返回 `Ok(())`。

## ensure_write_allowed_by_scope

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待写入路径。

返回值：
- `Result<()>`

功能：
- 写入前的权限门禁函数。
- 用于编辑、覆盖、批量修改等操作。

## ensure_create_allowed_by_scope

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待创建路径。

返回值：
- `Result<()>`

功能：
- 创建前的权限门禁函数。
- 用于创建文件或目录。

## ensure_delete_allowed_by_scope

参数：
- `ctx`：当前 CToolScopeContext。
- `path`：待删除路径。

返回值：
- `Result<()>`

功能：
- 删除前的权限门禁函数。
- 会阻止删除受保护路径。

## ensure_move_allowed_by_scope

参数：
- `ctx`：当前 CToolScopeContext。
- `from`：移动来源路径。
- `to`：移动目标路径。

返回值：
- `Result<()>`

功能：
- 移动或重命名前的权限门禁函数。
- 同时检查来源路径和目标路径。

## parse_ctool_scope_command

参数：
- `args`：`/cs` 后面的参数列表。

返回值：
- `Result<CToolScopeCommand>`

功能：
- 解析 CLI 输入的 `/cs` 命令。
- 负责区分文件、文件夹、添加、移除、只读、隐藏、基础视野和显示配置。

## handle_ctool_scope_command

参数：
- `ctx`：当前 CToolContext。
- `command`：已经解析好的 CToolScopeCommand。

返回值：
- `Result<String>`

功能：
- 执行 `/cs` 命令。
- 负责修改 session 级别 `config.toml` 或 `scope.toml`，以及返回给 CLI 的提示文本。

## show_ctool_scope

参数：
- `ctx`：当前 CToolContext。

返回值：
- `String`

功能：
- 生成当前视野状态的展示文本。
- 用于 `/cs show`。
- 展示基础视野、SessionRoot、CoolWorkspace、session 视野规则等信息。
