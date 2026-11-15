# Inside Windows Sandbox: a fresh install of the XcreenShot in C:\xs (*-setup.exe).
# - The installed CLI runs on a clean Windows (no Visual C++ Redistributable, APP-028).
# - A start after an update (the remembered version is older, installed from the window)
#   shows the window even started in the background, with the notice (APP-036).
. C:\xs\common.ps1
$setup = Get-ChildItem C:\xs\*-setup.exe | Select-Object -First 1
Install $setup.FullName
$cli = "$xsDir\xcreenshot.exe"
LogRun { & $cli --version }
Log "cli exit $LASTEXITCODE"
Start-Process "$xsDir\xcreenshot-app.exe"
Start-Sleep 20
LogRun { & $cli --no-launch app about --json }
Shot '01-first-start'
LogRun { & $cli --no-launch app quit --json }
while (Get-Process xcreenshot-app -ErrorAction SilentlyContinue) { Start-Sleep 1 }

# As if 0.0.1 had been updated from its window.
$state = "$env:APPDATA\XcreenShot\config\state.json"
$j = Get-Content $state -Raw | ConvertFrom-Json
Log "remembered version: $($j.memory.'app.lastVersion')"
$j.memory | Add-Member -Force -NotePropertyName 'app.lastVersion' -NotePropertyValue '0.0.1'
$j.memory | Add-Member -Force -NotePropertyName 'updates.reopen' -NotePropertyValue $true
# Without a BOM, as the app writes it.
[System.IO.File]::WriteAllText($state, ($j | ConvertTo-Json -Depth 20), (New-Object System.Text.UTF8Encoding $false))
LogRun { Get-Content $state -Raw }
Start-Process "$xsDir\xcreenshot-app.exe" -ArgumentList '--background'
Start-Sleep 15
Log 'after a start in the background, following an update from the window:'
Windows | Out-File -Append -Encoding utf8 "$out\log.txt"
Shot '02-after-update'
$j = Get-Content $state -Raw | ConvertFrom-Json
Log "remembered version: $($j.memory.'app.lastVersion'); reopen: $($j.memory.'updates.reopen')"
Copy-Item "$xsDir\data\logs\*.log" $out -ErrorAction SilentlyContinue
Done
