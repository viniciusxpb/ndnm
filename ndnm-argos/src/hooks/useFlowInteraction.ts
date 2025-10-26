// src/hooks/useFlowInteraction.ts
import { useState, useCallback, useRef, useEffect } from 'react';
import { useReactFlow, useKeyPress, type Node, type Edge } from '@xyflow/react';
import { type NodePaletteItem } from '@/nodes/registry';
import usePaneClickCombo from '@/hooks/usePaneClickCombo';
import { normalizeIOMode } from '@/utils/flowUtils';

type PendingConnect = {
  fromNodeId: string | null;
  fromHandleId: string | null;
  pos: { x: number; y: number };
};

type UseFlowInteractionProps = {
  nodes: Node[];
  edges: Edge[];
  setNodes: React.Dispatch<React.SetStateAction<Node[]>>;
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
  nodePalette: NodePaletteItem[];
  onNodeValueChange: (nodeId: string, value: string) => void; // 🔥 NOVA PROP
};

export function useFlowInteraction({ 
  nodes, 
  edges, 
  setNodes, 
  setEdges, 
  nodePalette,
  onNodeValueChange // 🔥 RECEBE A FUNÇÃO DO FLOWCONTROLLER
}: UseFlowInteractionProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [pendingConnect, setPendingConnect] = useState<PendingConnect | null>(null);
  const [panelPos, setPanelPos] = useState<{ x: number; y: number } | null>(null);
  const [selectedNodes, setSelectedNodes] = useState<string[]>([]);
  const [selectedEdges, setSelectedEdges] = useState<string[]>([]);
  const connectingNodeId = useRef<string | null>(null);
  const connectingHandleId = useRef<string | null>(null);
  const { screenToFlowPosition } = useReactFlow();
  
  // 🔥 CORREÇÃO: Geração de ID baseada no maior ID existente
  const nextId = useCallback(() => {
    if (nodes.length === 0) return 'n1';
    
    const maxId = Math.max(...nodes.map(node => {
      const match = node.id.match(/n(\d+)/);
      return match ? parseInt(match[1]) : 0;
    }));
    
    return `n${maxId + 1}`;
  }, [nodes]);

  const onConnectStart = useCallback((_event: React.MouseEvent | React.TouchEvent, { nodeId, handleId }: { nodeId: string | null, handleId: string | null }) => {
    connectingNodeId.current = nodeId;
    connectingHandleId.current = handleId;
  }, []);

  const onConnectEnd = useCallback(
    (event: MouseEvent | TouchEvent) => {
      const targetIsPane = (event.target as Element).classList.contains('react-flow__pane');
      if (targetIsPane && connectingNodeId.current && connectingHandleId.current) {
        const isTouch = 'changedTouches' in event && event.changedTouches?.length > 0;
        const point = isTouch ? event.changedTouches[0] : (event as MouseEvent);
        const pos = screenToFlowPosition({ x: point.clientX, y: point.clientY });
        setPendingConnect({
          fromNodeId: connectingNodeId.current,
          fromHandleId: connectingHandleId.current,
          pos,
        });
        setIsModalOpen(true);
      }
      connectingNodeId.current = null;
      connectingHandleId.current = null;
    }, [screenToFlowPosition]
  );

  const addNodeByType = useCallback((typeKey: string) => {
    const spec = nodePalette.find((n) => n.type === typeKey);
    if (!spec) { setIsModalOpen(false); setPendingConnect(null); return; }
    const id = nextId();
    const baseData: any = JSON.parse(JSON.stringify(spec.default_data ?? {}));

    if (!baseData.label) { baseData.label = spec.label; }

    baseData.onChange = onNodeValueChange;

    baseData.inputsMode = normalizeIOMode(baseData.inputsMode);
    baseData.outputsMode = normalizeIOMode(baseData.outputsMode);
    if (baseData.inputsMode === 'n') baseData.inputsCount = Math.max(baseData.inputsCount ?? 1, 1);
    if (baseData.outputsMode === 'n') baseData.outputsCount = Math.max(baseData.outputsCount ?? 1, 1);

    let newNodePosition = { x: 0, y: 0 };
    const newNode: Node = { id, type: spec.type, className: 'hacker-node', position: { x: 0, y: 0 }, data: baseData };

    console.log('🆕 Adicionando novo node:', {
      id,
      type: spec.type,
      currentNodesCount: nodes.length,
      input_fields: baseData.input_fields,
      hasInputFields: !!baseData.input_fields
    });

    if (pendingConnect) {
      newNodePosition = pendingConnect.pos;
      newNode.position = newNodePosition;
      const { fromNodeId, fromHandleId } = pendingConnect;
      setNodes((nds) => [...nds, newNode]);
      if (fromNodeId && fromHandleId) {
        const targetHandleId = 'in_0';
        setEdges((eds) => {
          const newEdge: Edge = { id: `e-${fromNodeId}-${id}-${Date.now()}`, source: fromNodeId, target: id, sourceHandle: fromHandleId, targetHandle: targetHandleId };
          return [...eds, newEdge];
        });
      }
      setPendingConnect(null);
    } else {
      const center = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
      newNodePosition = screenToFlowPosition(center);
      newNode.position = newNodePosition;
      setNodes((nds) => [...nds, newNode]);
    }
    setIsModalOpen(false);
  }, [nodePalette, pendingConnect, screenToFlowPosition, edges, setNodes, setEdges, nextId, onNodeValueChange, nodes.length]);

  const del = useKeyPress('Delete');
  const bsp = useKeyPress('Backspace');
  useEffect(() => {
    const pressed = del || bsp;
    if (!pressed || (selectedNodes.length === 0 && selectedEdges.length === 0)) return;
    const selectedNodeIds = new Set(selectedNodes);
    const selectedEdgeIds = new Set(selectedEdges);
    setEdges((eds) => eds.filter(e => !selectedEdgeIds.has(e.id) && !selectedNodeIds.has(e.source) && !selectedNodeIds.has(e.target)));
    setNodes((nds) => nds.filter((n) => !selectedNodeIds.has(n.id)));
    setSelectedNodes([]); setSelectedEdges([]);
  }, [del, bsp, selectedNodes, selectedEdges, setNodes, setEdges]);

  const PANEL_OFFSET_X = 280, PANEL_OFFSET_Y = 100;
  const onPaneClick = usePaneClickCombo({
    onSingle: () => setPanelPos(null),
    onDouble: (e: MouseEvent) => { setPanelPos({ x: e.clientX - PANEL_OFFSET_X, y: e.clientY - PANEL_OFFSET_Y }); },
  });

  const handleCloseModal = useCallback(() => {
    connectingNodeId.current = null;
    connectingHandleId.current = null;
    setPendingConnect(null);
    setIsModalOpen(false);
  }, []);

  return {
    isModalOpen,
    panelPos,
    selectedNodes,
    selectedEdges,
    setIsModalOpen,
    setPanelPos,
    setSelectedNodes,
    setSelectedEdges,
    onConnectStart,
    onConnectEnd,
    addNodeByType,
    onPaneClick,
    handleCloseModal,
  };
}