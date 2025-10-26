# 🚀 NDNM Quick Start Guide

## Passo 1: Rodar o Node

Abra um terminal e execute:

```bash
cd ndnm-backend/nodes/node-file-browser
cargo run
```

Você deve ver:
```
Loaded node configuration: 📂 Gerenciador de Arquivos (Dinâmico)
Target directory set to: "./managed_files"
Starting node server on 0.0.0.0:3001
```

✅ Node rodando na porta **3001**

---

## Passo 2: Rodar o Hermes

Abra **OUTRO terminal** e execute:

```bash
cd ndnm-backend
cargo run -p ndnm-hermes
```

Você deve ver:
```
Starting NDNM Hermes - The Orchestrator
Discovering nodes in ./nodes directory...
Discovered 1 nodes
  - 📂 Gerenciador de Arquivos (Dinâmico) (hash_sha256_de_viniciusxpb_node-file-browser): 2 sections, 2 input fields
Starting Hermes API server on 0.0.0.0:3000
```

✅ Hermes rodando na porta **3000**

---

## Passo 3: Testar!

Abra **OUTRO terminal** (ou PowerShell) e rode os testes:

### Testar Health

```powershell
.\test.ps1 health-node
.\test.ps1 health-hermes
```

### Ver Nodes Registrados

```powershell
.\test.ps1 registry
```

### Criar um Arquivo

```powershell
.\test.ps1 create-file
```

### Listar Arquivos

```powershell
.\test.ps1 list-files
```

### Executar um Grafo

```powershell
.\test.ps1 run-graph
```

### Salvar Workspace

```powershell
.\test.ps1 save-workspace
```

### Listar Workspaces

```powershell
.\test.ps1 list-workspaces
```

---

## Comandos Úteis

### Ver todos os comandos disponíveis
```powershell
.\test.ps1 help
```

### Build tudo
```bash
cd ndnm-backend
cargo build --all
```

### Rodar todos os testes
```bash
cd ndnm-backend
cargo test --all
```

---

## Estrutura de Portas

- **3000** - Hermes (Orchestrator)
- **3001** - Node File Browser
- **3002+** - Outros nodes que você criar

---

## Troubleshooting

### Erro "Address already in use"

```powershell
# Ver quem está usando a porta
netstat -ano | findstr :3000

# Matar o processo
taskkill /PID <número_do_pid> /F
```

### Node não aparece no registry

1. Verifique se o `config.yaml` existe em `nodes/node-file-browser/`
2. Restart o Hermes
3. Veja os logs do Hermes para erros

### Erro de compilação

```bash
cd ndnm-backend
cargo clean
cargo build --all
```

---

## Próximos Passos

Agora que tá funcionando:

1. Crie novos nodes copiando `node-file-browser` como template
2. Implemente `ndnm-brazil` para WebSocket
3. Implemente `ndnm-exdoida` para observability
4. Crie o frontend `ndnm-argos`

Veja `CLAUDE.md` para detalhes da arquitetura!
