// src/utils/api.ts
import axios from "axios";

// Configuração base do axios
const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:3003',
});

// Interceptor para logging de requisições
api.interceptors.request.use(
  (config) => {
    console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`);
    return config;
  },
  (error) => {
    console.error('API Request Error:', error);
    return Promise.reject(error);
  }
);

// Interceptor para logging de respostas
api.interceptors.response.use(
  (response) => {
    console.log(`API Response: ${response.status} ${response.config.url}`);
    return response;
  },
  (error) => {
    console.error('API Response Error:', error.response?.status || 'Network Error', error.message);
    return Promise.reject(error);
  }
);

// Funções para comunicação com o back-end
export const apiService = {
  // Obter registro de nós
  async getNodeRegistry() {
    try {
      const response = await api.get(import.meta.env.VITE_NODE_CONFIG_PATH || '/nodes/registry');
      return response.data;
    } catch (error) {
      console.error('Failed to fetch node registry', error);
      throw error;
    }
  },

  // Executar um grafo
  async executeGraph(graphData: any) {
    try {
      const response = await api.post('/graphs/run', graphData);
      return response.data;
    } catch (error) {
      console.error('Failed to execute graph', error);
      throw error;
    }
  },

  // Salvar workspace
  async saveWorkspace(name: string, data: any) {
    try {
      const response = await api.post('/nexus/save', { name, data });
      return response.data;
    } catch (error) {
      console.error('Failed to save workspace', error);
      throw error;
    }
  },

  // Carregar workspace
  async loadWorkspace(name: string) {
    try {
      const response = await api.get(`/nexus/load/${name}`);
      return response.data;
    } catch (error) {
      console.error('Failed to load workspace', error);
      throw error;
    }
  },

  // Listar workspaces
  async listWorkspaces() {
    try {
      const response = await api.get('/nexus/list');
      return response.data;
    } catch (error) {
      console.error('Failed to list workspaces', error);
      throw error;
    }
  },
};