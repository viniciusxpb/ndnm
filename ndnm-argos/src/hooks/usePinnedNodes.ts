// src/hooks/usePinnedNodes.ts
import { useState, useCallback } from 'react';

export interface PinnedNodeData {
  nodeId: string;
  label: string;
  type: string;
}

export function usePinnedNodes() {
  const [pinnedNodes, setPinnedNodes] = useState<PinnedNodeData[]>([]);

  const pinNode = useCallback((nodeId: string, label: string, type: string) => {
    setPinnedNodes(prev => {
      const alreadyPinned = prev.some(p => p.nodeId === nodeId);
      if (alreadyPinned) return prev;

      console.log('📌 Pinando node:', nodeId, label);
      return [...prev, { nodeId, label, type }];
    });
  }, []);

  const unpinNode = useCallback((nodeId: string) => {
    console.log('📍 Despinando node:', nodeId);
    setPinnedNodes(prev => prev.filter(p => p.nodeId !== nodeId));
  }, []);

  const isPinned = useCallback((nodeId: string) => {
    return pinnedNodes.some(p => p.nodeId === nodeId);
  }, [pinnedNodes]);

  return {
    pinnedNodes,
    pinNode,
    unpinNode,
    isPinned,
  };
}
