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

        Write-Host ""
        Write-Host "✓ " -ForegroundColor Green -NoNewline
        Write-Host "kyle $version installed successfully!"
        Write-Host ""
        Write-Host "Run 'kyle --help' to get started."
    }
    finally {
        # Cleanup
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Install-Kyle
