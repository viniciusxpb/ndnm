// src/hooks/useNodeConfig.ts
import { useState, useEffect } from 'react';
import { apiService } from '../utils/api';

// Hook para carregar a configuração do node-file-browser
export function useNodeConfig() {
  const [nodeConfig, setNodeConfig] = useState<any>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchNodeConfig = async () => {
      try {
        setLoading(true);
        const response = await apiService.getNodeRegistry();
        
        if (response && response.nodes) {
          // Encontrar a configuração do node-file-browser
          const fileBrowserConfig = response.nodes.find(
            (node: any) => node.type === 'filesystem' || node.node_type === 'filesystem' || node?.config?.node_type === 'filesystem'
          );
          
          if (fileBrowserConfig) {
            setNodeConfig(fileBrowserConfig);
            // Atualizar configuração no logger se necessário
            if (window.frontendLogger && typeof window.frontendLogger.setNodeConfig === 'function') {
              window.frontendLogger.setNodeConfig(fileBrowserConfig);
            }
          } else {
            setError('Configuração do node-file-browser não encontrada');
            console.error('Configuração do node-file-browser não encontrada');
          }
        }
      } catch (error) {
        setError('Erro ao carregar configuração do node');
        console.error('Erro ao carregar configuração do node', error);
      } finally {
        setLoading(false);
      }
    };

    fetchNodeConfig();
  }, []);

  return { nodeConfig, loading, error };
}