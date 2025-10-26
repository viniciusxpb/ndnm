// src/flow/FlowController.tsx
import React, { useState, useCallback, useEffect } from 'react';
import type { Node, Edge } from '@xyflow/react';
import LeftPanel from '@/components/LeftPanel';
import HackerModal from '@/components/HackerModal';
import NodeCatalog from '@/components/NodeCatalog';
import { PinnedPlayButtons } from '@/components/PinnedPlayButtons';
import { useNodePalette } from '@/hooks/useNodePalette';
import { useFlowStateSync } from '@/hooks/useFlowStateSync';
import { useFlowInteraction } from '@/hooks/useFlowInteraction';
import { useWorkspacePersistence } from '@/hooks/useWorkspacePersistence';
import { usePinnedNodes } from '@/hooks/usePinnedNodes';
import { useWorkflowExecution } from '@/hooks/useWorkflowExecution';
import { FlowCanvas } from './FlowCanvas';

type FlowControllerProps = {
  onReassignNodeData?: (nodes: Node[]) => Node[];
};

export default function FlowController({ onReassignNodeData }: FlowControllerProps) {
  const [workspaceName, setWorkspaceName] = useState('workspace-1');
  const nodePalette = useNodePalette();
  const { nodes, edges, setNodes, setEdges, onNodesChange, onEdgesChange, onConnect } = useFlowStateSync();
  const { pinnedNodes, pinNode, unpinNode, isPinned } = usePinnedNodes();
  const { executeWorkflow, executionState } = useWorkflowExecution();

  const handleNodeValueChange = useCallback((nodeId: string, value: string) => {
    console.log(`📝 Atualizando node ${nodeId} para valor:`, value);
    setNodes(nds => nds.map(node =>
      node.id === nodeId ? {
        ...node,
        data: {
          ...node.data,
          value
        }
      } : node
    ));
  }, [setNodes]);

  const handlePinNode = useCallback((nodeId: string, label: string) => {
    const node = nodes.find(n => n.id === nodeId);
    if (!node) return;
    pinNode(nodeId, label, node.type || 'unknown');
  }, [nodes, pinNode]);

  const handleExecuteFromNode = useCallback((nodeId: string) => {
    console.log('🚀 Iniciando execução a partir do node:', nodeId);
    executeWorkflow(nodes, edges, workspaceName);
  }, [executeWorkflow, nodes, edges, workspaceName]);

  const isExecuting = executionState.status === 'running';

  useEffect(() => {
    setNodes(nds => nds.map(node => {
      const isPlayNode = node.type === 'playButton' || node.type === 'comfyPlay';
      if (!isPlayNode) return node;

      return {
        ...node,
        data: {
          ...node.data,
          isPinned: isPinned(node.id),
          onPin: handlePinNode,
          onUnpin: unpinNode,
          onExecute: handleExecuteFromNode,
          isExecuting,
        }
      };
    }));
  }, [pinnedNodes, setNodes, isPinned, handlePinNode, unpinNode, handleExecuteFromNode, isExecuting]);

  const {
    isModalOpen, panelPos,
    setIsModalOpen, setPanelPos,
    onConnectStart, onConnectEnd, addNodeByType, onPaneClick, handleCloseModal
  } = useFlowInteraction({ 
    nodes, 
    edges, 
    setNodes, 
    setEdges, 
    nodePalette,
    onNodeValueChange: handleNodeValueChange // 🔥 PASSA A FUNÇÃO PARA USE_FLOW_INTERACTION
  });

  const { saveWorkspace, loadWorkspace } = useWorkspacePersistence();

  // 🔥 CORREÇÃO: Função para processar nós carregados do workspace
  const handleLoadWorkspaceFromPanel = useCallback((newNodes: Node[], newEdges: Edge[]) => {
    let processedNodes = newNodes;
    
    console.log('🔄 Carregando workspace com', newNodes.length, 'nodes e', newEdges.length, 'edges');
    
    // Aplica o reassign das funções se disponível
    if (onReassignNodeData) {
      processedNodes = onReassignNodeData(newNodes);
    } else {
      // Fallback: reassign local COM A FUNÇÃO CORRETA
      processedNodes = newNodes.map(node => ({
        ...node,
        data: {
          ...node.data,
          onChange: handleNodeValueChange
        }
      }));
    }
    
    setNodes(processedNodes);
    setEdges(newEdges);
    console.log('🎯 Workspace carregado via LeftPanel. Total nodes:', processedNodes.length);
  }, [setNodes, setEdges, onReassignNodeData, handleNodeValueChange]);

  const handleSaveWorkspace = async () => {
    const success = await saveWorkspace(workspaceName, nodes, edges);
    if (success) {
      alert(`✅ Workspace "${workspaceName}" salvo!`);
    } else {
      alert(`❌ Erro ao salvar workspace`);
    }
  };

  console.log('🔍 FlowController - nodes count:', nodes.length, 'edges count:', edges.length);

  const pinnedNodesData = nodes.filter(node =>
    pinnedNodes.some(p => p.nodeId === node.id)
  );

  return (
    <>
      <PinnedPlayButtons
        pinnedNodes={pinnedNodesData}
        onUnpin={unpinNode}
        onExecute={handleExecuteFromNode}
        isExecuting={isExecuting}
      />
      <div className="globalWrapper">
        <LeftPanel
          onOpenModal={() => setIsModalOpen(true)}
          nodes={nodes}
          edges={edges}
          onLoadWorkspace={handleLoadWorkspaceFromPanel}
          onReassignNodeData={onReassignNodeData}
        />
        <FlowCanvas
          nodes={nodes}
          edges={edges}
          nodePalette={nodePalette}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onConnectStart={onConnectStart}
          onConnectEnd={onConnectEnd}
          onPaneClick={onPaneClick}
          panelPos={panelPos}
          setPanelPos={setPanelPos}
        />
      </div>
      <HackerModal open={isModalOpen} onClose={handleCloseModal}>
        <NodeCatalog onPick={addNodeByType} nodePalette={nodePalette} />
      </HackerModal>
    </>
  );
}