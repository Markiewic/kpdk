param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

$Repository = "Markiewic/kpdk"
$InstallDir = if ($env:KPDK_INSTALL_DIR) {
    $env:KPDK_INSTALL_DIR
}
else {
    Join-Path $env:LOCALAPPDATA "Programs/kpdk/bin"
}
$ArchiveName = "kpdk-windows-x64.zip"
$BaseUrl = if ($Version -eq "latest") {
    "https://github.com/$Repository/releases/latest/download"
}
else {
    "https://github.com/$Repository/releases/download/$Version"
}
$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) "kpdk-install-$PID"
$Archive = Join-Path $WorkDir $ArchiveName
$Checksum = Join-Path $WorkDir "$ArchiveName.sha256"

try {
    New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
    Write-Host "Downloading $ArchiveName from $Version..."
    Invoke-WebRequest -Uri "$BaseUrl/$ArchiveName" -OutFile $Archive
    Invoke-WebRequest -Uri "$BaseUrl/$ArchiveName.sha256" -OutFile $Checksum

    $ExpectedSha256 = ((Get-Content $Checksum -Raw).Trim() -split "\s+")[0].ToLowerInvariant()
    $ActualSha256 = (Get-FileHash -Algorithm SHA256 $Archive).Hash.ToLowerInvariant()
    if ($ActualSha256 -ne $ExpectedSha256) {
        throw "SHA-256 mismatch: expected $ExpectedSha256, got $ActualSha256"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Expand-Archive -Path $Archive -DestinationPath $InstallDir -Force

    $Kpdk = Join-Path $InstallDir "kpdk.exe"
    $Programmer = Join-Path $InstallDir "easypdkprog.exe"
    $License = Join-Path $InstallDir "easypdkprog-LICENSE"
    foreach ($RequiredFile in @($Kpdk, $Programmer, $License)) {
        if (-not (Test-Path -PathType Leaf $RequiredFile)) {
            throw "The release bundle did not contain $RequiredFile"
        }
    }

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $PathEntries = @($UserPath -split ";" | Where-Object { $_ })
    if (-not ($PathEntries | Where-Object { $_.TrimEnd("\") -ieq $InstallDir.TrimEnd("\") })) {
        $NewUserPath = (@($InstallDir) + $PathEntries) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
    }
    if (-not (($env:Path -split ";") | Where-Object { $_.TrimEnd("\") -ieq $InstallDir.TrimEnd("\") })) {
        $env:Path = "$InstallDir;$env:Path"
    }

    & $Kpdk --version
    & $Programmer --version
    Write-Host "Installed kpdk and easypdkprog in $InstallDir"
    Write-Host "The directory was added to your user PATH."
    Write-Host "Next: kpdk sdk install"
}
finally {
    if (Test-Path $WorkDir) {
        Remove-Item -Recurse -Force $WorkDir
    }
}
