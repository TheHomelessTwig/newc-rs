# newc installer for Windows.
#
# Downloads the latest newc.exe release, installs it to
# %ProgramFiles%\newc, adds that directory to the machine PATH, and
# creates a Start Menu shortcut. Run elevated (the script will
# re-launch itself elevated if needed).
#
# Usage:
#   irm https://raw.githubusercontent.com/TheHomelessTwig/newc-rs/main/packaging/install.ps1 | iex

$ErrorActionPreference = "Stop"

$installDir = Join-Path $env:ProgramFiles "newc"
$exePath    = Join-Path $installDir "newc.exe"
$releaseUrl = "https://github.com/TheHomelessTwig/newc-rs/releases/latest/download/newc-x86_64-windows.exe"

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Admin)) {
    Write-Host "Re-launching with administrator privileges (needed for Program Files + machine PATH)..."
    $scriptPath = $MyInvocation.MyCommand.Path
    if ($scriptPath) {
        Start-Process powershell -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`""
    } else {
        # Invoked via `irm ... | iex` — no script file on disk, re-run the one-liner elevated.
        $oneLiner = "irm https://raw.githubusercontent.com/TheHomelessTwig/newc-rs/main/packaging/install.ps1 | iex"
        Start-Process powershell -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -Command `"$oneLiner`""
    }
    exit
}

Write-Host "Installing newc to $installDir ..."
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Invoke-WebRequest -Uri $releaseUrl -OutFile $exePath
Write-Host "newc.exe installed."

# Add to machine PATH (idempotent).
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if (($machinePath -split ";") -notcontains $installDir) {
    Write-Host "Adding $installDir to the system PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$machinePath;$installDir", "Machine")
} else {
    Write-Host "$installDir already on PATH."
}

# Start Menu shortcut.
$startMenuDir = Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs"
$shortcutPath = Join-Path $startMenuDir "newc.lnk"
Write-Host "Creating Start Menu shortcut..."
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $exePath
$shortcut.WorkingDirectory = $installDir
$shortcut.Description = "C project scaffolding & management"
$shortcut.Save()

Write-Host ""
Write-Host "newc installed successfully."
Write-Host "Open a new terminal for the updated PATH to take effect, or launch 'newc' from the Start Menu now."
