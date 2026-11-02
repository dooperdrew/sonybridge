<#
.SYNOPSIS
    Installs XMCompanion for the current user without requiring Administrator privileges.

.DESCRIPTION
    Installs XMCompanion into %LOCALAPPDATA%\Programs\XMCompanion, creates Start Menu
    and Desktop shortcuts, registers the Windows Installed Apps (Uninstall) entry, optionally configures
    autostart on Windows logon (HKCU), and launches the application.

.PARAMETER TargetDir
    Installation directory. Defaults to "$env:LOCALAPPDATA\Programs\XMCompanion".

.PARAMETER CreateDesktopShortcut
    Creates a desktop shortcut to XMCompanion. Default is on.

.PARAMETER NoDesktopShortcut
    Disables creating the desktop shortcut.

.PARAMETER CreateStartMenuShortcut
    Creates Start Menu shortcuts to XMCompanion and Diagnostic CLI. Default is on.

.PARAMETER NoStartMenuShortcut
    Disables creating Start Menu shortcuts.

.PARAMETER EnableAutostart
    Configures XMCompanion to launch automatically on Windows login (per-user HKCU). Default is on.

.PARAMETER NoAutostart
    Disables autostart on Windows login.

.PARAMETER LaunchNow
    Starts XMCompanion immediately after installation. Default is on.

.PARAMETER NoLaunch
    Prevents launching XMCompanion immediately after installation.

.EXAMPLE
    .\install.ps1
    .\install.ps1 -TargetDir "C:\Tools\XMCompanion" -NoAutostart
#>

[CmdletBinding()]
param (
    [string]$TargetDir = "$env:LOCALAPPDATA\Programs\XMCompanion",
    [switch]$CreateDesktopShortcut,
    [switch]$NoDesktopShortcut,
    [switch]$CreateStartMenuShortcut,
    [switch]$NoStartMenuShortcut,
    [switch]$EnableAutostart,
    [switch]$NoAutostart,
    [switch]$LaunchNow,
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
if (-not $ScriptDir) { $ScriptDir = (Get-Location).Path }

$WizardPath = Join-Path $ScriptDir "setup-wizard.ps1"
if (Test-Path $WizardPath) {
    & $WizardPath `
        -TargetDir $TargetDir `
        -NoDesktopShortcut:$NoDesktopShortcut `
        -NoStartMenuShortcut:$NoStartMenuShortcut `
        -NoAutostart:$NoAutostart `
        -NoLaunch:$NoLaunch `
        -Silent
} else {
    throw "setup-wizard.ps1 not found in $ScriptDir"
}
