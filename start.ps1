# reset-and-run.ps1
# Reinicia o backend do NDNM do zero e inicia o frontend.
# Uso:
#   .\reset-and-run.ps1              # Para tudo, inicia backend e frontend
#   .\reset-and-run.ps1 -NoFrontend  # Para tudo, inicia backend sem abrir o frontend

param(
  [switch]$NoFrontend
)

Write-Host "Resetando NDNM: matando nodes/serviços, reiniciando backend e iniciando frontend..." -ForegroundColor Cyan

# 1) Parar serviços e nodes se estiverem ativos
if (Test-Path "$PSScriptRoot\stop-all.ps1") {
  try {
    Write-Host "Parando serviços com stop-all.ps1..." -ForegroundColor Yellow
    & "$PSScriptRoot\stop-all.ps1"
  } catch {
    Write-Warning "Falha ao executar stop-all.ps1: $_"
  }
} else {
  Write-Warning "stop-all.ps1 não encontrado em $PSScriptRoot"
}

# 2) Limpar arquivo de PIDs (se existir)
$pidFile = Join-Path $PSScriptRoot ".ndnm-pids.txt"
if (Test-Path $pidFile) {
  try { Remove-Item $pidFile -Force -ErrorAction SilentlyContinue } catch { }
}

# 3) Iniciar backend do zero
if (Test-Path "$PSScriptRoot\start-all.ps1") {
  Write-Host "Iniciando backend com start-all.ps1..." -ForegroundColor Yellow
  & "$PSScriptRoot\start-all.ps1"
} else {
  Write-Error "start-all.ps1 não encontrado em $PSScriptRoot"
  exit 1
}

# 4) Iniciar frontend (opcional)
if (-not $NoFrontend) {
  $frontendPath = Join-Path $PSScriptRoot "ndnm-argos"
  if (Test-Path $frontendPath) {
    Write-Host "Abrindo frontend (npm run dev) em nova janela..." -ForegroundColor Yellow
    # Abre uma nova janela do PowerShell para rodar o dev server sem bloquear
    Start-Process -FilePath "powershell" -ArgumentList "-NoExit", "-Command", "cd `"$frontendPath`"; npm run dev" -WorkingDirectory $frontendPath
  } else {
    Write-Warning "Diretório do frontend não encontrado: $frontendPath"
  }
}

Write-Host "Reset concluído." -ForegroundColor Green