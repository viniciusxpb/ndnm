// src/hooks/useWsClient.ts
import { useEffect, useMemo, useRef, useState } from 'react';

export type WsStatus = 'idle' | 'connecting' | 'open' | 'closing' | 'closed';

export interface UseWsClientOptions {
  /** Envia "ping" a cada N ms para manter a conexão viva (0 = desabilita). */
  heartbeatMs?: number;
  /** Tempo base do backoff exponencial (ms). */
  backoffBaseMs?: number;
  /** Máximo entre tentativas (ms). */
  backoffMaxMs?: number;
  /** Quantidade máx. de tentativas de reconexão (Infinity = sem limite). */
  maxRetries?: number;
  /** Auto-reconectar ao fechar/erro. */
  autoreconnect?: boolean;
  /** Logar eventos básicos no console (open/close/error/retry). */
  debug?: boolean;
}

export interface UseWsClient {
  status: WsStatus;
  send: (data: string | ArrayBufferLike | Blob | ArrayBufferView) => boolean;
  /** Última mensagem recebida (raw). */
  lastMessage: MessageEvent<any> | null;
  /** Última mensagem parseada como JSON (se possível). */
  lastJson: unknown | null;
  /** Fechar manualmente (desativa auto-reconexão). */
  close: () => void;
}

// --- Helper de Log com Timestamp ---
function log(level: 'log' | 'warn' | 'error', ...args: any[]) {
  const ts = new Date().toISOString();
  console[level](`[${ts}]`, ...args);
}
// -----------------------------------

/**
 * Hook de WebSocket confiável com reconexão exponencial e heartbeat.
 * URL recomendada via env: import.meta.env.VITE_WS_URL
 */
export function useWsClient(url: string, opts?: UseWsClientOptions): UseWsClient {
  const options = useMemo<UseWsClientOptions>(() => ({
    heartbeatMs: 25_000,
    backoffBaseMs: 750,
    backoffMaxMs: 10_000,
    maxRetries: Infinity,
    autoreconnect: true,
    debug: true,
    ...opts,
  }), [opts]);

  const wsRef = useRef<WebSocket | null>(null);
  const hbRef = useRef<number | null>(null);
  const retriesRef = useRef(0);
  const manualCloseRef = useRef(false);

  const [status, setStatus] = useState<WsStatus>('idle');
  const [lastMessage, setLastMessage] = useState<MessageEvent<any> | null>(null);
  const [lastJson, setLastJson] = useState<unknown | null>(null);

  // Util: iniciar heartbeat
  const startHeartbeat = () => {
    if (!options.heartbeatMs || hbRef.current) return;
    hbRef.current = window.setInterval(() => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        options.debug && log('log', '🟡 [WS Front] Enviando Heartbeat (ping)');
        try { wsRef.current.send('ping'); } catch { /* ignore */ }
      }
    }, options.heartbeatMs);
  };

  const stopHeartbeat = () => {
    if (hbRef.current) {
      clearInterval(hbRef.current);
      hbRef.current = null;
    }
  };

  // Conectar (com backoff)
  useEffect(() => {
    if (!url) return;

    let cancelled = false;

    const connect = () => {
      if (cancelled) return;

      setStatus('connecting');
      options.debug && log('log', `🟡 [WS Front] Conectando em ${url}...`);
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        retriesRef.current = 0;
        setStatus('open');
        options.debug && log('log', '🟢 [WS Front] Conexão ABERTA:', url);
        startHeartbeat();
      };

      ws.onmessage = (ev) => {
        setLastMessage(ev);
        // tenta parsear JSON, se falhar deixa como null
        try {
          const parsed = JSON.parse(typeof ev.data === 'string' ? ev.data : '');
          setLastJson(parsed);
          // loga tudo que chegar (requisito)
          log('log', '🟢 [WS Front] Mensagem (JSON):', parsed);
        } catch {
          setLastJson(null);
          log('log', '🟢 [WS Front] Mensagem (RAW):', ev.data);
        }
      };

      ws.onerror = (err) => {
        // Erros de WS são notórios por serem inúteis. O 'err' é só um 'Event'.
        // O erro de verdade aparece no 'onclose'.
        options.debug && log('error', '🔴 [WS Front] ERROR EVENT:', err);
      };

      ws.onclose = (ev) => {
        setStatus('closed');
        stopHeartbeat();
        wsRef.current = null;
        
        // LOG LENDÁRIO AQUI: Isso é o que a gente quer ver
        options.debug && log('error',
          `🔴 [WS Front] CONEXÃO FECHADA: code=${ev.code} reason=${ev.reason || "''"} wasClean=${ev.wasClean}`
        );

        if (!cancelled && options.autoreconnect && !manualCloseRef.current) {
          const retry = retriesRef.current++;
          if (retry < (options.maxRetries ?? Infinity)) {
            const delay = Math.min(
              (options.backoffBaseMs ?? 750) * Math.pow(2, retry),
              options.backoffMaxMs ?? 10_000
            );
            options.debug && log('warn', `🟡 [WS Front] Reconectando em ${Math.round(delay)}ms (tentativa ${retry + 1})`);
            setStatus('connecting');
            const t = window.setTimeout(connect, delay);
            return () => clearTimeout(t);
          } else {
            options.debug && log('error', '🔴 [WS Front] Máximo de tentativas atingido. Desistindo.');
          }
        }
      };
    };

    connect();

    return () => {
      cancelled = true;
      manualCloseRef.current = true;
      stopHeartbeat();
      try { wsRef.current?.close(); } catch { /* ignore */ }
      wsRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url, options.autoreconnect, options.backoffBaseMs, options.backoffMaxMs, options.maxRetries, options.heartbeatMs, options.debug]);

  const send: UseWsClient['send'] = (data) => {
    const ok = !!wsRef.current && wsRef.current.readyState === WebSocket.OPEN;
    if (!ok) {
      options.debug && log('warn', '🔴 [WS Front] send(): socket não estava aberto');
      return false;
    }
    try {
      wsRef.current!.send(data);
      return true;
    } catch (e) {
      options.debug && log('error', '🔴 [WS Front] send() falhou:', e);
      return false;
    }
  };

  const close = () => {
    manualCloseRef.current = true;
    stopHeartbeat();
    try {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        setStatus('closing');
      }
      wsRef.current?.close();
    } catch { /* ignore */ }
  };

  return { status, send, lastMessage, lastJson, close };
}