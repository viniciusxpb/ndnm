param([switch]$Headless)
# Start All NDNM Services (single window backend)
# Backend inicia e permanece em UMA janela. Se qualquer serviço encerrar, todos são finalizados.

Write-Host "Starting NDNM System (single window)..." -ForegroundColor Cyan
Write-Host ""

$ProjectRoot = $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "ndnm-backend"

# Logs directory for headless mode
$LogsDir = Join-Path $ProjectRoot ".logs"
if (-not (Test-Path $LogsDir)) { New-Item -ItemType Directory -Path $LogsDir | Out-Null }

# Ensure cargo exists
try { $null = Get-Command cargo -ErrorAction Stop } catch {
    Write-Host "Error: Cargo not found. Make sure Rust is installed." -ForegroundColor Red
    exit 1
}

# Helpers: Port check/cleanup
function Get-PortOwnerPid {
    param([int]$Port)
    try {
        $conn = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
        if ($conn) { return $conn.OwningProcess } else { return $null }
    } catch { return $null }
}

function Ensure-Port-Free {
    param([int]$Port,[string]$ServiceName)
    $ownerPid = Get-PortOwnerPid -Port $Port
    if ($ownerPid) {
        Write-Host "[WARN] Porta $Port ocupada antes de iniciar $ServiceName (PID: $ownerPid). Tentando liberar..." -ForegroundColor Yellow
        try { Stop-Process -Id $ownerPid -Force -ErrorAction SilentlyContinue; Write-Host "  [OK] Porta $Port liberada." -ForegroundColor Green }
        catch { Write-Host "  [FAIL] Não foi possível liberar porta ${Port}: $($_.Exception.Message)" -ForegroundColor Red }
    }
}

# Start cargo process in same window
function Start-BackendProc {
    param(
        [string]$Name,
        [string]$WorkingDir,
        [string[]]$CargoArgs,
        [int]$PortToCheck
    )

    if ($PortToCheck -gt 0) { Ensure-Port-Free -Port $PortToCheck -ServiceName $Name }

    Write-Host "=== $Name ===" -ForegroundColor Cyan
    $outLog = Join-Path $LogsDir "$Name.out.log"
    $errLog = Join-Path $LogsDir "$Name.err.log"

    if ($Headless) {
        # Headless: redireciona para arquivos de log
        $proc = Start-Process -FilePath "cargo" -ArgumentList $CargoArgs -WorkingDirectory $WorkingDir -RedirectStandardOutput $outLog -RedirectStandardError $errLog -PassThru
    } else {
        # Única janela: sem redirecionar, outputs entram nesta janela
        $proc = Start-Process -FilePath "cargo" -ArgumentList $CargoArgs -WorkingDirectory $WorkingDir -NoNewWindow -PassThru
    }

    if ($proc) {
        Write-Host "  [OK] $Name started (PID: $($proc.Id))" -ForegroundColor Green
        return $proc
    } else {
        Write-Host "  [FAIL] Failed to start $Name" -ForegroundColor Red
        return $null
    }
}

# PIDs/procs tracking
$procs = New-Object System.Collections.ArrayList
$pidsFile = Join-Path $ProjectRoot ".ndnm-pids.txt"

# Start order: Hermes -> Brazil -> Exdoida -> Nodes
Write-Host ""; Write-Host "Step 1: Starting Hermes Orchestrator" -ForegroundColor Cyan
$hermes = Start-BackendProc -Name "Hermes" -WorkingDir $BackendDir -CargoArgs @("run","-p","ndnm-hermes") -PortToCheck 3000
if ($hermes) { [void]$procs.Add($hermes) }
Start-Sleep -Seconds 2

Write-Host ""; Write-Host "Step 2: Starting Brazil BFF" -ForegroundColor Cyan
$brazil = Start-BackendProc -Name "Brazil" -WorkingDir $BackendDir -CargoArgs @("run","-p","ndnm-brazil") -PortToCheck 3002
if ($brazil) { [void]$procs.Add($brazil) }
Start-Sleep -Seconds 2

Write-Host ""; Write-Host "Step 3: Starting Exdoida Observability" -ForegroundColor Cyan
$exdoida = Start-BackendProc -Name "Exdoida" -WorkingDir $BackendDir -CargoArgs @("run","-p","ndnm-exdoida") -PortToCheck 3003
if ($exdoida) { [void]$procs.Add($exdoida) }
Start-Sleep -Seconds 2

Write-Host ""; Write-Host "Step 4: Discovering and Starting Nodes" -ForegroundColor Cyan
$nodesDir = Join-Path $BackendDir "nodes"
if (Test-Path $nodesDir) {
    $nodeDirectories = Get-ChildItem -Path $nodesDir -Directory
    foreach ($nodeDir in $nodeDirectories) {
        $configPath = Join-Path $nodeDir.FullName "config.yaml"
        if (Test-Path $configPath) {
            $nodeName = $nodeDir.Name
            # Port check for known nodes
            $port = if ($nodeName -eq "node-file-browser") { 3001 } else { 0 }
            $nodeProc = Start-BackendProc -Name $nodeName -WorkingDir $nodeDir.FullName -CargoArgs @("run") -PortToCheck $port
            if ($nodeProc) { [void]$procs.Add($nodeProc) }
            Start-Sleep -Seconds 1
        }
    }
}

# Save PIDs
$procs | ForEach-Object { $_.Id } | Out-File -FilePath $pidsFile -Encoding UTF8

# Register cascading shutdown: if any exits, kill the rest
foreach ($proc in $procs) {
    try {
        $proc.EnableRaisingEvents = $true
        Register-ObjectEvent -InputObject $proc -EventName Exited -SourceIdentifier "ndnm-exit-$($proc.Id)" -Action {
            try {
                $pf = Join-Path $PSScriptRoot ".ndnm-pids.txt"
                if (Test-Path $pf) {
                    $ids = Get-Content $pf | ForEach-Object { [int]$_ }
                    foreach ($id in $ids) {
                        if ($id -ne $event.Sender.Id) {
                            try { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue } catch {}
                        }
                    }
                }
            } finally {
                Write-Host "[EXIT] $($event.Sender.ProcessName) saiu. Encerrando backend." -ForegroundColor Red
            }
        } | Out-Null
    } catch {}
}

Write-Host ""; Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "NDNM System Started Successfully!" -ForegroundColor Green
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host ""; Write-Host "Services running:" -ForegroundColor Yellow
Write-Host "  - Hermes: http://localhost:3000" -ForegroundColor White
Write-Host "  - Brazil: http://localhost:3002 (WebSocket: ws://localhost:3002/ws)" -ForegroundColor White
Write-Host "  - Exdoida: http://localhost:3003 (UDP Logs: port 9514)" -ForegroundColor White
Write-Host "  - Nodes: node-file-browser em http://localhost:3001" -ForegroundColor White
if ($Headless) { Write-Host "Logs gravados em: $LogsDir" -ForegroundColor Gray }

# Keep this window attached to backend lifecycle
$idsToWait = $procs | ForEach-Object { $_.Id }
if ($idsToWait.Count -gt 0) {
    Write-Host "Aguardando backend (Ctrl+C para encerrar manualmente)..." -ForegroundColor Gray
    try { Wait-Process -Id $idsToWait } catch {}
}
