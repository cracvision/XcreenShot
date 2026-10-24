# Starts N interactive region captures and cancels each with Esc once the
# selection is shown; prints each CLI result and exit code.
param([int]$Runs = 3)
Add-Type -Name K -Namespace W -MemberDefinition '[DllImport("user32.dll")] public static extern void keybd_event(byte v, byte s, uint f, UIntPtr e);'
$cli = "C:\Projects\XcreenShot\target\debug\xcreenshot.exe"
$log = "$env:TEMP\xs-dev.log"
for ($i = 1; $i -le $Runs; $i++) {
  $before = (Select-String -Path $log -Pattern "selection shown").Count
  $p = Start-Process -FilePath $cli -ArgumentList "capture","region","--copy","false","--json" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\xs-cap\esc$i.json"
  $deadline = (Get-Date).AddSeconds(10)
  while ((Select-String -Path $log -Pattern "selection shown").Count -le $before -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 50 }
  Start-Sleep -Milliseconds 300
  [W.K]::keybd_event(0x1B, 0, 0, [UIntPtr]::Zero); [W.K]::keybd_event(0x1B, 0, 2, [UIntPtr]::Zero)
  $p.WaitForExit(10000) | Out-Null
  "run $i exit=$($p.ExitCode) $(Get-Content "$env:TEMP\xs-cap\esc$i.json")"
  Start-Sleep -Milliseconds 500
}
