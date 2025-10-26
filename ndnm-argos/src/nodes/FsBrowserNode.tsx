// src/nodes/FsBrowserNode.tsx
import React, { useState, useEffect, useCallback } from 'react';
import type { NodeProps } from '@xyflow/react';
import { Handle, Position } from '@xyflow/react';
import type { BaseNodeData } from './BaseIONode';

// Interface para a resposta do backend node-fs-browser
interface DirectoryEntry {
    name: string;
    is_dir: boolean;
    path: string;
    modified: string;
}

interface FsBrowserApiResponse {
    current_path: string;
    entries: DirectoryEntry[];
}

// Props do nó
type FsBrowserNodeData = BaseNodeData & {
    // Campos específicos futuros aqui
};

// *** CLASSE CSS PARA IGNORAR O SCROLL/ZOOM DO FLOW ***
const NO_WHEEL_CLASS = 'prevent-flow-scroll'; // <<<<==== DEFINIDO AQUI

export function FsBrowserNode({ id, data }: NodeProps<FsBrowserNodeData>) {
    const [entries, setEntries] = useState<DirectoryEntry[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [currentListedPath, setCurrentListedPath] = useState<string | null>(null);
    const [currentPath, setCurrentPath] = useState<string>('C:\\');

    // Função para buscar dados no backend
    const fetchDirectoryContents = useCallback(async (path: string) => {
        if (!path) {
            setEntries([]);
            setCurrentListedPath(null);
            return;
        }
        setIsLoading(true);
        setError(null);
        setCurrentListedPath(path);
        console.log(`[FsBrowserNode:${id}] 📡 Buscando conteúdo para: ${path}`);
        try {
            const response = await fetch('http://localhost:3011/run', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ value: path }),
            });

            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`Erro ${response.status}: ${errorText || response.statusText}`);
            }

            const result: FsBrowserApiResponse = await response.json();
            console.log(`[FsBrowserNode:${id}] ✅ Resposta: ${result.entries?.length || 0} entradas.`);
            setEntries(result.entries.filter(e => e.name !== '..') || []);

        } catch (err) {
            console.error(`[FsBrowserNode:${id}] ❌ Erro:`, err);
            setError(err instanceof Error ? err.message : 'Erro desconhecido');
            setEntries([]);
        } finally {
            setIsLoading(false);
        }
    }, [id]);

    // Busca o conteúdo inicial em C:\ e quando currentPath muda
    useEffect(() => {
        if (currentPath && currentPath !== currentListedPath && !isLoading) {
            fetchDirectoryContents(currentPath);
        }
    }, [currentPath, fetchDirectoryContents, isLoading, currentListedPath]);

    // Handler para voltar uma pasta acima
    const handleGoUp = useCallback(() => {
        const pathParts = currentPath.split('\\').filter(p => p);
        if (pathParts.length > 1) {
            const parentPath = pathParts.slice(0, -1).join('\\') + '\\';
            console.log(`[FsBrowserNode:${id}] ⬆️ Voltando para: ${parentPath}`);
            setCurrentPath(parentPath);
        } else if (pathParts.length === 1) {
            const rootPath = pathParts[0] + '\\';
            console.log(`[FsBrowserNode:${id}] ⬆️ Já está na raiz: ${rootPath}`);
        }
    }, [currentPath, id]);

    // Handler para clique nas entradas da lista
    const handleEntryClick = useCallback((entry: DirectoryEntry) => {
        if (entry.is_dir) {
            console.log(`[FsBrowserNode:${id}] ➡️ Navegando para pasta: ${entry.path}`);
            setCurrentPath(entry.path);
        } else {
            console.log(`[FsBrowserNode:${id}] 📄 Arquivo clicado (sem ação): ${entry.name}`);
        }
    }, [id]);

    // Estilos
    const listStyle: React.CSSProperties = {
        marginTop: '8px',
        border: '1px solid rgba(0, 255, 153, 0.2)', padding: '4px',
        fontSize: '11px', background: 'rgba(0, 0, 0, 0.2)',
    };
    const entryStyleBase: React.CSSProperties = {
        position: 'relative', padding: '3px 16px 3px 6px', display: 'flex',
        alignItems: 'center', justifyContent: 'space-between',
        borderBottom: '1px dotted rgba(0, 255, 153, 0.1)', minHeight: '22px',
        userSelect: 'none',
        transition: 'background-color 0.1s ease',
    };
    const folderStyle: React.CSSProperties = { cursor: 'pointer' };
    const fileStyle: React.CSSProperties = { opacity: 0.8, cursor: 'default' };
    const handleBaseStyle: React.CSSProperties = {
        position: 'absolute', right: '2px', top: '50%',
        transform: 'translateY(-50%)', width: '8px', height: '8px',
        background: '#00ff99', border: '1px solid #005e38',
        borderRadius: '50%', cursor: 'crosshair',
        zIndex: 1,
        pointerEvents: 'auto'
    };

    return (
        <div className="hacker-node" style={{ minWidth: '350px' }}>
            <strong>{data.label ?? 'Navegador de Arquivos'}</strong>

            <div style={{ display: 'flex', gap: '4px', marginTop: '6px', alignItems: 'center' }}>
                <button
                    onClick={handleGoUp}
                    className="nodrag"
                    style={{
                        padding: '4px 8px',
                        background: 'rgba(0, 255, 153, 0.15)',
                        border: '1px solid rgba(0, 255, 153, 0.3)',
                        borderRadius: '2px',
                        color: '#00ff99',
                        cursor: 'pointer',
                        fontSize: '11px',
                        fontWeight: 'bold',
                        transition: 'background 0.15s ease'
                    }}
                    onMouseEnter={(e) => e.currentTarget.style.background = 'rgba(0, 255, 153, 0.25)'}
                    onMouseLeave={(e) => e.currentTarget.style.background = 'rgba(0, 255, 153, 0.15)'}
                >
                    ⬆️
                </button>
                <div style={{
                    flex: 1,
                    padding: '4px 6px',
                    background: 'rgba(0, 255, 153, 0.1)',
                    border: '1px solid rgba(0, 255, 153, 0.2)',
                    borderRadius: '2px',
                    fontSize: '11px',
                    fontFamily: 'monospace',
                    color: '#00ff99',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap'
                }}>
                    📁 {currentPath}
                </div>
            </div>

            {isLoading && <div style={{ fontSize: '11px', opacity: 0.7, marginTop: '5px' }}>Carregando...</div>}
            {error && <div style={{ fontSize: '11px', color: '#ff4444', marginTop: '5px' }}>Erro: {error}</div>}

            {!isLoading && !error && entries.length > 0 && (
                <div
                    style={listStyle}
                    className={`${NO_WHEEL_CLASS} nodrag`}
                >
                    {entries.map((entry, index) => {
                        const combinedStyle = {
                             ...entryStyleBase,
                             ...(entry.is_dir ? folderStyle : fileStyle)
                        };
                        return (
                            <div
                                key={entry.path || `entry-${index}`}
                                style={combinedStyle}
                                title={entry.path}
                                onClick={() => handleEntryClick(entry)}
                                onMouseEnter={(e) => { if (entry.is_dir) e.currentTarget.style.backgroundColor = 'rgba(0, 77, 51, 0.3)'; }}
                                onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                            >
                                <span>{entry.is_dir ? '📁' : '📄'} {entry.name}</span>
                                <Handle
                                    type="source"
                                    id={`out_${entry.path}`}
                                    position={Position.Right}
                                    style={handleBaseStyle}
                                />
                            </div>
                        );
                     })}
                </div>
            )}
             {!isLoading && !error && entries.length === 0 && currentPath && (
                <div style={{ fontSize: '11px', opacity: 0.5, marginTop: '5px' }}>(Pasta vazia ou sem permissão)</div>
            )}
        </div>
    );
}