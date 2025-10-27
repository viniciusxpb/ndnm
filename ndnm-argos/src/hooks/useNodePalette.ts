// src/hooks/useNodePalette.ts
import { useEffect, useState } from 'react';
import { useWsClient } from './useWsClient';
import { buildWsUrl } from '@/utils/wsUrl';
import { apiService } from '../utils/api';
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
  const [loading, setLoading] = useState<boolean>(false);
  const WS_URL = buildWsUrl();
  const client = useWsClient(WS_URL, {
    autoreconnect: true,
    heartbeatMs: 0,
    debug: false,
  });

  // Função para mapear a configuração do node para o formato do palette
  function mapConfigToNodePalette(config: any): NodePaletteItem {
    try {
      // Mapear a configuração do node-file-browser para o formato do palette
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

  // Carregar palette do backend se não tiver configuração inicial
  useEffect(() => {
    if (!initialConfig && nodePalette.length === 0) {
      const fetchNodeRegistry = async () => {
        try {
          setLoading(true);
          const registry = await apiService.getNodeRegistry();
          
          if (registry && registry.nodes && Array.isArray(registry.nodes)) {
            setNodePalette(registry.nodes);
            logger.info('Palette carregado do backend', registry.nodes);
          }
        } catch (error) {
          logger.error('Erro ao carregar palette do backend', error);
        } finally {
          setLoading(false);
        }
      };
      
      fetchNodeRegistry();
    }
  }, [initialConfig, nodePalette.length]);

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