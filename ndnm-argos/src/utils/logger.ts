// src/utils/logger.ts
// Frontend logger que intercepta console e envia para node-ex-doida

interface LogEntry {
  timestamp: string;
  level: 'log' | 'error' | 'warn' | 'info' | 'debug';
  message: string;
  data?: any;
}

class FrontendLogger {
  private exDoIdaUrl: string;
  private originalConsole: {
    log: typeof console.log;
    error: typeof console.error;
    warn: typeof console.warn;
    info: typeof console.info;
    debug: typeof console.debug;
  };
  private isEnabled: boolean = false;

  constructor(exDoIdaUrl: string = 'http://localhost:3999/log') {
    this.exDoIdaUrl = exDoIdaUrl;

    // Salva as funções originais do console
    this.originalConsole = {
      log: console.log.bind(console),
      error: console.error.bind(console),
      warn: console.warn.bind(console),
      info: console.info.bind(console),
      debug: console.debug.bind(console),
    };
  }

  /**
   * Inicia a interceptação dos logs do console
   */
  init() {
    if (this.isEnabled) {
      this.originalConsole.warn('[Logger] Already initialized');
      return;
    }

    this.isEnabled = true;
    this.originalConsole.log('[Logger] Initializing frontend logger -> node-ex-doida');

    // Intercepta console.log
    console.log = (...args: any[]) => {
      this.originalConsole.log(...args);
      this.sendLog('log', args);
    };

    // Intercepta console.error
    console.error = (...args: any[]) => {
      this.originalConsole.error(...args);
      this.sendLog('error', args);
    };

    // Intercepta console.warn
    console.warn = (...args: any[]) => {
      this.originalConsole.warn(...args);
      this.sendLog('warn', args);
    };

    // Intercepta console.info
    console.info = (...args: any[]) => {
      this.originalConsole.info(...args);
      this.sendLog('info', args);
    };

    // Intercepta console.debug
    console.debug = (...args: any[]) => {
      this.originalConsole.debug(...args);
      this.sendLog('debug', args);
    };

    this.originalConsole.log('[Logger] Console interception active');
  }

  /**
   * Para a interceptação e restaura o console original
   */
  stop() {
    if (!this.isEnabled) return;

    console.log = this.originalConsole.log;
    console.error = this.originalConsole.error;
    console.warn = this.originalConsole.warn;
    console.info = this.originalConsole.info;
    console.debug = this.originalConsole.debug;

    this.isEnabled = false;
    this.originalConsole.log('[Logger] Console interception stopped');
  }

  /**
   * Envia o log para o node-ex-doida
   */
  private async sendLog(level: LogEntry['level'], args: any[]) {
    try {
      // Converte os argumentos em uma mensagem
      const message = args
        .map(arg => {
          if (typeof arg === 'object') {
            try {
              return JSON.stringify(arg, null, 2);
            } catch {
              return String(arg);
            }
          }
          return String(arg);
        })
        .join(' ');

      // Prepara os dados adicionais (se houver objetos)
      const dataObjects = args.filter(arg => typeof arg === 'object' && arg !== null);
      const data = dataObjects.length > 0 ? dataObjects : undefined;

      const entry: LogEntry = {
        timestamp: new Date().toISOString(),
        level,
        message,
        data,
      };

      // Envia para node-ex-doida (fire-and-forget, não bloqueia)
      fetch(this.exDoIdaUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(entry),
      }).catch(err => {
        // Falha silenciosa - não queremos quebrar a aplicação por causa de logs
        // Mas usa o console original para debug
        this.originalConsole.error('[Logger] Failed to send log to node-ex-doida:', err);
      });
    } catch (err) {
      this.originalConsole.error('[Logger] Error in sendLog:', err);
    }
  }

  /**
   * Acesso às funções originais do console (útil para debug interno)
   */
  get original() {
    return this.originalConsole;
  }
}

// Instância singleton
export const frontendLogger = new FrontendLogger();

// Auto-inicializa em desenvolvimento
if (import.meta.env.DEV) {
  // Pequeno delay para garantir que tudo carregou
  setTimeout(() => {
    frontendLogger.init();
  }, 100);
}

export default frontendLogger;
