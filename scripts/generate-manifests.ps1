<#
.SYNOPSIS
    Generates WinGet (multi-file 1.9.0 + singleton) and Scoop package manifests for a given SonyBridge release.

.DESCRIPTION
    Calculates the SHA256 of the distribution zip and updates the manifest files in packaging/
    ready for submission to microsoft/winget-pkgs or Scoop buckets.

.PARAMETER Version
    The release version (e.g. "0.2.0"). If omitted, reads from Cargo.toml.

.PARAMETER ArchivePath
    Path to the release ZIP. Defaults to dist\SonyBridge-v<Version>-windows-x64.zip.

.EXAMPLE
    .\scripts\generate-manifests.ps1
    .\scripts\generate-manifests.ps1 -Version "0.2.0"
#>

[CmdletBinding()]
param(
    [string]$Version,
    [string]$ArchivePath
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "..")).Path

if (-not $Version) {
    $CargoToml = Join-Path $RepoRoot "Cargo.toml"
    if (Test-Path $CargoToml) {
        $content = Get-Content $CargoToml -Raw
        if ($content -match '\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"') {
            $Version = $matches[1].Trim()
        }
    }
}
if (-not $Version) { $Version = "0.2.0" }

Write-Host "Generating package manifests for XMCompanion / SonyBridge v$Version..." -ForegroundColor Cyan

if (-not $ArchivePath) {
    $CandArchives = @(
        (Join-Path $RepoRoot "dist\v$Version\XMCompanion-v$Version-windows-x64.zip"),
        (Join-Path $RepoRoot "dist\XMCompanion-v$Version-windows-x64.zip"),
        (Join-Path $RepoRoot "dist\v$Version\SonyBridge-v$Version-windows-x64.zip"),
        (Join-Path $RepoRoot "dist\SonyBridge-v$Version-windows-x64.zip")
    )
    foreach ($cand in $CandArchives) {
        if (Test-Path $cand) {
            $ArchivePath = $cand
            break
        }
    }
}

if (-not (Test-Path $ArchivePath)) {
    throw "Archive not found at '$ArchivePath'. Please run scripts\package-release.ps1 first."
}

$Hash = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToUpper()
$HashLower = $Hash.ToLowerInvariant()
Write-Host "Archive: $ArchivePath" -ForegroundColor Gray
Write-Host "SHA256:  $Hash" -ForegroundColor Green

$Utf8NoBom = New-Object System.Text.UTF8Encoding $false

# 1. Output WinGet Multi-File 1.9.0 Manifests (packaging/winget/manifests/d/DooperDrew/SonyBridge/<Version>/)
$WingetDir = Join-Path $RepoRoot "packaging\winget\manifests\d\DooperDrew\SonyBridge\$Version"
New-Item -ItemType Directory -Path $WingetDir -Force | Out-Null

$VersionYaml = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.9.0.schema.json
PackageIdentifier: DooperDrew.SonyBridge
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.9.0
"@

$InstallerYaml = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.9.0.schema.json
PackageIdentifier: DooperDrew.SonyBridge
PackageVersion: $Version
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: sonybridge-app.exe
    PortableCommandAlias: sonybridge
  - RelativeFilePath: sonybridge-cli.exe
    PortableCommandAlias: sonybridge-cli
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/DooperDrew/xmcompanion/releases/download/v$Version/XMCompanion-v$Version-windows-x64.zip
    InstallerSha256: $Hash
    UpgradeBehavior: install
ManifestType: installer
ManifestVersion: 1.9.0
"@

$LocaleYaml = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.9.0.schema.json
PackageIdentifier: DooperDrew.SonyBridge
PackageVersion: $Version
PackageLocale: en-US
Publisher: DooperDrew
PublisherUrl: https://github.com/DooperDrew
PublisherSupportUrl: https://github.com/DooperDrew/xmcompanion/issues
Author: DooperDrew
PackageName: SonyBridge
PackageUrl: https://github.com/DooperDrew/xmcompanion
License: MIT
LicenseUrl: https://github.com/DooperDrew/xmcompanion/blob/main/LICENSE
Copyright: Copyright (c) 2026 DooperDrew
ShortDescription: Native Windows 10/11 system tray companion for Sony WH-1000XM3/XM4/XM5 Bluetooth headphones.
Description: |
  SonyBridge / XMCompanion is a lightweight, native Windows 10/11 x64 system tray companion for Sony Bluetooth headphones
  (WH-1000XM3, WH-1000XM4, WH-1000XM5, and WF series). Features real-time battery monitoring, Active Noise Canceling (ANC)
  and 20-step Ambient Sound control, 5-band hardware Equalizer and Clear Bass tuning, 1-click Sound Profiles,
  and smart Windows Core Audio Stereo (A2DP) routing with 0% idle CPU.
Moniker: sonybridge
Tags:
  - sony
  - headphones
  - wh-1000xm3
  - wh-1000xm4
  - wh-1000xm5
  - bluetooth
  - equalizer
  - anc
  - audio
  - rust
  - windows11
ReleaseNotesUrl: https://github.com/DooperDrew/xmcompanion/releases/tag/v$Version
ManifestType: defaultLocale
ManifestVersion: 1.9.0
"@

[System.IO.File]::WriteAllText((Join-Path $WingetDir "DooperDrew.SonyBridge.yaml"), $VersionYaml, $Utf8NoBom)
[System.IO.File]::WriteAllText((Join-Path $WingetDir "DooperDrew.SonyBridge.installer.yaml"), $InstallerYaml, $Utf8NoBom)
[System.IO.File]::WriteAllText((Join-Path $WingetDir "DooperDrew.SonyBridge.locale.en-US.yaml"), $LocaleYaml, $Utf8NoBom)
Write-Host "  -> WinGet 1.9.0 manifests written to: $WingetDir" -ForegroundColor Green

# Also keep singleton convenience manifest at packaging/winget/DooperDrew.SonyBridge.yaml
$SingletonPath = Join-Path $RepoRoot "packaging\winget\DooperDrew.SonyBridge.yaml"
$SingletonYaml = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.singleton.1.6.0.schema.json
PackageIdentifier: DooperDrew.SonyBridge
PackageVersion: $Version
PackageLocale: en-US
Publisher: DooperDrew
PackageName: SonyBridge
PackageUrl: https://github.com/DooperDrew/xmcompanion
License: MIT
ShortDescription: Windows 10/11 system tray companion for Sony WH-1000XM3/XM4/XM5 headphones (Battery, ANC, 5-Band EQ, Clear Bass, A2DP routing).
ManifestType: singleton
ManifestVersion: 1.6.0
Installers:
  - Architecture: x64
    InstallerType: zip
    NestedInstallerType: portable
    NestedInstallerFiles:
      - RelativeFilePath: sonybridge-app.exe
        PortableCommandAlias: sonybridge
      - RelativeFilePath: sonybridge-cli.exe
        PortableCommandAlias: sonybridge-cli
    InstallerUrl: https://github.com/DooperDrew/xmcompanion/releases/download/v$Version/XMCompanion-v$Version-windows-x64.zip
    InstallerSha256: $Hash
"@
[System.IO.File]::WriteAllText($SingletonPath, $SingletonYaml, $Utf8NoBom)

# 2. Output Scoop Manifest
$ScoopDir = Join-Path $RepoRoot "packaging\scoop"
New-Item -ItemType Directory -Path $ScoopDir -Force | Out-Null

$ScoopJson = @"
{
    "version": "$Version",
    "description": "Lightweight Windows 10/11 system tray companion for Sony Bluetooth headphones (WH-1000XM3/XM4/XM5): real-time battery, ANC/Ambient Sound, 5-band Equalizer & Clear Bass, and smart A2DP audio routing.",
    "homepage": "https://github.com/DooperDrew/xmcompanion",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/DooperDrew/xmcompanion/releases/download/v$Version/XMCompanion-v$Version-windows-x64.zip",
            "hash": "$HashLower"
        }
    },
    "bin": [
        [
            "sonybridge-app.exe",
            "sonybridge"
        ],
        "sonybridge-cli.exe"
    ],
    "shortcuts": [
        [
            "sonybridge-app.exe",
            "SonyBridge"
        ]
    ],
    "checkver": "github",
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/DooperDrew/xmcompanion/releases/download/v`$version/XMCompanion-v`$version-windows-x64.zip"
            }
        }
    }
}
"@

[System.IO.File]::WriteAllText((Join-Path $ScoopDir "sonybridge.json"), $ScoopJson, $Utf8NoBom)
Write-Host "  -> Scoop manifest written to: $(Join-Path $ScoopDir 'sonybridge.json')" -ForegroundColor Green

Write-Host "`nAll package distribution manifests generated successfully!" -ForegroundColor Cyan
