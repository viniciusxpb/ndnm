// src/hooks/useNodePalette.ts
import { useEffect, useState } from 'react';
import { useWsClient } from './useWsClient';
import { buildWsUrl } from '@/utils/wsUrl';
import { logger } from '../utils/logger';

export interface NodePaletteItem {
  type: string;
  label: string;
  default_data: {
    inputsMode: 0 | 1 | 'n';
    outputsMode: 0 | 1 | 'n';
    input_fields: Array<{
      type: 'text' | 'selector';
      name: string;
      label: string;
      default?: string;
    }>;
    value: any;
  };
}

export function useNodePalette(initialConfig?: any): NodePaletteItem[] {
  const [nodePalette, setNodePalette] = useState<NodePaletteItem[]>(initialConfig ? [mapConfigToNodePalette(initialConfig)] : []);
  const WS_URL = buildWsUrl();
  const client = useWsClient(WS_URL, {
    autoreconnect: true,
    heartbeatMs: 25000,
    debug: true,
  });

  // Função para mapear a configuração do node para o formato do palette
  function mapConfigToNodePalette(config: any): NodePaletteItem {
    try {
      return {
        type: config.node_type || 'filesystem',
        label: config.label || '📂 Gerenciador de Arquivos',
        default_data: {
          inputsMode: 'n',
          outputsMode: 'n',
          input_fields: config.input_fields?.map((field: any) => ({
            type: field.type || 'text',
            name: field.name,
            label: field.label,
            default: field.default,
          })) || [],
          value: {},
        },
      };
    } catch (error) {
      logger.error('Erro ao mapear configuração do node para palette', error);
      return {
        type: 'filesystem',
        label: '📂 Gerenciador de Arquivos',
        default_data: {
          inputsMode: 'n',
          outputsMode: 'n',
          input_fields: [],
          value: {},
        },
      };
    }
  }

  // Receber palette via WebSocket (NODE_CONFIG)
  useEffect(() => {
    if (client.lastJson) {
      const message = client.lastJson as any;
      if (message?.type === 'NODE_CONFIG' && Array.isArray(message.payload)) {
        logger.info('[useNodePalette] Recebido NODE_CONFIG:', message.payload);
        const validPaletteItems = message.payload.filter(
          (item: any): item is NodePaletteItem =>
            typeof item.type === 'string' && typeof item.label === 'string'
        );
        validPaletteItems.sort((a, b) => a.label.localeCompare(b.label));
        setNodePalette(validPaletteItems);
      }
    }
  }, [client.lastJson]);

  return nodePalette;
}