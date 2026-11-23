#define MyAppName "Crimson System Boost"
#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif

[Setup]
AppId={{A8B2F6C1-4D37-4E91-9A62-7F5C3B18D204}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=Crimson System Boost
DefaultDirName={localappdata}\Programs\Crimson System Boost
DefaultGroupName=Crimson System Boost
DisableProgramGroupPage=yes
CloseApplications=yes
RestartApplications=yes
CloseApplicationsFilter=crimson-system-boost.exe
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=Output
OutputBaseFilename=Crimson-System-Boost-Setup-v{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName=Crimson System Boost

[Files]
Source: "..\target\release\crimson-system-boost.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Crimson System Boost"; Filename: "{app}\crimson-system-boost.exe"
Name: "{autodesktop}\Crimson System Boost"; Filename: "{app}\crimson-system-boost.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Run]
Filename: "{app}\crimson-system-boost.exe"; Description: "Launch Crimson System Boost"; Flags: postinstall nowait skipifsilent
