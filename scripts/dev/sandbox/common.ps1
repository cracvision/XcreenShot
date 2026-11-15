# Helpers for the tests that run inside Windows Sandbox (dot-sourced by them).
$ErrorActionPreference = 'Continue'
$out = 'C:\xs\out'
New-Item -ItemType Directory -Force $out | Out-Null
$xsDir = "$env:LOCALAPPDATA\XcreenShot"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing

function Log($m) { "$(Get-Date -Format HH:mm:ss.fff) $m" | Out-File -Append -Encoding utf8 "$out\log.txt" }

# The output of a command, into the log.
function LogRun([scriptblock] $b) { (& $b 2>&1 | Out-String).Trim() | Out-File -Append -Encoding utf8 "$out\log.txt" }

# BitBlt with CAPTUREBLT (Graphics.CopyFromScreen refuses that flag). XcreenShot's own
# windows are WebView2, composed by DirectComposition: a GDI copy does not see them
# (live drawing's shapes included). To check what is on screen, ask the app instead.
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Gdi {
  [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr dc);
  [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr d, int x, int y, int w, int h, IntPtr s, int sx, int sy, int op);
}
"@
function Shot($name) {
  try {
    $r = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object System.Drawing.Bitmap $r.Width, $r.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $dst = $g.GetHdc(); $src = [Gdi]::GetDC([IntPtr]::Zero)
    # SRCCOPY | CAPTUREBLT
    [void][Gdi]::BitBlt($dst, 0, 0, $r.Width, $r.Height, $src, $r.X, $r.Y, 0x40CC0020)
    [void][Gdi]::ReleaseDC([IntPtr]::Zero, $src); $g.ReleaseHdc($dst)
    $bmp.Save("$out\$name.png", [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
  } catch { Log "shot failed: $_" }
}

# XcreenShot's processes and installers, with their command lines.
function Procs {
  Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'xcreenshot*' -or $_.Name -like '*setup*' -or $_.Name -like '*installer*' } |
    ForEach-Object { "  pid=$($_.ProcessId) $($_.Name) :: $($_.CommandLine)" }
}

# Visible top-level windows of XcreenShot.
function Windows {
  Get-Process xcreenshot-app -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } |
    ForEach-Object { "  window: '$($_.MainWindowTitle)'" }
}

function Install($setup) {
  Log "installing $setup (silent)"
  $p = Start-Process $setup -ArgumentList '/S' -PassThru -Wait
  Log "install exit $($p.ExitCode)"
}

function Done {
  Log 'DONE'
  Start-Sleep 5
  shutdown.exe /s /t 0
}
