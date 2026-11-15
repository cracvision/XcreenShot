# Inside Windows Sandbox: installs the older XcreenShot (the first *-setup.exe in C:\xs),
# then updates it from the published manifest with "Install and restart", as from the
# window, and records how the installer ran and whether the app came back (APP-036).
# Needs C:\xs\xcreenshot.exe: a CLI that runs on a clean Windows (0.2.3 or later).
. C:\xs\common.ps1
$setup = Get-ChildItem C:\xs\*-setup.exe | Select-Object -First 1
Install $setup.FullName
$cli = 'C:\xs\xcreenshot.exe'
Start-Process "$xsDir\xcreenshot-app.exe"
Start-Sleep 20
LogRun { & $cli --no-launch app about --json }
LogRun { & $cli --no-launch app check-update --json }
Log 'install-update now'
Start-Process $cli -ArgumentList '--no-launch', 'app', 'install-update', '--json'
$seen = @{}
for ($i = 0; $i -lt 60; $i++) {
  Start-Sleep 1
  foreach ($line in (Procs)) { if (-not $seen.ContainsKey($line)) { $seen[$line] = 1; Log "seen $line" } }
  if ($i % 5 -eq 0) { Shot ('t{0:D3}' -f $i) }
}
Log 'after 60 s:'; Procs | Out-File -Append -Encoding utf8 "$out\log.txt"; Windows | Out-File -Append -Encoding utf8 "$out\log.txt"
LogRun { & $cli --no-launch app about --json }
LogRun { & "$xsDir\xcreenshot.exe" --version }
Done
