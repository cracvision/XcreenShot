; Installer hooks of the Windows package (APP-028, APP-029). Tauri inserts these macros
; into its NSIS script; `tauri.bundle.json` points here.
!include LogicLib.nsh

; MCP clients such as Claude Desktop keep `xcreenshot.exe mcp` running, and Windows does
; not let a running program be overwritten: the install stopped with "Error opening file
; for writing". It does let it be renamed. So a copy in use moves aside, the client keeps
; its connection, and it gets the new version the next time it starts the server.
; Copying it back keeps the name in place if the install is cancelled before its files
; are written; nothing is moved when the file is free.
!macro NSIS_HOOK_PREINSTALL
  Push $0
  Push $1
  ; Copies set aside by earlier updates, once nothing runs them any more.
  Delete "$INSTDIR\xcreenshot.exe.*.old"
  ${If} ${FileExists} "$INSTDIR\xcreenshot.exe"
    ClearErrors
    FileOpen $0 "$INSTDIR\xcreenshot.exe" a
    ${If} ${Errors}
      StrCpy $1 1
      ${While} ${FileExists} "$INSTDIR\xcreenshot.exe.$1.old"
        IntOp $1 $1 + 1
      ${EndWhile}
      Rename "$INSTDIR\xcreenshot.exe" "$INSTDIR\xcreenshot.exe.$1.old"
      CopyFiles /SILENT "$INSTDIR\xcreenshot.exe.$1.old" "$INSTDIR\xcreenshot.exe"
    ${Else}
      FileClose $0
    ${EndIf}
  ${EndIf}
  ClearErrors
  Pop $1
  Pop $0
!macroend

; PDF in "Open with" (CAP-041): XcreenShot can open them, but never becomes their
; default app, so it only adds itself to the list of apps that open them.
!macro NSIS_HOOK_POSTINSTALL
  ; `xcreenshot` in new terminals (APP-037), unless /nopath. The CLI edits the PATH:
  ; this script's strings are cut at 1024 characters and would damage a long one.
  ClearErrors
  ${GetOptions} $CMDLINE "/nopath" $0
  ${If} ${Errors}
    nsExec::Exec '"$INSTDIR\xcreenshot.exe" setup --add-path "$INSTDIR"'
    Pop $0
  ${EndIf}
  ; /nolaunchatlogin: the app does not turn on starting with Windows (APP-037).
  ClearErrors
  ${GetOptions} $CMDLINE "/nolaunchatlogin" $0
  ${IfNot} ${Errors}
    FileOpen $0 "$INSTDIR\no-launch-at-login" w
    FileClose $0
  ${EndIf}
  ClearErrors
  WriteRegStr SHCTX "Software\Classes\XcreenShot.Pdf" "" "PDF (XcreenShot)"
  WriteRegStr SHCTX "Software\Classes\XcreenShot.Pdf\DefaultIcon" "" "$INSTDIR\xcreenshot-app.exe,0"
  WriteRegStr SHCTX "Software\Classes\XcreenShot.Pdf\shell\open\command" "" '"$INSTDIR\xcreenshot-app.exe" "%1"'
  WriteRegStr SHCTX "Software\Classes\.pdf\OpenWithProgids" "XcreenShot.Pdf" ""
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Out of the PATH (APP-042), not when this is an update.
  ${If} $UpdateMode <> 1
    nsExec::Exec '"$INSTDIR\xcreenshot.exe" setup --remove-path "$INSTDIR"'
    Pop $0
  ${EndIf}
  Delete "$INSTDIR\no-launch-at-login"
  Delete "$INSTDIR\xcreenshot.exe.*.old"
  DeleteRegValue SHCTX "Software\Classes\.pdf\OpenWithProgids" "XcreenShot.Pdf"
  DeleteRegKey SHCTX "Software\Classes\XcreenShot.Pdf"
!macroend

; Deleting the settings and data when the person ticks the box (APP-042): XcreenShot's
; own folders, never the library. If a library was put inside them (a library.db is
; there), they stay as they are.
Var XsLibraryInside
Function un.XsFoundLibrary
  StrCpy $XsLibraryInside 1
  Push "StopLocate"
FunctionEnd

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    StrCpy $XsLibraryInside 0
    ${Locate} "$APPDATA\XcreenShot" "/L=F /M=library.db" "un.XsFoundLibrary"
    ${Locate} "$LOCALAPPDATA\XcreenShot" "/L=F /M=library.db" "un.XsFoundLibrary"
    ${If} $XsLibraryInside = 0
      RMDir /r "$APPDATA\XcreenShot"
      RMDir /r "$LOCALAPPDATA\XcreenShot"
    ${EndIf}
  ${EndIf}
!macroend
