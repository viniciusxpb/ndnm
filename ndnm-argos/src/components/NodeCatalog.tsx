// src/components/NodeCatalog.tsx
import { useMemo, useState } from 'react';
import { type NodePaletteItem } from '@/nodes/registry';

type Props = {
  onPick: (type: string) => void;
  nodePalette: NodePaletteItem[];
};

export default function NodeCatalog({ onPick, nodePalette }: Props) {
  const [q, setQ] = useState('');
  const filtered = useMemo(() => {
    const s = q.trim().toLowerCase();
    if (!s) return nodePalette;
    return nodePalette.filter(n => n.label.toLowerCase().includes(s));
  }, [q, nodePalette]);

  return (
    <div>
      <input
        className="nodrag"
        placeholder="Buscar node..."
        value={q}
        onChange={(e) => setQ(e.target.value)}
        style={{ width: '100%', marginBottom: 12 }}
      />

      <div className="node-palette">
        {filtered.map((item) => (
          <button
            key={item.type}
            className="hacker-btn"
            style={{ width: '100%' }}
            onClick={() => onPick(item.type)}
          >
            {item.label}
          </button>
        ))}
        {nodePalette.length > 0 && filtered.length === 0 && (
          <div style={{ opacity: .7, fontSize: 12 }}>Nada encontrado…</div>
        )}
        {nodePalette.length === 0 && (
           <div style={{ opacity: .7, fontSize: 12 }}>Carregando nodes do backend...</div>
        )}
      </div>
    </div>
  );
}