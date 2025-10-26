// src/utils/wsUrl.ts
/**
 * Monta a URL do WebSocket de forma robusta.
 * Regras:
 * - Usa wss:// se a página estiver em https://
 * - Substitui 0.0.0.0 -> localhost (pois 0.0.0.0 é só endereço de bind)
 * - Permite override por env: VITE_WS_URL, VITE_WS_HOST, VITE_WS_PORT, VITE_WS_PATH
 */
export function buildWsUrl(): string {
  // 1) Se VITE_WS_URL estiver setado, usa direto (mas corrige 0.0.0.0)
  const raw = (import.meta as any)?.env?.VITE_WS_URL as string | undefined;
  if (raw && raw.trim().length > 0) {
    try {
      const u = new URL(raw);
      if (u.hostname === "0.0.0.0") u.hostname = "localhost";
      return u.toString();
    } catch {
      // cai pro fallback abaixo
    }
  }

  // 2) Monta pela página atual
  const loc = window.location;
  const isHttps = loc.protocol === "https:";
  const proto = isHttps ? "wss:" : "ws:";

  // Overrides por env
  let host = ((import.meta as any)?.env?.VITE_WS_HOST as string) || loc.hostname || "localhost";
  let port = ((import.meta as any)?.env?.VITE_WS_PORT as string) || "3100";
  let path = ((import.meta as any)?.env?.VITE_WS_PATH as string) || "/ws";

  if (host === "0.0.0.0") host = "localhost";
  if (!path.startsWith("/")) path = "/" + path;

  // Se o host atual já tem a porta do backend, mantenha; senão usa a porta indicada
  const hostWithPort =
    (isHttps && port === "443") || (!isHttps && port === "80")
      ? host
      : `${host}:${port}`;

  return `${proto}//${hostWithPort}${path}`;
}
