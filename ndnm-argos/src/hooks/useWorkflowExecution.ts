// src/hooks/useWorkflowExecution.ts
//
// Hook para gerenciar execução de workflows

import { useState, useCallback, useEffect } from 'react';
import type { Node, Edge } from '@xyflow/react';
import { useWsClient } from './useWsClient';
import { buildWsUrl } from '@/utils/wsUrl';
import {
  convertGraphForBackend,
  findPlayNodes,
  validateGraphForExecution,
} from '@/utils/graphConverter';

export type ExecutionStatus = 'idle' | 'running' | 'completed' | 'error';

export interface ExecutionState {
  status: ExecutionStatus;
  runId: string | null;
  totalNodes: number;
  executedNodes: number;
  cachedNodes: number;
  durationMs: number;
  error: string | null;
  currentNode: string | null;
}

const initialState: ExecutionState = {
  status: 'idle',
  runId: null,
  totalNodes: 0,
  executedNodes: 0,
  cachedNodes: 0,
  durationMs: 0,
  error: null,
  currentNode: null,
};

export function useWorkflowExecution() {
  const [executionState, setExecutionState] = useState<ExecutionState>(initialState);
  const WS_URL = buildWsUrl();
  const client = useWsClient(WS_URL, {
    autoreconnect: true,
    heartbeatMs: 25000,
    debug: true,
  });

  /**
   * Executa o workflow
   */
  const executeWorkflow = useCallback(
    (nodes: Node[], edges: Edge[], workspaceName: string = 'default') => {
      console.log('🚀 Iniciando execução do workflow...');

      // Valida o grafo
      const validation = validateGraphForExecution(nodes, edges);
      if (!validation.valid) {
        console.error('❌ Validação do grafo falhou:', validation.errors);
        setExecutionState({
          ...initialState,
          status: 'error',
          error: validation.errors.join('; '),
        });
        return;
      }

      // Encontra o primeiro Play node
      const playNodes = findPlayNodes(nodes);
      if (playNodes.length === 0) {
        setExecutionState({
          ...initialState,
          status: 'error',
          error: 'Nenhum node Play encontrado',
        });
        return;
      }

      const playNode = playNodes[0]; // Usa o primeiro Play node encontrado
      console.log('🎯 Play node selecionado:', playNode.id);

      // Converte o grafo para o formato do backend
      const backendGraph = convertGraphForBackend(nodes, edges);

      // Monta a mensagem EXECUTE_PLAY
      const message = {
        type: 'EXECUTE_PLAY',
        play_node_id: playNode.id,
        workspace_id: workspaceName,
        graph: backendGraph,
      };

      console.log('📤 Enviando mensagem EXECUTE_PLAY:', message);

      // Envia via WebSocket
      if (client.status === 'open') {
        client.send(JSON.stringify(message));
        setExecutionState({
          ...initialState,
          status: 'running',
        });
      } else {
        console.error('❌ WebSocket não está conectado. Status:', client.status);
        setExecutionState({
          ...initialState,
          status: 'error',
          error: `WebSocket não conectado (status: ${client.status})`,
        });
      }
    },
    [client]
  );

  /**
   * Escuta mensagens do backend
   */
  useEffect(() => {
    if (!client.lastJson) return;

    const message = client.lastJson as any;
    console.log('📨 Mensagem recebida do backend:', message);

    switch (message.type) {
      case 'EXECUTION_STATUS':
        // Atualização de status em tempo real
        console.log('⚙️ Status de execução:', message);
        setExecutionState(prev => ({
          ...prev,
          status: 'running',
          runId: message.run_id,
          currentNode: message.current_node,
          totalNodes: message.total_nodes || prev.totalNodes,
        }));
        break;

      case 'EXECUTION_COMPLETE':
        // Execução concluída com sucesso
        console.log('✅ Execução completa:', message);
        setExecutionState({
          status: 'completed',
          runId: message.run_id,
          totalNodes: message.total_nodes,
          executedNodes: message.executed_nodes,
          cachedNodes: message.cached_nodes,
          durationMs: message.duration_ms,
          error: null,
          currentNode: null,
        });
        break;

      case 'EXECUTION_ERROR':
        // Erro durante a execução
        console.error('❌ Erro na execução:', message);
        setExecutionState(prev => ({
          ...prev,
          status: 'error',
          runId: message.run_id,
          error: message.error,
          currentNode: message.failed_node || prev.currentNode,
        }));
        break;

      default:
        // Ignora outras mensagens (NODE_CONFIG, ECHO, etc)
        break;
    }
  }, [client.lastJson]);

  /**
   * Reseta o estado de execução
   */
  const resetExecution = useCallback(() => {
    setExecutionState(initialState);
  }, []);

  return {
    executionState,
    executeWorkflow,
    resetExecution,
    isWebSocketConnected: client.status === 'open',
    websocketStatus: client.status,
  };
}
