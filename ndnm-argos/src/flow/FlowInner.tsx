// src/flow/FlowInner.tsx
import { useState, useCallback, useRef, useEffect, useMemo } from 'react';
import {
  ReactFlow,
  applyNodeChanges,
  applyEdgeChanges,
  useOnSelectionChange,
  addEdge,
  Panel,
  useReactFlow,
  useKeyPress,
  useUpdateNodeInternals,
  type NodeTypes,
  type EdgeChange, type Connection, type Node, type Edge
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import LeftPanel from '@/components/LeftPanel';
import HackerModal from '@/components/HackerModal';
import NodeCatalog from '@/components/NodeCatalog';
import usePaneClickCombo from '@/hooks/usePaneClickCombo';
import { BaseIONode } from '@/nodes/BaseIONode';
import { type NodePaletteItem } from '@/nodes/registry';
import { useWsClient } from '@/hooks/useWsClient';
import { buildWsUrl } from '@/utils/wsUrl';

type IOmode = 0 | 1 | 'n' | string;
function isDynInputsNode(n: Node) {
  const m: IOmode | undefined = (n.data as any)?.inputsMode;
  return m === 'n';
}
function isDynOutputsNode(n: Node) {
  const m: IOmode | undefined = (n.data as any)?.outputsMode;
  return m === 'n';
}
function normalizeIOMode(mode: string | undefined): IOmode {
    if (mode === '0') return 0;
    if (mode === '1') return 1;
    if (mode === 'n') return 'n';
    return 1;
}
function normalizeNodeIOCounts(node: Node, edges: Edge[]): Node {
  let updatedNode = node;
  const data: any = updatedNode.data ?? {};

  const inputsMode = normalizeIOMode(data.inputsMode);
  if (inputsMode === 'n') {
    const currentInCount: number = Number.isFinite(data.inputsCount) ? Math.max(data.inputsCount, 1) : 1;
    const usedInHandles = new Set(
      edges
        .filter((e) => e.target === updatedNode.id && e.targetHandle && String(e.targetHandle).startsWith('in_'))
        .map((e) => String(e.targetHandle))
    );
    const shouldInCount = Math.max(usedInHandles.size + 1, 1);
    if (shouldInCount !== currentInCount) {
      updatedNode = {
          ...updatedNode,
          data: { ...data, inputsMode: 'n', inputsCount: shouldInCount }
      };
    }
  }

  const outputsMode = normalizeIOMode(data.outputsMode);
  if (outputsMode === 'n') {
    const currentData = updatedNode.data ?? {};
    const currentOutCount: number = Number.isFinite(currentData.outputsCount) ? Math.max(currentData.outputsCount, 1) : 1;
    const usedOutHandles = new Set(
      edges
        .filter((e) => e.source === updatedNode.id && e.sourceHandle && String(e.sourceHandle).startsWith('out_'))
        .map((e) => String(e.sourceHandle))
    );
    const shouldOutCount = Math.max(usedOutHandles.size + 1, 1);
    if (shouldOutCount !== currentOutCount) {
      updatedNode = {
          ...updatedNode,
          data: { ...(updatedNode.data ?? {}), outputsMode: 'n', outputsCount: shouldOutCount }
      };
    }
  }

  return updatedNode;
}

function normalizeAllNodesIO(nodes: Node[], edges: Edge[]): Node[] {
    return nodes.map(n => normalizeNodeIOCounts(n, edges));
}

const initialNodes: Node[] = [];
const initialEdges: Edge[] = [];

type PendingConnect = {
  fromNodeId: string | null;
  fromHandleId: string | null;
  pos: { x: number; y: number };
};

export default function FlowInner() {
  const [nodes, setNodes] = useState<Node[]>(initialNodes);
  const [edges, setEdges] = useState<Edge[]>(initialEdges);
  const [panelPos, setPanelPos] = useState<{ x: number; y: number } | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [pendingConnect, setPendingConnect] = useState<PendingConnect | null>(null);
  const [nodePalette, setNodePalette] = useState<NodePaletteItem[]>([]);
  const { screenToFlowPosition } = useReactFlow();
  const updateNodeInternals = useUpdateNodeInternals();
  const [selectedNodes, setSelectedNodes] = useState<string[]>([]);
  const [selectedEdges, setSelectedEdges] = useState<string[]>([]);
  const connectingNodeId = useRef<string | null>(null);
  const connectingHandleId = useRef<string | null>(null);

  const WS_URL = buildWsUrl();
  const client = useWsClient(WS_URL, {
    autoreconnect: true,
    heartbeatMs: 25000,
    debug: true,
  });

  useEffect(() => {
    if (client.lastJson) {
      const message = client.lastJson as any;
      if (message?.type === 'NODE_CONFIG' && Array.isArray(message.payload)) {
        console.log('Recebido NODE_CONFIG do backend:', message.payload);
        const validPaletteItems = message.payload.filter(
          (item: any): item is NodePaletteItem =>
            typeof item.type === 'string' && typeof item.label === 'string'
        );
         validPaletteItems.sort((a, b) => a.label.localeCompare(b.label));
        setNodePalette(validPaletteItems);
      }
    }
  }, [client.lastJson]);

  const dynamicNodeTypes: NodeTypes = useMemo(() => {
    const types: NodeTypes = {};
    nodePalette.forEach(item => {
      types[item.type] = BaseIONode;
    });
    types['default'] = BaseIONode;
    return types;
  }, [nodePalette]);

  useOnSelectionChange({
    onChange: ({ nodes, edges }) => {
      setSelectedNodes(nodes.map((n: any) => n.id));
      setSelectedEdges(edges.map((e: any) => e.id));
    },
  });

  const idRef = useRef(1);
  const nextId = () => `n${idRef.current++}`;

  const prevIOCountsRef = useRef<Record<string, { inputs?: number; outputs?: number }>>({});
  useEffect(() => {
    const toUpdate: string[] = [];
    nodes.forEach(n => {
        const data: any = n.data ?? {};
        const inputsMode = normalizeIOMode(data.inputsMode);
        const outputsMode = normalizeIOMode(data.outputsMode);
        let changed = false;

        const prevCounts = prevIOCountsRef.current[n.id] ?? {};

        if (inputsMode === 'n') {
            const count = data.inputsCount ?? 1;
            if (prevCounts.inputs === undefined || prevCounts.inputs !== count) {
                prevIOCountsRef.current[n.id] = { ...prevCounts, inputs: count };
                changed = true;
            }
        }
         if (outputsMode === 'n') {
            const count = data.outputsCount ?? 1;
             if (prevCounts.outputs === undefined || prevCounts.outputs !== count) {
                prevIOCountsRef.current[n.id] = { ...prevCounts, outputs: count };
                changed = true;
            }
        }

        if (changed && !toUpdate.includes(n.id)) {
            toUpdate.push(n.id);
        }
    });
    if (toUpdate.length > 0) {
        updateNodeInternals(toUpdate);
    }
  }, [nodes, updateNodeInternals]);

  const onNodesChange = useCallback(
    (changes) => setNodes((nds) => applyNodeChanges(changes, nds)),
    [setNodes]
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    [setEdges]
  );

  // CORREÇÃO: Removi o comentário placeholder inválido
  const onConnect = useCallback((connection: Connection) => {
      connectingNodeId.current = null;
      connectingHandleId.current = null;
      setEdges((eds) => addEdge(connection, eds));
    }, [setEdges]); // Mantém a dependência correta

  useEffect(() => {
    setNodes((nds) => normalizeAllNodesIO(nds, edges));
  }, [edges, setNodes]);

  const onConnectStart = useCallback((_: any, { nodeId, handleId }: { nodeId: string | null, handleId: string | null }) => {
    connectingNodeId.current = nodeId;
    connectingHandleId.current = handleId;
  }, []);

  const onConnectEnd = useCallback(
    (event: MouseEvent | TouchEvent) => {
      const targetIsPane = (event.target as Element).classList.contains('react-flow__pane');

      if (targetIsPane && connectingNodeId.current && connectingHandleId.current) {
        const isTouch = 'changedTouches' in event && event.changedTouches && event.changedTouches.length > 0;
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
    },
    [screenToFlowPosition]
  );

  const addNodeByType = useCallback((typeKey: string) => {
    const spec = nodePalette.find((n) => n.type === typeKey);

    if (!spec) {
       console.error(`Erro: Tentativa de adicionar node tipo '${typeKey}' que não foi encontrado na palette recebida.`);
       setIsModalOpen(false);
       setPendingConnect(null);
       return;
    }

    const id = nextId();
    const baseData: any = JSON.parse(JSON.stringify(spec.default_data ?? {}));

    if (!baseData.label) {
      baseData.label = spec.label;
    }

    baseData.inputsMode = normalizeIOMode(baseData.inputsMode);
    baseData.outputsMode = normalizeIOMode(baseData.outputsMode);
    if (baseData.inputsMode === 'n') baseData.inputsCount = Math.max(baseData.inputsCount ?? 1, 1);
    if (baseData.outputsMode === 'n') baseData.outputsCount = Math.max(baseData.outputsCount ?? 1, 1);

    let newNodePosition = { x: 0, y: 0 };
    const newNode: Node = {
        id, type: spec.type, className: 'hacker-node',
        position: { x: 0, y: 0 }, data: baseData,
    };

    if (pendingConnect) {
      newNodePosition = pendingConnect.pos;
      newNode.position = newNodePosition;
      const { fromNodeId, fromHandleId } = pendingConnect;

      setNodes((nds) => [...nds, newNode]);

      if (fromNodeId && fromHandleId) {
        const targetHandleId = 'in_0';
        setEdges((eds) => {
          const newEdge: Edge = {
            id: `e-${fromNodeId}-${id}-${Date.now()}`, source: fromNodeId, target: id,
            sourceHandle: fromHandleId, targetHandle: targetHandleId,
          };
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
  }, [nodePalette, pendingConnect, screenToFlowPosition, edges, setNodes, setEdges]);

  const del = useKeyPress('Delete');
  const bsp = useKeyPress('Backspace');
  useEffect(() => {
    const pressed = del || bsp;
    if (!pressed || (selectedNodes.length === 0 && selectedEdges.length === 0)) return;

    const selectedNodeIds = new Set(selectedNodes);
    const selectedEdgeIds = new Set(selectedEdges);

    setEdges((eds) => eds.filter(e =>
        !selectedEdgeIds.has(e.id) &&
        !selectedNodeIds.has(e.source) &&
        !selectedNodeIds.has(e.target)
    ));
    setNodes((nds) => nds.filter((n) => !selectedNodeIds.has(n.id)));

    setSelectedNodes([]);
    setSelectedEdges([]);
  }, [del, bsp, selectedNodes, selectedEdges, setNodes, setEdges]);

  const PANEL_OFFSET_X = 280, PANEL_OFFSET_Y = 100;
  const onPaneClick = usePaneClickCombo({
    onSingle: () => setPanelPos(null),
    onDouble: (e: MouseEvent) => {
      setPanelPos({ x: e.clientX - PANEL_OFFSET_X, y: e.clientY - PANEL_OFFSET_Y });
    },
  });

  const handleCloseModal = useCallback(() => {
    connectingNodeId.current = null;
    connectingHandleId.current = null;
    setPendingConnect(null);
    setIsModalOpen(false);
  }, []);

  return (
    <>
      <div className="globalWrapper">
        <LeftPanel onOpenModal={() => setIsModalOpen(true)} />
        <div className="mainBoard">
          <ReactFlow
            colorMode="dark"
            nodes={nodes}
            edges={edges}
            nodeTypes={dynamicNodeTypes}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onConnectStart={onConnectStart}
            onConnectEnd={onConnectEnd}
            onPaneClick={onPaneClick}
            fitView
            onInit={(instance) => ((window as any).reactFlowInstance = instance)}
          >
            {panelPos && (
              <Panel style={{ left: panelPos.x, top: panelPos.y, position: 'absolute' }}>
                <div className="hacker-panel">
                  <p>⚡ Painel Hacker</p>
                  <button onClick={() => setPanelPos(null)}>Fechar</button>
                </div>
              </Panel>
            )}
          </ReactFlow>
        </div>
      </div>
      <HackerModal open={isModalOpen} onClose={handleCloseModal}>
        <NodeCatalog onPick={addNodeByType} nodePalette={nodePalette} />
      </HackerModal>
    </>
  );
}