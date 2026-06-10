
 一定要让 CoolReadWrite 这个 CommissionProfile  启动的时候能知道自己当下可调用的 CTool工具都有哪些

schema 化是把“这个工具需要什么参数、参数是什么类型、哪些必填、哪些选、默认值/限制是什么”用机器能读懂的结构写出来。
  

# CTool 基础类

本文档整理 CTool 基础工具。

基础工具分为三类：

- 读取工具
- 修改工具
- 文件操作工具


所有基础工具都必须符合 CToolScope 视野规则。

除 `CToolCommandRequest` 和 `CToolTavilySearch` 之外，CTool 不应具备联网、下载、编译可执行文件、运行可执行文件的能力。




# 读取工具

读取工具只负责查看文件系统和文本内容。
读取工具不能修改文件、创建文件、删除文件、移动文件。

读取工具必须遵守以下规则：

- 必须经过 CToolScope 读取或搜索权限检查。
- 不能读取 `.cool`、`.cool-system` 等硬保护配置区。
- 不能绕过 hide 规则展示隐藏路径。
- 默认只处理 UTF-8 文本内容。
- 读取完整文件时必须有大小上限。


## CToolListDirectory

Rust 类型名：
- `CToolListDirectory`

文件名：
- `ctool_list_directory.rs`

工具名：
- `ctool_list_directory`

功能介绍：
- 查看目录结构。
- 用于快速理解项目文件布局。
- 支持限制递归深度和最大返回条目数。

权限要求：
- 需要对目标目录有读取权限。
- 展示目录内容时，不能显示 CToolScope 判定为隐藏的路径。

安全要求：
- 只能列目录。
- 不允许修改文件系统。
- 不允许读取文件正文。


## CToolRgSearch

Rust 类型名：
- `CToolRgSearch`

文件名：
- `ctool_rg_search.rs`

工具名：
- `ctool_rg_search`

功能介绍：
- 在文本文件中搜索关键词、类型名、函数名、报错文本等。
- 用于快速定位代码或配置中的相关位置。
- 支持限制搜索深度、最大结果数、是否包含隐藏文件等参数。

权限要求：
- 需要对搜索根目录有搜索权限。
- 搜索结果不能暴露 CToolScope 判定为隐藏的路径或内容。

安全要求：
- 默认只搜索 UTF-8 文本文件。
- 不搜索二进制文件。
- 不允许联网。
- 不允许执行项目代码。


## CToolReadCodeRange

Rust 类型名：
- `CToolReadCodeRange`

文件名：
- `ctool_read_code_range.rs`

工具名：
- `ctool_read_code_range`

功能介绍：
- 读取指定文件的指定行号范围。
- 用于查看搜索命中位置附近的代码或配置。
- 适合读取较大文件中的局部片段。

权限要求：
- 需要对目标文件有读取权限。
- 不能读取 CToolScope 判定为隐藏或硬保护的文件。

安全要求：
- 只能读取 UTF-8 文本文件。
- 必须限制返回行数或返回内容大小。
- 不允许修改文件。


## CToolReadFile

Rust 类型名：
- `CToolReadFile`

文件名：
- `ctool_read_file.rs`

工具名：
- `ctool_read_file`

功能介绍：
- 读取完整的小型文本文件或配置文件。
- 用于查看单个文件的完整内容。

权限要求：
- 需要对目标文件有读取权限。
- 不能读取 CToolScope 判定为隐藏或硬保护的文件。

安全要求：
- 只能读取 UTF-8 文本文件。
- 必须有文件大小上限。
- 不允许读取大型文件、二进制文件、可执行文件。
- 不允许修改文件。


# 修改工具

修改工具负责对已有文本文件做受控修改。
修改工具不负责创建新文件、删除文件、移动文件或目录。

修改工具必须遵守以下规则：

- 必须同时检查读取权限和写入权限。
- 必须只处理 UTF-8 文本文件。
- 不能修改 `.cool`、`.cool-system` 等硬保护配置区。
- 不能生成或写入可执行文件。
- 修改前应尽量可预览。
- 批量修改必须先完成预检，再执行实际写入。


## CToolEditReplace

Rust 类型名：
- `CToolEditReplace`

文件名：
- `ctool_edit_replace.rs`

工具名：
- `ctool_edit_replace`

功能介绍：
- 对单个 UTF-8 文本文件执行一次精确替换。
- 适合小范围、确定性的文本修改。

参数语义：
- `path`：目标文件路径。
- `old_string`：需要被替换的精确文本。
- `new_string`：替换后的文本。

权限要求：
- 需要对目标文件有读取权限。
- 需要对目标文件有写入权限。

安全要求：
- `old_string` 必须存在且只出现一次。
- 如果 `old_string` 不存在或出现多次，必须失败。
- 不允许模糊替换。
- 不允许修改二进制文件。


## CToolEditInsert

Rust 类型名：
- `CToolEditInsert`

文件名：
- `ctool_edit_insert.rs`

工具名：
- `ctool_edit_insert`

功能介绍：
- 在 UTF-8 文本文件的指定行后插入内容。
- 适合追加段落、补充配置、插入函数或注释。

参数语义：
- `path`：目标文件路径。
- `insert_after_line`：插入位置。`0` 表示插入到文件开头，否则表示插入到指定行之后。
- `content`：要插入的文本内容。

权限要求：
- 需要对目标文件有读取权限。
- 需要对目标文件有写入权限。

安全要求：
- 插入行号必须有效。
- 只能修改 UTF-8 文本文件。
- 不允许修改二进制文件。


## CToolPreviewDiff

Rust 类型名：
- `CToolPreviewDiff`

文件名：
- `ctool_preview_diff.rs`

工具名：
- `ctool_preview_diff`

功能介绍：
- 预览替换或插入操作会产生的 diff。
- 本工具只展示结果，不写入文件。
- 用于在实际修改前确认修改范围。

参数语义：
- `path`：目标文件路径。
- `operations`：待预览的替换或插入操作列表。

权限要求：
- 需要对目标文件有读取权限。

安全要求：
- 不允许写入文件。
- 不允许修改文件系统。
- 预览逻辑必须与实际修改工具保持一致，避免预览和执行结果不一致。


## CToolEditBatch

Rust 类型名：
- `CToolEditBatch`

文件名：
- `ctool_edit_batch.rs`

工具名：
- `ctool_edit_batch`

功能介绍：
- 对一个或多个 UTF-8 文本文件执行多个精确替换或插入操作。
- 适合一次性完成一组相关修改。

参数语义：
- `operations`：批量操作列表。每一项包含操作类型、目标路径和具体修改内容。

权限要求：
- 每个目标文件都需要读取权限。
- 每个目标文件都需要写入权限。

安全要求：
- 必须先预检全部操作。
- 任意一项预检失败，整个批量操作都应失败。
- 不应在预检未完成时提前写入部分文件。
- 替换操作仍然要求精确且唯一命中。
- 不允许修改二进制文件。


# 文件操作工具

文件操作工具负责创建、删除、移动或重命名文件和目录。
文件操作工具的风险高于读取工具和普通修改工具，因此必须严格受 CToolScope 约束。

文件操作工具必须遵守以下规则：

- 必须经过对应的 create、delete、move 权限检查。
- 不能操作 `.cool`、`.cool-system` 等硬保护配置区。
- 不能删除或移动 `CoolWorkspace` 根目录。
- 不能删除或移动 SessionRoot 关键目录。
- 不能创建可执行文件或二进制文件。
- 删除目录只能删除空目录，不能递归删除。
- 移动目录不能覆盖已有目录。


## CToolCreateFile

Rust 类型名：
- `CToolCreateFile`

文件名：
- `ctool_create_file.rs`

工具名：
- `ctool_create_file`

功能介绍：
- 创建一个新的 UTF-8 文本、源码或配置文件。
- 可以选择是否允许覆盖已有文件。

参数语义：
- `path`：要创建的文件路径。
- `content`：文件内容。
- `overwrite`：是否允许覆盖已有文件。

权限要求：
- 需要对目标路径有创建权限。
- 如果覆盖已有文件，还需要对已有文件有写入权限。

安全要求：
- 默认不允许覆盖已有文件。
- 只能创建安全文本文件。
- 不允许创建可执行文件、二进制文件、脚本启动器等高风险文件。


## CToolDeleteFile

Rust 类型名：
- `CToolDeleteFile`

文件名：
- `ctool_delete_file.rs`

工具名：
- `ctool_delete_file`

功能介绍：
- 删除一个文件。
- 可通过 `expected_content` 做额外安全确认。

参数语义：
- `path`：要删除的文件路径。
- `expected_content`：可选。若提供，则只有文件当前内容完全等于该值时才允许删除。

权限要求：
- 需要对目标文件有删除权限。

安全要求：
- 只能删除文件，不能删除目录。
- 不能删除硬保护路径。
- 如果提供 `expected_content`，内容不一致时必须失败。


## CToolMoveFile

Rust 类型名：
- `CToolMoveFile`

文件名：
- `ctool_move_file.rs`

工具名：
- `ctool_move_file`

功能介绍：
- 移动或重命名单个文件。

参数语义：
- `from`：来源文件路径。
- `to`：目标文件路径。
- `overwrite`：是否允许覆盖目标文件。

权限要求：
- 需要对来源路径和目标路径通过移动权限检查。
- 如果覆盖目标文件，还需要对目标文件有写入或删除权限。

安全要求：
- 只能移动文件，不能移动目录。
- 默认不允许覆盖目标文件。
- 不能移动硬保护路径。
- 不能把文件移动成可执行文件或二进制文件。


## CToolCreateDirectory

Rust 类型名：
- `CToolCreateDirectory`

文件名：
- `ctool_create_directory.rs`

工具名：
- `ctool_create_directory`

功能介绍：
- 创建一个目录。
- 只创建指定的一个目录。

参数语义：
- `path`：要创建的目录路径。

权限要求：
- 需要对目标路径有创建权限。

安全要求：
- 父目录必须已经存在。
- 不做递归创建。
- 不能创建硬保护配置目录。


## CToolDeleteDirectory

Rust 类型名：
- `CToolDeleteDirectory`

文件名：
- `ctool_delete_directory.rs`

工具名：
- `ctool_delete_directory`

功能介绍：
- 删除一个空目录。

参数语义：
- `path`：要删除的目录路径。

权限要求：
- 需要对目标目录有删除权限。

安全要求：
- 只能删除空目录。
- 不能递归删除。
- 不能删除硬保护路径。
- 不能删除 `CoolWorkspace` 根目录或 SessionRoot 关键目录。


## CToolMoveDirectory

Rust 类型名：
- `CToolMoveDirectory`

文件名：
- `ctool_move_directory.rs`

工具名：
- `ctool_move_directory`

功能介绍：
- 移动或重命名单个目录。

参数语义：
- `from`：来源目录路径。
- `to`：目标目录路径。

权限要求：
- 需要对来源路径和目标路径通过移动权限检查。

安全要求：
- 只能移动目录，不能移动文件。
- 不允许覆盖已有目录。
- 不能移动硬保护路径。
- 不能移动 `CoolWorkspace` 根目录或 SessionRoot 关键目录。


# 特殊类

## CToolAnnotateMarkdown

Rust 类型名：

* `CToolAnnotateMarkdown`

文件名：

* `ctool_annotate_markdown.rs`

工具名：

* `ctool_annotate_markdown`

功能介绍：

* 给 Markdown 文档添加批注 / 高亮标记。
* 只用于 `.md` / `.markdown` 文档。
* 通过插入 HTML `<mark>` 标签，对指定文本进行视觉标注。
* 普通批注使用绿色高亮：

```html
<mark style="background:#d3f8b6">被批注文本</mark>
```

* 重要批注使用红色高亮：

```html
<mark style="background:#ff4d4f">被批注文本</mark>
```

* 用于在不破坏原文结构的前提下，对设计文档、任务文档、审查文档进行标注。

权限要求：

* 目标文件必须位于 CToolScope 允许访问的范围内。
* 目标文件必须不是 Hidden。
* 目标文件可以是 ReadWrite。


* 目标文件也可以是 ReadOnly 。但绝不可以是Hidden
* 这是唯一允许对 ReadOnly 文件进行受限写入的 基础类 CTool。
* 对 ReadOnly 文件的写入只能是添加 Markdown 批注，不能进行普通编辑。

权限例外：

* 普通写入工具不能修改 ReadOnly 文件。
* `ctool_annotate_markdown` 可以修改 ReadOnly 文件，但只能执行“批注添加”这一种操作。
* 该例外不能扩展到代码文件。
* 该例外不能扩展到普通文本重写。
* 该例外不能用于删除、替换、移动、重排原文内容。

安全要求：

* 只能添加批注。
* 不允许删除任何已有内容。
* 不允许改写已有正文。
* 不允许重排段落。
* 不允许格式化整个文件。
* 不允许修改代码块内容。
* 不允许在代码块内部添加 `<mark>`。
* 不允许在行内代码内部添加 `<mark>`。
* 不允许修改 YAML front matter。
* 不允许修改 HTML 注释中的内容。
* 不允许执行 Markdown 中的任何链接、脚本、命令。
* 不允许联网。
* 不允许运行命令。
* 不允许调用 shell。
* 不允许调用 Python。
* 不允许编译。
* 不允许安装依赖。
* 不允许创建新文件。
* 不允许删除文件。
* 不允许重命名文件。


代码段规则：

* fenced code block 内禁止批注。
* 行内代码禁止批注。
* 代码块边界包括：

````markdown
```text
code
````

````

- 也包括：

```markdown
~~~text
code
~~~
````

* 如果目标文本位于代码块中，工具必须拒绝操作。
* 如果目标文本跨越代码块边界，工具必须拒绝操作。
* 如果无法可靠判断是否处于代码块中，工具必须拒绝操作。

批注类型：

* `normal`

  * 普通批注。
  * 使用绿色背景。
  * 格式：

```html
<mark style="background:#d3f8b6">原文</mark>
```

* `important`

  * 重要批注。
  * 使用红色背景。
  * 格式：

```html
<mark style="background:#ff4d4f">原文</mark>
```

批注方向诉求：

* 批注可以带有方向标记。
* 方向标记用于表示批注意见指向原文的上方还是下方。
* 向上方向使用：`↑`
* 向下方向使用：`↓`
* 方向标记应出现在批注内容的最开头。


参数建议：

* `path`

  * Markdown 文件路径。
* `target_text`

  * 需要批注的原文文本。
* `annotation_kind`

  * `normal` 或 `important`。
* `annotation_direction`

  * 可选参数。
  * `up` 表示批注意见指向上方内容。
  * `down` 表示批注意见指向下方内容。
  * 如果不传，默认不添加方向标记，或按定稿规则选择默认值。

* `occurrence`

  * 当 `target_text` 出现多次时，指定第几个匹配项。
* `allow_readonly`

  * 是否允许对 ReadOnly 文件执行批注。
  * 默认允许，但只能用于本工具。
* `dry_run`

  * 只预览，不写入。
  * 默认可以为 false，但建议支持。

行为要求：

* 工具应先读取 Markdown 文件。
* 工具应解析代码块范围。
* 工具应定位 `target_text`。
* 工具应确认目标文本不在代码块、块范围。
* 工具应定位 `target_text`。
* 工具应确认目标文本不在代码块、行内代码、YAML front matter 中。
* 工具应只在目标文本两侧插入 `<mark>` 开始和结束标签。
* 原文文本本身必须完整保留。
* 修改后文件除新增 `<mark>` 标签外，不应发生其他变化。
* 如果匹配到多个位置且未指定 `occurrence`，必须拒绝并提示需要指定。
* 如果未匹配到目标文本，必须拒绝。
* 如果目标文本已经被 `<mark>` 包裹，默认拒绝重复批注。
* 如果文件不是 Markdown，必须拒绝。
* 如果文件是 Hidden，必须拒绝。
* 如果路径超出 Scope，必须拒绝。

输出要求：

* 返回是否成功。
* 返回批注类型。
* 返回文件路径。
* 返回命中的行号。
* 返回修改前后的局部预览。
* 如果拒绝操作，必须说明拒绝原因。
* 如果目标文件是 ReadOnly，输出中必须明确说明这是 Markdown 批注工具的受限例外。

示例：

普通批注：

```markdown
<mark style="background:#d3f8b6">士大夫</mark>
```

重要批注：

```markdown
<mark style="background:#ff4d4f">阿斯蒂芬阿斯蒂芬</mark>
```


# 基础功能增强类：只读型 CTool

基础功能增强类只读型 CTool 负责更高效地读取日志、搜索结果和错误摘要。

这类工具必须遵守以下规则：

- 只能读取和搜索，不能修改文件系统。
- 必须经过 CToolScope 读取或搜索权限检查。
- 不能读取 `.cool`、`.cool-system` 等硬保护配置区。
- 不能绕过 hide 规则展示隐藏路径。
- 默认只处理 UTF-8 文本内容。
- 必须限制返回结果数量和单文件读取大小。


## CToolTailFile

Rust 类型名：
- `CToolTailFile`

文件名：
- `ctool_tail_file.rs`

工具名：
- `ctool_tail_file`

功能介绍：
- 读取 UTF-8 文本文件末尾内容。
- 用于查看测试日志、构建日志、长输出日志的最后部分。
- 支持按末尾行数和最大返回字节数限制输出。

参数语义：
- `path`：目标文件路径。
- `lines`：可选。最多返回末尾多少行。
- `max_bytes`：可选。最多返回多少字节。

权限要求：
- 需要对目标文件有读取权限。

安全要求：
- 只能读取 UTF-8 文本文件。
- 不允许读取隐藏或硬保护路径。
- 必须限制返回大小。


## CToolRgSearchContext

Rust 类型名：
- `CToolRgSearchContext`

文件名：
- `ctool_rg_search_context.rs`

工具名：
- `ctool_rg_search_context`

功能介绍：
- 搜索字面量文本，并返回命中行前后的上下文。
- 用于查看错误、失败测试、关键日志附近的相关内容。

参数语义：
- `path`：搜索根路径。
- `query`：要搜索的字面量文本。
- `before`：每个命中点前返回多少行。
- `after`：每个命中点后返回多少行。
- `case_sensitive`：是否区分大小写。
- `max_depth`：最大搜索深度。
- `max_results`：最大返回命中数。
- `include_hidden`：是否包含点号开头的隐藏文件名。

权限要求：
- 需要对搜索根路径有搜索权限。
- 每个被读取文件都需要读取权限。

安全要求：
- 只搜索 UTF-8 文本文件。
- 不返回隐藏或硬保护路径内容。
- 必须限制上下文行数和最大结果数。


## CToolRegexSearch

Rust 类型名：
- `CToolRegexSearch`

文件名：
- `ctool_regex_search.rs`

工具名：
- `ctool_regex_search`

功能介绍：
- 使用 Rust regex 搜索 UTF-8 文本文件。
- 用于搜索失败测试名、错误码、summary、结构化日志片段等。

参数语义：
- `path`：搜索根路径。
- `pattern`：Rust regex 正则表达式。
- `case_sensitive`：是否区分大小写。
- `max_depth`：最大搜索深度。
- `max_results`：最大返回命中数。
- `include_hidden`：是否包含点号开头的隐藏文件名。

权限要求：
- 需要对搜索根路径有搜索权限。
- 每个被读取文件都需要读取权限。

安全要求：
- 只搜索 UTF-8 文本文件。
- 正则表达式必须先编译校验。
- 不返回隐藏或硬保护路径内容。
- 必须限制最大结果数。


## CToolCountMatches

Rust 类型名：
- `CToolCountMatches`

文件名：
- `ctool_count_matches.rs`

工具名：
- `ctool_count_matches`

功能介绍：
- 统计匹配数量，不返回所有匹配行。
- 用于判断问题规模，例如某个错误出现了多少行、多少文件受影响。
- 默认按字面量匹配，也可以通过 `is_regex` 使用正则匹配。

参数语义：
- `path`：搜索根路径。
- `query`：要匹配的文本或正则表达式。
- `is_regex`：是否把 `query` 当作正则表达式。
- `case_sensitive`：是否区分大小写。
- `max_depth`：最大搜索深度。
- `include_hidden`：是否包含点号开头的隐藏文件名。

返回信息：
- `file_count`：参与统计的 UTF-8 文本文件数量。
- `matching_file_count`：至少有一行命中的文件数量。
- `line_match_count`：命中的行数。

权限要求：
- 需要对搜索根路径有搜索权限。
- 每个被读取文件都需要读取权限。

安全要求：
- 只统计 UTF-8 文本文件。
- 不返回匹配行正文，避免输出过大。
- 不统计隐藏或硬保护路径内容。


## CToolExtractLinesMatching

Rust 类型名：
- `CToolExtractLinesMatching`

文件名：
- `ctool_extract_lines_matching.rs`

工具名：
- `ctool_extract_lines_matching`

功能介绍：
- 提取匹配行并输出为列表。
- 用于整理失败测试名、错误摘要、日志 summary。
- 支持去重和排序。
- 默认按字面量匹配，也可以通过 `is_regex` 使用正则匹配。

参数语义：
- `path`：搜索根路径。
- `query`：要匹配的文本或正则表达式。
- `is_regex`：是否把 `query` 当作正则表达式。
- `case_sensitive`：是否区分大小写。
- `unique`：是否按行文本去重。
- `sort`：是否按行文本排序。
- `max_depth`：最大搜索深度。
- `max_results`：最大返回行数。
- `include_hidden`：是否包含点号开头的隐藏文件名。

权限要求：
- 需要对搜索根路径有搜索权限。
- 每个被读取文件都需要读取权限。

安全要求：
- 只提取 UTF-8 文本文件中的行。
- 不返回隐藏或硬保护路径内容。
- 必须限制最大返回行数。
