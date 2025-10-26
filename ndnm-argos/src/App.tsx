// src/App.tsx
import { ReactFlowProvider } from '@xyflow/react';
import FlowController from '@/flow/FlowController';
import '@/styles/hacker.scss';
import '@/style.scss';
import { useWsClient } from '@/hooks/useWsClient';
import { WebSocketStatus } from '@/components/WebSocketStatus';
import { buildWsUrl } from '@/utils/wsUrl';
import '@/utils/logger'; // Inicia o logger automaticamente

export default function App() {
  const WS_URL = buildWsUrl();

  const client = useWsClient(WS_URL, {
    autoreconnect: true,
    heartbeatMs: 25000,
    debug: true,
  });

  return (
    <ReactFlowProvider>
      <WebSocketStatus status={client.status} />
      <FlowController />
    </ReactFlowProvider>
  );
}