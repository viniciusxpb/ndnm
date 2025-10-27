# Documentação das Alterações - Integração Frontend/Backend

## Resumo das Alterações

Este documento descreve as alterações realizadas para configurar o frontend `ndnm-argos` para se comunicar com o backend `ndnm-brazil` utilizando o arquivo de configuração `node-file-browser/config.yaml`.

### 1. Arquivos Criados/Modificados

- **Novos Arquivos:**
  - `.env` - Configuração de variáveis de ambiente
  - `src/utils/api.ts` - Serviço de API para comunicação com o backend
  - `src/hooks/useNodeConfig.ts` - Hook para carregar configurações de nodes

- **Arquivos Modificados:**
  - `package.json` - Adicionada dependência axios
  - `src/utils/logger.ts` - Atualizado para suportar configuração de nodes
  - `src/App.tsx` - Integrado hook useNodeConfig
  - `src/flow/FlowController.tsx` - Atualizado para receber configuração de nodes
  - `src/hooks/useNodePalette.ts` - Atualizado para usar configuração do node-file-browser

### 2. Variáveis de Ambiente Configuradas

```
VITE_WS_HOST=localhost
VITE_WS_PORT=3002
VITE_WS_PATH=/ws
VITE_API_BASE_URL=http://localhost:3002
VITE_NODE_CONFIG_PATH=/nodes/registry
```

### 3. Dependências Adicionadas

- **axios**: ^1.6.2 - Cliente HTTP para comunicação com o backend

### 4. Funcionalidades Implementadas

1. **Sistema de API**: Implementado serviço de API com axios para comunicação com o backend
2. **Carregamento de Configuração**: Hook useNodeConfig para carregar configuração do node-file-browser
3. **Integração com Logger**: Atualizado sistema de logging para diagnóstico de problemas
4. **Adaptação do Palette**: Modificado useNodePalette para usar configuração do node-file-browser

### 5. Fluxo de Comunicação

1. O frontend inicia e carrega as variáveis de ambiente
2. O hook useNodeConfig faz uma requisição para o backend para obter a configuração dos nodes
3. A configuração é passada para o FlowController e useNodePalette
4. O useNodePalette mapeia a configuração para o formato esperado pelo frontend
5. A comunicação em tempo real é mantida via WebSocket

### 6. Testes Realizados

- Verificação de carregamento da configuração do node-file-browser
- Teste de comunicação WebSocket entre frontend e backend
- Validação da exibição correta dos nodes no palette

## Como Testar

1. Inicie o backend: `cd ndnm-backend/ndnm-brazil; cargo run`
2. Inicie o frontend: `cd ndnm-argos; npm run dev`
3. Acesse o frontend no navegador: `http://localhost:5173`
4. Verifique se o node File Browser aparece no palette

## Próximos Passos

- Implementar testes automatizados para validar a integração
- Melhorar o tratamento de erros na comunicação entre frontend e backend
- Adicionar suporte para mais tipos de nodes

## Comando de Reset e Execução

Para matar quaisquer nodes/serviços que estejam rodando, reiniciar todo o backend do zero e iniciar o frontend em uma nova janela, use:

```
# Na raiz do projeto (c:\Projetos\new\ndnm)
.\start.ps1
```

- Para executar sem abrir o frontend (apenas backend):
```
.\start.ps1 -NoFrontend
```

Observações:
- Este script chama `stop-all.ps1` para encerrar processos e `start-all.ps1` para iniciar Hermes, Brazil, Exdoida e nodes.
- O frontend é iniciado com `npm run dev` em uma nova janela do PowerShell.
- Em PowerShell, para encadear comandos use `;` (não `&&`).