# Start All NDNM Services
# Opens each service in a separate PowerShell window

Write-Host "Starting NDNM System..." -ForegroundColor Cyan
Write-Host ""

# Get the project root directory
$ProjectRoot = $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "ndnm-backend"

# Check if cargo is available
try {
    $null = Get-Command cargo -ErrorAction Stop
} catch {
    Write-Host "Error: Cargo not found. Make sure Rust is installed." -ForegroundColor Red
    exit 1
}

# Function to start a service in a new window
function Start-Service {
    param(
        [string]$Name,
        [string]$WorkingDir,
        [string]$Command
    )

    Write-Host "Starting $Name..." -ForegroundColor Yellow

    $process = Start-Process powershell -ArgumentList @(
        "-NoExit",
        "-Command",
        "cd '$WorkingDir'; Write-Host '=== $Name ===' -ForegroundColor Cyan; Write-Host ''; $Command"
    ) -PassThru

    if ($process) {
        Write-Host "  [OK] $Name started (PID: $($process.Id))" -ForegroundColor Green
        return $process.Id
    } else {
        Write-Host "  [FAIL] Failed to start $Name" -ForegroundColor Red
        return $null
    }
}

# Array to store PIDs
$pids = @()

# Start Hermes first
Write-Host ""
Write-Host "Step 1: Starting Hermes Orchestrator" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
$hermesPid = Start-Service -Name "Hermes" -WorkingDir $BackendDir -Command "cargo run -p ndnm-hermes"
if ($hermesPid) {
    $pids += $hermesPid
}
Start-Sleep -Seconds 2

# Start Brazil BFF
Write-Host ""
Write-Host "Step 2: Starting Brazil BFF" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan
$brazilPid = Start-Service -Name "Brazil" -WorkingDir $BackendDir -Command "cargo run -p ndnm-brazil"
if ($brazilPid) {
    $pids += $brazilPid
}
Start-Sleep -Seconds 2

# Discover nodes
Write-Host ""
Write-Host "Step 3: Discovering and Starting Nodes" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan

$nodesDir = Join-Path $BackendDir "nodes"
if (Test-Path $nodesDir) {
    $nodeDirectories = Get-ChildItem -Path $nodesDir -Directory

    foreach ($nodeDir in $nodeDirectories) {
        $configPath = Join-Path $nodeDir.FullName "config.yaml"

        if (Test-Path $configPath) {
            $nodeName = $nodeDir.Name
            $nodePid = Start-Service -Name $nodeName -WorkingDir $nodeDir.FullName -Command "cargo run"

            if ($nodePid) {
                $pids += $nodePid
            }

            Start-Sleep -Seconds 1
        }
    }
}

# Save PIDs to file for stop-all.ps1
$pidsFile = Join-Path $ProjectRoot ".ndnm-pids.txt"
$pids | Out-File -FilePath $pidsFile -Encoding UTF8

Write-Host ""
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "NDNM System Started Successfully!" -ForegroundColor Green
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Services running:" -ForegroundColor Yellow
Write-Host "  - Hermes: http://localhost:3000" -ForegroundColor White
Write-Host "  - Brazil: http://localhost:3002 (WebSocket: ws://localhost:3002/ws)" -ForegroundColor White
Write-Host "  - Nodes: Check individual windows" -ForegroundColor White
Write-Host ""
Write-Host "To test the system:" -ForegroundColor Yellow
Write-Host "  .\test.ps1 health-hermes" -ForegroundColor White
Write-Host ""
Write-Host "To stop all services:" -ForegroundColor Yellow
Write-Host "  .\stop-all.ps1" -ForegroundColor White
Write-Host ""
Write-Host "Process IDs saved to: $pidsFile" -ForegroundColor Gray
