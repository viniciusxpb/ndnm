// src/hooks/useWorkspacePersistence.ts
import { useCallback } from 'react';
import { type Node, type Edge } from '@xyflow/react';

export function useWorkspacePersistence() {
  const saveWorkspace = useCallback(async (name: string, nodes: Node[], edges: Edge[]) => {
    console.log(`[Workspace] Salvando '${name}'...`);
    try {
      const response = await fetch('http://localhost:3100/workspace/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name,
          nodes: nodes.map(n => ({
            id: n.id,
            type: n.type,
            position: n.position,
            data: n.data,
          })),
          edges: edges.map(e => ({
            id: e.id,
            source: e.source,
            target: e.target,
            sourceHandle: e.sourceHandle,
            targetHandle: e.targetHandle,
          })),
        }),
      });

      if (!response.ok) throw new Error('Falha ao salvar');
      console.log(`[Workspace] '${name}' salvo!`);
      return true;
    } catch (e) {
      console.error(`[Workspace] Erro ao salvar:`, e);
      return false;
    }
  }, []);

  const loadWorkspace = useCallback(async (name: string) => {
    console.log(`[Workspace] Carregando '${name}'...`);
    try {
      const response = await fetch(`http://localhost:3100/workspace/load/${name}`);
      if (!response.ok) throw new Error('Workspace não encontrado');

      const data = await response.json();
      console.log(`[Workspace] '${name}' carregado!`);
      return data;
    } catch (e) {
      console.error(`[Workspace] Erro ao carregar:`, e);
      return null;
    }
  }, []);

  return { saveWorkspace, loadWorkspace };
}