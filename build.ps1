# Environment variables:
#
# If JIEBA_VIM_BUILD_FROM_SOURCE=1, then skip downloading cdylib and build from
# source directly.
#
# If JIEBA_VIM_DOWNLOAD_BASE_URL is non-empty, then download cdylib from that
# base url without falling back to building from source. This is intended to be
# used in tests.
#
# If JIEBA_VIM_INSTALL_NVIM=1, then install lua5.1 binding for nvim; otherwise
# install py3 binding for vim. If this variable is not set, install py3 binding
# for vim.

param()

function Has-Command {
    param(
        [Parameter(ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )
    foreach ($c in $Commands) {
        if (-not (Get-Command $c -ErrorAction SilentlyContinue)) { return $false }
    }
    return $true
}

# Define these variables:
#   - SCRIPT_DIR
#   - DEST_DIR
#   - DEST_NAME
#   - BINDING
#   - ASSET_NAME
#   - LIB_NAME
function Prepare-Release {
    $Script:SCRIPT_DIR = $PSScriptRoot
    if (-not $Script:SCRIPT_DIR) {
        $Script:SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Definition
    }
    if ($env:JIEBA_VIM_INSTALL_NVIM -eq "1") {
        $Script:BINDING = "lua51"
        $Script:DEST_DIR = "lua\jieba_vim"
        $Script:DEST_NAME = "jieba_vim_rs.dll"
        $Script:LIB_NAME = "jieba_vim_jieba_vim_rs.dll"
    } else {
        $Script:BINDING = "py3"
        $Script:DEST_DIR = "pythonx\jieba_vim"
        $Script:DEST_NAME = "jieba_vim_rs.pyd"
        $Script:LIB_NAME = "jieba_vim_rs.dll"
    }
    # 简化：release 中只有 x86_64 Windows 的 DLL
    $Script:ASSET_NAME = "jieba_vim_rs-x86_64-pc-windows-msvc-$Script:BINDING.dll"
}

function Download-Release {
    try {
        $curr_commit = (& git rev-parse HEAD) -join ''
    } catch {
        return $false
    }
    $curr_tag = (& git tag --points-at $curr_commit) -join "`n"
    if (-not $curr_tag) { return $false }
    $baseUrl = "https://github.com/kkew3/jieba.vim/releases/download/$curr_tag/"

    $url = $baseUrl + $Script:ASSET_NAME
    $dest = Join-Path $Script:DEST_DIR $Script:DEST_NAME
    try {
        if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
            & curl.exe -fsSL -o $dest $url
        } else {
            Invoke-WebRequest -Uri $url -OutFile $dest -ErrorAction Stop
        }
        if (Test-Path $dest) { return $true }
    } catch {
        if (Test-Path $dest) { Remove-Item $dest -ErrorAction SilentlyContinue }
    }
    return $false
}

function Download-Release-Url {
    Remove-Item "$Script:DEST_DIR\$Script:DEST_NAME" -Force -ErrorAction SilentlyContinue
    $baseUrl = "$env:JIEBA_VIM_DOWNLOAD_BASE_URL/"
    $url = $baseUrl + $Script:ASSET_NAME
    $dest = Join-Path $Script:DEST_DIR $Script:DEST_NAME
    try {
        if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
            & curl.exe -fsSL -o $dest $url
        } else {
            Invoke-WebRequest -Uri $url -OutFile $dest -ErrorAction Stop
        }
        if (Test-Path $dest) { return $true }
    } catch {
        if (Test-Path $dest) { Remove-Item $dest -ErrorAction SilentlyContinue }
    }
    return $false
}

function Build-From-Source {
    $color_when = if ($env:VIMRUNTIME) { 'never' } else { 'auto' }

    & cargo clean --color=$color_when --manifest-path rust_backend\Cargo.toml
    & cargo build -r --color=$color_when --manifest-path rust_backend\Cargo.toml --package jieba_vim_rs_binding_$Script:BINDING
    if ($LASTEXITCODE -ne 0) { return $false }

    # Remove-Item: used to delete $DEST_NAME in case it's a symlink
    Remove-Item "$Script:DEST_DIR\$Script:DEST_NAME" -Force -ErrorAction SilentlyContinue
    Copy-Item "rust_backend\target\release\$Script:LIB_NAME" -Destination "$Script:DEST_DIR\$Script:DEST_NAME"
    return $?
}

Prepare-Release
Push-Location -Path $Script:SCRIPT_DIR
try {
    if ($env:JIEBA_VIM_DOWNLOAD_BASE_URL) {
        if (Download-Release-Url) { exit 0 } else { exit 1 }
    }
    if ($env:JIEBA_VIM_BUILD_FROM_SOURCE -ne "1" -and (Has-Command git)) {
        if (Download-Release) { exit 0 }
    }
    if (Has-Command cargo) {
        if (Build-From-Source) { exit 0 } else { exit 1 }
    } else {
        Write-Error "jieba.vim build: cannot build from source: 'cargo' not found"
        exit 1
    }
} finally {
    Pop-Location
}
