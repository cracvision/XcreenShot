# Starts a Universal capture, presses Enter once the selection is shown, prints the result.
param([string]$Extra = "")
Add-Type -Name K -Namespace W -MemberDefinition '[DllImport("user32.dll")] public static extern void keybd_event(byte v, byte s, uint f, UIntPtr e);'
$cli = "C:\Projects\XcreenShot\target\debug\xcreenshot.exe"
$log = "$env:TEMP\xs-dev.log"
$out = "$env:TEMP\xs-enter-universal.json"
$before = (Select-String -Path $log -Pattern "selection shown").Count
$p = Start-Process -FilePath $cli -ArgumentList "capture","universal","--copy","false","--open","none","--json" -NoNewWindow -PassThru -RedirectStandardOutput $out
$deadline = (Get-Date).AddSeconds(10)
while ((Select-String -Path $log -Pattern "selection shown").Count -le $before -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 50 }
Start-Sleep -Milliseconds 400
[W.K]::keybd_event(0x0D, 0, 0, [UIntPtr]::Zero); [W.K]::keybd_event(0x0D, 0, 2, [UIntPtr]::Zero)
if (-not $p.WaitForExit(5000)) {
  # Nothing happened with Enter: cancel so the session ends.
  [W.K]::keybd_event(0x1B, 0, 0, [UIntPtr]::Zero); [W.K]::keybd_event(0x1B, 0, 2, [UIntPtr]::Zero)
  $p.WaitForExit(5000) | Out-Null
}
$r = Get-Content $out -Raw | ConvertFrom-Json
if ($r.ok) { "status=$($r.result.status) size=$($r.result.width)x$($r.result.height)" } else { "error=$($r.error.code)" }
