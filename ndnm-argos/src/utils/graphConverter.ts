// src/utils/graphConverter.ts
//
// Converte o grafo do React Flow (frontend) para o formato esperado pelo executor (backend)

import type { Node, Edge } from '@xyflow/react';

/**
 * Formato esperado pelo backend (ndnm-brazil/src/execution/types.rs)
 */
export interface BackendGraphNode {
  id: string;
  node_type: string;
  port: number;
  label: string;
  data: Record<string, any>;
}

export interface BackendConnection {
  from_node_id: string;
  from_output_index: number;
  to_node_id: string;
  to_input_index: number;
}

export interface BackendWorkflowGraph {
  nodes: BackendGraphNode[];
  connections: BackendConnection[];
}

/**
 * Extrai o índice do handle a partir do ID
 * Exemplos:
 *   "in_0" → 0
 *   "in_1" → 1
 *   "out_2" → 2
 */
function extractHandleIndex(handleId: string | null | undefined): number {
  if (!handleId) return 0;
  const match = handleId.match(/_(\d+)$/);
  return match ? parseInt(match[1], 10) : 0;
}

/**
 * Mapeia tipo de node do frontend para porta do backend
 * Consulta o config.yaml de cada node para saber a porta
 */
function getNodePort(nodeType: string): number {
  // TODO: Idealmente isso deveria vir do NODE_CONFIG que o backend envia
  // Por enquanto, mapeamento hardcoded baseado nos configs conhecidos
  const portMap: Record<string, number> = {
    'add': 3000,           // node-sum
    'subtract': 3001,      // node-subtract
    'fixedValue': 3010,    // node-fixed-value
    'fsBrowser': 3011,     // node-fs-browser
    'playButton': 3020,    // node-play-button
    'comfyPlay': 3021,     // node-comfy-play
    'emptyLatentImage': 3002, // node-empty-latent-image
    'loadCheckpoint': 3003,   // node-load-checkpoint
    'ksampler': 3004,         // node-ksampler
    'listDirectory': 3005,    // node-list-directory
  };

  return portMap[nodeType] || 3000;
}

/**
 * Converte nodes do React Flow para o formato do backend
 */
function convertNodes(nodes: Node[]): BackendGraphNode[] {
  return nodes.map(node => ({
    id: node.id,
    node_type: node.type || 'unknown',
    port: getNodePort(node.type || 'unknown'),
    label: (node.data as any)?.label || node.type || 'Unknown',
    data: {
      // Envia todos os campos de data do node
      ...(node.data || {}),
      // Remove funções (como onChange) que não são serializáveis
      onChange: undefined,
    }
  }));
}

/**
 * Converte edges do React Flow para connections do backend
 */
function convertEdges(edges: Edge[]): BackendConnection[] {
  return edges.map(edge => ({
    from_node_id: edge.source,
    from_output_index: extractHandleIndex(edge.sourceHandle),
    to_node_id: edge.target,
    to_input_index: extractHandleIndex(edge.targetHandle),
  }));
}

/**
 * Converte grafo completo do React Flow para formato do backend
 */
export function convertGraphForBackend(
  nodes: Node[],
  edges: Edge[]
): BackendWorkflowGraph {
  console.log('🔄 Convertendo grafo para formato do backend...');
  console.log('   Nodes no frontend:', nodes.length);
  console.log('   Edges no frontend:', edges.length);

  const backendGraph: BackendWorkflowGraph = {
    nodes: convertNodes(nodes),
    connections: convertEdges(edges),
  };

  console.log('✅ Grafo convertido:');
  console.log('   Nodes no backend:', backendGraph.nodes.length);
  console.log('   Connections no backend:', backendGraph.connections.length);

  return backendGraph;
}

/**
 * Encontra todos os nodes do tipo Play no grafo
 */
export function findPlayNodes(nodes: Node[]): Node[] {
  return nodes.filter(node =>
    node.type === 'playButton' || node.type === 'comfyPlay'
  );
}

/**
 * Valida se o grafo está pronto para execução
 */
export function validateGraphForExecution(nodes: Node[], edges: Edge[]): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  // Verifica se há pelo menos um node
  if (nodes.length === 0) {
    errors.push('O grafo está vazio. Adicione pelo menos um node.');
  }

  // Verifica se há pelo menos um Play node
  const playNodes = findPlayNodes(nodes);
  if (playNodes.length === 0) {
    errors.push('Nenhum node Play encontrado. Adicione um node Play para executar o workflow.');
  }

  // Verifica se há nodes isolados (sem conexões)
  const connectedNodeIds = new Set<string>();
  edges.forEach(edge => {
    connectedNodeIds.add(edge.source);
    connectedNodeIds.add(edge.target);
  });

  const isolatedNodes = nodes.filter(node =>
    !connectedNodeIds.has(node.id) &&
    node.type !== 'playButton' &&
    node.type !== 'comfyPlay'
  );

  if (isolatedNodes.length > 0) {
    console.warn('⚠️ Nodes isolados detectados:', isolatedNodes.map(n => n.id));
    // Não é um erro crítico, apenas um aviso
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
