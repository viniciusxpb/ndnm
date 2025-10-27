// ndnm-argos/src/utils/logReporter.ts

const EXDOIDA_URL = import.meta.env.VITE_EXDOIDA_LOG_URL || 'http://localhost:3003';

export type LogLevel = 'info' | 'warn' | 'error' | 'debug' | 'trace';

export async function reportLog(
  level: LogLevel,
  source: string,
  message: string,
  metadata?: Record<string, unknown>
): Promise<void> {
  try {
    await fetch(`${EXDOIDA_URL}/logs`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ level, source, message, metadata }),
      keepalive: true,
    });
  } catch {
    // Falha em observabilidade não bloqueia app
  }
}

export function initLogForwarding(source = 'ndnm-argos') {
  // Evitar duplicação em hot-reload
  if ((window as any).__logForwardingInitialized) return;
  (window as any).__logForwardingInitialized = true;

  const original = {
    log: console.log,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
  };

  console.log = (...args: any[]) => {
    original.log.apply(console, args);
    void reportLog('info', source, args.map(stringifyArg).join(' '));
  };
  console.warn = (...args: any[]) => {
    original.warn.apply(console, args);
    void reportLog('warn', source, args.map(stringifyArg).join(' '));
  };
  console.error = (...args: any[]) => {
    original.error.apply(console, args);
    void reportLog('error', source, args.map(stringifyArg).join(' '));
  };
  console.debug = (...args: any[]) => {
    original.debug.apply(console, args);
    void reportLog('debug', source, args.map(stringifyArg).join(' '));
  };
}

function stringifyArg(arg: unknown): string {
  if (typeof arg === 'string') return arg;
  try {
    return JSON.stringify(arg);
  } catch {
    return String(arg);
  }
}