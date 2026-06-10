# CodexRelease.bat

```bat
@echo off
setlocal

rem LauncherDir = this BAT file's directory.
set "LAUNCHER_DIR=%~dp0"
set "COOL_SYSTEM_DIR=%LAUNCHER_DIR%.cool-system"

rem Compatibility env for current/old code paths.
rem Future code should prefer COOL_SYSTEM_DIR and derive config/scope/command from it.
set "COOL_SYSTEM_CONFIG=%COOL_SYSTEM_DIR%\config.toml"

cd /d C:\CodexLab\codex

echo ================================
echo Before launch: related processes
echo ================================
tasklist | findstr /i "codex app-server exec-server"
if errorlevel 1 echo No related process before launch.

echo.
echo ================================
echo Cool paths
echo ================================
echo LauncherDir=%LAUNCHER_DIR%
echo CoolSystemDir=%COOL_SYSTEM_DIR%
echo SessionRoot=%CD%
echo CoolDir=%CD%\.cool
echo COOL_SYSTEM_CONFIG=%COOL_SYSTEM_CONFIG%

echo.
echo ================================
echo Launching Codex RELEASE WITHOUT --safe-mode
echo It should default to SafeMode ON.
echo ================================
echo.

codex-rs\target\release\codex.exe

echo.
echo ================================
echo Codex exited. Waiting 3 seconds...
echo ================================
timeout /t 3 /nobreak >nul

echo.
echo ================================
echo After exit: related processes
echo ================================
tasklist | findstr /i "codex app-server exec-server"
if errorlevel 1 echo No related process after exit.

echo.
echo Test finished.
pause
```

# C:\Arsenal.cool-system\config.toml

```toml
[cool]
name = "CoolCode System Config"
version = 1
```

# C:\Arsenal.cool-system\scope.toml

```toml
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

# C:\Arsenal.cool-system\command.toml

```toml
[ctool_command]
enabled = true

green_exact_commands = []
green_prefixes = []

yellow_prefixes = [
  "cargo check",
  "cargo build",
  "cargo test",
  "cargo fmt",
  "git status",
  "git diff",
  "git add",
  "git commit",
  "rg"
]

red_prefixes = [
  "del",
  "erase",
  "rmdir",
  "rd",
  "remove-item",
  "git reset --hard",
  "git clean -fd",
  "git clone",
  "powershell",
  "pwsh",
  "cmd",
  "python",
  "py",
  "python3",
  "node",
  "curl",
  "wget",
  "invoke-webrequest",
  "invoke-restmethod",
  "shutdown",
  "taskkill",
  "reg",
  "netsh",
  "start",
  "start-process",
  "explorer"
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
  ".gz",
  "&&",
  "||",
  ">",
  ">>",
  "|"
]

# Target design:
# These fields require the future Blocked implementation.
# Current code may ignore them until Blocked is added.
blocked_prefixes = [
  "python -m venv",
  "py -m venv",
  "python3 -m venv",
  "virtualenv",
  "python -m virtualenv",
  "py -m virtualenv",
  "pip install",
  "pip3 install",
  "python -m pip install",
  "py -m pip install",
  "python3 -m pip install",
  "pipx install",
  "uv venv",
  "uv python install",
  "uv tool install",
  "rye",
  "hatch env",
  "pipenv install",
  "poetry install",
  "poetry env",
  "conda create",
  "conda install",
  "mamba create",
  "mamba install",
  "micromamba create",
  "micromamba install",
  "winget install python",
  "choco install python",
  "scoop install python",
  "msiexec",
  "python -m ensurepip",
  "py -m ensurepip"
]

blocked_contains = [
  "python.org",
  "install python",
  "python installer",
  "python-3.",
  "python3.",
  "venv",
  "virtualenv",
  "ensurepip",
  "pip install",
  "conda create",
  "conda install",
  "pyenv",
  "pyenv-win",
  "AppData\\Local\\Programs\\Python",
  "C:\\Python",
  "Program Files\\Python",
  "WindowsApps\\python",
  "setx PATH",
  "set PATH"
]
```

# C:\CodexLab\codex.cool\config.toml 示例

```toml
[cool]
ctool_scope_base = "CoolWorkspace"

# 不写或留空时，CoolWorkspace 默认等于 SessionRoot。
# 如果希望真正工作区是 codex-rs，则写：
workspace = "codex-rs"
```

# C:\CodexLab\codex.cool\scope.toml 示例

```toml
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

# C:\CodexLab\codex.cool\command.toml 示例

```toml
[ctool_command]
enabled = true

green_exact_commands = [
  "git status",
  "git diff"
]

green_prefixes = []

yellow_prefixes = [
  "cargo check",
  "cargo build",
  "cargo test",
  "cargo fmt",
  "git add",
  "git commit",
  "rg"
]

red_prefixes = []

red_contains = []
```
