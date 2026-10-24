# Window, client and DWM frame rectangles (physical pixels), styles and extended
# styles of the visible xcreenshot-app windows.
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class WR {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int s);
  [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr h, int i);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
"@
[WR]::SetProcessDPIAware() | Out-Null
$procs = Get-Process xcreenshot-app -ErrorAction SilentlyContinue | ForEach-Object { $_.Id }
[WR]::EnumWindows({ param($h, $l)
  $pid2 = 0; [void][WR]::GetWindowThreadProcessId($h, [ref]$pid2)
  if ($procs -contains $pid2 -and [WR]::IsWindowVisible($h)) {
    $sb = New-Object System.Text.StringBuilder 256; [void][WR]::GetWindowText($h, $sb, 256)
    $w = New-Object WR+RECT; $c = New-Object WR+RECT; $d = New-Object WR+RECT
    [void][WR]::GetWindowRect($h, [ref]$w); [void][WR]::GetClientRect($h, [ref]$c)
    [void][WR]::DwmGetWindowAttribute($h, 9, [ref]$d, 16)
    $style = '{0:X8}' -f [WR]::GetWindowLong($h, -16); $ex = '{0:X8}' -f [WR]::GetWindowLong($h, -20)
    Write-Host "'$($sb)' window=$($w.L),$($w.T) $($w.R-$w.L)x$($w.B-$w.T) client=$($c.R)x$($c.B) frame=$($d.L),$($d.T) $($d.R-$d.L)x$($d.B-$d.T) style=$style ex=$ex"
  }
  return $true }, [IntPtr]::Zero) | Out-Null
