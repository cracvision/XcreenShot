# Inside Windows Sandbox: XcreenShot's clipboard (TRY-018). Two captures; `tray insert`
# without paste leaves the first one's original image on the clipboard; with paste, both
# land in the window that was in front when the panel opened (a small rich-text window
# made here, which accepts pasted images). Never on Carlos's computer: it writes the
# clipboard and presses Ctrl+V.
. C:\xs\common.ps1
$setup = Get-ChildItem C:\xs\*-setup.exe | Select-Object -First 1
Install $setup.FullName
$cli = "$xsDir\xcreenshot.exe"
Start-Process "$xsDir\xcreenshot-app.exe"
Start-Sleep 20
LogRun { & $cli --no-launch app about --json }

Log 'two captures, put in the tray (a capture from the CLI does not open the editor)'
foreach ($n in 1, 2) {
  $c = (& $cli --no-launch capture fullscreen --json | ConvertFrom-Json).result
  Log "captured $($c.documentId) $($c.width)x$($c.height)"
  LogRun { & $cli --no-launch tray open --path $c.path --activate false --json }
  Start-Sleep 2
}
$tray = (& $cli --no-launch tray list --json | ConvertFrom-Json).result.items
Log "tray: $($tray.Count) items: $(($tray | ForEach-Object { $_.documentId + '/' + $_.format }) -join ', ')"
$ids = @($tray | Select-Object -First 2 | ForEach-Object { $_.documentId })

Log 'copy only (no paste), after emptying the clipboard'
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.Clipboard]::Clear()
LogRun { & $cli --no-launch tray insert $ids[0] --json }
Add-Type -AssemblyName System.Windows.Forms
$img = [System.Windows.Forms.Clipboard]::GetImage()
Log "clipboard image: $(if ($img) { "$($img.Width)x$($img.Height)" } else { 'none' })"

# The paste target: a rich-text window that records how many pictures it got.
$target = @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form
$f.Text = 'Paste target'; $f.Width = 900; $f.Height = 700; $f.TopMost = $true
$r = New-Object System.Windows.Forms.RichTextBox; $r.Dock = 'Fill'; $f.Controls.Add($r)
$t = New-Object System.Windows.Forms.Timer; $t.Interval = 25000
$t.Add_Tick({ $n = ([regex]::Matches($r.Rtf, '\\pict')).Count; "pictures=$n" | Out-File C:\xs\out\target.txt; $f.Close() })
$f.Add_Shown({ $f.Activate(); $r.Focus(); $t.Start() })
[void]$f.ShowDialog()
'@
Set-Content C:\xs\target.ps1 $target -Encoding utf8
Start-Process powershell.exe -ArgumentList '-ExecutionPolicy', 'Bypass', '-STA', '-File', 'C:\xs\target.ps1'
Start-Sleep 4
# Notepad in front: the window you were in is not the one we paste into ("Insert into").
Start-Process notepad.exe
Start-Sleep 3
Log 'panel open'
LogRun { & $cli --no-launch tray clipboard --open true --json }
Start-Sleep 2
Shot 'panel'
$windows = (& $cli --no-launch tray targets --json | ConvertFrom-Json).result.windows
Log "targets: $(($windows | ForEach-Object { $_.app + ' - ' + $_.title + $(if ($_.previous) { ' (previous)' } else { '' }) }) -join ' | ')"
$target = ($windows | Where-Object { $_.title -eq 'Paste target' } | Select-Object -First 1).id
Log "insert both into the paste target ($target), not the previous window"
$p = (@{ documentIds = $ids; paste = $true; target = $target } | ConvertTo-Json -Compress).Replace('"', '\"')
LogRun { & $cli --no-launch tray insert --params $p --json }
Start-Sleep 25
Log "target: $(Get-Content C:\xs\out\target.txt -ErrorAction SilentlyContinue)"
Copy-Item "$xsDir\data\logs\*.log" $out -ErrorAction SilentlyContinue
Done
