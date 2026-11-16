# The documentation package for Jarvis, Carlos's agent that runs the OQ on CRAC_BEAST:
# the briefing, the OQ, the manual guides, the spec, AGENTS.md and the project status,
# in one .zip. Jarvis can also read them in place (C:\Projects\XcreenShot).
#
#   powershell -File scripts\make-jarvis-kit.ps1
#
# Leaves target\jarvis\XcreenShot_Jarvis_Kit.zip. Not included: spec\research (process
# evidence), the business notes and anything with Carlos's captures.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$out = Join-Path $root 'target\jarvis'
$stage = Join-Path $out 'XcreenShot_Jarvis_Kit'
Remove-Item $out -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $stage | Out-Null

function Put($from, $to) {
  $dest = Join-Path $stage $to
  New-Item -ItemType Directory -Force (Split-Path $dest -Parent) | Out-Null
  Copy-Item (Join-Path $root $from) $dest -Recurse -Force
}

Put 'docs\jarvis\LEEME-JARVIS.md' '00-LEEME-JARVIS.md'
Put 'tests\manual\XcreenShot_OQ_Windows_v1_4.xlsx' '01-OQ\XcreenShot_OQ_Windows_v1_4.xlsx'
Put 'tests\manual\E2E-windows.md' '01-OQ\E2E-windows.md'
Put 'AGENTS.md' '02-AGENTS.md'
Put 'tests\manual' '03-guias'
Remove-Item (Join-Path $stage '03-guias\XcreenShot_OQ_Windows_v1_4.xlsx') -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force (Join-Path $stage '04-spec') | Out-Null
Get-ChildItem (Join-Path $root 'spec') -File -Filter *.md | Where-Object { $_.Name -ne 'IDEAS.md' } |
  ForEach-Object { Copy-Item $_.FullName (Join-Path $stage "04-spec\$($_.Name)") }
Put 'docs\ESTADO.md' '05-estado\ESTADO.md'
Put 'docs\adr' '05-estado\adr'
Put 'docs\formats' '05-estado\formats'

$zip = Join-Path $out 'XcreenShot_Jarvis_Kit.zip'
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
$n = (Get-ChildItem $stage -Recurse -File).Count
"Package: $zip ($n files, $([math]::Round((Get-Item $zip).Length / 1KB)) KB)"
