# Stop All NDNM Services
# Kills all processes started by start-all.ps1 and any lingering backend processes/ports

Write-Host "Stopping NDNM System..." -ForegroundColor Cyan
Write-Host ""

$ProjectRoot = $PSScriptRoot
$pidsFile = Join-Path $ProjectRoot ".ndnm-pids.txt"

function Kill-IfExists {
    param([int]$TargetPid)
    if ($TargetPid -le 0) { return }
    try { Stop-Process -Id $TargetPid -Force -ErrorAction SilentlyContinue } catch {}
}

function Kill-ByPort {
    param([int]$Port)
    try {
        $conn = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
        if ($conn) {
            $pid = $conn.OwningProcess
            Write-Host "  Liberando porta $Port (PID: $pid)..." -ForegroundColor Gray
            Kill-IfExists -TargetPid $pid
        }
    } catch {}
}

# If no PID file, still search and clean up
if (-not (Test-Path $pidsFile)) {
    Write-Host "No running services found (no PID file)" -ForegroundColor Yellow
    Write-Host ""; Write-Host "Searching for cargo/ndnm/node processes manually..." -ForegroundColor Yellow

    $cargoProcesses = Get-Process -Name "cargo" -ErrorAction SilentlyContinue
    $namedProcesses = Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match "ndnm|node-file-browser" }
    $allProcesses = @($cargoProcesses) + @($namedProcesses) | Select-Object -Unique

    if ($allProcesses.Count -gt 0) {
        Write-Host "Found $($allProcesses.Count) related processes" -ForegroundColor Yellow
        foreach ($proc in $allProcesses) {
            try {
                Write-Host "  Stopping $($proc.ProcessName) (PID: $($proc.Id))..." -ForegroundColor Gray
                Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
                Write-Host "    [OK] Stopped" -ForegroundColor Green
            } catch {
                Write-Host "    [SKIP] Could not stop (may already be closed)" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "No NDNM processes found running" -ForegroundColor Gray
    }

    # Free common ports
    Write-Host "Liberando portas comuns (3000,3001,3002,3003,9514)..." -ForegroundColor Yellow
    foreach ($p in 3000,3001,3002,3003,9514) { Kill-ByPort -Port $p }

    Write-Host ""; Write-Host "Done!" -ForegroundColor Green
    exit 0
}

# Read PIDs from file
$pids = Get-Content $pidsFile
Write-Host "Found $($pids.Count) processes to stop" -ForegroundColor Yellow
Write-Host ""

$stoppedCount = 0
$notFoundCount = 0

foreach ($processId in $pids) {
    if ([string]::IsNullOrWhiteSpace($processId)) { continue }
    $pidNum = [int]$processId

    $process = Get-Process -Id $pidNum -ErrorAction SilentlyContinue
    if ($process) {
        try {
            Write-Host "Stopping PID $pidNum ($($process.ProcessName))..." -ForegroundColor Gray
            Stop-Process -Id $pidNum -Force
            Write-Host "  [OK] Stopped" -ForegroundColor Green
            $stoppedCount++
        } catch {
            Write-Host "  [FAIL] Could not stop: $($_.Exception.Message)" -ForegroundColor Red
        }
    } else {
        Write-Host "PID $pidNum not found (may already be closed)" -ForegroundColor Yellow
        $notFoundCount++
    }
}

# Also try to kill any remaining cargo/ndnm/node processes
Write-Host ""; Write-Host "Cleaning up remaining processes..." -ForegroundColor Yellow
$remainingCargo = Get-Process -Name "cargo" -ErrorAction SilentlyContinue
$remainingNamed = Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match "ndnm|node-file-browser" }
$remaining = @($remainingCargo) + @($remainingNamed) | Select-Object -Unique
foreach ($proc in $remaining) { Kill-IfExists -TargetPid $proc.Id }

# Free common ports
Write-Host "Liberando portas comuns (3000,3001,3002,3003,9514)..." -ForegroundColor Yellow
foreach ($p in 3000,3001,3002,3003,9514) { Kill-ByPort -Port $p }

# Remove PID file
Remove-Item $pidsFile -ErrorAction SilentlyContinue

Write-Host ""; Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "Summary:" -ForegroundColor Yellow
Write-Host "  Stopped: $stoppedCount" -ForegroundColor Green
Write-Host "  Not found: $notFoundCount" -ForegroundColor Yellow
Write-Host ""; Write-Host "NDNM System Stopped" -ForegroundColor Green
Write-Host "=======================================" -ForegroundColor Cyan
