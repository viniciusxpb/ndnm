# Testing NDNM System

## Quick Start

### 1. Start the node-file-browser

```bash
cd ndnm-backend/nodes/node-file-browser
cargo run
```

Should see: `Starting node server on 0.0.0.0:3001`

### 2. Start ndnm-hermes (in another terminal)

```bash
cd ndnm-backend
cargo run -p ndnm-hermes
```

Should see:
```
Discovered 1 nodes
  - 📂 Gerenciador de Arquivos (Dinâmico) (hash_sha256_de_viniciusxpb_node-file-browser): 2 sections, 2 input fields
Starting Hermes API server on 0.0.0.0:3000
```

---

## Test Commands

### Check Health

**Node:**
```bash
curl http://localhost:3001/health
```

**Hermes:**
```bash
curl http://localhost:3000/health
```

### Get Node Registry (Hermes)

```bash
curl http://localhost:3000/nodes/registry | json_pp
```

### Get Node Config (Node direct)

```bash
curl http://localhost:3001/config | json_pp
```

### Test Node Execution (Direct)

Create a test file:
```bash
curl -X POST http://localhost:3001/run \
  -H "Content-Type: application/json" \
  -d '{
    "inputs": {
      "copy_input_0": {
        "filename": "test.txt",
        "content": "Hello from NDNM!"
      }
    }
  }' | json_pp
```

List files in the node:
```bash
curl http://localhost:3001/list | json_pp
```

### Test Graph Execution (via Hermes)

Create a simple graph (just the file browser copying a file):

```bash
curl -X POST http://localhost:3000/graphs/run \
  -H "Content-Type: application/json" \
  -d '{
    "graph": {
      "nodes": [
        {
          "instance_id": "file_browser_1",
          "node_type_id": "hash_sha256_de_viniciusxpb_node-file-browser",
          "input_values": {
            "target_directory": "./test_workspace"
          }
        }
      ],
      "connections": []
    }
  }' | json_pp
```

### Save a Workspace

```bash
curl -X POST http://localhost:3000/nexus/save \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_test_workspace",
    "data": {
      "graph": {
        "nodes": [
          {
            "instance_id": "node1",
            "node_type_id": "hash_sha256_de_viniciusxpb_node-file-browser",
            "input_values": {}
          }
        ],
        "connections": []
      },
      "metadata": {
        "description": "Test workspace",
        "created_by": "tester"
      }
    }
  }'
```

### List Workspaces

```bash
curl http://localhost:3000/nexus/list | json_pp
```

### Load a Workspace

```bash
curl http://localhost:3000/nexus/load/my_test_workspace | json_pp
```

---

## Windows PowerShell Commands

If curl is not working well, use PowerShell:

### Health Check
```powershell
Invoke-RestMethod -Uri http://localhost:3000/health
```

### Get Registry
```powershell
Invoke-RestMethod -Uri http://localhost:3000/nodes/registry | ConvertTo-Json -Depth 10
```

### Execute Graph
```powershell
$body = @{
    graph = @{
        nodes = @(
            @{
                instance_id = "file_browser_1"
                node_type_id = "hash_sha256_de_viniciusxpb_node-file-browser"
                input_values = @{
                    target_directory = "./test_workspace"
                }
            }
        )
        connections = @()
    }
} | ConvertTo-Json -Depth 10

Invoke-RestMethod -Uri http://localhost:3000/graphs/run -Method Post -Body $body -ContentType "application/json"
```

---

## Expected Behaviors

### ✅ Working
- Hermes discovers the node-file-browser
- Node registry returns full config structure
- Node can execute directly via /run
- Files are created in managed_files directory
- Workspaces save to nexus/ directory

### 🚧 Limitations (since brazil/exdoida not implemented yet)
- No WebSocket communication
- No real-time updates
- No observability/logging system
- No authentication

---

## Troubleshooting

### Port Already in Use
```bash
# Windows
netstat -ano | findstr :3000
netstat -ano | findstr :3001

# Kill process
taskkill /PID <pid> /F
```

### Node Not Discovered
- Make sure config.yaml exists in nodes/node-file-browser/
- Check Hermes logs for discovery errors
- Verify nodes/ directory path is correct

### Connection Refused
- Make sure services are running
- Check firewall settings
- Verify ports 3000 and 3001 are accessible
