# Inside Windows Sandbox: the modes that hold the mouse and keyboard, with their real
# hooks and simulated input (VID-048 live drawing, CAP-067 multi-capture). Nothing here
# can reach the computer outside the Sandbox. Checks that a drag draws, that a click
# captures, and above all that `Esc` gives the input back.
. C:\xs\common.ps1
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Inp {
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte sc, uint f, UIntPtr e);
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int i);
}
"@
$W = [Inp]::GetSystemMetrics(0); $H = [Inp]::GetSystemMetrics(1)
function MoveTo($x, $y) { [Inp]::mouse_event(0x8001, [int]($x * 65535 / $W), [int]($y * 65535 / $H), 0, [UIntPtr]::Zero) }
function Down { [Inp]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero) }
function Up { [Inp]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero) }
function Key($vk) { [Inp]::keybd_event($vk, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 50; [Inp]::keybd_event($vk, 0, 2, [UIntPtr]::Zero) }
function Drag($x1, $y1, $x2, $y2) {
  MoveTo $x1 $y1; Start-Sleep -Milliseconds 100; Down
  for ($i = 1; $i -le 10; $i++) { MoveTo ($x1 + ($x2 - $x1) * $i / 10) ($y1 + ($y2 - $y1) * $i / 10); Start-Sleep -Milliseconds 30 }
  Up
}

$setup = Get-ChildItem C:\xs\*-setup.exe | Select-Object -First 1
Install $setup.FullName
$cli = "$xsDir\xcreenshot.exe"
Start-Process "$xsDir\xcreenshot-app.exe" -ArgumentList '--background'
Start-Sleep 15
LogRun { & $cli --no-launch app about --json }

# --- Live drawing ---------------------------------------------------------------
LogRun { & $cli --no-launch settings set video.liveDrawAnytime true --json }
MoveTo ($W / 2) ($H / 2)
Log 'live drawing on'
LogRun { & $cli --no-launch record draw --params '{\"on\":true,\"tool\":\"arrow\",\"fade\":0}' --json }
Start-Sleep 2
Drag ($W * 0.3) ($H * 0.3) ($W * 0.6) ($H * 0.5)
Start-Sleep 1
Shot 'livedraw-1-drawn'
Log 'shapes after a drag:'
LogRun { (& $cli --no-launch record draw --params '{\"query\":true}' --json | ConvertFrom-Json).result.shapes.Count }
Log 'Esc'
Key 0x1B
Start-Sleep 2
LogRun { & $cli --no-launch record draw --params '{\"query\":true}' --json }
Shot 'livedraw-2-after-esc'

# --- Multi-capture -------------------------------------------------------------
Start-Process notepad.exe
Start-Sleep 3
$np = Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
Log "notepad window: $($np.MainWindowTitle)"
Log 'multi-capture on'
LogRun { & $cli --no-launch capture multi-start --json }
Start-Sleep 3
# A click in the middle of the screen (notepad, or the desktop): one capture.
MoveTo ($W / 2) ($H / 2); Start-Sleep -Milliseconds 300; Down; Start-Sleep -Milliseconds 60; Up
Start-Sleep 3
Shot 'multi-1-clicked'
LogRun { & $cli --no-launch capture multi-state --json }
Log 'Esc'
Key 0x1B
Start-Sleep 2
LogRun { & $cli --no-launch capture multi-state --json }
Shot 'multi-2-after-esc'

# The input reaches the apps again: typing in notepad.
if ($np) {
  $ws = New-Object -ComObject WScript.Shell
  $null = $ws.AppActivate($np.Id)
  Start-Sleep 1
  Key 0x4F; Key 0x4B  # "ok"
  Start-Sleep 1
  Shot 'after-typing'
}
LogRun { & $cli --no-launch settings set video.liveDrawAnytime false --json }
Copy-Item "$xsDir\data\logs\*.log" $out -ErrorAction SilentlyContinue
Done
