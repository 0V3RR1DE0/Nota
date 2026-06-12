!include "MUI2.nsh"
!include "FileFunc.nsh"

Name "Nota"
OutFile "NotaSetup-0.1.0.exe"
InstallDir "$PROGRAMFILES64\Nota"
InstallDirRegKey HKLM "Software\Nota" ""
RequestExecutionLevel admin

!define MUI_ICON "assets\icon.ico"
!define MUI_UNICON "assets\icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "assets\installer_banner.bmp"  ; 164x314px

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Finnish"
!insertmacro MUI_LANGUAGE "English"

Section "Nota" SecMain
  SectionIn RO  ; Required section
  SetOutPath "$INSTDIR"
  File "target\release\Nota.exe"
  File "target\release\NotaUpdater.exe"

  ; Registry for uninstaller
  WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Nota" "DisplayName"     "Nota"
  WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Nota" "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Nota" "DisplayIcon"     "$INSTDIR\Nota.exe"
  WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Nota" "Publisher"       "Nota"
  WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Nota" "DisplayVersion"  "0.1.0"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Start Menu shortcut" SecStartMenu
  CreateDirectory "$SMPROGRAMS\Nota"
  CreateShortcut  "$SMPROGRAMS\Nota\Nota.lnk"      "$INSTDIR\Nota.exe"
  CreateShortcut  "$SMPROGRAMS\Nota\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Desktop shortcut" SecDesktop
  CreateShortcut "$DESKTOP\Nota.lnk" "$INSTDIR\Nota.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\Nota.exe"
  Delete "$INSTDIR\NotaUpdater.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir  "$INSTDIR"
  Delete "$SMPROGRAMS\Nota\Nota.lnk"
  Delete "$SMPROGRAMS\Nota\Uninstall.lnk"
  RMDir  "$SMPROGRAMS\Nota"
  Delete "$DESKTOP\Nota.lnk"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Nota"
SectionEnd