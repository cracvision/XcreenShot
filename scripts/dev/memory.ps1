param([int]$appPid, [int]$samples = 1, [int]$every = 0)
# Memory and CPU of an XcreenShot process and everything it started (its WebView2
# processes), for APP-051. Private bytes is what the process keeps for itself; the
# working set is what is in RAM now. -samples and -every (seconds) repeat the reading.
function Tree([int]$root) {
  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name
  $ids = New-Object System.Collections.Generic.List[int]
  $ids.Add($root)
  $i = 0
  while ($i -lt $ids.Count) {
    foreach ($p in $all) { if ($p.ParentProcessId -eq $ids[$i] -and -not $ids.Contains([int]$p.ProcessId)) { $ids.Add([int]$p.ProcessId) } }
    $i++
  }
  return $ids
}
for ($n = 0; $n -lt $samples; $n++) {
  $ids = Tree $appPid
  $procs = $ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue }
  $private = ($procs | Measure-Object PrivateMemorySize64 -Sum).Sum / 1MB
  $ws = ($procs | Measure-Object WorkingSet64 -Sum).Sum / 1MB
  $cpu = ($procs | Measure-Object CPU -Sum).Sum
  "{0:HH:mm:ss}  processes={1}  private={2:N0} MB  workingSet={3:N0} MB  cpuSeconds={4:N2}" -f (Get-Date), $procs.Count, $private, $ws, $cpu
  if ($n -lt $samples - 1) { Start-Sleep -Seconds $every }
}
