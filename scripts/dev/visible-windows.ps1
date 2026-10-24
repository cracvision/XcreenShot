# Lists visible top-level windows of xcreenshot-app with title, class and rect
# (selection windows are excluded from screenshots, so this is how to see them).
Add-Type @'
using System; using System.Text; using System.Collections.Generic; using System.Runtime.InteropServices;
public static class Wins {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
  public static List<string> List(uint[] pids) {
    var o = new List<string>();
    EnumWindows((h, l) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (Array.IndexOf(pids, pid) >= 0 && IsWindowVisible(h)) {
        var t = new StringBuilder(256); GetWindowText(h, t, 256);
        var c = new StringBuilder(256); GetClassName(h, c, 256);
        RECT r; GetWindowRect(h, out r);
        o.Add(String.Format("'{0}' class={1} rect=({2},{3}) {4}x{5}", t, c, r.L, r.T, r.R - r.L, r.B - r.T));
      }
      return true; }, IntPtr.Zero);
    return o;
  }
}
'@
[Wins]::SetProcessDPIAware() | Out-Null
$pids = [uint32[]]@(Get-Process xcreenshot-app -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
if ($pids.Count -eq 0) { "xcreenshot-app is not running"; return }
"process: " + ((Get-CimInstance Win32_Process -Filter "Name='xcreenshot-app.exe'" | Select-Object -First 1).CommandLine)
[Wins]::List($pids)
