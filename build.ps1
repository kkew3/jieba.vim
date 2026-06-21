
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

function Download-Release {
	if ($env:JIEBA_VIM_BUILD_FROM_SOURCE -eq "1") {
		return $false
	}

	if (-not (Has-Command git)) { return $false }
 	try {
 		$curr_commit = (& git rev-parse HEAD) -join ''
 	} catch {
 		return $false
 	}
 	$curr_tag = (& git tag --points-at $curr_commit) -join "`n"
 	if (-not $curr_tag) { return $false }
 	$baseUrl = "https://github.com/kkew3/jieba.vim/releases/download/$curr_tag/"

	if ($env:JIEBA_VIM_INSTALL_NVIM -eq "1") {
		$binding = "lua51"
		$dest_dir = 'lua\jieba_vim'
		$dest_name = 'jieba_vim_rs.dll'
	} else {
		$binding = "py3"
		$dest_dir = 'pythonx\jieba_vim'
		$dest_name = 'jieba_vim_rs.pyd'
	}

 	# 简化：release 中只有 x86_64 Windows 的 DLL
 	$name = "jieba_vim_rs-x86_64-pc-windows-msvc-$binding.dll"

 	$scriptDir = $PSScriptRoot
 	if (-not $scriptDir) { $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition }
 	$saveDir = Join-Path $scriptDir $dest_dir
 	if (-not (Test-Path $saveDir)) { New-Item -ItemType Directory -Force -Path $saveDir | Out-Null }

 	$url = $baseUrl + $name
 	$dest = Join-Path $saveDir $dest_name
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

	# Assume that build.ps1 only runs on Windows.
	if ($env:JIEBA_VIM_INSTALL_NVIM -eq "1") {
		$binding = "lua51"
		$dest_dir = 'lua\jieba_vim'
		$cdylib_name = 'jieba_vim_jieba_vim_rs.dll'
		$dest_name = 'jieba_vim_rs.dll'
	} else {
		$binding = "py3"
		$dest_dir = 'pythonx\jieba_vim'
		$cdylib_name = 'jieba_vim_rs.dll'
		$dest_name = 'jieba_vim_rs.pyd'
	}

	& cargo clean --color=$color_when --manifest-path rust_backend\Cargo.toml
	& cargo build -r --color=$color_when --manifest-path rust_backend\Cargo.toml --package jieba_vim_rs_binding_$binding
	if ($LASTEXITCODE -ne 0) { return $false }

	# Remove-Item: used to delete $dest_name in case it's a symlink
	Remove-Item "$dest_dir\$dest_name" -Force -ErrorAction SilentlyContinue
	Copy-Item "rust_backend\target\release\$cdylib_name" -Destination "$dest_dir\$dest_name"
	return $?
}

Push-Location -Path $PSScriptRoot
try {
	if (Has-Command git) {
		if (Download-Release) { exit 0 }
	}
	if (Has-Command cargo) {
		if (Build-From-Source) { exit 0 } else { exit 1 }
	} else {
		Write-Error 'cargo not found; cannot build from source.'
		exit 1
	}
} finally {
	Pop-Location
}
