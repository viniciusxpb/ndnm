// viniciusxpb/frontend/frontend-e2ff74bfb8b04c2182486c99304ae2e139575034/src/components/FileBrowser.tsx
import React, { useState, useEffect, useCallback } from 'react';

// Definindo a estrutura da entrada que vem do Rust
interface DirectoryEntry {
    name: string;
    is_dir: boolean;
    path: string;
    modified: string; // Vem como string ISO 8601 do Rust
}

interface FileBrowserProps {
    onSelectPath: (path: string) => void;
    initialPath: string;
    isFolderPicker: boolean; // Seletor de pasta ou de arquivo
}

export function FileBrowser({ onSelectPath, initialPath, isFolderPicker }: FileBrowserProps) {
    const [currentPath, setCurrentPath] = useState(initialPath || 'C:\\');
    const [entries, setEntries] = useState<DirectoryEntry[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Função para buscar os arquivos/diretórios via HTTP
    const fetchDirectoryContents = useCallback(async (path: string) => {
        setIsLoading(true);
        setError(null);
        
        try {
            console.log(`[FileBrowser] 📡 Buscando conteúdo do diretório: ${path}`);
            
            const response = await fetch('http://localhost:3011/run', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ path: path }),
            });

            if (!response.ok) {
                throw new Error(`Erro HTTP: ${response.status}`);
            }

            const data = await response.json();
            console.log(`[FileBrowser] ✅ Resposta recebida. ${data.entries?.length || 0} entradas.`);
            
            // Atualiza o caminho atual e as entradas
            setCurrentPath(data.current_path || path);
            setEntries(data.entries || []);
            
        } catch (err) {
            console.error('[FileBrowser] ❌ Erro ao buscar diretório:', err);
            setError(`Falha ao carregar diretório: ${err instanceof Error ? err.message : 'Erro desconhecido'}`);
            setEntries([]);
        } finally {
            setIsLoading(false);
        }
    }, []);

    // Efeito para carregar o diretório inicial quando o componente monta
    useEffect(() => {
        fetchDirectoryContents(initialPath || 'C:\\');
    }, [initialPath, fetchDirectoryContents]);

    const handleEntryClick = useCallback((entry: DirectoryEntry) => {
        if (entry.is_dir) {
            console.log(`[FileBrowser] ➡️ Navegando para: ${entry.path}`);
            fetchDirectoryContents(entry.path);
        } else if (!isFolderPicker) {
            onSelectPath(entry.path);
        }
    }, [isFolderPicker, onSelectPath, fetchDirectoryContents]);

    // Simplesmente seleciona a pasta atual
    const handleSelectCurrent = useCallback(() => {
        console.log(`[FileBrowser] ✅ Selecionado o caminho atual: ${currentPath}`);
        onSelectPath(currentPath);
    }, [currentPath, onSelectPath]);

    // Botão para recarregar o diretório atual
    const handleReload = useCallback(() => {
        fetchDirectoryContents(currentPath);
    }, [currentPath, fetchDirectoryContents]);

    // Renderização no estilo hacker
    return (
        <div style={{ padding: '8px', minHeight: '300px' }}>
            <div style={{ marginBottom: '8px', fontSize: '14px', color: '#00ff99' }}>
                Caminho: {isLoading ? `Carregando...` : currentPath}
            </div>

            <div style={{ marginBottom: '10px', display: 'flex', gap: '8px' }}>
                {isFolderPicker && (
                    <button 
                        className="hacker-btn nodrag"
                        onClick={handleSelectCurrent} 
                        disabled={isLoading}
                    >
                        ✅ Selecionar esta pasta
                    </button>
                )}
                <button 
                    className="hacker-btn nodrag"
                    onClick={handleReload} 
                    disabled={isLoading}
                >
                    🔄 Atualizar
                </button>
            </div>

            {error && (
                <div style={{ 
                    color: '#ff4444', 
                    backgroundColor: 'rgba(255, 0, 0, 0.1)', 
                    padding: '8px', 
                    marginBottom: '10px',
                    border: '1px solid #ff4444',
                    fontSize: '12px'
                }}>
                    ❌ {error}
                </div>
            )}

            {isLoading ? (
                <div style={{ opacity: 0.8, padding: '20px', textAlign: 'center' }}>
                    Buscando arquivos e diretórios...
                </div>
            ) : (
                <div style={{ maxHeight: '250px', overflowY: 'auto', border: '1px solid #1f5329', padding: '4px' }}>
                    {entries.map((entry) => (
                        <div 
                            key={entry.path + entry.name}
                            onClick={() => handleEntryClick(entry)}
                            className="nodrag"
                            style={{
                                cursor: entry.is_dir || !isFolderPicker ? 'pointer' : 'default',
                                padding: '3px',
                                fontSize: '12px',
                                opacity: entry.is_dir ? 1 : 0.7,
                                color: entry.is_dir ? '#00ccff' : '#b7f397',
                                borderBottom: '1px dotted #17351c',
                                display: 'flex',
                                justifyContent: 'space-between',
                                transition: 'background 0.1s ease',
                                userSelect: 'none',
                            }}
                            onMouseEnter={(e) => { e.currentTarget.style.background = 'rgba(0, 77, 51, 0.2)'; }}
                            onMouseLeave={(e) => { e.currentTarget.style.background = 'transparent'; }}
                        >
                            <span>
                                {entry.name === '..' ? '⬆️' : entry.is_dir ? '📁' : '📄'} {entry.name}
                            </span>
                            <span style={{ opacity: 0.5, fontSize: '10px' }}>
                                {entry.modified ? entry.modified.substring(0, 10) : ''}
                            </span>
                        </div>
                    ))}
                    {entries.length === 0 && !error && (
                        <div style={{ opacity: 0.5, padding: '10px' }}>Pasta vazia</div>
                    )}
                </div>
            )}
        </div>
    );
}