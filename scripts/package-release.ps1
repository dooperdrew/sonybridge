<#
.SYNOPSIS
    Builds and packages SonyBridge into a distributable portable ZIP archive, standalone setup executable, and generates WinGet/Scoop manifests.

.DESCRIPTION
    Compiles the workspace in release mode, validates artifacts, creates a single standalone installer (SonyBridge-Setup.exe),
    assembles a clean portable bundle (SonyBridge-v<Version>-portable.zip and SonyBridge-v<Version>-windows-x64.zip),
    creates an isolated per-version distribution folder (dist/v<Version>/) containing all standalone executables,
    scripts, and archives for seamless testing, and updates WinGet and Scoop manifests.

.PARAMETER Version
    Override the version string. If omitted, extracted from workspace Cargo.toml.

.PARAMETER OutputDir
    Destination root folder for releases (default: "dist").

.PARAMETER SkipBuild
    If specified, skips running 'cargo build --workspace --release' and uses existing binaries.

.EXAMPLE
    .\scripts\package-release.ps1
    .\scripts\package-release.ps1 -Version "0.3.0"
    .\scripts\package-release.ps1 -SkipBuild
#>

[CmdletBinding()]
param (
    [string]$Version,
    [string]$OutputDir = "dist",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "..")).Path
Set-Location $RepoRoot

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "        SonyBridge Release Packaging Tool         " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Repository root: $RepoRoot"

# Ensure cargo is available in PATH
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path (Join-Path $CargoBin "cargo.exe")) {
        Write-Host "Adding $CargoBin to PATH..." -ForegroundColor Yellow
        $env:PATH = "$CargoBin;$env:PATH"
    } else {
        throw "cargo was not found on PATH and was not located at $CargoBin\cargo.exe"
    }
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    $CargoTomlPath = Join-Path $RepoRoot "Cargo.toml"
    $CargoContent = Get-Content $CargoTomlPath -Raw
    if ($CargoContent -match '\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
    } else {
        $Version = "0.3.0"
    }
}

$DistRoot = Join-Path $RepoRoot $OutputDir
if (-not (Test-Path $DistRoot)) {
    New-Item -ItemType Directory -Path $DistRoot -Force | Out-Null
}

# Dedicated isolated folder for this release version
$VersionDistDir = Join-Path $DistRoot "v$Version"
if (-not (Test-Path $VersionDistDir)) {
    New-Item -ItemType Directory -Path $VersionDistDir -Force | Out-Null
}

$PackageName = "SonyBridge-v$Version-windows-x64"
$PortableName = "SonyBridge-v$Version-portable"
$StageDir = Join-Path $DistRoot "staging\$PackageName"
$ZipPath = Join-Path $VersionDistDir "$PackageName.zip"
$PortableZipPath = Join-Path $VersionDistDir "$PortableName.zip"

# 1. Build workspace in release mode
if (-not $SkipBuild) {
    Write-Host "`n[1/6] Building workspace in release mode..." -ForegroundColor Cyan
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    cargo build --workspace --release
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build --workspace --release failed with exit code $LASTEXITCODE"
    }
    Write-Host "Release compilation succeeded." -ForegroundColor Green
} else {
    Write-Host "`n[1/6] Skipping build as requested (-SkipBuild)..." -ForegroundColor Yellow
}

# 2. Verify release binaries and assets
Write-Host "`n[2/6] Verifying release binaries and assets..." -ForegroundColor Cyan
$ReleaseDir = Join-Path $RepoRoot "target\release"
$AppExe = Join-Path $ReleaseDir "sonybridge-app.exe"
$CliExe = Join-Path $ReleaseDir "sonybridge-cli.exe"

if (-not (Test-Path $AppExe)) { throw "Missing release binary: $AppExe" }
if (-not (Test-Path $CliExe)) { throw "Missing release binary: $CliExe" }
Write-Host "  - Found application binary: $AppExe" -ForegroundColor Green
Write-Host "  - Found diagnostic binary:  $CliExe" -ForegroundColor Green

# 3. Create standalone setup executable and stage binaries into dedicated version folder
Write-Host "`n[3/6] Staging single-file installer & binaries into dist\v$Version\..." -ForegroundColor Cyan
$SetupExeName = "SonyBridge-Setup.exe"
$VersionedSetupExeName = "SonyBridge-v$Version-Setup.exe"

# Copy binaries into dedicated version directory
Copy-Item -Path $AppExe -Destination (Join-Path $VersionDistDir "sonybridge-app.exe") -Force
Copy-Item -Path $CliExe -Destination (Join-Path $VersionDistDir "sonybridge-cli.exe") -Force
Copy-Item -Path $AppExe -Destination (Join-Path $VersionDistDir $SetupExeName) -Force
Copy-Item -Path $AppExe -Destination (Join-Path $VersionDistDir $VersionedSetupExeName) -Force

# Also provide root dist copies for compatibility
Copy-Item -Path $AppExe -Destination (Join-Path $DistRoot $SetupExeName) -Force
Copy-Item -Path $AppExe -Destination (Join-Path $DistRoot $VersionedSetupExeName) -Force

Write-Host "  - Staged standalone installer: $VersionDistDir\$SetupExeName" -ForegroundColor Green

# 4. Optional Inno Setup compilation
Write-Host "`n[4/6] Checking for Inno Setup compiler (iscc.exe)..." -ForegroundColor Cyan
$CandidateIscc = @(
    (Get-Command iscc.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -ErrorAction SilentlyContinue),
    "${env:ProgramFiles(x86)}\Inno Setup 6\iscc.exe",
    "$env:ProgramFiles\Inno Setup 6\iscc.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 5\iscc.exe",
    "$env:ProgramFiles\Inno Setup 5\iscc.exe"
)
$IsccExe = $CandidateIscc | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
$InstallerIss = Join-Path $ScriptDir "installer.iss"

if ($IsccExe -and (Test-Path $InstallerIss)) {
    Write-Host "  Found Inno Setup compiler: $IsccExe" -ForegroundColor Green
    Write-Host "  Compiling Inno Setup installer..." -ForegroundColor Cyan
    & $IsccExe "/DMyAppVersion=$Version" $InstallerIss
    if ($LASTEXITCODE -eq 0) {
        $CompiledSetup = Join-Path $DistRoot "SonyBridge-v$Version-Setup.exe"
        if (Test-Path $CompiledSetup) {
            Copy-Item -Path $CompiledSetup -Destination (Join-Path $VersionDistDir $VersionedSetupExeName) -Force
            $SetupItem = Get-Item $CompiledSetup
            $SetupSizeMB = [math]::Round($SetupItem.Length / 1MB, 2)
            Write-Host "  Built Inno Setup installer: $CompiledSetup ($SetupSizeMB MB)" -ForegroundColor Green
        }
    } else {
        Write-Warning "Inno Setup compilation exited with code $LASTEXITCODE"
    }
} else {
    Write-Host "  Inno Setup compiler not found; standalone single-file installer and PowerShell wizard active." -ForegroundColor Gray
}

# 5. Assemble clean portable package & copy standalone assets into dist\v$Version\
Write-Host "`n[5/6] Creating portable ZIP packages and copying scripts into dist\v$Version\..." -ForegroundColor Cyan
if (Test-Path $StageDir) { Remove-Item -Path $StageDir -Recurse -Force }
New-Item -ItemType Directory -Path $StageDir -Force | Out-Null

Copy-Item $AppExe -Destination $StageDir -Force
Copy-Item $CliExe -Destination $StageDir -Force

$LocalesSrc = Join-Path $RepoRoot "crates\sonybridge-app\locales"
$LocalesDst = Join-Path $StageDir "locales"
$LocalesVersionDst = Join-Path $VersionDistDir "locales"

if (Test-Path $LocalesSrc) {
    New-Item -ItemType Directory -Path $LocalesDst -Force | Out-Null
    New-Item -ItemType Directory -Path $LocalesVersionDst -Force | Out-Null
    Get-ChildItem -Path $LocalesSrc -Filter "*.toml" | ForEach-Object {
        Copy-Item $_.FullName -Destination $LocalesDst -Force
        Copy-Item $_.FullName -Destination $LocalesVersionDst -Force
    }
}

@("install.ps1", "uninstall.ps1", "setup-wizard.ps1", "setup.cmd", "installer.iss") | ForEach-Object {
    $ScriptFile = Join-Path $ScriptDir $_
    if (Test-Path $ScriptFile) {
        Copy-Item $ScriptFile -Destination $StageDir -Force
        Copy-Item $ScriptFile -Destination $VersionDistDir -Force
    }
}

Copy-Item (Join-Path $RepoRoot "README.md") -Destination $StageDir -Force
Copy-Item (Join-Path $RepoRoot "LICENSE") -Destination $StageDir -Force
Copy-Item (Join-Path $RepoRoot "CHANGELOG.md") -Destination $StageDir -Force
Copy-Item (Join-Path $RepoRoot "README.md") -Destination $VersionDistDir -Force
Copy-Item (Join-Path $RepoRoot "LICENSE") -Destination $VersionDistDir -Force
Copy-Item (Join-Path $RepoRoot "CHANGELOG.md") -Destination $VersionDistDir -Force

$UserGuidePath = Join-Path $RepoRoot "docs\user-guide.md"
if (Test-Path $UserGuidePath) {
    Copy-Item $UserGuidePath -Destination (Join-Path $StageDir "user-guide.md") -Force
    Copy-Item $UserGuidePath -Destination (Join-Path $VersionDistDir "user-guide.md") -Force
}

if (Test-Path $ZipPath) { Remove-Item -Path $ZipPath -Force }
if (Test-Path $PortableZipPath) { Remove-Item -Path $PortableZipPath -Force }

Compress-Archive -Path "$StageDir\*" -DestinationPath $ZipPath -CompressionLevel Optimal -Force
Copy-Item -Path $ZipPath -Destination $PortableZipPath -Force

# Also copy zips to root dist/ for convenience
Copy-Item -Path $ZipPath -Destination (Join-Path $DistRoot "$PackageName.zip") -Force
Copy-Item -Path $PortableZipPath -Destination (Join-Path $DistRoot "$PortableName.zip") -Force

# Cleanup staging directory
Remove-Item -Path (Join-Path $DistRoot "staging") -Recurse -Force

$ZipInfo = Get-Item $ZipPath
$SizeMB = [math]::Round($ZipInfo.Length / 1MB, 2)
Write-Host "  - Portable package created: $ZipPath ($SizeMB MB)" -ForegroundColor Green
Write-Host "  - Portable alias created:   $PortableZipPath" -ForegroundColor Green

# 6. Auto-generate WinGet and Scoop package manifests
Write-Host "`n[6/6] Generating distribution manifests (WinGet & Scoop)..." -ForegroundColor Cyan
$ManifestGen = Join-Path $ScriptDir "generate-manifests.ps1"
if (Test-Path $ManifestGen) {
    & $ManifestGen -Version $Version -ArchivePath $ZipPath
}

Write-Host "`n==================================================" -ForegroundColor Green
Write-Host "       RELEASE PACKAGING COMPLETE SUCCESS         " -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Green
Write-Host "Dedicated Folder: $VersionDistDir" -ForegroundColor Yellow
Write-Host "Installer:        $VersionDistDir\$SetupExeName" -ForegroundColor Yellow
Write-Host "App Binary:       $VersionDistDir\sonybridge-app.exe" -ForegroundColor Yellow
Write-Host "Archive:          $ZipPath ($SizeMB MB)" -ForegroundColor Yellow
