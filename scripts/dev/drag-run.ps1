# Runs an interactive capture command, drags from (X1,Y1) to (X2,Y2) in physical
# pixels, then optionally presses keys (e.g. "I"), and prints the CLI result.
param([string]$Command = "region", [int]$X1 = 1000, [int]$Y1 = 800, [int]$X2 = 1301, [int]$Y2 = 1001,
      [string[]]$Keys = @(), [switch]$ClickOnly)
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class In {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte v, byte s, uint f, UIntPtr e);
}
'@
[In]::SetProcessDPIAware() | Out-Null
$cli = "C:\Projects\XcreenShot\target\debug\xcreenshot.exe"
$log = "$env:TEMP\xs-dev.log"
$out = "$env:TEMP\xs-cap\drag.json"
$before = (Select-String -Path $log -Pattern "selection shown").Count
$args = @("capture", $Command, "--copy", "false", "--out", "$env:TEMP\xs-cap\drag.png", "--json")
if ($Command -eq "window") { $args += @("--target", "interactive") }
$p = Start-Process -FilePath $cli -ArgumentList $args -NoNewWindow -PassThru -RedirectStandardOutput $out
$deadline = (Get-Date).AddSeconds(10)
while ((Select-String -Path $log -Pattern "selection shown").Count -le $before -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 50 }
Start-Sleep -Milliseconds 300
[In]::SetCursorPos($X1, $Y1) | Out-Null; Start-Sleep -Milliseconds 200
[In]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 80
if (-not $ClickOnly) {
  foreach ($i in 1..10) { [In]::SetCursorPos($X1 + [int](($X2-$X1)*$i/10), $Y1 + [int](($Y2-$Y1)*$i/10)) | Out-Null; Start-Sleep -Milliseconds 25 }
}
[In]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 400
foreach ($k in $Keys) { $vk = [byte][char]$k.ToUpper(); [In]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); [In]::keybd_event($vk, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200 }
$p.WaitForExit(8000) | Out-Null
if (-not $p.HasExited) { "still waiting (no result)"; [In]::keybd_event(0x1B,0,0,[UIntPtr]::Zero); [In]::keybd_event(0x1B,0,2,[UIntPtr]::Zero); Start-Sleep 1 }
$r = Get-Content $out -Raw | ConvertFrom-Json
if ($r.ok) { "ok " + $r.result.width + "x" + $r.result.height + " rect=" + ($r.result.rect | ConvertTo-Json -Compress) + " mode=" + $r.result.metadata.mode + " app=" + $r.result.metadata.app.name } else { "error " + $r.error.code }
