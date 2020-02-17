[Setup]
AppName=Wasmer
AppVersion=0.13.1
DefaultDirName={pf}\Wasmer
DefaultGroupName=Wasmer
Compression=lzma2
SolidCompression=yes
OutputDir=.\
DisableProgramGroupPage=yes
ChangesEnvironment=yes
OutputBaseFilename=WasmerInstaller
WizardImageFile=media\wizard_logo_2.bmp
WizardSmallImageFile=media\wizard_logo_small.bmp
SetupIconFile=media\wizard_logo.ico
DisableWelcomePage=no

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType:string; ValueName: "WASMER_DIR"; \
    ValueData: "{%USERPROFILE}\.wasmer"; Flags: preservestringtype
Root: HKCU; Subkey: "Environment"; ValueType:string; ValueName: "WASMER_CACHE_DIR"; \
    ValueData: "{%USERPROFILE}\.wasmer\cache"; Flags: preservestringtype

[Files]
Source: "..\..\target\release\wasmer.exe"; DestDir: "{app}\bin"
Source: "..\..\wapm-cli\target\release\wapm.exe"; DestDir: "{app}\bin"

[Dirs]
Name: "{%USERPROFILE}\.wasmer"

[Code]
const EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

procedure EnvAddPath(Path: string);
var
    Paths: string;
begin
    { Retrieve current path (use empty string if entry not exists) }
    if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths)
    then Paths := '';

    { Skip if string already found in path }
    if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') > 0 then exit;

    { App string to the end of the path variable }
    Paths := Paths + ';'+ Path +';'

    { Overwrite (or create if missing) path environment variable }
    if RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths)
    then Log(Format('The [%s] added to PATH: [%s]', [Path, Paths]))
    else Log(Format('Error while adding the [%s] to PATH: [%s]', [Path, Paths]));
end;

procedure EnvRemovePath(Path: string);
var
    Paths: string;
    P: Integer;
begin
    { Skip if registry entry not exists }
    if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
        exit;

    { Skip if string not found in path }
    P := Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';');
    if P = 0 then exit;

    { Update path variable }
    Delete(Paths, P - 1, Length(Path) + 1);

    { Overwrite path environment variable }
    if RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths)
    then Log(Format('The [%s] removed from PATH: [%s]', [Path, Paths]))
    else Log(Format('Error while removing the [%s] from PATH: [%s]', [Path, Paths]));
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
    if CurStep = ssPostInstall 
     then begin 
     EnvAddPath(ExpandConstant('{app}') +'\bin');
     EnvAddPath(ExpandConstant('{%USERPROFILE}') +'\globals\wapm_packages\.bin');
     end
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
    if CurUninstallStep = usPostUninstall
    then begin 
    EnvRemovePath(ExpandConstant('{app}') +'\bin');
    EnvAddPath(ExpandConstant('{%USERPROFILE}') +'\globals\wapm_packages\.bin');
    end
end;
