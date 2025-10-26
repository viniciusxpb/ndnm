// viniciusxpb/frontend/frontend-e2ff74bfb8b04c2182486c99304ae2e139575034/src/nodes/BaseIONode.tsx
import { useMemo } from 'react';
import type { NodeProps } from '@xyflow/react';
import { Handle, Position, NodeToolbar, useConnection } from '@xyflow/react';
import { SelectorControl } from '@/components/SelectorControl'; // IMPORTADO

type IOmode = 0 | 1 | 'n';

interface InputField {
  name: string;
  type: string;
}

export interface BaseNodeData {
  label?: string;
  value?: string;
  inputsMode?: IOmode;
  outputsMode?: IOmode;
  inputsCount?: number;
  outputsCount?: number;
  onChange?: (id: string, val: string) => void;
  toolbarPosition?: Position;
  input_fields?: InputField[];
}

type BaseProps = NodeProps<BaseNodeData> & { children?: React.ReactNode };

export function BaseIONode({ id, data, children }: BaseProps) {
  // Logs de renderização removidos para focar no fluxo de WS
  const connection = useConnection();
  const isTargetOrigin = connection.inProgress && connection.fromNode?.id === id;

  const inCount = 1;
  const outCount = 1;
  const inOffsetY = 0;
  const outOffsetY = 0; // FIX LENDÁRIO: Variável reintroduzida com 'const'

  const renderableField = useMemo(() => {
    console.log(`🔍 [BaseIONode ${id}] Checking input_fields:`, {
      hasInputFields: !!data.input_fields,
      isArray: Array.isArray(data.input_fields),
      input_fields: data.input_fields,
      dataKeys: Object.keys(data)
    });

    if (!data.input_fields || !Array.isArray(data.input_fields)) {
      return undefined;
    }

    const field = data.input_fields.find((f) => f.type === 'text' || f.type === 'selector');
    console.log(`🎯 [BaseIONode ${id}] Renderable field:`, field);
    return field;
  }, [data.input_fields, id, data]);

  const showValueInput = !!renderableField;
  
  // Função para renderizar o controle de input
  const renderInputControl = (field: InputField) => {
    const isSelector = field.type === 'selector';

    if (isSelector) {
      // Usa o SelectorControl para campos 'selector'
      // Deduzimos que é seletor de pasta se o nome tiver 'path' ou 'directory'
      const isFolderSelector = field.name.toLowerCase().includes('path') || field.name.toLowerCase().includes('directory');
      
      // Assumimos que data.onChange existe, pois é garantido pelo FlowController
      return (
        <SelectorControl
          id={id}
          name={field.name}
          value={data.value ?? ''}
          onChange={data.onChange!} 
          isFolderSelector={isFolderSelector}
        />
      );
    }

    // Input type="text" padrão (para fixedValue, etc.)
    return (
      <input
        id={`${id}-${field.name}`}
        type="text"
        value={data.value ?? ''}
        className="nodrag"
        onChange={(e) => data.onChange?.(id, e.target.value)}
        placeholder={`Enter ${field.name}...`}
        style={{
          display: 'block',
          width: 'calc(100% - 10px)',
          padding: '4px',
          fontFamily: 'monospace',
          // Estilo básico para o input de texto normal (que o CSS não cobre)
          background: 'rgba(0, 0, 0, 0.3)',
          border: '1px solid rgba(0, 255, 136, 0.3)',
          color: '#c8ffdf',
          borderRadius: '4px',
          boxSizing: 'border-box',
        }}
      />
    );
  };

  return (
    <>
      <NodeToolbar isVisible={isTargetOrigin} position={data.toolbarPosition ?? Position.Top}>
        <button className="xy-theme__button">cut</button>
        <button className="xy-theme__button">copy</button>
        <button className="xy-theme__button">
          <img src="/src/assets/icons/wire_cutter.svg" alt="Wire Cutter" style={{ width: 16, height: 16 }} />
        </button>
      </NodeToolbar>
      <div className="hacker-node base-io">
        <strong>{data.label ?? 'Node'}</strong>
        {showValueInput && renderableField ? (
          <div style={{ marginTop: '4px' }}>
            {renderInputControl(renderableField)}
          </div>
        ) : (
          <div style={{ marginTop: '4px', fontSize: '11px', opacity: 0.5, color: '#888' }}>
            (sem campos de input definidos)
          </div>
        )}
        {children}

        {/* Handles */}
        {Array.from({ length: inCount }).map((_, i) => (
          <Handle
            key={`in_${i}`}
            id={`in_${i}`}
            type="target"
            position={Position.Left}
            style={{ transform: `translateY(${i * inOffsetY}px)` }}
          />
        ))}
        {Array.from({ length: outCount }).map((_, i) => (
          <Handle
            key={`out_${i}`}
            id={`out_${i}`}
            type="source"
            position={Position.Right}
            style={{ transform: `translateY(${i * outOffsetY}px)` }}
          />
        ))}
      </div>
    </>
  );
}