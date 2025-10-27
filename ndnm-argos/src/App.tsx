// src/App.tsx
import { useState, useEffect } from 'react';
import { ReactFlowProvider } from '@xyflow/react';
import FlowController from '@/flow/FlowController';
import '@/styles/hacker.scss';
import '@/style.scss';
import { useWsClient } from '@/hooks/useWsClient';
import { WebSocketStatus } from '@/components/WebSocketStatus';
import { buildWsUrl } from '@/utils/wsUrl';
import { useNodeConfig } from '@/hooks/useNodeConfig';

export default function App() {
  const WS_URL = buildWsUrl();
  const { nodeConfig, loading, error } = useNodeConfig();

  const client = useWsClient(WS_URL, {
    autoreconnect: true,
    heartbeatMs: 25000,
    debug: true,
  });

  useEffect(() => {
    // Log de inicialização
    console.log('Aplicação NDNM-Argos inicializada');
    console.log('WebSocket URL:', WS_URL);
    
    if (error) {
      console.error('Erro ao carregar configuração do node', error);
    }
  }, [error, WS_URL]);

  return (
    <ReactFlowProvider>
      <WebSocketStatus status={client.status} />
      {loading ? (
        <div className="loading">Carregando configuração do node...</div>
      ) : error ? (
        <div className="error">
          Erro ao carregar configuração do node: {error.message}
        </div>
      ) : (
        <FlowController nodeConfig={nodeConfig} />
      )}
    </ReactFlowProvider>
  );
}