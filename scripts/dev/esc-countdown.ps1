# Starts a delayed full-screen capture, presses Esc during the countdown and prints the
# CLI result, the exit code and how long it took.
Add-Type -Name K -Namespace W -MemberDefinition '[DllImport("user32.dll")] public static extern void keybd_event(byte v, byte s, uint f, UIntPtr e);'
$cli = "C:\Projects\XcreenShot\target\debug\xcreenshot.exe"
$out = "$env:TEMP\xs-esc-countdown.json"
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $cli -ArgumentList "capture","fullscreen","--delay","5","--copy","false","--open","none","--json" -NoNewWindow -PassThru -RedirectStandardOutput $out
Start-Sleep -Milliseconds 1500
[W.K]::keybd_event(0x1B, 0, 0, [UIntPtr]::Zero); [W.K]::keybd_event(0x1B, 0, 2, [UIntPtr]::Zero)
$p.WaitForExit(15000) | Out-Null
"exit=$($p.ExitCode) after=$($sw.ElapsedMilliseconds)ms $(Get-Content $out)"
