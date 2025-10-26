import { useState, useEffect } from 'react';
import { useWsClient } from '@/hooks/useWsClient';
import { buildWsUrl } from '@/utils/wsUrl';
import { type NodePaletteItem } from '@/nodes/registry';

export function useNodePalette(): NodePaletteItem[] {
  const [nodePalette, setNodePalette] = useState<NodePaletteItem[]>([]);
  const WS_URL = buildWsUrl();
  const client = useWsClient(WS_URL, {
    autoreconnect: true,
    heartbeatMs: 0,
    debug: false,
  });

  useEffect(() => {
    if (client.lastJson) {
      const message = client.lastJson as any;
      if (message?.type === 'NODE_CONFIG' && Array.isArray(message.payload)) {
        console.log('[useNodePalette] Recebido NODE_CONFIG:', message.payload);
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