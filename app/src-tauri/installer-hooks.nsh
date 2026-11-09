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

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\xcreenshot.exe.*.old"
!macroend
