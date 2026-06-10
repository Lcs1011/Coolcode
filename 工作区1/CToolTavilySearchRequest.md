# ctool_tavily_search_request 产品方案

## 1. 产品定位

`ctool_tavily_search_request` 是 CoolCode / CTool 体系中的受控联网搜索工具。

它不是 Codex 原生联网搜索，也不是默认自由联网能力。它属于 `ctool_` 工具体系，必须受到 SafeMode、系统配置、Session 配置、风险分级、用户确认、缓存落盘和日志审计的管制。

核心目标：

```text
在本地 CTool 读取工具无法提供足够信息时，
允许 Codex 通过 Tavily 获取外部资料；
但每一次联网请求都必须可展示、可记录、可复查、可限制、可缓存。
```

`ctool_tavily_search_request` 是 CTool 中少数允许联网的特殊工具之一。除本工具和 `ctool_command_request` 外，基础 CTool 不应具备联网、下载或执行外部命令能力。

---

## 2. 三条铁律

### 2.1 能不用就不用

优先使用本地 CTool：

```text
ctool_read_file
ctool_rg_search
ctool_read_code_range
ctool_regex_search
ctool_extract_lines_matching
```

只有本地资料不足、需要外部文档、公开网页或公开资料时，才允许申请 Tavily 搜索。

### 2.2 必须完整展示请求

每次 Tavily 请求必须展示：

```text
Provider
Action
Risk
CurrentDir
Query / URL / Source
Will write
确认提示
```

禁止：

```text
搜索一下
查一下网页
打开上面的链接
省略 query
隐藏 URL
模型自己补全但不展示
```

### 2.3 结果必须落盘

Tavily 返回结果不直接大段塞回模型上下文。

工具只返回：

```text
生成的 Markdown 文件路径
简短摘要
下一步建议
```

完整结果写入：

```text
CoolDir\cache\web_search\YYYY-MM-DD\
```

其中：

```text
CoolDir = SessionRoot\.cool
```

---

## 3. SafeMode 关系

`ctool_tavily_search_request` 不能绕过 SafeMode。

执行链路：

```text
SafeMode ON
=> CoolReadWrite
=> CTool allowed
=> ctool_tavily_search_request
=> 读取 CoolSystemDir\config.toml 中的 Tavily token
=> 读取 CoolDir\config.toml / CoolDir\scope.toml 中的 Session 限制
=> 请求分类
=> 展示完整请求
=> 必要时等待用户确认
=> 调用 Tavily 或拒绝
=> 写入 CoolDir\cache\web_search\YYYY-MM-DD\
=> 追加 request_log.md
=> 返回短摘要和文件路径
```

---

## 4. 配置文件

配置分两类：系统级密钥配置和 Session 级行为配置。

```text
CoolSystemDir\config.toml   系统级，保存 Tavily token 和全局硬限制
CoolDir\config.toml         Session 级，保存本 Session 的搜索策略
```

说明：

```text
CoolSystemDir = LauncherDir\.cool-system
CoolDir       = SessionRoot\.cool
```

### 4.1 Tavily Token

Tavily token 只能放在系统级 config 中。

```toml
[ctool_tavily_search]
enabled = true
provider = "tavily"
tavily_api_key = "tvly-..."
```

禁止把 token 写入：

```text
CoolDir\config.toml
CoolDir\scope.toml
CoolDir\command.toml
项目源码文件
任务文档
日志文件
搜索缓存文件
模型上下文展示文本
```

工具输出、日志和错误信息中都不得打印完整 token。

### 4.2 Session 行为配置

Session 级配置可以限制搜索行为，但不能覆盖系统级更严格规则。

示例：

```toml
[ctool_tavily_search]
enabled = true

allow_text_search = true
allow_extract = true
allow_zoom = true
allow_research = true
allow_image_search = false

max_search_results = 8
max_extract_chars = 12000
max_zoom_chars = 6000

write_request_log = true
write_result_markdown = true
```

系统配置可以强制关闭图片搜索、降低最大字符数、禁用某些 action，Session 配置不能放宽这些限制。

---

## 5. 配置段设计

统一配置段名：

```toml
[ctool_tavily_search]
```

完整示例：

```toml
[ctool_tavily_search]
enabled = true
provider = "tavily"

allow_text_search = true
allow_extract = true
allow_zoom = true
allow_research = true
allow_image_search = false

max_search_results = 8
max_extract_chars = 12000
max_zoom_chars = 6000

sensitive_keywords = [
  "token",
  "api key",
  "apikey",
  "password",
  "secret",
  "bearer",
  "sk-",
  "tvly-",
  "private key",
  "ssh key"
]

blocked_keywords = [
  "leak token",
  "steal api key",
  "dump password",
  "bypass login",
  "exploit private key"
]
```

---

## 6. 工具形态

第一版只做一个工具：

```text
 
```

支持 action：

```text
search
extract
zoom
research
```

图片能力作为同一工具的受控模式保留：

```text
search_with_images
extract_with_images
```

第一版默认关闭图片能力。

---

## 7. 风险等级

风险等级分四类：

```text
Green   = 普通公开文本搜索，可自动执行
Yellow  = 需要用户确认一次的边界搜索
Red     = 高风险联网搜索，确认两次
Blocked = 硬拒绝请求，直接拒绝执行
```

风险顺序：

```text
Green < Yellow < Red < Blocked
```

### 7.1 Green

普通公开技术搜索默认 Green。

示例：

```text
rust cargo workspace dependency
ratatui textarea enter handler
windows rust path canonicalize
Tavily API search endpoint docs
```

Green 可以自动执行，但必须展示请求并写日志。

### 7.2 Yellow

以下情况至少 Yellow：

```text
query 很宽泛，可能返回大量不相关内容
extract 非官方网页
research 需要多轮外部资料综合
请求包含公司名、项目名但不包含敏感内容
```

Yellow 必须确认一次。

### 7.3 Red

以下情况至少 Red：

```text
图片搜索
抽取大量网页内容
query 包含 token / key / secret / password 等敏感词
query 包含大段本地源码
请求搜索漏洞利用细节
请求打开网站或模拟浏览器行为
```

Red 必须确认两次。

### 7.4 Blocked

以下情况必须 Blocked：

```text
请求搜索、泄露、恢复、绕过 token / key / password / private key
请求上传本地私有文件全文到外部服务
请求下载 exe / msi / dll / bat / cmd / ps1 / sh / zip / rar / 7z / tar / gz
请求执行网页 JavaScript
请求打开浏览器
请求绕过登录、验证码、付费墙或访问控制
请求把 Tavily token 打印到日志或输出
Session 配置中出现 tavily_api_key
系统级配置缺少 Tavily token 但请求要求联网执行
```

Blocked 请求必须展示，但不能确认执行。

---

## 8. 请求展示格式

所有 Tavily 请求都必须明显展示。

### 8.1 Green 文本搜索

```text
==============================
🟢 TAVILY SEARCH REQUEST: GREEN
Provider: Tavily
Action: search
CurrentDir: C:\CodexLab\codex\codex-rs
Auto Approved: text search allowed by config

Query:
rust cargo workspace dependency

Will write:
CoolDir\cache\web_search\2026-06-09\00000_search_rust_cargo_workspace_dependency.md
==============================
```

### 8.2 Red 图片搜索

```text
==============================
🔴 TAVILY SEARCH REQUEST: RED
Provider: Tavily
Action: search_with_images
CurrentDir: C:\CodexLab\codex\codex-rs

Query:
unreal engine material graph example

Allowed Image Extensions:
.jpg, .jpeg, .png, .webp

Will write:
CoolDir\cache\web_search\2026-06-09\00001_image_search_unreal_material_graph.md

First confirm? Type Y:
Second confirm? Type Y:
==============================
```

### 8.3 Blocked 请求

```text
==============================
🔴🔴🔴 TAVILY SEARCH REQUEST: BLOCKED
Provider: Tavily
Action: search
CurrentDir: C:\CodexLab\codex\codex-rs
Blocked: hard policy

Query:
find leaked tvly token examples

Will write:
CoolDir\cache\web_search\2026-06-09\00002_blocked_tavily_request.md
==============================
```

---

## 9. 确认规则

### 9.1 Green

Green 自动执行。

### 9.2 Yellow

```text
Y / y 开头 = 同意执行
N / n 开头 = 拒绝执行
其他任何输入 = 拒绝执行
空输入 = 拒绝执行
```

### 9.3 Red

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

### 9.4 Blocked

```text
不进入确认流程
直接拒绝执行
仍然返回展示信息、拒绝原因、日志路径
```

### 9.5 拒绝反馈

用户可以输入：

```text
N 不要联网，先看本地 Cargo.toml
```

处理方式：

```text
首字母 N => 拒绝执行
N 后面的内容 => 作为用户反馈返回给 Codex
```

如果输入不是 Y/N，也按拒绝处理，并把完整输入作为反馈。

---

## 10. 缓存目录结构

搜索缓存目录：

```text
CoolDir\cache\web_search\YYYY-MM-DD\
```

示例：

```text
SessionRoot\.cool\cache\web_search\2026-06-09\
  request_log.md
  00000_search_rust_cargo_workspace_dependency.md
  00001_extract_cargo_docs_workspace_dependencies.md
  00002_zoom_workspace_dependencies_section.md
  00003_research_ratatui_textarea_enter.md
  00004_blocked_tavily_request.md
```

编号规则：

```text
当天从 00000 开始
每新增一个结果文件编号 +1
超过 99999 后继续 100000、100001
不截断
不覆盖
```

搜索缓存目录应加入 Scope 只读保护：

```text
ctool_tavily_search_request 可以写入 CoolDir\cache\web_search
普通 CTool 可以读取 CoolDir\cache\web_search
普通 CTool 不允许改、删、移动 CoolDir\cache\web_search
```

---

## 11. 文件命名规则

格式：

```text
00000_<action>_<slug>.md
```

示例：

```text
00000_search_rust_cargo_workspace_dependency.md
00001_extract_cargo_docs_workspace_dependencies.md
00002_zoom_workspace_dependencies_section.md
00003_research_ratatui_textarea_enter.md
00004_blocked_tavily_request.md
```

`slug` 来源：

```text
优先使用 file_name_hint
否则根据 query / title / URL 自动生成
非法字符替换为 _
长度限制 80 字符左右
```

---

## 12. request_log.md 设计

每次搜索申请都追加到当天日志。

```markdown
# Tavily Search Request Log

Date: 2026-06-09

## 00000

Time: 2026-06-09 15:30:12
Provider: Tavily
Tool: ctool_tavily_search_request
Action: search
Risk: Green
Approved: Auto
CurrentDir: C:\CodexLab\codex\codex-rs

Query:
rust cargo workspace dependency

Output:
00000_search_rust_cargo_workspace_dependency.md

---

## 00001

Time: 2026-06-09 15:35:00
Provider: Tavily
Tool: ctool_tavily_search_request
Action: search
Risk: Blocked
Approved: No
Status: Blocked
CurrentDir: C:\CodexLab\codex\codex-rs

Query:
find leaked tvly token examples

Reason:
matched blocked keyword: token leak request

Output:
00001_blocked_tavily_request.md
```

日志文件属于 protected 搜索缓存，Codex 只能读，不能改。

---

## 13. 搜索结果文件格式

### 13.1 Search 文件

```markdown
# Tavily Search Result

Provider: Tavily
Kind: Search
Time: 2026-06-09 15:30:12
CurrentDir: C:\CodexLab\codex\codex-rs
Risk: Green
Approved: Auto

## Query

rust cargo workspace dependency

## Short Summary

Found official Cargo documentation and examples related to workspace dependencies.

## Results

### 1. Cargo Workspaces - The Cargo Book

URL:
https://doc.rust-lang.org/cargo/reference/workspaces.html

Summary:
...
```

### 13.2 Extract 文件

```markdown
# Tavily Extract Result

Provider: Tavily
Kind: Extract
Time: 2026-06-09 15:32:44
Source Search:
00000_search_rust_cargo_workspace_dependency.md

## URL

https://doc.rust-lang.org/cargo/reference/workspaces.html

## Short Summary

This page explains Cargo workspaces, workspace dependencies, and shared package configuration.

## Content

...
```

### 13.3 Blocked 文件

```markdown
# Tavily Search Request Result

Provider: Tavily
Kind: Blocked
Time: 2026-06-09 15:35:00
Risk: Blocked
Approved: No
Status: Blocked

## Query

find leaked tvly token examples

## Reason

Request matched blocked policy.

## Commands / Network

No Tavily request was sent.
```

---

## 14. 工具返回给 Codex 的内容

不要返回完整正文。

只返回：

```text
Search completed.

Output:
CoolDir\cache\web_search\2026-06-09\00000_search_rust_cargo_workspace_dependency.md

Summary:
Found official Cargo documentation and examples related to workspace dependencies.

Suggested next step:
Read the output file. If result 1 is useful, request extract for its URL.
```

Blocked 返回：

```text
Search blocked.

Output:
CoolDir\cache\web_search\2026-06-09\00001_blocked_tavily_request.md

Reason:
Request matched blocked policy.
```

---

## 15. Action 设计

### 15.1 search

输入：

```json
{
  "action": "search",
  "query": "rust cargo workspace dependency",
  "file_name_hint": "rust_cargo_workspace_dependency",
  "risk_confirmation": null,
  "red_first_confirmation": null,
  "red_second_confirmation": null
}
```

输出：

```text
生成 search MD 文件
追加 request_log.md
返回文件路径和摘要
```

### 15.2 extract

输入：

```json
{
  "action": "extract",
  "url": "https://doc.rust-lang.org/cargo/reference/workspaces.html",
  "source_file": "CoolDir\\cache\\web_search\\2026-06-09\\00000_search_rust_cargo_workspace_dependency.md",
  "file_name_hint": "cargo_docs_workspace_dependencies"
}
```

输出：

```text
生成 extract MD 文件
追加 request_log.md
返回文件路径和摘要
```

### 15.3 zoom

输入：

```json
{
  "action": "zoom",
  "source_file": "CoolDir\\cache\\web_search\\2026-06-09\\00001_extract_cargo_docs_workspace_dependencies.md",
  "target": "workspace.dependencies",
  "file_name_hint": "workspace_dependencies_section"
}
```

输出：

```text
生成 zoom MD 文件
返回文件路径和摘要
```

### 15.4 research

输入：

```json
{
  "action": "research",
  "query": "ratatui textarea enter submit handler",
  "file_name_hint": "ratatui_textarea_enter"
}
```

输出：

```text
生成 research MD 文件
返回文件路径和摘要
```

---

## 16. 网络安全规则

默认禁止：

```text
不打开浏览器
不执行网页 JavaScript
不下载二进制文件
不下载压缩包
不下载脚本
不保存 HTML 为可执行网页
不自动嵌入远程图片
不自动打开搜索结果文件
不在日志中写入 Tavily token
```

以下行为必须 Blocked：

```text
下载 exe / msi / dll
下载 bat / cmd / ps1 / sh
下载 zip / rar / 7z / tar / gz
下载未知二进制
打开网站或浏览器
执行网页 JavaScript
git clone 外部仓库
curl / wget / Invoke-WebRequest 下载文件
泄露或搜索 token / key / password / secret
```

普通技术查询可以 Green：

```text
query 仅为普通技术问题
不包含 token / key / secret
不包含大段私有代码
不要求上传本地文件
不要求下载内容
```

---

## 17. 图片搜索规则

图片搜索默认关闭。

```toml
[ctool_tavily_search]
allow_image_search = false
```

开启后仍至少 Red，必须二次确认。

```text
第一次确认 Y / y
第二次确认 Y / y
```

允许类型：

```text
.jpg
.jpeg
.png
.webp
```

禁止类型：

```text
.svg
.gif
.bmp
.ico
.tif
.tiff
.heic
.avif
```

SVG 按网页/脚本风险处理，不按普通图片处理。

图片文件保存规则：

```text
只保存到 CoolDir\cache\web_search\YYYY-MM-DD\
不自动打开
不自动预览
不自动嵌入 HTML
不允许远程 URL 图片直接进入网页
不允许 data:image
需要校验扩展名和文件头
限制文件大小
```

---

## 18. Provider 设计

第一版只实现：

```text
provider = "tavily"
```

系统级 config 必须提供 Tavily token。

未来可以扩展：

```text
provider = "openai_web"
provider = "bocha"
provider = "querit"
provider = "bing"
```

第一版不要同时做太多 provider。

---

## 19. 与原生搜索的关系

OpenAI 原生 web_search 更像云端托管搜索：

```text
模型自己搜索
模型自己打开页面
模型自己页内查找
模型自己组织引用
```

Tavily 方案更像安全工程化搜索：

```text
Codex 申请
工具搜索
写入缓存
写入日志
Codex 读取缓存
Codex 决定下一步
```

最终体验接近原生搜索，但多了：

```text
缓存
日志
权限
可审计
可复查
不会直接撑爆上下文
```

---

## 20. 推荐开发顺序

第一阶段：

```text
ctool_tavily_search_request
只支持 search
只支持文本结果
读取系统级 Tavily token
写 MD
写 request_log
普通搜索默认 Green
敏感搜索 Red
Blocked 直接拒绝并写记录
```

第二阶段：

```text
支持 extract
```

第三阶段：

```text
支持 zoom
```

第四阶段：

```text
支持 research
```

第五阶段：

```text
支持图片搜索
默认关闭
二次确认
只允许 jpg/jpeg/png/webp
```

---

## 21. 最终采用版

本方案正式采用以下原则：

```text
ctool_tavily_search_request 是受 SafeMode 管制的联网搜索工具。
Tavily token 只允许放在 CoolSystemDir\config.toml。
普通文本搜索默认 Green，但必须展示和记录日志。
Yellow 确认一次。
Red 确认两次。
Blocked 必须展示和记录，但不能执行。
搜索结果必须写入 CoolDir\cache\web_search\YYYY-MM-DD\ Markdown 缓存文件。
工具只把缓存文件路径和简短摘要返回给 Codex。
Codex 通过普通 CTool 只读缓存文件。
每一次 search / extract / zoom / research 都生成新文件。
图片搜索默认关闭，开启后至少 Red。
下载内容、打开网站、执行浏览器、泄露 token，一律 Blocked。
```
