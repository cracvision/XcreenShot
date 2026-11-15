# Runs a test inside Windows Sandbox: a clean, throwaway Windows (Windows 11 Pro, feature
# "Windows Sandbox"). Nothing touches this computer's XcreenShot, and a test that takes
# the mouse or keyboard only takes them inside the Sandbox window.
#
#   powershell -File scripts\dev\sandbox.ps1 -Inside scripts\dev\sandbox\update.ps1 `
#     -Files target\release\xcreenshot.exe, <installer>.exe
#
# The files land in C:\xs inside, with common.ps1 next to the test; the test writes
# C:\xs\out (log.txt, screenshots), ends its log with DONE and shuts the Sandbox down.
# Only one Sandbox runs at a time: never kill its processes while it starts.
param(
  [Parameter(Mandatory)] [string] $Inside,
  [string[]] $Files = @(),
  [int] $TimeoutSec = 600
)
$ErrorActionPreference = 'Stop'
if (Get-Process WindowsSandbox, WindowsSandboxServer -ErrorAction SilentlyContinue) {
  throw 'A Sandbox is already running: wait for it to end.'
}
# A Sandbox that shut down may leave its remote session behind for a while; alone,
# without the Sandbox, it is a leftover.
$t = 0
while ((Get-Process WindowsSandboxRemoteSession -ErrorAction SilentlyContinue) -and $t -lt 60) { Start-Sleep 2; $t += 2 }
Get-Process WindowsSandboxRemoteSession -ErrorAction SilentlyContinue | Stop-Process -Force
$dir = Join-Path $env:TEMP ("xs-sandbox-" + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force $dir | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'sandbox\common.ps1') $dir
Copy-Item $Inside (Join-Path $dir 'test.ps1')
foreach ($f in $Files) { Copy-Item $f $dir }
@"
<Configuration>
  <MappedFolders>
    <MappedFolder>
      <HostFolder>$dir</HostFolder>
      <SandboxFolder>C:\xs</SandboxFolder>
      <ReadOnly>false</ReadOnly>
    </MappedFolder>
  </MappedFolders>
  <ClipboardRedirection>Disable</ClipboardRedirection>
  <LogonCommand>
    <Command>powershell.exe -ExecutionPolicy Bypass -WindowStyle Minimized -File C:\xs\test.ps1</Command>
  </LogonCommand>
</Configuration>
"@ | Set-Content (Join-Path $dir 'test.wsb') -Encoding utf8
Start-Process (Join-Path $dir 'test.wsb')
$log = Join-Path $dir 'out\log.txt'
$t = 0
while ($t -lt $TimeoutSec) {
  if ((Test-Path $log) -and (Select-String -Path $log -Pattern 'DONE' -Quiet)) { break }
  Start-Sleep 5; $t += 5
}
"Folder: $dir (after $t s)"
if (Test-Path $log) { Get-Content $log } else { 'No log: the test did not run.' }
