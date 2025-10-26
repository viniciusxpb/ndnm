// src/flow/FlowCanvas.tsx
import React, { useMemo, useCallback, useEffect } from 'react';
import {
    ReactFlow,
    Panel,
    type NodeTypes,
    type Node,
    type Edge,
    type Connection,
    type NodeChange,
    type EdgeChange,
    type OnConnectStart,
    type OnConnectEnd,
    useReactFlow
} from '@xyflow/react';
import { BaseIONode } from '@/nodes/BaseIONode';
import { FsBrowserNode } from '@/nodes/FsBrowserNode';
import { PlayNode } from '@/nodes/PlayNode';
import { type NodePaletteItem } from '@/nodes/registry';

type FlowCanvasProps = {
    nodes: Node[];
    edges: Edge[];
    nodePalette: NodePaletteItem[];
    onNodesChange: (changes: NodeChange[]) => void;
    onEdgesChange: (changes: EdgeChange[]) => void;
    onConnect: (connection: Connection) => void;
    onConnectStart: OnConnectStart;
    onConnectEnd: OnConnectEnd;
    onPaneClick: (event: React.MouseEvent) => void;
    panelPos: { x: number; y: number } | null;
    setPanelPos: React.Dispatch<React.SetStateAction<{ x: number; y: number } | null>>;
};

// *** CLASSE CSS PARA IGNORAR O SCROLL/ZOOM DO FLOW ***
const NO_WHEEL_CLASS = 'prevent-flow-scroll'; // <<<<==== DEFINIDO AQUI TAMBÉM

export function FlowCanvas({
    nodes, edges, nodePalette, onNodesChange, onEdgesChange,
    onConnect, onConnectStart, onConnectEnd, onPaneClick,
    panelPos, setPanelPos
}: FlowCanvasProps) {

    const { deleteElements } = useReactFlow();

    const dynamicNodeTypes: NodeTypes = useMemo(() => {
        const types: NodeTypes = {};
        nodePalette.forEach(item => {
            if (item.type === 'fsBrowser') {
                types[item.type] = FsBrowserNode;
            } else if (item.type === 'playButton' || item.type === 'comfyPlay') {
                types[item.type] = PlayNode;
            } else {
                types[item.type] = BaseIONode;
            }
        });
        types['default'] = BaseIONode;
        return types;
    }, [nodePalette]);

    const handleKeyDown = useCallback((event: KeyboardEvent) => {
        if (event.key === 'Delete' || event.key === 'Backspace') {
            const selectedNodes = nodes.filter(node => node.selected);
            const selectedEdges = edges.filter(edge => edge.selected);

            if (selectedNodes.length > 0 || selectedEdges.length > 0) {
                console.log('🗑️ Deletando elementos selecionados:', {
                    nodes: selectedNodes.length,
                    edges: selectedEdges.length
                });
                deleteElements({ nodes: selectedNodes, edges: selectedEdges });
            }
        }
    }, [nodes, edges, deleteElements]);

    useEffect(() => {
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [handleKeyDown]);

    return (
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
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                onInit={(instance: any) => ((window as any).reactFlowInstance = instance)}
                // *** PROP ADICIONADA AQUI ***
                noWheelClassName={NO_WHEEL_CLASS} // <<<<==== PASSANDO A CLASSE PARA O REACT FLOW
            >
                {panelPos && (
                    <Panel style={{ left: panelPos.x, top: panelPos.y, position: 'absolute' }}>
                        <div className="hacker-panel">
                            <p>⚡ Painel Hacker</p>
                            <button className="hacker-btn ghost" onClick={() => setPanelPos(null)}>Fechar</button>
                        </div>
                    </Panel>
                )}
            </ReactFlow>
        </div>
    );
}