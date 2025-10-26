// src/nodes/PlayNode.tsx
import type { NodeProps } from '@xyflow/react';
import { BaseIONode, type BaseNodeData } from '@/nodes/BaseIONode';

type PlayData = BaseNodeData & {
  onPin?: (nodeId: string, label: string) => void;
  onUnpin?: (nodeId: string) => void;
  onExecute?: (nodeId: string) => void;
  isPinned?: boolean;
  isExecuting?: boolean;
};

export function PlayNode(props: NodeProps<PlayData>) {
  const { id, data } = props;

  const handleTogglePin = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (data.isPinned) {
      data.onUnpin?.(id);
    } else {
      data.onPin?.(id, data.label || 'Play');
    }
  };

  const handleExecute = (e: React.MouseEvent) => {
    e.stopPropagation();
    console.log('▶️ Play button clicado para node:', id);
    data.onExecute?.(id);
  };

  return (
    <div style={{ position: 'relative' }}>
      <button
        onClick={handleTogglePin}
        className="nodrag pin-button"
        style={{
          position: 'absolute',
          top: -8,
          right: -8,
          width: 20,
          height: 20,
          padding: 0,
          background: data.isPinned ? 'rgba(0, 255, 136, 0.9)' : 'rgba(0, 0, 0, 0.7)',
          border: data.isPinned ? '2px solid #00ff88' : '1px solid rgba(0, 255, 136, 0.5)',
          color: data.isPinned ? '#000' : '#c8ffdf',
          borderRadius: '4px',
          cursor: 'pointer',
          fontSize: 10,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          boxShadow: data.isPinned ? '0 0 12px rgba(0, 255, 136, 0.8)' : '0 2px 4px rgba(0, 0, 0, 0.3)',
          transition: 'all 0.2s ease',
          zIndex: 10,
        }}
        title={data.isPinned ? 'Despinar da UI' : 'Pinar na UI'}
      >
        {data.isPinned ? '📌' : '📍'}
      </button>
      <BaseIONode
        {...props}
        data={{
          label: data.label,
          value: data.value ?? '',
          inputsMode: data.inputsMode ?? 'n',
          inputsCount: data.inputsCount ?? 1,
          outputsMode: data.outputsMode ?? 1,
          outputsCount: data.outputsCount ?? 1,
          onChange: data.onChange,
        }}
      >
        <div style={{ marginTop: 8, display: 'flex', flexDirection: 'column', gap: 4 }}>
          <button
            onClick={handleExecute}
            disabled={data.isExecuting}
            className="nodrag"
            style={{
              padding: '6px 12px',
              background: data.isExecuting
                ? 'rgba(255, 255, 0, 0.3)'
                : 'linear-gradient(135deg, #00ff88 0%, #00cc6a 100%)',
              border: '1px solid #00ff88',
              borderRadius: '4px',
              color: data.isExecuting ? '#ffff00' : '#000',
              fontWeight: 'bold',
              fontSize: 12,
              cursor: data.isExecuting ? 'wait' : 'pointer',
              boxShadow: data.isExecuting
                ? '0 0 10px rgba(255, 255, 0, 0.5)'
                : '0 2px 8px rgba(0, 255, 136, 0.3)',
              transition: 'all 0.2s ease',
            }}
          >
            {data.isExecuting ? '⏳ Executando...' : '▶️ Executar'}
          </button>
          <div style={{ opacity: 0.6, fontSize: 10, color: '#888', textAlign: 'center' }}>
            Entradas: {data.inputsCount ?? 1}
          </div>
        </div>
      </BaseIONode>
    </div>
  );
}