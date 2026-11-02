; Inno Setup Script for XMCompanion
; Zero-elevation Windows installer for Sony Bluetooth Headphones (WH-1000XM3 / XM4 / XM5)
#define MyAppName "XMCompanion"
#define MyAppVersion "0.3.0"
#define MyAppPublisher "DooperDrew & XMCompanion Contributors"
#define MyAppURL "https://github.com/DooperDrew/xmcompanion"
#define MyAppExeName "XMCompanion.exe"
#define MyLegacyExeName "sonybridge-app.exe"
#define MyCliExeName "sonybridge-cli.exe"

[Setup]
AppId={{8E74A9D2-51C0-4A9E-9B42-7C8F901D2A3B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Programs\XMCompanion
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; Zero Administrator elevation required (per-user install into %LOCALAPPDATA%)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist
OutputBaseFilename=XMCompanion-v{#MyAppVersion}-Setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
CloseApplicationsFilter={#MyAppExeName},{#MyLegacyExeName},{#MyCliExeName}
UninstallDisplayIcon={app}\{#MyAppExeName}
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=XMCompanion Setup (Sony Headphones Companion for Windows 10/11)

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "autostart"; Description: "Start XMCompanion automatically at Windows sign-in"; GroupDescription: "Startup options:"

[Files]
Source: "..\target\release\{#MyLegacyExeName}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\target\release\{#MyLegacyExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\{#MyCliExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\crates\sonybridge-app\locales\*"; DestDir: "{app}\locales"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\docs\user-guide.md"; DestDir: "{app}"; Flags: ignoreversion; DestName: "user-guide.md"
Source: "uninstall.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "setup-wizard.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "install.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "setup.cmd"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\XMCompanion"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Comment: "Sony Bluetooth Headphones System Tray Companion"
Name: "{group}\XMCompanion Diagnostic Probe"; Filename: "powershell.exe"; Parameters: "-NoExit -Command ""& '{app}\{#MyCliExeName}' probe"""; WorkingDir: "{app}"; Comment: "Run Bluetooth RFCOMM & Windows Core Audio diagnostic probe"
Name: "{group}\Uninstall XMCompanion"; Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\uninstall.ps1"""; Comment: "Uninstall XMCompanion"
Name: "{autodesktop}\XMCompanion"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon; Comment: "XMCompanion"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "XMCompanion"; ValueData: """{app}\{#MyAppExeName}"""; Tasks: autostart; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch XMCompanion"; Flags: nowait postinstall skipifsilent checked

[UninstallDelete]
Type: files; Name: "{app}\*.log"
Type: dirifempty; Name: "{app}\locales"
Type: dirifempty; Name: "{app}"

[Code]
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/F /IM XMCompanion.exe /IM sonybridge-app.exe /IM sonybridge-cli.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := '';
end;

function InitializeUninstall(): Boolean;
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/F /IM XMCompanion.exe /IM sonybridge-app.exe /IM sonybridge-cli.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := True;
end;
