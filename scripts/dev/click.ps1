# Left click at (x, y) in physical desktop pixels (simulated input).
param([int]$x, [int]$y)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class M {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
}
"@
[M]::SetProcessDPIAware() | Out-Null
[M]::SetCursorPos($x, $y) | Out-Null
Start-Sleep -Milliseconds 80
[M]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 40
[M]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
