; SPDX-License-Identifier: GPL-3.0-only
#ifndef Version
  #error Version requerida
#endif
[Setup]
AppId={{A758C875-3CA1-42F0-A1E8-85F1D58F7E42}
AppName=Icaro Launcher
AppVersion={#Version}
AppPublisher=00faithlss
AppPublisherURL=https://github.com/00faithlss/Icaro-launcher
DefaultDirName={localappdata}\Programs\Icaro Launcher
DefaultGroupName=Icaro Launcher
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#Salida}
OutputBaseFilename={#Nombre}
SetupIconFile={#Raiz}\crates\ui\assets\icono\icaro.ico
UninstallDisplayIcon={app}\IcaroLauncher.exe
LicenseFile={#Raiz}\LICENSE
Compression=lzma2/fast
SolidCompression=yes
WizardStyle=modern
CloseApplications=no
RestartApplications=no
UninstallFilesDir={app}\uninstall
VersionInfoVersion={#Version}
VersionInfoProductName=Icaro Launcher

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "Crear un acceso directo en el escritorio"; Flags: unchecked

[Files]
Source: "{#Preparacion}\IcaroLauncher.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Preparacion}\icaro-sync-host.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Preparacion}\licencias\*"; DestDir: "{app}\licencias"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Icaro Launcher"; Filename: "{app}\IcaroLauncher.exe"
Name: "{autodesktop}\Icaro Launcher"; Filename: "{app}\IcaroLauncher.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\IcaroLauncher.exe"; Description: "Abrir Icaro Launcher"; Flags: nowait postinstall skipifsilent

; No se eliminan carpetas de AppData ni se incluye [UninstallDelete].
