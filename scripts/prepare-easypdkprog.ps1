param(
    [Parameter(Mandatory = $true)]
    [string] $Destination
)

$ErrorActionPreference = "Stop"
$Version = "1.3"
$ExpectedSha256 = "d907384d08152fe452cc65f51a0c2751d39f3e3a832632bfadacc8fd349be840"
$SourceUrl = "https://github.com/free-pdk/easy-pdk-programmer-software/releases/download/$Version/EASYPDKPROG_WIN_20200713_1.3.zip"
$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) "kpdk-easypdkprog-$PID"
$Archive = Join-Path $WorkDir "easypdkprog.zip"
$Extracted = Join-Path $WorkDir "extracted"

try {
    New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
    Invoke-WebRequest -Uri $SourceUrl -OutFile $Archive

    $ActualSha256 = (Get-FileHash -Algorithm SHA256 $Archive).Hash.ToLowerInvariant()
    if ($ActualSha256 -ne $ExpectedSha256) {
        throw "SHA-256 mismatch for $($SourceUrl): expected $ExpectedSha256, got $ActualSha256"
    }

    Expand-Archive -Path $Archive -DestinationPath $Extracted
    $Source = Join-Path $Extracted "EASYPDKPROG/easypdkprog.exe"
    if (-not (Test-Path -PathType Leaf $Source)) {
        throw "easypdkprog.exe was not found in the official release archive"
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Destination) | Out-Null
    Copy-Item -Path $Source -Destination $Destination -Force
    $LicenseSource = Join-Path $Extracted "EASYPDKPROG/LICENSE"
    $LicenseDestination = Join-Path (Split-Path -Parent $Destination) "easypdkprog-LICENSE"
    Copy-Item -Path $LicenseSource -Destination $LicenseDestination -Force
    & $Destination --version
    if ($LASTEXITCODE -ne 0) {
        throw "installed easypdkprog.exe exited with status $LASTEXITCODE"
    }
}
finally {
    if (Test-Path $WorkDir) {
        Remove-Item -Recurse -Force $WorkDir
    }
}
