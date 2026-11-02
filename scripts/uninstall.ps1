<#
.SYNOPSIS
    Uninstalls XMCompanion, removes autostart entries, shortcuts, Windows Installed Apps entry, and application files.

.DESCRIPTION
    Stops running XMCompanion processes, removes the autostart entry from the
    current user registry (HKCU\Software\Microsoft\Windows\CurrentVersion\Run),
    removes the Windows Add/Remove Programs registration (HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\XMCompanion),
    deletes Start Menu and Desktop shortcuts, removes program files, and optionally purges configuration data
    in %APPDATA%\XMCompanion.

.PARAMETER TargetDir
    The installation directory to remove. Defaults to the folder containing this script if installed,
    or "$env:LOCALAPPDATA\Programs\XMCompanion".

.PARAMETER PurgeData
    If specified, permanently removes user sound profiles and settings in %APPDATA%\XMCompanion without prompting.

.PARAMETER Silent
    Runs the uninstaller silently without prompts or message boxes.

.EXAMPLE
    .\uninstall.ps1
    .\uninstall.ps1 -Silent
    .\uninstall.ps1 -Silent -PurgeData
#>

[CmdletBinding()]
param (
    [string]$TargetDir,
    [switch]$PurgeData,
    [switch]$Silent
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
if (-not $ScriptDir) {
    $ScriptDir = (Get-Location).Path
}

if (-not $TargetDir) {
    if ((Test-Path (Join-Path $ScriptDir "XMCompanion.exe")) -or (Test-Path (Join-Path $ScriptDir "sonybridge-app.exe"))) {
        $TargetDir = $ScriptDir
    } else {
        $TargetDir = "$env:LOCALAPPDATA\Programs\XMCompanion"
        if ((-not (Test-Path $TargetDir)) -and (Test-Path "$env:LOCALAPPDATA\Programs\SonyBridge")) {
            $TargetDir = "$env:LOCALAPPDATA\Programs\SonyBridge"
        }
    }
}

if (-not $Silent) {
    try {
        Add-Type -AssemblyName System.Windows.Forms -ErrorAction SilentlyContinue
        $confirm = [System.Windows.Forms.MessageBox]::Show(
            "Are you sure you want to completely uninstall XMCompanion (Sony Headphones Companion for Windows)?",
            "Uninstall XMCompanion",
            [System.Windows.Forms.MessageBoxButtons]::YesNo,
            [System.Windows.Forms.MessageBoxIcon]::Question
        )
        if ($confirm -ne [System.Windows.Forms.DialogResult]::Yes) {
            Write-Host "Uninstallation cancelled by user."
            return
        }

        if (-not $PurgeData) {
            $askPurge = [System.Windows.Forms.MessageBox]::Show(
                "Do you also want to remove your saved Sound Profiles and settings stored in %APPDATA%\XMCompanion?",
                "Remove User Data?",
                [System.Windows.Forms.MessageBoxButtons]::YesNo,
                [System.Windows.Forms.MessageBoxIcon]::Question
            )
            if ($askPurge -eq [System.Windows.Forms.DialogResult]::Yes) {
                $PurgeData = $true
            }
        }
    } catch {
        Write-Host "==================================================" -ForegroundColor Cyan
        Write-Host "         XMCompanion Uninstallation Tool          " -ForegroundColor Cyan
        Write-Host "==================================================" -ForegroundColor Cyan
        $ans = Read-Host "Are you sure you want to uninstall XMCompanion? (y/N)"
        if ($ans -notmatch "^[yY]") {
            Write-Host "Uninstallation cancelled."
            return
        }
    }
}

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "         XMCompanion Uninstallation Tool          " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Target Directory: $TargetDir"

Write-Host "`n[1/5] Stopping running XMCompanion processes..." -ForegroundColor Cyan
Get-Process -Name "XMCompanion", "sonybridge-app", "sonybridge-cli" -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "  Stopping $($_.ProcessName) (PID $($_.Id))..."
    try {
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    } catch {}
}
Start-Sleep -Milliseconds 400

Write-Host "`n[2/5] Removing autostart entry from HKCU Run..." -ForegroundColor Cyan
$RunKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
@("XMCompanion", "SonyBridge") | ForEach-Object {
    $prop = $_
    try {
        $existing = Get-ItemProperty -Path $RunKey -Name $prop -ErrorAction SilentlyContinue
        if ($existing) {
            Remove-ItemProperty -Path $RunKey -Name $prop -ErrorAction SilentlyContinue
            Write-Host "  Removed '$prop' from $RunKey" -ForegroundColor Green
        }
    } catch {}
}

Write-Host "`n[3/5] Removing Windows Add/Remove Programs entry..." -ForegroundColor Cyan
@("HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\XMCompanion",
  "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\SonyBridge") | ForEach-Object {
    $uk = $_
    try {
        if (Test-Path $uk) {
            Remove-Item -Path $uk -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "  Removed registry key: $uk" -ForegroundColor Green
        }
    } catch {}
}

Write-Host "`n[4/5] Removing Start Menu and Desktop shortcuts..." -ForegroundColor Cyan
@((Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\XMCompanion"),
  (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\SonyBridge")) | ForEach-Object {
    $smd = $_
    if (Test-Path $smd) {
        Remove-Item -Path $smd -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  Removed Start Menu folder: $smd" -ForegroundColor Green
    }
}

$DesktopDir = [Environment]::GetFolderPath([Environment+SpecialFolder]::Desktop)
if (Test-Path $DesktopDir) {
    @("XMCompanion.lnk", "SonyBridge.lnk", "SonyBridge Settings.lnk") | ForEach-Object {
        $deskLnk = Join-Path $DesktopDir $_
        if (Test-Path $deskLnk) {
            Remove-Item -Path $deskLnk -Force -ErrorAction SilentlyContinue
            Write-Host "  Removed Desktop shortcut: $deskLnk" -ForegroundColor Green
        }
    }
}

Write-Host "`n[5/5] Removing application files..." -ForegroundColor Cyan
if (Test-Path $TargetDir) {
    if ($ScriptDir -and ($ScriptDir.TrimEnd('\') -eq $TargetDir.TrimEnd('\'))) {
        Get-ChildItem -Path $TargetDir -Exclude "uninstall.ps1" | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
        Start-Process cmd.exe -ArgumentList "/c cd /d %TEMP% & timeout /t 1 /nobreak >nul & rmdir /s /q `"$TargetDir`"" -WindowStyle Hidden
        Write-Host "  Scheduled target directory cleanup on exit." -ForegroundColor Green
    } else {
        Remove-Item -Path $TargetDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  Removed installation directory: $TargetDir" -ForegroundColor Green
    }
}

if ($PurgeData) {
    @((Join-Path $env:APPDATA "XMCompanion"), (Join-Path $env:APPDATA "SonyBridge")) | ForEach-Object {
        $ad = $_
        if (Test-Path $ad) {
            Remove-Item -Path $ad -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "  Purged user configuration directory: $ad" -ForegroundColor Green
        }
    }
} else {
    Write-Host "  User settings in %APPDATA%\XMCompanion were preserved." -ForegroundColor Gray
}

Write-Host "`n==================================================" -ForegroundColor Green
Write-Host "       XMCOMPANION UNINSTALLATION COMPLETED       " -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Green

if (-not $Silent) {
    try {
        [System.Windows.Forms.MessageBox]::Show(
            "XMCompanion has been successfully removed from your computer.",
            "Uninstall XMCompanion",
            [System.Windows.Forms.MessageBoxButtons]::OK,
            [System.Windows.Forms.MessageBoxIcon]::Information
        ) | Out-Null
    } catch {}
}
