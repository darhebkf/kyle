# Kyle installer for Windows
# Usage: irm https://kylefile.dev/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "darhebkf/kyle"
$Target = "x86_64-pc-windows-msvc"
$InstallDir = if ($env:KYLE_INSTALL_DIR) { $env:KYLE_INSTALL_DIR } else { "$env:USERPROFILE\.kyle\bin" }

function Write-Info { param($Message) Write-Host "info: " -ForegroundColor Green -NoNewline; Write-Host $Message }
function Write-Warn { param($Message) Write-Host "warn: " -ForegroundColor Yellow -NoNewline; Write-Host $Message }
function Write-Err { param($Message) Write-Host "error: " -ForegroundColor Red -NoNewline; Write-Host $Message; exit 1 }

function Get-LatestVersion {
    $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
    return $response.tag_name
}

function Install-Kyle {
    Write-Info "Detected platform: windows-x86_64 ($Target)"

    # Get version
    $version = if ($env:KYLE_VERSION) { $env:KYLE_VERSION } else { Get-LatestVersion }
    if (-not $version) {
        Write-Err "Could not determine latest version"
    }
    Write-Info "Installing kyle $version"

    # Download URL
    $url = "https://github.com/$Repo/releases/download/$version/kyle-$Target.zip"

    # Create temp directory
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tmpDir | Out-Null

    try {
        # Download
        Write-Info "Downloading from $url"
        $zipPath = Join-Path $tmpDir "kyle.zip"
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

        # Extract
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        # Install
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        $exePath = Join-Path $InstallDir "kyle.exe"
        Move-Item -Path (Join-Path $tmpDir "kyle.exe") -Destination $exePath -Force

        Write-Info "Installed kyle to $exePath"

        # Check PATH
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$InstallDir*") {
            Write-Warn "$InstallDir is not in your PATH"
            Write-Host ""
            $addToPath = Read-Host "Add to PATH? [Y/n]"
            if ($addToPath -ne "n" -and $addToPath -ne "N") {
                [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
                $env:Path = "$env:Path;$InstallDir"
                Write-Info "Added to PATH"
            } else {
                Write-Host ""
                Write-Host "To add manually, run:"
                Write-Host ""
                Write-Host "  `$env:Path += `";$InstallDir`""
                Write-Host ""
            }
        }

        # Auto-upgrade prompt
        Write-Host ""
        $autoUpgrade = Read-Host "Enable automatic updates? [y/N]"
        if ($autoUpgrade -eq "y" -or $autoUpgrade -eq "Y") {
            & $exePath config set auto_upgrade true 2>$null
            Write-Info "Auto-upgrade enabled"
        }

        # Shell completions prompt
        Write-Host ""
        $completions = Read-Host "Install shell completions? [Y/n]"
        if ($completions -ne "n" -and $completions -ne "N") {
            $profilePath = $PROFILE
            if (-not (Test-Path $profilePath)) {
                New-Item -ItemType File -Path $profilePath -Force | Out-Null
            }
            Add-Content -Path $profilePath -Value "`n# Kyle completions"
            Add-Content -Path $profilePath -Value "& kyle completions bash | Out-String | Invoke-Expression"
            Write-Info "Shell completions added to $profilePath"
        }

        # MCP setup prompt
        Write-Host ""
        $mcpSetup = Read-Host "Set up MCP for AI tools? [y/N]"
        if ($mcpSetup -eq "y" -or $mcpSetup -eq "Y") {
            Write-Host ""
            Write-Host "  1) Claude Code"
            Write-Host "  2) Cursor"
            Write-Host "  3) Windsurf"
            Write-Host "  4) Skip"
            Write-Host ""
            $client = Read-Host "Select AI client [1-4]"

            $mcpConfig = @{
                mcpServers = @{
                    kyle = @{
                        command = $exePath
                        args = @("mcp")
                    }
                }
            } | ConvertTo-Json -Depth 4

            switch ($client) {
                "1" {
                    $ccDir = "$env:USERPROFILE\.claude"
                    $ccFile = "$ccDir\claude_desktop_config.json"
                    if (Test-Path $ccFile) {
                        Write-Warn "$ccFile already exists - add kyle MCP manually:"
                        Write-Host ""
                        Write-Host "  $exePath mcp --config"
                    } else {
                        New-Item -ItemType Directory -Path $ccDir -Force | Out-Null
                        $mcpConfig | Out-File -FilePath $ccFile -Encoding utf8
                        Write-Info "MCP config written to $ccFile"
                    }
                }
                "2" {
                    $cursorDir = "$env:USERPROFILE\.cursor"
                    $cursorFile = "$cursorDir\mcp.json"
                    if (Test-Path $cursorFile) {
                        Write-Warn "$cursorFile already exists - add kyle MCP manually:"
                        Write-Host ""
                        Write-Host "  $exePath mcp --config"
                    } else {
                        New-Item -ItemType Directory -Path $cursorDir -Force | Out-Null
                        $mcpConfig | Out-File -FilePath $cursorFile -Encoding utf8
                        Write-Info "MCP config written to $cursorFile"
                    }
                }
                "3" {
                    $wsDir = "$env:USERPROFILE\.codeium\windsurf"
                    $wsFile = "$wsDir\mcp_config.json"
                    if (Test-Path $wsFile) {
                        Write-Warn "$wsFile already exists - add kyle MCP manually:"
                        Write-Host ""
                        Write-Host "  $exePath mcp --config"
                    } else {
                        New-Item -ItemType Directory -Path $wsDir -Force | Out-Null
                        $mcpConfig | Out-File -FilePath $wsFile -Encoding utf8
                        Write-Info "MCP config written to $wsFile"
                    }
                }
                default {
                    Write-Info "Skipped MCP setup. Run 'kyle mcp --config' anytime to get the config."
                }
            }
        }

        Write-Host ""
        Write-Host "✓ " -ForegroundColor Green -NoNewline
        Write-Host "kyle $version installed successfully!"

        # Verify
        $installedVersion = & $exePath --version 2>$null
        if ($installedVersion) {
            Write-Info "Verified: $installedVersion"
        }

        Write-Host ""
        Write-Host "Run 'kyle --help' to get started."
    }
    finally {
        # Cleanup
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Install-Kyle
