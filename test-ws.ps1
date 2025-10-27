# test-ws.ps1
param(
  [string]$Url = "ws://127.0.0.1:3002/ws",
  [int]$TimeoutMs = 5000
)

Add-Type -AssemblyName System.Net.Http
Add-Type -AssemblyName System.Net.WebSockets

$ws = [System.Net.WebSockets.ClientWebSocket]::new()
$cts = [System.Threading.CancellationTokenSource]::new()
$cts.CancelAfter($TimeoutMs)

try {
  Write-Host "Conectando a $Url ..." -ForegroundColor Cyan
  $uri = [System.Uri]::new($Url)
  $ws.ConnectAsync($uri, $cts.Token).GetAwaiter().GetResult()
  Write-Host "Conexão aberta." -ForegroundColor Green

  # Envia uma mensagem simples
  $msg = "ping"
  $bytes = [System.Text.Encoding]::UTF8.GetBytes($msg)
  $seg = [System.ArraySegment[byte]]::new($bytes)
  $ws.SendAsync($seg, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, $cts.Token).GetAwaiter().GetResult()
  Write-Host "Mensagem enviada: $msg" -ForegroundColor Yellow

  # Recebe uma resposta (até Timeout)
  $buffer = New-Object byte[] 4096
  $bufSeg = [System.ArraySegment[byte]]::new($buffer)
  $result = $ws.ReceiveAsync($bufSeg, $cts.Token).GetAwaiter().GetResult()
  if ($result.Count -gt 0) {
    $text = [System.Text.Encoding]::UTF8.GetString($buffer, 0, $result.Count)
    Write-Host "Mensagem recebida: $text" -ForegroundColor Green
  } else {
    Write-Warning "Sem dados recebidos dentro do timeout ($TimeoutMs ms)."
  }

} catch {
  Write-Error "Falha no WebSocket: $_"
} finally {
  try { $ws.CloseAsync([System.Net.WebSockets.WebSocketCloseStatus]::NormalClosure, "bye", $cts.Token).GetAwaiter().GetResult() } catch {}
  $ws.Dispose()
  $cts.Dispose()
}