# CToolScope 视野详细设计

## 基础视野

基础视野由 CToolScopeBase 决定。

CToolScopeBase 位于：

```text
SessionRoot\.cool\config.toml
```

如果没有配置，默认：

```text
None
```

---

## CToolScopeBase 枚举

```text
None
CoolWorkspace
SelectedOnly
TheEyeofProvidence
```

### None

无基础视野。

即使 CoolWorkspace 默认等于 SessionRoot，CTool 也仍然没有基础视野。

### CoolWorkspace

使用 CoolWorkspace 作为基础视野。

只有在 CToolScopeBase 设置为 CoolWorkspace 时，CoolWorkspace 路径才会成为基础可见范围。

### SelectedOnly

不开放基础视野，只依赖显式文件/文件夹规则。

### TheEyeofProvidence

全局视野。

暂时不启用。

---

## CoolWorkspace

CoolWorkspace 位于：

```text
SessionRoot\.cool\config.toml
```

如果未设置：

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

CTool 的普通相对路径应基于 CoolWorkspace 解析。

---

## 视野详细配置文件

视野详细配置分为系统级和 SessionRoot 级。

系统级：

```text
LauncherDir\.cool-system\scope.toml
```

SessionRoot 级：

```text
SessionRoot\.cool\scope.toml
```

配置结构：

```toml
[files]
readwrite = []
readonly = []
hide = []

[folders]
readwrite = []
readonly = []
hide = []
```

---

## 推荐默认隐藏

系统级 `scope.toml` 推荐默认隐藏：

```toml
[folders]
hide = [
  ".codex",
  ".cool"
]
```

注意：

`.cool`、`.cool-system` 属于硬保护配置区，应该由代码层强制保护，不应只依赖配置文件。

---

## 视野设置权限原则

### 总原则

```text
系统级 > SessionRoot 级
文件规则 > 文件夹规则
hide > readonly > readwrite
显式规则 > CToolScopeBase
```

### 详细优先级

```text
System.files.hide
System.files.readonly
System.files.readwrite

System.folders.hide
System.folders.readonly
System.folders.readwrite

Session.files.hide
Session.files.readonly
Session.files.readwrite

Session.folders.hide
Session.folders.readonly
Session.folders.readwrite

CToolScopeBase
```

说明：

1. 系统级规则永远高于 SessionRoot 级规则。
    
2. SessionRoot 级规则不能越过系统级规则。
    
3. 文件规则高于文件夹规则。
    
4. hide 优先级最高。
    
5. readonly 允许读，不允许普通写。
    
6. readwrite 允许读写。
    
7. CToolScopeBase 只提供最后的基础可见性。
    

---

## 路径解析规则

### 用户输入路径

用户输入的相对路径按 CoolWorkspace 解析。

例如：

```text
path = "src/lib.rs"
```

解析为：

```text
CoolWorkspace\src\lib.rs
```

绝对路径保持绝对路径，但仍必须经过 CToolScope 判断。

### 配置文件路径

`scope.toml` 中的相对路径默认按 CoolWorkspace 解析。

也就是说：

```toml
[folders]
hide = [".codex"]
```

表示：

```text
CoolWorkspace\.codex
```

如果需要保护 SessionRoot 下的固定目录，建议由代码层硬保护处理，而不是依赖相对路径配置。

---

## 硬保护路径

以下路径应由代码强制保护：

```text
CoolDir = SessionRoot\.cool
CoolSystemDir = LauncherDir\.cool-system
```

硬保护路径不允许普通 CTool 读写。

普通 CTool 不允许：

```text
读取
搜索
写入
创建
删除
移动
重命名
```

这些目录中的内容。

如果未来需要修改配置，必须通过专门的 Cool 配置命令或专门的配置工具。

---

## 读取判断

CTool 可以读取路径，当且仅当：

```text
路径不属于硬保护路径
路径不是 Hidden
路径命中 readonly 或 readwrite
或路径被 CToolScopeBase 允许
```

### 读取允许

```text
ReadWrite
Readonly
CToolScopeBase 允许
```

### 读取拒绝

```text
Hidden
Unspecified
硬保护路径
范围外路径
```

---

## 搜索判断

搜索权限通常等同于读取权限。

搜索时必须保证：

- 搜索根目录可搜索。
    
- 每个返回结果对应文件可读。
    
- 不能返回 Hidden 路径。
    
- 不能返回硬保护路径。
    
- 不能通过搜索泄露隐藏路径名称或内容。
    

---

## 写入判断

普通写入工具只能写入 ReadWrite 路径。

Readonly 路径不能被普通写入工具修改。

Hidden 路径不能读写。

硬保护路径不能读写。

例外：

```text
CToolAnnotateMarkdown
```

该工具可以对 ReadOnly Markdown 文件执行受限批注写入，但只能添加批注，不能删除、替换、重排或改写正文。

---

## 创建判断

创建路径时，主要检查目标父目录。

创建文件或目录必须满足：

```text
父目录可写
目标路径不属于 Hidden
目标路径不属于 Readonly
目标路径不属于硬保护路径
```

不允许在硬保护目录中创建任何内容。

---

## 删除判断

删除文件或目录必须满足：

```text
目标路径可写
目标路径不属于硬保护路径
目标路径不属于受保护路径
```

目录删除只能删除空目录。

不允许递归删除目录。

不允许删除：

```text
CoolWorkspace 根目录
SessionRoot 关键目录
CoolDir
CoolSystemDir
```

---

## 移动判断

移动或重命名必须同时检查来源和目标。

必须满足：

```text
来源路径可写
目标父目录可创建
来源不属于硬保护路径
目标不属于硬保护路径
```

不允许移动：

```text
CoolWorkspace 根目录
SessionRoot 关键目录
CoolDir
CoolSystemDir
```

---

## /cs 视野命令

`/cs` 是 CToolScope 的简写命令。

所有 `/cs` 命令只修改 SessionRoot 级配置，不修改系统级配置。

### 文件规则

```text
/cs f <path>
/cs f ro <path>
/cs f hide <path>

/cs f - <path>
/cs f ro - <path>
/cs f hide - <path>
```

### 文件夹规则

```text
/cs <path>
/cs ro <path>
/cs hide <path>

/cs - <path>
/cs ro - <path>
/cs hide - <path>
```

### 基础视野

```text
/cs base none
/cs base coolworkspace
/cs base selectedonly
```

### 显示当前视野

```text
/cs show
```

`/cs show` 应展示：

```text
SessionRoot
CoolDir
CoolWorkspace
CToolScopeBase
SessionRoot 级视野规则
系统级视野规则
硬保护路径
```

---

## 启动加载顺序

```text
1. 初始化 SafeMode
2. 解析 PermissionProfile
3. 捕获 LauncherDir
4. 捕获 SessionRoot
5. 定位 CoolSystemDir = LauncherDir\.cool-system
6. 定位 CoolDir = SessionRoot\.cool

7. 读取 CoolSystemDir\config.toml
   找不到：按空配置
   格式错误：CTool 不启用，并报错

8. 读取 CoolDir\config.toml
   找不到：按空配置
   格式错误：CTool 不启用，并报错

9. 根据 CoolDir\config.toml 确定：
   CToolScopeBase
   CoolWorkspace

10. 读取 CoolSystemDir\scope.toml
    找不到：按空配置
    格式错误：CTool 不启用，并报错

11. 读取 CoolDir\scope.toml
    找不到：按空配置
    格式错误：CTool 不启用，并报错

12. 合并系统级和 SessionRoot 级 scope 配置

13. 构造 CToolScopeContext

14. 读取 CoolSystemDir\command.toml
    找不到：按空配置
    格式错误：ctool_command_request 不启用，并报错

15. 读取 CoolDir\command.toml
    找不到：按空配置
    格式错误：ctool_command_request 不启用，并报错

16. 合并系统级和 SessionRoot 级 command 配置

17. 构造 CToolContext

18. 注册 CTool
```

---

## 视野相关工具函数

### locate_cool_dir

参数：

```text
session_root
```

返回：

```text
SessionRoot\.cool
```

功能：

定位当前 session 的 `.cool` 目录。

### locate_cool_config_path

返回：

```text
SessionRoot\.cool\config.toml
```

功能：

定位 session 级基础配置文件。

### locate_cool_scope_path

返回：

```text
SessionRoot\.cool\scope.toml
```

功能：

定位 session 级视野配置文件。

### locate_cool_system_dir

返回：

```text
LauncherDir\.cool-system
```

功能：

定位系统级 Cool 目录。

### locate_cool_system_config_path

返回：

```text
LauncherDir\.cool-system\config.toml
```

功能：

定位系统级基础配置文件。

### locate_cool_system_scope_path

返回：

```text
LauncherDir\.cool-system\scope.toml
```

功能：

定位系统级视野配置文件。

### locate_cool_system_command_path

返回：

```text
LauncherDir\.cool-system\command.toml
```

功能：

定位系统级命令策略文件。

### resolve_user_path

相对路径按 CoolWorkspace 解析。

### can_read_path

判断 CTool 是否可以读取该路径。

### can_search_path

判断 CTool 是否可以搜索该路径。

### can_write_path

判断 CTool 是否可以写入该路径。

### can_create_path

判断 CTool 是否可以创建该路径。

### ensure_read_allowed_by_scope

读取前的门禁函数。

### ensure_search_allowed_by_scope

搜索前的门禁函数。

### ensure_write_allowed_by_scope

写入前的门禁函数。

### ensure_create_allowed_by_scope

创建前的门禁函数。

### ensure_delete_allowed_by_scope

删除前的门禁函数。

### ensure_move_allowed_by_scope

移动前的门禁函数。

### is_hard_protected_config_path

判断路径是否属于硬保护配置区。

当前应包括：

```text
CoolDir
CoolSystemDir
```