// src/components/PinnedPlayButtons.tsx
import type { Node } from '@xyflow/react';

interface PinnedPlayButtonsProps {
  pinnedNodes: Node[];
  onUnpin: (nodeId: string) => void;
  onExecute?: (nodeId: string) => void;
  isExecuting?: boolean;
}

export function PinnedPlayButtons({ pinnedNodes, onUnpin, onExecute, isExecuting }: PinnedPlayButtonsProps) {
  if (pinnedNodes.length === 0) return null;

  const handleExecute = (nodeId: string) => {
    console.log('▶️ Executando workflow a partir do pinned node:', nodeId);
    onExecute?.(nodeId);
  };

  return (
    <div className="pinned-play-container">
      <div className="pinned-play-header">
        <span>📌 Pinned Nodes</span>
      </div>
      <div className="pinned-play-nodes">
        {pinnedNodes.map(node => (
          <div key={node.id} className="pinned-node-wrapper">
            <button
              onClick={() => onUnpin(node.id)}
              className="pinned-close-btn nodrag"
              title="Desancorar"
            >
              ✕
            </button>
            <div className="pinned-node-mini">
              <div className="pinned-node-label">
                {(node.data as any)?.label || node.type}
              </div>
              <button
                onClick={() => handleExecute(node.id)}
                disabled={isExecuting}
                className="nodrag"
                style={{
                  marginTop: 8,
                  padding: '4px 8px',
                  width: '100%',
                  background: isExecuting
                    ? 'rgba(255, 255, 0, 0.3)'
                    : 'linear-gradient(135deg, #00ff88 0%, #00cc6a 100%)',
                  border: '1px solid #00ff88',
                  borderRadius: '3px',
                  color: isExecuting ? '#ffff00' : '#000',
                  fontWeight: 'bold',
                  fontSize: 11,
                  cursor: isExecuting ? 'wait' : 'pointer',
                  boxShadow: isExecuting
                    ? '0 0 8px rgba(255, 255, 0, 0.5)'
                    : '0 2px 6px rgba(0, 255, 136, 0.3)',
                  transition: 'all 0.2s ease',
                }}
              >
                {isExecuting ? '⏳' : '▶️'}
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
