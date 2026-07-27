param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"
$repository = "Markiewic/kpdk"
$installDirectory = Join-Path $env:LOCALAPPDATA "kpdk\bin"
$archive = Join-Path $env:TEMP "kpdk-windows-x64.zip"

if ($Version -eq "latest") {
    $release = Invoke-RestMethod "https://api.github.com/repos/$repository/releases/latest"
    $downloadUrl = ($release.assets | Where-Object name -eq "kpdk-windows-x64.zip").browser_download_url
} else {
    $downloadUrl = "https://github.com/$repository/releases/download/$Version/kpdk-windows-x64.zip"
}

if (-not $downloadUrl) {
    throw "The release does not contain kpdk-windows-x64.zip"
}

New-Item -ItemType Directory -Force $installDirectory | Out-Null
Invoke-WebRequest $downloadUrl -OutFile $archive
Expand-Archive $archive $installDirectory -Force
Remove-Item $archive

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ";") -notcontains $installDirectory) {
    [Environment]::SetEnvironmentVariable(
        "Path",
        ($userPath.TrimEnd(";") + ";" + $installDirectory),
        "User"
    )
}

Write-Host "Installed kpdk to $installDirectory"
Write-Host "Open a new terminal and run: kpdk --help"
