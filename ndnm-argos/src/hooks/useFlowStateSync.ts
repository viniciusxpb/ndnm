import { useState, useEffect, useCallback, useRef } from 'react';
import {
  applyNodeChanges, applyEdgeChanges, addEdge,
  useUpdateNodeInternals,
  type NodeChange, type EdgeChange, type Connection,
  type Node, type Edge
} from '@xyflow/react';
import { normalizeAllNodesIO, normalizeIOMode } from '@/utils/flowUtils';

const initialNodes: Node[] = [];
const initialEdges: Edge[] = [];

export function useFlowStateSync() {
  const [nodes, setNodes] = useState<Node[]>(initialNodes);
  const [edges, setEdges] = useState<Edge[]>(initialEdges);
  const updateNodeInternals = useUpdateNodeInternals();

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => setNodes((nds) => applyNodeChanges(changes, nds)),
    [setNodes]
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    [setEdges]
  );

  const onConnect = useCallback(
    (connection: Connection) => {
      setEdges((eds) => addEdge(connection, eds));
    },
    [setEdges]
  );

  useEffect(() => {
    setNodes((nds) => normalizeAllNodesIO(nds, edges));
  }, [edges, setNodes]);

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
        if (changed && !toUpdate.includes(n.id)) { toUpdate.push(n.id); }
    });
    if (toUpdate.length > 0) { updateNodeInternals(toUpdate); }
  }, [nodes, updateNodeInternals]);

  return { nodes, edges, setNodes, setEdges, onNodesChange, onEdgesChange, onConnect };
}