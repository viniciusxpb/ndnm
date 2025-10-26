# Stop All NDNM Services
# Kills all processes started by start-all.ps1

Write-Host "Stopping NDNM System..." -ForegroundColor Cyan
Write-Host ""

$ProjectRoot = $PSScriptRoot
$pidsFile = Join-Path $ProjectRoot ".ndnm-pids.txt"

# Check if PID file exists
if (-not (Test-Path $pidsFile)) {
    Write-Host "No running services found (no PID file)" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Searching for cargo processes manually..." -ForegroundColor Yellow

    # Try to find cargo processes
    $cargoProcesses = Get-Process -Name "cargo" -ErrorAction SilentlyContinue
    $rustProcesses = Get-Process | Where-Object { $_.ProcessName -match "ndnm" }

    $allProcesses = @($cargoProcesses) + @($rustProcesses) | Select-Object -Unique

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

    Write-Host ""
    Write-Host "Done!" -ForegroundColor Green
    exit 0
}

# Read PIDs from file
$pids = Get-Content $pidsFile

Write-Host "Found $($pids.Count) processes to stop" -ForegroundColor Yellow
Write-Host ""

$stoppedCount = 0
$notFoundCount = 0

foreach ($processId in $pids) {
    if ([string]::IsNullOrWhiteSpace($processId)) {
        continue
    }

    $pidNum = [int]$processId

    # Check if process exists
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

# Also try to kill any remaining cargo/ndnm processes
Write-Host ""
Write-Host "Cleaning up remaining processes..." -ForegroundColor Yellow

$remainingCargo = Get-Process -Name "cargo" -ErrorAction SilentlyContinue
$remainingNdnm = Get-Process | Where-Object { $_.ProcessName -match "ndnm" }

$remaining = @($remainingCargo) + @($remainingNdnm) | Select-Object -Unique

foreach ($proc in $remaining) {
    try {
        Write-Host "  Stopping $($proc.ProcessName) (PID: $($proc.Id))..." -ForegroundColor Gray
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    } catch {
        # Ignore errors
    }
}

# Remove PID file
Remove-Item $pidsFile -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "Summary:" -ForegroundColor Yellow
Write-Host "  Stopped: $stoppedCount" -ForegroundColor Green
Write-Host "  Not found: $notFoundCount" -ForegroundColor Yellow
Write-Host ""
Write-Host "NDNM System Stopped" -ForegroundColor Green
Write-Host "=======================================" -ForegroundColor Cyan
