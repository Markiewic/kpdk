param(
    [switch]$Yes
)

$ErrorActionPreference = "Stop"

$InstallDir = if ($env:KPDK_INSTALL_DIR) {
    $env:KPDK_INSTALL_DIR
}
else {
    Join-Path $env:LOCALAPPDATA "Programs/kpdk/bin"
}
$DataDir = Join-Path $env:LOCALAPPDATA "kpdk"
$ExpectedDataDir = [System.IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA "kpdk"))
if ([System.IO.Path]::GetFullPath($DataDir) -ne $ExpectedDataDir) {
    throw "Refusing to remove unexpected data directory: $DataDir"
}

if (-not $Yes) {
    if (-not [Environment]::UserInteractive -or [Console]::IsInputRedirected) {
        throw "No interactive console detected. Retry with -Yes."
    }
    $Answer = Read-Host "Remove kpdk from $InstallDir and all SDK data from $DataDir? [y/N]"
    if ($Answer -notmatch "^(?i:y|yes)$") {
        Write-Host "Uninstall cancelled."
        exit 0
    }
}

foreach ($File in @("kpdk.exe", "easypdkprog.exe", "easypdkprog-LICENSE")) {
    $Path = Join-Path $InstallDir $File
    if (Test-Path -PathType Leaf $Path) {
        Remove-Item -Force $Path
    }
}
if (Test-Path -PathType Container $DataDir) {
    Remove-Item -Recurse -Force $DataDir
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$RemainingEntries = @(
    $UserPath -split ";" |
        Where-Object { $_ -and $_.TrimEnd("\") -ine $InstallDir.TrimEnd("\") }
)
[Environment]::SetEnvironmentVariable("Path", ($RemainingEntries -join ";"), "User")

Write-Host "Removed kpdk, Easy PDK Programmer, all kpdk SDK data, and the kpdk PATH entry."
