
# 四层结构


## 第 1 层：SafeMode  

SafeMode AboveAll 原则

一旦开启，会强制关闭所有的 Codex 原生工具
只有CTool 可以正常使用

## 第 2 层：PermissionProfile

加入 Cool开头的模式。
核心是 CoolReadWrite

Cool开头的模式
只能使用 “CTool工具“





## 第 3 层：CToolScope  

CToolScope 并非是直接限制 PermissionProfile 的视野范围
而是 直接限制 CTool的视野范围 
从而达到对 Cool开头 PermissionProfile 绝对控制
因为 Cool开头 PermissionProfile 只能用 CTool进行读写


### CToolScopeBase

分为以下四种类型
None
CoolWorkspace
SelectedOnly
TheEyeofProvidence


CToolScopeBase 决定了 CToolScope的 基础视野 

CToolScopeBase 位于 
SessionRoot 的 .cool 中的 config.toml 设定


无设定 默认为 None 无任何视野
关于视野设计 详细位于《CToolScope 视野 详细设计 》
这里没有太多赘




### CoolWorkspace
可以设置

CoolWorkspace 位于 SessionRoot\.cool\config.toml 中设置；未设置时默认为 SessionRoot。




## 启动Codex时要显示

SafeMode 状态
PermissionProfile 状态
CToolScope 状态
CoolWorkspace 路径

之后考虑 在下方也持续显示

## 第4层 CTool


1/
CTool  制作 要考虑它的操作权限范围。 必须要符合我们的视野规则。
2/
然后 一定要让 CoolReadWrite 这个 CommissionProfile  启动的时候能知道自己当下可调用的工具都有哪些。 具体该怎么用。
3/ 
CTool 都属于安全级别非常高的工具。 
除 CToolCommandRequest 和 CToolTavilySearch 之外
不能有 联网、下载和 编译可执行文件能力


### 读取工具

| 功能                  | Rust 类型名             | 文件名                        | 工具名                     |
| ------------------- | -------------------- | -------------------------- | ----------------------- |
| 看项目结构               | `CToolListDirectory` | `ctool_list_directory.rs`  | `ctool_list_directory`  |
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

/


### Telegram 对话工具

| 功能     | Rust 类型名                   | 文件名                              | 工具名                           |
| ------ | -------------------------- | -------------------------------- | ----------------------------- |
| TG 发消息 | `CToolTelegramSendMessage` | `ctool_telegram_send_message.rs` | `ctool_telegram_send_message` |
| TG 收指令 | `CToolTelegramPollCommand` | `ctool_telegram_poll_command.rs` | `ctool_telegram_poll_command` |



### CToolCommandRequest
这个会在专门的文档中详细介绍



### Codex 自带命令参考

我现在能用的工具主要是： - 
shell：在当前工作区运行命令，比如 rg、git、just、cargo、pwsh 等。 
apply_patch：按补丁方式修改文件。 

update_plan：维护任务计划和进度。
view_image：查看本地图片。 -

multi_tool_use.parallel：并行运行多个读取/搜索类工具调用，适合同时看多个文件或命令输出。


### CTool 文件夹设计结构

```
codex-rs/   //Codex自带
  utils/    //Codex自带
    ctool/
      Cargo.toml
      src/
        lib.rs

        scope.rs
        context.rs
        error.rs
        gate.rs

        tool.rs
        registry.rs

        tools/
          mod.rs

          read/
            mod.rs
            ctool_list_directory.rs
            ctool_rg_search.rs
            ctool_read_code_range.rs
            ctool_read_file.rs

          edit/
            mod.rs
            ctool_edit_replace.rs
            ctool_edit_insert.rs
            ctool_edit_batch.rs
            ctool_preview_diff.rs

          file_ops/
            mod.rs
            ctool_create_file.rs
            ctool_delete_file.rs
            ctool_move_file.rs

          tg/
            mod.rs
            ctool_telegram_send_message.rs
            ctool_telegram_poll_command.rs
            
```




# 制作规划


## 第一阶段

创建  CToolScopeBase 枚举

包含
None
CoolWorkspace
SelectedOnly
TheEyeofProvidence


前期做 CTool 的时候，只考虑 CoolWorkspace 这一种情况。 
其他三种我们先暂时不考虑。
然后我们第一步是把这个枚举给创建出来。 


然后保证在启动 Codex 的时候，

最开头显示的状态包括：

SafeMode 状态
PermissionProfile 状态
CToolScope 状态
CoolWorkspace 路径


## 第二阶段

PermissionProfile 添加 CoolReadWrite 


这个模式，先不给任何工具，它不能调用任何 Codex默认工具。 
它只能使用 CTool工具
并且让CoolReadWrite  作为默认的 permission profile 。


## 第三步
陆续制作CTool 一个一个送给 
CoolReadWrite