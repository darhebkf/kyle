$ErrorActionPreference = "Stop"

$Repo = "behradeslamifar/kyle"
$DefaultInstallDir = "$env:LOCALAPPDATA\kyle"

function Write-Banner {
    Write-Host ""
    Write-Host "  Kyle Installer" -ForegroundColor Cyan
    Write-Host "  ---------------" -ForegroundColor DarkGray
    Write-Host ""
}

function Write-Step {
    param([string]$Message)
    Write-Host "  → " -NoNewline -ForegroundColor Cyan
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "  ✓ " -NoNewline -ForegroundColor Green
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "  ! " -NoNewline -ForegroundColor Yellow
    Write-Host $Message
}

function Prompt-Input {
    param([string]$Question, [string]$Default)
    Write-Host "  $Question [$Default]: " -NoNewline
    $input = Read-Host
    if ([string]::IsNullOrWhiteSpace($input)) {
        return $Default
    }
    return $input
}

function Get-Architecture {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        "X64" { return "amd64" }
        "Arm64" { return "arm64" }
        default {
            Write-Warn "Unsupported architecture: $arch"
            exit 1
        }
    }
}

function Install-FromRelease {
    param([string]$InstallDir, [string]$Version, [string]$Arch)

    $Url = "https://github.com/$Repo/releases/download/$Version/kyle-windows-$Arch.exe"

    Write-Step "Downloading kyle $Version..."

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

    $OutFile = Join-Path $InstallDir "kyle.exe"
    Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing
}

function Install-FromSource {
    param([string]$InstallDir)

    $GoPath = Get-Command go -ErrorAction SilentlyContinue
    if (-not $GoPath) {
        Write-Warn "Go is required to build from source"
        Write-Host "  Install Go from https://go.dev/doc/install" -ForegroundColor White
        exit 1
    }

    $TmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

    Push-Location $TmpDir

    Write-Step "Cloning repository..."
    git clone --depth 1 --quiet "https://github.com/$Repo.git" kyle
    Set-Location kyle

    Write-Step "Building..."
    go build -o kyle.exe ./cmd/do

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Move-Item kyle.exe (Join-Path $InstallDir "kyle.exe") -Force

    Pop-Location
    Remove-Item $TmpDir -Recurse -Force
}

function Add-ToPath {
    param([string]$Dir)

    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -notlike "*$Dir*") {
        Write-Host ""
        Write-Warn "Add to your PATH:"
        Write-Host ""
        Write-Host "      [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$Dir', 'User')" -ForegroundColor DarkGray
        Write-Host ""
        Write-Host "  Or add manually via System Properties → Environment Variables"
        return $false
    }
    return $true
}

function Main {
    Write-Banner

    $Arch = Get-Architecture
    Write-Host "  Detected: windows/$Arch" -ForegroundColor White
    Write-Host ""

    $InstallDir = Prompt-Input "Install location" $DefaultInstallDir

    Write-Host ""
    Write-Step "Fetching latest release..."

    try {
        $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
        $Version = $Release.tag_name

        Install-FromRelease -InstallDir $InstallDir -Version $Version -Arch $Arch
    }
    catch {
        Write-Warn "No releases found, building from source..."
        Install-FromSource -InstallDir $InstallDir
    }

    Write-Host ""
    Write-Success "Installed kyle to $InstallDir\kyle.exe"

    Add-ToPath -Dir $InstallDir

    Write-Host ""
    Write-Host "  Run " -NoNewline
    Write-Host "kyle help" -ForegroundColor White -NoNewline
    Write-Host " to get started."
    Write-Host ""
}

Main
