// src/components/LeftPanel.tsx
import { useState } from 'react';
import { useWorkspacePersistence } from '@/hooks/useWorkspacePersistence';
import type { Node, Edge } from '@xyflow/react';

type Props = {
  onOpenModal: () => void;
  nodes: Node[];
  edges: Edge[];
  onLoadWorkspace: (nodes: Node[], edges: Edge[]) => void;
  onReassignNodeData?: (nodes: Node[]) => Node[];
};

export default function LeftPanel({ onOpenModal, nodes, edges, onLoadWorkspace, onReassignNodeData }: Props) {
  const [workspaceName, setWorkspaceName] = useState('workspace-1');
  const { saveWorkspace, loadWorkspace } = useWorkspacePersistence();

  const handleSaveWorkspace = async () => {
    const success = await saveWorkspace(workspaceName, nodes, edges);
    if (success) {
      alert(`✅ Workspace "${workspaceName}" salvo!`);
    } else {
      alert(`❌ Erro ao salvar workspace`);
    }
  };

  const handleLoadWorkspace = async () => {
    console.log('🔄 Iniciando carregamento do workspace...');
    console.log('📁 Nome do workspace:', workspaceName);

    const data = await loadWorkspace(workspaceName);
    console.log('📦 Dados retornados do loadWorkspace:', data);

    if (data && data.nodes && data.edges) {
      console.log('✅ Dados válidos encontrados, carregando...');

      // AQUI ESTÁ A SOLUÇÃO: Reassignar as funções perdidas
      let nodesToLoad = data.nodes;
      if (onReassignNodeData) {
        nodesToLoad = onReassignNodeData(data.nodes);
        console.log('🔄 Nós modificados com onReassignNodeData:', nodesToLoad);
      }

      console.log('📊 Número de nodes:', nodesToLoad.length);
      console.log('📊 Número de edges:', data.edges.length);
      onLoadWorkspace(nodesToLoad, data.edges);
      alert(`✅ Workspace "${workspaceName}" carregado!`);
    } else {
      console.log('❌ Dados inválidos ou não encontrados');
      alert(`❌ Workspace não encontrado`);
    }
  };


  return (
    <aside className="leftPanel">
      <div style={{ marginBottom: 10, opacity: 0.9 }}>⚙️ Ações</div>
      <button className="hacker-btn" onClick={onOpenModal}>
        Abrir painel
      </button>

      <div style={{ marginTop: 16, paddingTop: 16, borderTop: '1px solid rgba(0, 255, 136, 0.2)' }}>
        <div style={{ marginBottom: 8, fontSize: 12, opacity: 0.8 }}>📝 Workspace</div>
        <input
          type="text"
          value={workspaceName}
          onChange={(e) => setWorkspaceName(e.target.value)}
          placeholder="Nome do workspace..."
          className="nodrag"
          style={{
            display: 'block',
            width: '100%',
            padding: '6px',
            marginBottom: 8,
            background: 'rgba(0, 0, 0, 0.3)',
            border: '1px solid rgba(0, 255, 136, 0.3)',
            color: '#b7f397',
            borderRadius: 4,
            fontFamily: 'monospace',
            fontSize: 12,
            boxSizing: 'border-box',
          }}
        />
        <button
          onClick={handleSaveWorkspace}
          className="hacker-btn"
          style={{
            display: 'block',
            width: '100%',
            marginBottom: 6,
            background: 'rgba(0, 150, 0, 0.3)',
            borderColor: '#00ff99',
            color: '#00ff99',
          }}
        >
          💾 Salvar
        </button>
        <button
          onClick={handleLoadWorkspace}
          className="hacker-btn"
          style={{
            display: 'block',
            width: '100%',
            background: 'rgba(0, 100, 150, 0.3)',
            borderColor: '#00ccff',
            color: '#00ccff',
          }}
        >
          📂 Carregar
        </button>
      </div>
    </aside>
  );
}