# Helpers for the tests that run inside Windows Sandbox (dot-sourced by them).
$ErrorActionPreference = 'Continue'
$out = 'C:\xs\out'
New-Item -ItemType Directory -Force $out | Out-Null
$xsDir = "$env:LOCALAPPDATA\XcreenShot"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing

function Log($m) { "$(Get-Date -Format HH:mm:ss.fff) $m" | Out-File -Append -Encoding utf8 "$out\log.txt" }

# The output of a command, into the log.
function LogRun([scriptblock] $b) { (& $b 2>&1 | Out-String).Trim() | Out-File -Append -Encoding utf8 "$out\log.txt" }

function Shot($name) {
  try {
    $r = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object System.Drawing.Bitmap $r.Width, $r.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    # CaptureBlt: with the layered (transparent) windows, such as live drawing's overlay.
    $g.CopyFromScreen($r.Location, [System.Drawing.Point]::Empty, $r.Size, [System.Drawing.CopyPixelOperation]'SourceCopy, CaptureBlt')
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
