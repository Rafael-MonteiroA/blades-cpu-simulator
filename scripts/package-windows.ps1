param(
    [string]$OutputDirectory,
    [string]$CargoTargetDirectory
)

$ErrorActionPreference = "Stop"
$repositoryRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $repositoryRoot "dist"
}
if ([string]::IsNullOrWhiteSpace($CargoTargetDirectory)) {
    $CargoTargetDirectory = Join-Path $repositoryRoot "target"
}
elseif (-not [System.IO.Path]::IsPathRooted($CargoTargetDirectory)) {
    $CargoTargetDirectory = Join-Path $repositoryRoot $CargoTargetDirectory
}

Push-Location $repositoryRoot
try {
    cargo build --release --locked --target-dir $CargoTargetDirectory
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed with exit code $LASTEXITCODE"
    }

    $manifest = Get-Content -LiteralPath (Join-Path $repositoryRoot "Cargo.toml") -Raw
    if ($manifest -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
        throw "Could not read the package version from Cargo.toml"
    }
    $version = $Matches[1]
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $packageName = "BladesCpuSimulator-$version-windows-x64-$timestamp"
    $stagingRoot = Join-Path $repositoryRoot "target\package"
    $stagingDirectory = Join-Path $stagingRoot $packageName
    New-Item -ItemType Directory -Path $stagingDirectory -Force | Out-Null
    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

    $releaseExecutable = Join-Path $CargoTargetDirectory "release\blades-cpu-simulator.exe"
    Copy-Item -LiteralPath $releaseExecutable -Destination $stagingDirectory
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "README.md") -Destination $stagingDirectory
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs") -Destination $stagingDirectory -Recurse
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "examples") -Destination $stagingDirectory -Recurse

    @"
BLADE-16 CPU Simulator $version

Run blades-cpu-simulator.exe to start the desktop application.
The docs folder contains the architecture, instruction set, and Assembly references.
The examples folder contains ready-to-assemble programs.
"@ | Set-Content -LiteralPath (Join-Path $stagingDirectory "RUNNING.txt") -Encoding utf8

    $archivePath = Join-Path $OutputDirectory "$packageName.zip"
    Compress-Archive -Path (Join-Path $stagingDirectory "*") -DestinationPath $archivePath -CompressionLevel Optimal
    Write-Output "Created portable package: $archivePath"
}
finally {
    Pop-Location
}
