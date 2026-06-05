<#
.SYNOPSIS
    Run the ragas CLI over every dataset in testdata/scenarios and summarize the metric means.

.DESCRIPTION
    Builds the `ragas` binary once, then runs `ragas evaluate` over each *.jsonl scenario,
    collecting the aggregate means (rouge_l always; faithfulness / context_recall when an API
    key is configured). Prints a per-file table plus a "discrimination" table that compares the
    good/bad pairs so you can see the intended axis separate.

.PARAMETER Offline
    Temporarily move `.env` aside so NO API key resolves -> only the offline rouge_l metric runs
    (zero API calls / cost). Useful to verify every dataset parses and to inspect ROUGE behaviour.

.EXAMPLE
    .\scripts\run-scenarios.ps1 -Offline      # parse-check + rouge_l only, no network
    .\scripts\run-scenarios.ps1               # full run (uses .env / env vars for the LLM metrics)
#>
[CmdletBinding()]
param(
    [switch]$Offline,
    [string]$ScenarioDir,
    [string]$ReportDir
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (-not $ScenarioDir) { $ScenarioDir = Join-Path $root 'testdata\scenarios' }
if (-not $ReportDir)   { $ReportDir   = Join-Path $ScenarioDir 'reports' }
New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null

Write-Host 'Building ragas binary...' -ForegroundColor Cyan
& cargo build --quiet --bin ragas
$exe = Join-Path $root 'target\debug\ragas.exe'
if (-not (Test-Path $exe)) { throw "binary not found: $exe" }

$files = Get-ChildItem -LiteralPath $ScenarioDir -Filter '*.jsonl' | Sort-Object Name

$envPath = Join-Path $root '.env'
$moved = $false
$results = @()
try {
    if ($Offline -and (Test-Path $envPath)) {
        Rename-Item -LiteralPath $envPath "$envPath.off"
        $moved = $true
        Write-Host 'Offline mode: .env moved aside -> rouge_l only, no API calls.' -ForegroundColor Yellow
    }

    foreach ($f in $files) {
        $report = Join-Path $ReportDir ($f.BaseName + '.json')
        $out = & $exe evaluate --dataset $f.FullName --report $report
        $code = $LASTEXITCODE

        $row = [ordered]@{ file = $f.Name; rouge_l = '-'; faithfulness = '-'; context_recall = '-'; errors = 0; status = 'ok' }
        if ($code -ne 0) {
            $row.status = "EXIT $code"
        }
        else {
            try {
                $j = $out | ConvertFrom-Json
                foreach ($m in $j.metrics) {
                    if ($row.Contains($m.metric)) {
                        $row[$m.metric] = if ([int]$m.count -gt 0) { '{0:N3}' -f [double]$m.mean } else { 'err' }
                    }
                    $row.errors += [int]$m.errors
                }
            }
            catch {
                $row.status = 'PARSE-FAIL'
            }
        }
        $results += [pscustomobject]$row
    }
}
finally {
    if ($moved) {
        Rename-Item -LiteralPath "$envPath.off" $envPath
        Write-Host '.env restored.' -ForegroundColor Yellow
    }
}

Write-Host ''
Write-Host '=== Per-file metric means ===' -ForegroundColor Green
$results | Format-Table -AutoSize | Out-String | Write-Host

# ---- discrimination pairs: good vs bad on the isolated axis ----
function MetricOf([string]$file, [string]$metric) {
    $r = $results | Where-Object { $_.file -eq $file }
    if ($r) { return $r.$metric } else { return 'n/a' }
}

$pairs = @(
    @{ axis = 'faithfulness';   good = '01-it-helpdesk-grounded.jsonl'; bad = '02-it-helpdesk-hallucinated.jsonl' },
    @{ axis = 'faithfulness';   good = '03-medical-grounded.jsonl';     bad = '04-medical-overclaim.jsonl' },
    @{ axis = 'faithfulness';   good = '05-chinese-kb-grounded.jsonl';  bad = '06-chinese-kb-hallucinated.jsonl' },
    @{ axis = 'context_recall'; good = '07-ecommerce-recall-hit.jsonl'; bad = '08-ecommerce-recall-miss.jsonl' },
    @{ axis = 'context_recall'; good = '09-hr-policy-recall-hit.jsonl'; bad = '10-hr-policy-recall-miss.jsonl' },
    @{ axis = 'rouge_l';        good = '11-devdocs-verbatim.jsonl';     bad = '12-devdocs-paraphrase.jsonl' }
)

$disc = foreach ($p in $pairs) {
    [pscustomobject]@{
        axis      = $p.axis
        good_file = $p.good
        good_mean = MetricOf $p.good $p.axis
        bad_file  = $p.bad
        bad_mean  = MetricOf $p.bad $p.axis
    }
}

Write-Host '=== Discrimination (good should beat bad on the listed axis) ===' -ForegroundColor Green
$disc | Format-Table -AutoSize | Out-String | Write-Host

if ($Offline) {
    Write-Host 'Note: offline run computes rouge_l only. faithfulness / context_recall show "-".' -ForegroundColor DarkGray
    Write-Host 'For the verbatim-vs-paraphrase pair, rouge_l should be clearly higher for verbatim.' -ForegroundColor DarkGray
}
Write-Host "Full per-sample reports written to: $ReportDir" -ForegroundColor DarkGray
