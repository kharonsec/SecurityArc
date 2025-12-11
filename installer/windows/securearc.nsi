; SecureArc NSIS Installer Script

; Define application constants
!define APPNAME "SecureArc"
!define COMPANYNAME "SecureArc Team"
!define DESCRIPTION "Self-Destructing Encrypted Archive Tool"
!define VERSIONMAJOR 0
!define VERSIONMINOR 1
!define VERSIONBUILD 0
!define HELPURL "http://localhost:3000" ; Placeholder
!define UPDATEURL "http://localhost:3000" ; Placeholder
!define ABOUTURL "http://localhost:3000" ; Placeholder
!define INSTALLSIZE 15000 ; Estimated size in KB

RequestExecutionLevel admin ;Require admin rights on NT6+ (When UAC is turned on)

InstallDir "$PROGRAMFILES\${APPNAME}"

; Registry key to check for directory (so if you install again, it will overwrite the old one automatically)
InstallDirRegKey HKLM "Software\${APPNAME}" "Install_Dir"

; Main Install settings
Name "${APPNAME}"
OutFile "SecureArc_Installer.exe"
BrandingText "${APPNAME} Installer"
ShowInstDetails show
AutoCloseWindow false

; Components page - allowing user to choose what to install
; Page components
Page directory
Page instfiles

Section "SecureArc GUI (Main App)" SecGUI
    SectionIn RO ; Read only, always installed
    SetOutPath $INSTDIR
    
    ; GUI Binary
    ; NOTE: This path assumes you run makensis from the project root and have built release binaries
    File "..\..\target\release\securearc-gui.exe"
    ; Add any other GUI resources if needed (e.g. icons, webview2loader.dll)
    ; File "..\..\target\release\WebView2Loader.dll" ; Uncomment if dynamic linking

    ; Create Uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Start Menu Shortcuts
    CreateDirectory "$SMPROGRAMS\${APPNAME}"
    CreateShortCut "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\securearc-gui.exe"
    CreateShortCut "$SMPROGRAMS\${APPNAME}\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "SecureArc CLI (Command Line)" SecCLI
    SetOutPath $INSTDIR
    
    ; CLI Binary
    File "..\..\target\release\securearc-cli.exe"

    ; Add to PATH (Basic implementation, might need a more robust macro/plugin for PATH modification)
    ; For now, just putting it in the same dir. User can add to PATH manually or we use EnVar plugin.
SectionEnd

Section "Desktop Shortcut" SecDesktop
    CreateShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\securearc-gui.exe"
SectionEnd

; Uninstaller Section
Section "Uninstall"
    ; Remove Start Menu Shortcuts
    RMDir /r "$SMPROGRAMS\${APPNAME}"
    
    ; Remove Desktop Shortcut
    Delete "$DESKTOP\${APPNAME}.lnk"

    ; Remove Files
    Delete "$INSTDIR\securearc-gui.exe"
    Delete "$INSTDIR\securearc-cli.exe"
    Delete "$INSTDIR\uninstall.exe"
    ; Delete "$INSTDIR\WebView2Loader.dll" 

    ; Remove Directory
    RMDir "$INSTDIR"

    ; Remove Registry Keys
    DeleteRegKey HKCR ".sarc"
    DeleteRegKey HKCR "SecureArc.Archive"
    DeleteRegKey HKCR "Directory\shell\SecureArc"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
    DeleteRegKey HKLM "Software\${APPNAME}"
SectionEnd

; Post-install section to write registry keys for Add/Remove programs
Section -Post
    WriteRegStr HKLM "Software\${APPNAME}" "Install_Dir" "$INSTDIR"

    ; File Association for .sarc
    WriteRegStr HKCR ".sarc" "" "SecureArc.Archive"
    WriteRegStr HKCR "SecureArc.Archive" "" "SecureArc Encrypted Archive"
    WriteRegStr HKCR "SecureArc.Archive\DefaultIcon" "" "$INSTDIR\securearc-gui.exe,0"
    WriteRegStr HKCR "SecureArc.Archive\shell\open\command" "" '"$INSTDIR\securearc-gui.exe" "%1"'

    ; Context Menu "Add to SecureArc Archive" (for Folders)
    WriteRegStr HKCR "Directory\shell\SecureArc" "" "Add to SecureArc Archive"
    WriteRegStr HKCR "Directory\shell\SecureArc\command" "" '"$INSTDIR\securearc-gui.exe" --create "%1"'
    
    ; Write the uninstall keys for Windows
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$INSTDIR\securearc-gui.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "${COMPANYNAME}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoRepair" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "EstimatedSize" ${INSTALLSIZE}
SectionEnd
