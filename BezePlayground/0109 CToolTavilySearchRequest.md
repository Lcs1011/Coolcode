
位于 CoolSystemDir\tavily.toml 中设置 token

示范：(里面的 token 是瞎写的)

``` toml
  enabled = true  # 表示是否允许使用

  [[tokens]]
  name = "main"
  api_key = "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"
  enabled = true

  [[tokens]]
  name = "backup_1"
  api_key = "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"
  enabled = true

  [[tokens]]
  name = "backup_2"
  api_key = "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"
  enabled = false
```

#### KP
在 TOML 语法中，双括号 `[[表格名]]` 声明的是一个表数组
因而叫tokens而不是 token
toml文件需要用 # 注释而不是 //
