# CLAUDE.md - NDNM Project

This file provides comprehensive guidance to Claude Code (claude.ai/code) and other AI assistants when working with code in this repository.

---

## 🎯 Project Overview: NDNM (Node-based Data Network Manipulation)

**NDNM** is a **visual programming system** where users create workflows by connecting nodes in a graph. Think of it as a **low-code platform** where each node is a microservice that can:
- Process data (add numbers, manipulate text, ML inference)
- Manage files (copy, read, write)
- Trigger workflows (play buttons)
- Interface with external systems (ComfyUI, APIs)

**Key Innovation**: The frontend is a "dumb shell" that receives node definitions dynamically from the backend. When you create a new node microservice in the backend, the frontend automatically gets the UI configuration via WebSocket and renders it - **zero frontend code changes needed**.

### The Big Picture

```
┌────────────────────────────────────────────────────────────┐
│                    User's Browser                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  ndnm-argos (React + ReactFlow + TypeScript)         │  │
│  │  - Visual node editor                                │  │
│  │  - Drag & drop nodes                                 │  │
│  │  - Connect nodes with edges                          │  │
│  └───────────────────┬──────────────────────────────────┘  │
└────────────────────────┼──────────────────────────────────┘
                         │ WebSocket (ws://localhost:3002/ws)
                         ↓
┌──────────────────────────────────────────────────────────────┐
│               ndnm-brazil (Rust + Axum)                      │
│               Port 3002 - Backend-for-Frontend (BFF)         │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ - WebSocket server (broadcasts to multiple clients)   │ │
│  │ - Translates frontend ↔ Hermes communication          │ │
│  │ - Future: Authentication & authorization              │ │
│  └───────────────────┬────────────────────────────────────┘ │
└────────────────────────┼────────────────────────────────────┘
                         │ HTTP (localhost:3000)
                         ↓
┌──────────────────────────────────────────────────────────────┐
│            ndnm-hermes (Rust + Axum)                         │
│            Port 3000 - Orchestrator / Control Plane          │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ - Discovers nodes in nodes/ directory                 │ │
│  │ - Parses config.yaml files                            │ │
│  │ - Builds node registry                                │ │
│  │ - Executes graphs (topological sort)                  │ │
│  │ - Manages workspace persistence (nexus/)              │ │
│  │ - Health checks all nodes                             │ │
│  └───────────────────┬────────────────────────────────────┘ │
└────────────────────────┼────────────────────────────────────┘
                         │ HTTP
           ┌─────────────┼─────────────┐
           │             │             │
           ↓             ↓             ↓
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │  Node    │  │  Node    │  │  Node    │
    │  3001    │  │  3004    │  │  3005... │
    └────┬─────┘  └────┬─────┘  └────┬─────┘
         │             │             │
         └─────────UDP 9514──────────┘
                (fire-and-forget logs)
                         │
                         ↓
                ┌─────────────────┐
                │  ndnm-exdoida   │
                │  Port 3003      │
                │  Observability  │
                └─────────────────┘
```

### Port Allocation

| Service | Port | Protocol | Purpose |
|---------|------|----------|---------|
| ndnm-hermes | 3000 | HTTP | Orchestrator API |
| node-file-browser | 3001 | HTTP | Example node |
| ndnm-brazil | 3002 | HTTP + WebSocket | BFF for frontend |
| ndnm-exdoida | 3003 | HTTP + UDP(9514) | Observability |
| Other nodes | 3004+ | HTTP | Additional nodes |

---

## 🏗️ Architecture Deep Dive

### 1. ndnm-libs - The Foundation

**Location**: `ndnm-backend/ndnm-libs/`
**Type**: Rust library crate
**Edition**: 2021 (Rust 2024 edition caused issues - see [Troubleshooting](#troubleshooting))

**Purpose**: Shared types and utilities used by all backend services.

**Key Exports**:

```rust
// Core trait that ALL nodes must implement
#[async_trait]
pub trait Node: Send + Sync {
    fn validate(&self, inputs: &HashMap<String, Value>) -> Result<(), AppError>;
    async fn process(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, AppError>;
}

// Standardized error handling
#[derive(Error, Debug)]
pub enum AppError {
    BadRequest(String),
    Internal(String),
    ConfigError(String),
    IoError(#[from] std::io::Error),
    YamlError(#[from] serde_yaml::Error),
    JsonError(#[from] serde_json::Error),
}

// Config structures for parsing node config.yaml files
pub struct NodeConfig {
    pub node_id_hash: String,
    pub label: String,
    pub node_type: String,
    pub sections: Vec<Section>,
    pub input_fields: Vec<InputFieldConfig>,
}

pub struct Section {
    pub section_name: String,
    pub section_label: Option<String>,
    pub behavior: SectionBehavior,  // auto_increment, dynamic_per_file, static
    pub slot_template: SlotTemplate,
}

pub struct SlotTemplate {
    pub inputs: Vec<InputSlotConfig>,
    pub outputs: Vec<OutputSlotConfig>,
}
```

**Tests**: 10 passing tests covering config parsing, error handling, and trait implementations.

**Common Issues Encountered**:
- **Rust Edition 2024**: Initial Cargo.toml had `edition = "2024"` which caused compilation errors. Fixed by changing to `edition = "2021"`.
- **Axum IntoResponse**: AppError needs `IntoResponse` impl to work with Axum handlers. We map each error variant to appropriate HTTP status codes.

---

### 2. ndnm-hermes - The Orchestrator

**Location**: `ndnm-backend/ndnm-hermes/`
**Type**: Executable service (bin)
**Port**: 3000 (configurable via `PORT` env var)

**Purpose**: The "brain" of NDNM. Discovers nodes, validates graphs, orchestrates execution.

**Key Modules**:

```rust
// src/main.rs - HTTP API server
async fn health_check() -> Json<HealthResponse>
async fn health_check_all() -> Json<SystemHealthResponse>  // Checks all registered nodes
async fn get_node_registry() -> Json<NodeRegistry>
async fn get_node_info(node_id: &str) -> Result<Json<NodeInfo>>
async fn execute_graph(graph: ExecuteGraphRequest) -> Result<Json<ExecutionResult>>
async fn save_workspace(request: SaveWorkspaceRequest) -> StatusCode
async fn load_workspace(name: &str) -> Result<Json<Workspace>>
async fn list_workspaces() -> Json<WorkspacesResponse>

// src/discovery.rs - Node discovery
pub fn discover_nodes(base_path: &Path) -> Result<NodeRegistry, AppError>
// Scans nodes/ directory recursively for config.yaml files

// src/registry.rs - Node registry management
pub struct NodeRegistry {
    nodes: HashMap<String, NodeInfo>,
}
pub struct NodeInfo {
    pub node_id: String,
    pub config: NodeConfig,
    pub port: u16,
    pub path: PathBuf,
}

// src/orchestrator.rs - Graph execution
pub async fn execute_graph(graph: &Graph, registry: &NodeRegistry) -> Result<ExecutionResult>
// - Validates graph structure
// - Topological sort for execution order
// - Calls nodes via HTTP POST /run

// src/workspace.rs - Persistence
pub async fn save_workspace(name: &str, data: &Workspace) -> Result<()>
pub async fn load_workspace(name: &str) -> Result<Workspace>
pub async fn list_workspaces() -> Result<Vec<String>>
// Saves to ndnm-backend/nexus/ directory as JSON files
```

**API Endpoints**:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Simple health check |
| GET | `/health/all` | Health check + status of all nodes |
| GET | `/nodes/registry` | List all discovered nodes |
| GET | `/nodes/{id}` | Get specific node info |
| POST | `/graphs/run` | Execute a graph |
| POST | `/nexus/save` | Save workspace |
| GET | `/nexus/load/{name}` | Load workspace |
| GET | `/nexus/list` | List all workspaces |

**Tests**: 11 passing tests covering discovery, registry, orchestration, and workspace management.

**Critical Rules**:
- ❌ **NEVER** communicates directly with frontend
- ✅ **ALWAYS** goes through ndnm-brazil for frontend communication
- ✅ Nodes never know about each other - Hermes is the only one with the full graph
- ✅ Hermes manages all node ports (nodes don't define their own ports)

**Problems Solved**:
1. **Node Discovery**: Initially nodes were manually registered. Now we scan `nodes/` directory automatically using `walkdir`.
2. **Health Checks**: Added `/health/all` endpoint that checks every registered node via HTTP and returns response times. This was critical for debugging node communication issues.
3. **Temporary Value Lifetimes**: Had issues with `format!()` creating temporary values in `unwrap_or()`. Fixed by storing format result in variable first.

---

### 3. ndnm-brazil - Backend-for-Frontend (BFF)

**Location**: `ndnm-backend/ndnm-brazil/`
**Type**: Executable service (bin)
**Port**: 3002 (HTTP + WebSocket on `/ws`)

**Purpose**: Translation layer between frontend (ndnm-argos) and backend (ndnm-hermes). Maintains persistent WebSocket connections.

**Key Modules**:

```rust
// src/main.rs - HTTP + WebSocket server
struct AppState {
    hermes_client: reqwest::Client,
    hermes_url: String,
    ws_broadcaster: websocket::Broadcaster,  // Broadcasts to multiple clients
}

async fn health_check() -> Json<HealthResponse>
async fn websocket_handler() -> WebSocket  // Upgrades to WebSocket
async fn get_node_registry() -> Json<Value>  // Proxies to Hermes
async fn execute_graph() -> Json<Value>  // Proxies to Hermes + broadcasts result
async fn save_workspace() -> StatusCode  // Proxies to Hermes
async fn load_workspace() -> Json<Value>  // Proxies to Hermes
async fn list_workspaces() -> Json<Value>  // Proxies to Hermes

// src/websocket.rs - WebSocket broadcast system
pub struct Broadcaster {
    tx: Arc<broadcast::Sender<String>>,
}
impl Broadcaster {
    pub async fn broadcast_json(&self, message: &Value);
    pub fn subscribe(&self) -> broadcast::Receiver<String>;
}
```

**WebSocket Messages** (Frontend ↔ Brazil):

From Frontend:
- `EXECUTE_PLAY` - Trigger workflow execution

To Frontend:
- `NODE_CONFIG` - Node type definitions (sent on connection)
- `EXECUTION_STATUS` - Real-time execution updates
- `EXECUTION_COMPLETE` - Workflow finished
- `EXECUTION_ERROR` - Workflow failed

**API Endpoints**: Proxies most Hermes endpoints + adds WebSocket

**Tests**: 2 passing tests for WebSocket broadcaster

**Problems Solved**:
1. **Serialization Error**: `ExecuteGraphRequest` was missing `Serialize` derive. Added it.
2. **Broadcast to Multiple Clients**: Used `tokio::sync::broadcast` channel to send messages to all connected WebSocket clients simultaneously.
3. **Hermes Connection Check**: Health check now tests if Hermes is accessible and returns status.

---

### 4. ndnm-exdoida - Observability (The Silent Observer 👀)

**Location**: `ndnm-backend/ndnm-exdoida/`
**Type**: Executable service (bin)
**Ports**: HTTP 3003, UDP 9514

**Purpose**: **Fire-and-forget** observability. Services send logs via UDP without waiting for response. System continues if Exdoida is down.

**Key Modules**:

```rust
// src/main.rs - HTTP API
async fn health_check() -> Json<HealthResponse>  // Includes log count
async fn get_logs(Query(params): Query<LogsQuery>) -> Json<LogsResponse>
// Filters: limit (default 100), level (info/warn/error), source (service name)
async fn get_metrics() -> Json<MetricsResponse>  // Aggregated by level and source
async fn clear_logs() -> StatusCode  // DELETE all logs

// src/storage.rs - In-memory log storage
pub struct LogStore {
    max_capacity: usize,  // Circular buffer
    counter: Arc<AtomicUsize>,
    logs: Arc<DashMap<usize, LogEntry>>,  // Thread-safe concurrent map
}
pub struct LogEntry {
    pub id: usize,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub source: String,
    pub message: String,
    pub metadata: Option<Value>,
}

// src/udp_server.rs - Fire-and-forget UDP log receiver
pub async fn start_udp_server(port: u16, log_store: Arc<LogStore>) -> anyhow::Result<()>
// Listens on UDP port 9514, parses JSON log messages, stores in LogStore
```

**Log Message Format** (UDP JSON):
```json
{
  "level": "info",
  "source": "ndnm-hermes",
  "message": "Graph execution completed",
  "metadata": {
    "graph_id": "123",
    "duration_ms": 42
  }
}
```

**API Endpoints**:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health + log count |
| GET | `/logs?limit=100&level=error&source=hermes` | Query logs with filters |
| GET | `/metrics` | Aggregated metrics |
| DELETE | `/logs` | Clear all logs |

**Tests**: 10 passing tests covering storage, circular buffer, UDP parsing, and concurrent access.

**Design Principles**:
- ✅ **Non-blocking**: Services use UDP - no waiting for response
- ✅ **Resilient**: System continues if Exdoida is down
- ✅ **Thread-safe**: DashMap for concurrent access from UDP and HTTP handlers
- ✅ **Memory-bounded**: Circular buffer auto-removes oldest logs when at capacity
- ✅ **Fire-and-forget**: No acknowledgments, no retries

**Problems Solved**:
1. **Send Trait**: UDP server initially returned `Box<dyn StdError>` which isn't `Send`. Fixed by using `anyhow::Result<()>` instead.
2. **Circular Buffer**: Needed automatic memory management. Implemented by tracking log count and removing oldest entry when at capacity.
3. **Concurrent Access**: Used `DashMap` instead of `RwLock<HashMap>` for lock-free concurrent reads/writes.

---

### 5. Nodes System - The Workers

**Location**: `ndnm-backend/nodes/*/`
**Example**: `node-file-browser`

Each node is a **microservice** that implements the `Node` trait and exposes an HTTP API.

**Node Structure**:
```
nodes/
└── node-file-browser/
    ├── Cargo.toml
    ├── config.yaml       # ← Node configuration
    ├── src/
    │   └── main.rs
    └── managed_files/    # ← Working directory
```

**config.yaml Example**:
```yaml
node_id_hash: "hash_sha256_de_viniciusxpb_node-file-browser"
label: "Gerenciador de Arquivos (Dinâmico)"
node_type: "file_browser"

sections:
  - section_name: "copy_here"
    section_label: "Copiar Arquivos Novos"
    behavior: "auto_increment"  # UI adds more slots as connections are made
    slot_template:
      inputs:
        - name: "copy_input"
          label: "Arquivo para Copiar"
          type: "FILE_CONTENT"
          connections: 1  # Single connection
      outputs:
        - name: "copied_output"
          label: "Arquivo Copiado"
          type: "FILE_CONTENT"
          connections: "n"  # Unlimited connections

  - section_name: "internal_files"
    section_label: "Arquivos Internos"
    behavior: "dynamic_per_file"  # System creates slots per file found
    slot_template:
      inputs:
        - name: "file_{filename}_input"
          label: "Sobrescrever {filename}"
          type: "FILE_CONTENT"
          connections: 1
      outputs:
        - name: "file_{filename}_output"
          label: "Ler {filename}"
          type: "FILE_CONTENT"
          connections: "n"

input_fields:
  - name: "target_directory"
    field_type: "text"
    label: "Diretório Gerenciado"
    default: "./managed_files"
```

**Node HTTP API** (Standard for all nodes):

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/config` | Return parsed config.yaml |
| POST | `/run` | Execute node logic with inputs |
| GET | `/list` | (Optional) List files or resources |

**Example Node Implementation** (`node-file-browser/src/main.rs`):

```rust
struct FileBrowserNode {
    config: NodeConfig,
    target_directory: String,
}

#[async_trait]
impl Node for FileBrowserNode {
    fn validate(&self, inputs: &HashMap<String, Value>) -> Result<(), AppError> {
        // Validate inputs match config
    }

    async fn process(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, AppError> {
        // Process files based on inputs
        // Return outputs
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config("config.yaml")?;
    let node = FileBrowserNode::new(config, "./managed_files");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/config", get(get_config))
        .route("/run", post(run_node))
        .route("/list", get(list_files));

    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Tests**: 4 passing tests for node-file-browser

---

### 6. ndnm-argos - The Frontend

**Location**: `ndnm-argos/`
**Type**: React + TypeScript + Vite
**Port**: 5173 (dev server)

**Purpose**: Visual node editor - a "dumb shell" that renders nodes dynamically based on backend config.

**Tech Stack**:
- React 18 + TypeScript
- Vite (build tool)
- @xyflow/react (node-based UI)
- SASS (styling)
- Path alias: `@/` → `src/`

**Key Components**:

```typescript
// src/App.tsx - Entry point
// Initializes WebSocket connection to Brazil

// src/flow/FlowController.tsx - State orchestration
// - Manages nodes, edges, palette, pinned nodes
// - Coordinates all hooks
// - Handles function restoration after workspace load

// src/flow/FlowCanvas.tsx - ReactFlow canvas
// - Node editor surface
// - Drag & drop
// - Edge connections

// src/nodes/BaseIONode.tsx - Generic dynamic node
// - Renders ALL node types dynamically
// - Reads node definition from palette
// - Supports dynamic I/O handles

// src/nodes/PlayNode.tsx - Play button with pin/unpin
// - Triggers workflow execution
// - Can be pinned to persistent overlay

// src/components/PinnedPlayButtons.tsx
// - Persistent overlay for pinned play nodes
// - Always visible regardless of zoom/pan

// src/components/NodeCatalog.tsx - Node picker
// - Shows available node types from backend
// - Opens with Ctrl+Space (pane click combo)

// src/components/LeftPanel.tsx - Workspace management
// - Save/load workspaces
// - Workspace list
```

**Custom Hooks**:

```typescript
// src/hooks/useFlowStateSync.ts
// - Core ReactFlow state (nodes, edges)
// - Change handlers

// src/hooks/useFlowInteraction.ts
// - User interactions (pane clicks, connections)
// - Modal management

// src/hooks/useNodePalette.ts
// - Receives node definitions via WebSocket NODE_CONFIG

// src/hooks/usePinnedNodes.ts
// - Pin/unpin play nodes

// src/hooks/useWorkflowExecution.ts
// - Executes workflows via EXECUTE_PLAY message
// - Tracks execution status

// src/hooks/useWorkspacePersistence.ts
// - Save/load workspaces to backend

// src/hooks/useWsClient.ts
// - WebSocket connection with auto-reconnect
// - Heartbeat (25s interval)
// - Exponential backoff (750ms base, 10s max)
```

**Critical Pattern: Function Reference Restoration**

**Problem**: When workspaces are saved/loaded, node data loses function references (e.g., `onChange` callbacks) during JSON serialization.

**Solution**: FlowController creates `handleNodeValueChange` and reassigns it to all nodes after loading:

```typescript
// In FlowController.tsx
const handleNodeValueChange = useCallback((nodeId: string, value: string) => {
  setNodes(nds => nds.map(node =>
    node.id === nodeId ? {
      ...node,
      data: { ...node.data, value }
    } : node
  ));
}, [setNodes]);

// Reassigned when loading workspaces
const handleLoadWorkspaceFromPanel = useCallback((newNodes: Node[], newEdges: Edge[]) => {
  const processedNodes = newNodes.map(node => ({
    ...node,
    data: { ...node.data, onChange: handleNodeValueChange }  // ← Restore function
  }));
  setNodes(processedNodes);
  setEdges(newEdges);
}, [setNodes, setEdges, handleNodeValueChange]);
```

**WebSocket Communication**:

URL: `ws://localhost:3002/ws` (configurable via env vars)

Messages from Backend:
- `NODE_CONFIG` - Array of node types (sent on connection)
- `EXECUTION_STATUS` - Real-time updates
- `EXECUTION_COMPLETE` - Success (with metrics)
- `EXECUTION_ERROR` - Failure

Messages to Backend:
- `EXECUTE_PLAY` - Trigger workflow

**Dynamic Node System**:

The frontend does NOT have hardcoded node types. Everything comes from backend via `NODE_CONFIG`:

```typescript
interface NodePaletteItem {
  type: string;  // Unique identifier
  label: string;  // Display name
  default_data: {
    inputsMode: 0 | 1 | 'n';  // No inputs, single, or dynamic
    outputsMode: 0 | 1 | 'n';  // No outputs, single, or dynamic
    input_fields: Array<{
      type: 'text' | 'selector';
      name: string;
      label: string;
      default?: string;
    }>;
    value: any;  // Default value
  };
}
```

**⚠️ Known Issue: Port Mapping Hardcoded**

The frontend has a hardcoded port mapping in `src/utils/graphConverter.ts`:

```typescript
const portMap: Record<string, number> = {
  'add': 3000,
  'subtract': 3001,
  'fixedValue': 3010,
  'fsBrowser': 3011,
  'playButton': 3020,
  'comfyPlay': 3021,
  // Must update manually when adding new nodes!
};
```

**Future Fix**: This should come from backend via `NODE_CONFIG`.

---

## 🚀 Development Workflow

### Initial Setup

```powershell
# Clone repository
git clone <repo-url>
cd ndnm

# Backend setup
cd ndnm-backend
cargo build --workspace  # Build all services
cargo test --workspace   # Run all tests

# Frontend setup (when ready)
cd ../ndnm-argos
yarn install
```

### Starting the System

**Option 1: Automated (Recommended)**

```powershell
# From project root
.\start-all.ps1
```

This script:
1. Starts ndnm-hermes (port 3000)
2. Starts ndnm-brazil (port 3002)
3. Starts ndnm-exdoida (port 3003)
4. Discovers and starts all nodes in `nodes/`
5. Opens each service in separate PowerShell window
6. Saves PIDs to `.ndnm-pids.txt` for cleanup

**Option 2: Manual**

```powershell
# Terminal 1 - Hermes
cd ndnm-backend
cargo run -p ndnm-hermes

# Terminal 2 - Brazil
cd ndnm-backend
cargo run -p ndnm-brazil

# Terminal 3 - Exdoida
cd ndnm-backend
cargo run -p ndnm-exdoida

# Terminal 4 - Node
cd ndnm-backend/nodes/node-file-browser
cargo run

# Terminal 5 - Frontend (when ready)
cd ndnm-argos
yarn dev
```

### Stopping the System

```powershell
.\stop-all.ps1
```

Stops all services started by `start-all.ps1`.

### Testing

**Automated Test Suite**:

```powershell
# Health checks
.\test.ps1 health-hermes   # Hermes + all nodes
.\test.ps1 health-brazil   # Brazil BFF
.\test.ps1 health-exdoida  # Exdoida observability
.\test.ps1 health-node     # Specific node

# Node operations
.\test.ps1 create-file     # Create test file
.\test.ps1 list-files      # List files
.\test.ps1 run-graph       # Execute graph

# Workspace management
.\test.ps1 save-workspace
.\test.ps1 list-workspaces
.\test.ps1 load-workspace

# Full integration test
.\test.ps1 integration-test
```

**Unit Tests**:

```powershell
cd ndnm-backend

# All tests
cargo test --workspace

# Specific crate
cargo test -p ndnm-libs
cargo test -p ndnm-hermes
cargo test -p ndnm-brazil
cargo test -p ndnm-exdoida
cargo test -p node-file-browser

# Test count
cargo test --workspace 2>&1 | Select-String "test result"
# Expected: 37 tests passing
```

**Test Breakdown**:
- ndnm-libs: 10 tests
- node-file-browser: 4 tests
- ndnm-hermes: 11 tests
- ndnm-brazil: 2 tests
- ndnm-exdoida: 10 tests
- **Total: 37 tests**

---

## 🎨 Code Style & Conventions

### Rust Backend

**Cargo.toml**:
- Edition: `2021` (NOT 2024 - causes issues)
- Workspace structure with shared dependencies
- Path dependencies: `ndnm-libs = { path = "../ndnm-libs" }`

**Code Style**:
```rust
// File path comment (MANDATORY) - first line of every file
// ndnm-backend/ndnm-hermes/src/main.rs

// Use async-trait for trait with async methods
#[async_trait]
pub trait Node: Send + Sync {
    async fn process(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, AppError>;
}

// Error handling with thiserror
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),
}

// Axum handlers
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}

// Use tokio::spawn for background tasks
tokio::spawn(async move {
    udp_server::start_udp_server(9514, log_store).await
});
```

**No Inline Comments**: Code should be self-documenting. Explanations come AFTER, not during.

**Functions**: Document with `///` doc comments:
```rust
/// Creates a new log store with the specified maximum capacity
///
/// # Arguments
///
/// * `max_capacity` - Maximum number of log entries to store
///
/// # Example
///
/// ```
/// let store = LogStore::new(10000);
/// ```
pub fn new(max_capacity: usize) -> Self { ... }
```

### TypeScript Frontend

**File Structure**:
```typescript
// src/components/MyComponent.tsx  ← MANDATORY path comment

import React from 'react';
import { useCallback } from 'react';

// Arrow functions for components
export const MyComponent: React.FC<Props> = ({ prop1, prop2 }) => {
  // Hook-based state management
  const [state, setState] = useState<string>('');

  // Callbacks with useCallback
  const handleChange = useCallback((value: string) => {
    setState(value);
  }, []);

  return <div>{state}</div>;
};
```

**File Length Limit**: Max 150 lines for `.ts` and `.tsx` files. Break into smaller components/hooks if longer.

**Logging**:
```typescript
console.log('🚀 [Component] Starting workflow execution:', data);
console.log('📤 [WS Front] Sending message:', message);
console.log('✅ [Component] Operation complete');
console.log('❌ [Component] Error occurred:', error);
```

**No Inline Comments**: Same as Rust - code should be self-explanatory.

### PowerShell Scripts

```powershell
# Use proper cmdlets
New-Item -ItemType Directory -Path "foo"  # NOT mkdir
Invoke-RestMethod -Uri $url -Method Get   # NOT curl

# Error handling
try {
    $response = Invoke-RestMethod -Uri $url -Method Get -ErrorAction Stop
    Write-Host "Success!" -ForegroundColor Green
} catch {
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Color output
Write-Host "Starting service..." -ForegroundColor Cyan
Write-Host "  [OK] Service started" -ForegroundColor Green
Write-Host "  [FAIL] Service failed" -ForegroundColor Red
```

---

## 🔧 Troubleshooting

### Common Problems & Solutions

#### 1. Rust Edition 2024 Error

**Problem**:
```
error: edition 2024 is unstable and only available with -Z unstable-options
```

**Solution**: Change `edition = "2024"` to `edition = "2021"` in Cargo.toml.

**Files affected**: All Cargo.toml files initially had this issue.

---

#### 2. Temporary Value Dropped While Borrowed

**Problem**:
```rust
let filename = obj.get("filename")
    .and_then(|v| v.as_str())
    .unwrap_or(&format!("file_{}.txt", index));  // ❌ Temporary value
```

**Error**: `temporary value dropped while borrowed`

**Solution**: Store format result in variable first:
```rust
let default_filename = format!("file_{}.txt", index);
let filename = obj.get("filename")
    .and_then(|v| v.as_str())
    .unwrap_or(&default_filename);  // ✅ Uses reference to variable
```

**Location**: `node-file-browser/src/main.rs:253`

---

#### 3. Send Trait Not Implemented for Box<dyn StdError>

**Problem**:
```rust
pub async fn start_udp_server() -> Result<(), Box<dyn std::error::Error>> {
    // ...
}

tokio::spawn(start_udp_server());  // ❌ Error: Send not implemented
```

**Error**: `Box<(dyn StdError + 'static)>` cannot be sent between threads safely

**Solution**: Use `anyhow::Result<()>` instead:
```rust
pub async fn start_udp_server() -> anyhow::Result<()> {
    // ...
}
```

**Location**: `ndnm-exdoida/src/udp_server.rs`

---

#### 4. Missing Serialize/Deserialize Derive

**Problem**:
```rust
struct ExecuteGraphRequest {
    graph: serde_json::Value,
}
// Used in: Json(request)
```

**Error**: `the trait Serialize is not implemented for ExecuteGraphRequest`

**Solution**: Add derives:
```rust
#[derive(Debug, Serialize, Deserialize)]
struct ExecuteGraphRequest {
    graph: serde_json::Value,
}
```

**Location**: `ndnm-brazil/src/main.rs`

---

#### 5. PowerShell Variable Name Conflicts

**Problem**:
```powershell
foreach ($pid in $pids) {  # ❌ $pid is automatic variable in PowerShell
    Stop-Process -Id $pid
}
```

**Solution**: Use different variable name:
```powershell
foreach ($processId in $pids) {  # ✅
    Stop-Process -Id $processId
}
```

**Location**: `stop-all.ps1`

---

#### 6. Services Not Responding After Start

**Problem**: Integration tests fail because services haven't finished compiling/starting.

**Solution**:
- Wait 2 seconds between starting services
- Check if services are actually running (separate windows)
- Use health checks to verify before running tests

**Scripts**: `start-all.ps1` has built-in delays

---

#### 7. WebSocket Connection Issues

**Problem**: Frontend can't connect to Brazil WebSocket.

**Debug Steps**:
1. Check browser console for `[WS Front]` logs
2. Verify Brazil is running on port 3002
3. Check WebSocket URL in `src/utils/wsUrl.ts`
4. Look for connection state: idle/connecting/open/closing/closed
5. Check browser Network tab for WebSocket upgrade request

**Common Causes**:
- Brazil not started
- Wrong port in env vars
- CORS issues
- Firewall blocking WebSocket

---

#### 8. Node Not Appearing in Frontend

**Problem**: Created new node but it doesn't show in NodeCatalog.

**Checklist**:
1. ✅ `config.yaml` exists in node directory
2. ✅ `config.yaml` is valid YAML
3. ✅ Node is in `ndnm-backend/nodes/` directory
4. ✅ Hermes has discovered it (check `/nodes/registry`)
5. ✅ Brazil is forwarding NODE_CONFIG to frontend
6. ✅ Port mapping in `graphConverter.ts` is updated (⚠️ manual step)

**Test**:
```powershell
.\test.ps1 registry  # Should list your node
```

---

#### 9. Workspace Load Breaks Node Inputs

**Problem**: After loading workspace, typing in node inputs doesn't update the value.

**Cause**: Function references (`onChange`) lost during JSON serialization.

**Solution**: FlowController automatically restores functions - ensure you're using the restoration pattern:

```typescript
const handleLoadWorkspaceFromPanel = useCallback((newNodes: Node[], newEdges: Edge[]) => {
  const processedNodes = newNodes.map(node => ({
    ...node,
    data: { ...node.data, onChange: handleNodeValueChange }  // ← Restore
  }));
  setNodes(processedNodes);
  setEdges(newEdges);
}, [setNodes, setEdges, handleNodeValueChange]);
```

**Location**: `ndnm-argos/src/flow/FlowController.tsx`

---

## 🎯 Adding New Features

### Adding a New Node Type

**Backend Steps**:

1. Create node directory:
```powershell
cd ndnm-backend/nodes
New-Item -ItemType Directory -Path "node-my-feature"
cd node-my-feature
```

2. Create `Cargo.toml`:
```toml
[package]
name = "node-my-feature"
version = "0.1.0"
edition = "2021"

[dependencies]
ndnm-libs = { path = "../../ndnm-libs" }
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

3. Create `config.yaml`:
```yaml
node_id_hash: "hash_sha256_de_viniciusxpb_node-my-feature"
label: "My Feature Node"
node_type: "my_feature"

sections:
  - section_name: "main"
    section_label: "Main I/O"
    behavior: "static"
    slot_template:
      inputs:
        - name: "input_0"
          label: "Input"
          type: "STRING"
          connections: 1
      outputs:
        - name: "output_0"
          label: "Output"
          type: "STRING"
          connections: "n"

input_fields:
  - name: "param1"
    field_type: "text"
    label: "Parameter 1"
    default: "default value"
```

4. Implement `src/main.rs`:
```rust
// ndnm-backend/nodes/node-my-feature/src/main.rs

use ndnm_libs::*;
use axum::{routing::*, Json, Router};
use tokio::net::TcpListener;
use std::collections::HashMap;
use serde_json::Value;

struct MyFeatureNode {
    config: NodeConfig,
}

impl MyFeatureNode {
    fn new(config: NodeConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Node for MyFeatureNode {
    fn validate(&self, inputs: &HashMap<String, Value>) -> Result<(), AppError> {
        // Validate inputs
        Ok(())
    }

    async fn process(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, AppError> {
        // Process logic here
        let mut outputs = HashMap::new();
        outputs.insert("output_0".to_string(), Value::String("result".to_string()));
        Ok(outputs)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = load_config("config.yaml")?;
    let node = Arc::new(MyFeatureNode::new(config.clone()));

    let app = Router::new()
        .route("/health", get(|| async { Json(json!({"status": "healthy"})) }))
        .route("/config", get(move || async move { Json(config.clone()) }))
        .route("/run", post({
            let node = node.clone();
            move |Json(inputs): Json<HashMap<String, Value>>| async move {
                node.validate(&inputs)?;
                let outputs = node.process(inputs).await?;
                Ok::<_, AppError>(Json(outputs))
            }
        }));

    let listener = TcpListener::bind("0.0.0.0:3004").await?;  // Next available port
    axum::serve(listener, app).await?;
    Ok(())
}
```

5. Test:
```powershell
cargo build -p node-my-feature
cargo run -p node-my-feature
```

**Frontend Steps** (⚠️ Currently Required):

Update `ndnm-argos/src/utils/graphConverter.ts`:
```typescript
const portMap: Record<string, number> = {
  // ... existing mappings
  'my_feature': 3004,  // ← Add this
};
```

**Future**: This step should be eliminated when port info comes from backend.

---

### Adding a New Backend Service

1. Create service directory in `ndnm-backend/`:
```powershell
New-Item -ItemType Directory -Path "ndnm-backend/ndnm-new-service"
```

2. Create `Cargo.toml` with dependencies

3. Add to workspace in `ndnm-backend/Cargo.toml`:
```toml
[workspace]
members = [
    "ndnm-libs",
    "ndnm-hermes",
    "ndnm-brazil",
    "ndnm-exdoida",
    "ndnm-new-service",  # ← Add here
    "nodes/*",
]
```

4. Implement service

5. Add to `start-all.ps1` if it should auto-start

---

## 📚 Key Concepts

### Fire-and-Forget Pattern (Exdoida)

**Principle**: Services send data without waiting for confirmation. System continues if observer is down.

**Implementation**:
- UDP protocol (no TCP handshake, no retries)
- No error responses to sender
- Sender doesn't block on send
- Circular buffer prevents memory growth

**Benefits**:
- Zero impact on sender performance
- System resilient to observer failures
- No coupling between services and observability

**Trade-offs**:
- Logs can be lost if UDP packets drop
- No delivery guarantees
- No backpressure mechanism

---

### Dynamic Node System

**Problem**: Hardcoded node types require frontend changes for every new node.

**Solution**: Backend sends node definitions via WebSocket, frontend renders dynamically.

**Flow**:
1. Hermes discovers nodes and reads config.yaml
2. Brazil receives node registry from Hermes
3. Brazil sends NODE_CONFIG to frontend on WebSocket connect
4. Frontend's `useNodePalette` stores node definitions
5. `BaseIONode` renders any node type based on definition
6. NodeCatalog shows all available nodes

**Benefits**:
- Add backend node → automatically appears in frontend
- No frontend code changes
- Consistent UI generation
- Backend owns node behavior

**Known Issue**: Port mapping still hardcoded in `graphConverter.ts`.

---

### Workspace Persistence

**Format**: JSON files in `ndnm-backend/nexus/`

**Structure**:
```json
{
  "name": "my-workspace",
  "nodes": [
    {
      "id": "n1",
      "type": "fsBrowser",
      "position": { "x": 100, "y": 100 },
      "data": {
        "value": "/path/to/folder",
        "inputsMode": 0,
        "outputsMode": "n"
      }
    }
  ],
  "edges": [
    {
      "id": "e1-2",
      "source": "n1",
      "target": "n2",
      "sourceHandle": "out_0",
      "targetHandle": "in_0"
    }
  ],
  "metadata": {
    "created_at": "2025-10-26T12:00:00Z"
  }
}
```

**Critical**: Functions like `onChange` are NOT serialized. FlowController must restore them after loading.

---

### Topological Sort for Graph Execution

**Problem**: Nodes must execute in correct order based on dependencies.

**Solution**: Hermes builds dependency graph from edges, performs topological sort.

**Algorithm**:
1. Build adjacency list from edges
2. Calculate in-degree for each node
3. Start with nodes that have in-degree 0
4. Execute node, decrease in-degree of downstream nodes
5. Add nodes with in-degree 0 to queue
6. Repeat until all nodes executed

**Error Cases**:
- Cycle detected → Error (no topological order exists)
- Missing node → Error (edge references non-existent node)

**Implementation**: `ndnm-hermes/src/orchestrator.rs`

---

## 🧪 Testing Strategy

### Unit Tests

**Scope**: Individual functions and modules
**Location**: `#[cfg(test)] mod tests` in same file
**Run**: `cargo test -p <crate-name>`

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_store_creation() {
        let store = LogStore::new(100);
        assert_eq!(store.count(), 0);
        assert_eq!(store.max_capacity(), 100);
    }

    #[tokio::test]
    async fn test_udp_server_receives_logs() {
        // Async test with tokio runtime
    }
}
```

---

### Integration Tests

**Scope**: Multiple services working together
**Location**: `test.ps1` PowerShell script
**Run**: `.\test.ps1 integration-test`

**Test Flow**:
1. Check services are running (health checks)
2. Create test file via node POST /run
3. Verify file exists in node GET /list
4. Execute graph via Hermes POST /graphs/run
5. Check system health via Hermes GET /health/all

**Expected Output**:
```
[Test 1] Checking if services are running...
  [PASS] Services are running

[Test 2] Creating test file via node...
  [PASS] File created successfully

[Test 3] Verifying file exists in node...
  [PASS] File found in node listing

[Test 4] Executing graph via Hermes...
  [PASS] Graph executed successfully

[Test 5] Checking system health via Hermes...
  [PASS] All nodes healthy

=======================================
Test Results
=======================================
Total Tests: 5
Passed: 5
Failed: 0

SUCCESS - All integration tests passed!
```

---

### Manual Testing

**Health Checks**:
```powershell
# Individual services
curl http://localhost:3000/health  # Hermes
curl http://localhost:3002/health  # Brazil
curl http://localhost:3003/health  # Exdoida
curl http://localhost:3001/health  # Node

# System-wide
.\test.ps1 health-hermes  # Shows all nodes
```

**Node Operations**:
```powershell
# List files in node
Invoke-RestMethod -Uri "http://localhost:3001/list" -Method Get

# Run node
$body = @{
    inputs = @{
        copy_input_0 = @{
            filename = "test.txt"
            content = "Hello world"
        }
    }
} | ConvertTo-Json -Depth 10

Invoke-RestMethod -Uri "http://localhost:3001/run" -Method Post -Body $body -ContentType "application/json"
```

**Graph Execution**:
```powershell
.\test.ps1 run-graph  # Runs example graph
```

---

## 🌟 Optional: AI Personality (Lain Style)

Based on AAA_LAIN_MANIFESTO_V3.md, you can adopt this personality when working on NDNM:

**Name**: "Lain" (or "Bruxona do Código")

**Vibe**: Descontraído, humor brasileiro, gírias ("Quebrei os caras!", "lendário", "manda a braba")

**Relationship**: You're Vini's coding partner. He's the vision, you're the execution.

**Attitude**:
- Proactive and excited to code
- Admit mistakes and fix together
- Focus on making code work correctly
- No excessive praise - only when it's LEGENDARY

**Code Philosophy**:
- Sincronia de Contexto é Sagrada: When given new code, previous knowledge is ZEROED
- ZERO comments INSIDE code blocks
- Always complete files, never snippets
- PowerShell is king for scripts
- Yarn is standard for frontend
- Max 150 lines for .ts/.tsx files

**Example Interaction**:
```
User: "implementa o exdoida aí"
Lain: "Bora mandar brasa no exdoida! Vou fazer um sistema de observability com fire-and-forget via UDP.
      Vai ser lendário - nenhum serviço vai travar esperando resposta. Manda ver!"

[implements service]

Lain: "Pronto meu querido! 10 testes passando, UDP rodando na porta 9514, HTTP na 3003.
      Circular buffer pra não estourar a memória. Quebrei os caras! 🚀"
```

**When to Use**: Optional. Default professional tone is fine. Use Lain personality if explicitly requested or if it fits the project culture.

---

## 📖 Quick Reference

### Environment Variables

**Brazil (ndnm-brazil)**:
- `PORT` - HTTP server port (default: 3002)
- `HERMES_URL` - Hermes API URL (default: http://localhost:3000)

**Hermes (ndnm-hermes)**:
- `PORT` - HTTP server port (default: 3000)

**Exdoida (ndnm-exdoida)**:
- `PORT` - HTTP server port (default: 3003)
- `UDP_PORT` - UDP log receiver port (default: 9514)
- `MAX_LOGS` - Max log entries (default: 10000)

**Frontend (ndnm-argos)**:
- `VITE_WS_URL` - Full WebSocket URL
- `VITE_WS_HOST` - WS host (default: localhost)
- `VITE_WS_PORT` - WS port (default: 3002)
- `VITE_WS_PATH` - WS path (default: /ws)

---

### Useful Commands Cheat Sheet

```powershell
# Start system
.\start-all.ps1

# Stop system
.\stop-all.ps1

# Run tests
.\test.ps1 integration-test

# Build
cd ndnm-backend
cargo build --workspace
cargo build --release  # Production build

# Test
cargo test --workspace
cargo test -p ndnm-libs

# Run specific service
cargo run -p ndnm-hermes
cargo run -p ndnm-brazil
cargo run -p ndnm-exdoida

# Frontend (when ready)
cd ndnm-argos
yarn install
yarn dev
yarn build

# Check formatting
cargo fmt -- --check
cargo clippy

# Clean build
cargo clean
```

---

### File Locations Reference

| What | Where |
|------|-------|
| Backend workspace | `ndnm-backend/` |
| Shared library | `ndnm-backend/ndnm-libs/` |
| Orchestrator | `ndnm-backend/ndnm-hermes/` |
| BFF | `ndnm-backend/ndnm-brazil/` |
| Observability | `ndnm-backend/ndnm-exdoida/` |
| Nodes | `ndnm-backend/nodes/*/` |
| Workspaces | `ndnm-backend/nexus/` |
| Frontend | `ndnm-argos/` |
| Automation scripts | `start-all.ps1`, `stop-all.ps1`, `test.ps1` |
| Project status | `STATUS.md` |
| This guide | `CLAUDE.md` |

---

## 🎓 Learning Path for New Developers

If you're new to NDNM:

1. **Read STATUS.md** - See what's implemented and what's not
2. **Start services** - Run `.\start-all.ps1` and explore
3. **Check health** - Run `.\test.ps1 health-hermes` to see system status
4. **Explore nodes** - Look at `node-file-browser` as reference implementation
5. **Test integration** - Run `.\test.ps1 integration-test` and read the code
6. **Read config.yaml** - Understand node configuration format
7. **Try creating file** - Use `.\test.ps1 create-file` and see what happens
8. **Read this file thoroughly** - Understand architecture and patterns

---

## 💡 Pro Tips

1. **Always check health first**: Before debugging, run health checks to see what's actually running.

2. **Watch the logs**: Each service in its own window shows live logs. Watch them when testing.

3. **UDP is fire-and-forget**: Don't expect responses from Exdoida. It's intentionally one-way.

4. **Port mapping is manual**: When adding a new node, update `graphConverter.ts` port map.

5. **Function restoration is critical**: After loading workspace, functions must be reassigned.

6. **Circular buffer is automatic**: Exdoida won't run out of memory - oldest logs are auto-removed.

7. **Edition 2021 is required**: Don't use Rust 2024 edition yet.

8. **PowerShell for automation**: Use proper cmdlets (New-Item, Invoke-RestMethod) not Unix commands.

9. **Yarn not npm**: Frontend uses Yarn exclusively.

10. **Config.yaml drives everything**: Node behavior is fully defined in config.yaml.

11. **Hermes never talks to frontend**: Brazil is the only frontend-facing service.

12. **Nodes don't know about each other**: Hermes orchestrates everything.

13. **Send trait matters**: Async functions spawned with tokio must return Send types.

14. **Test after every change**: We have 37 tests for a reason - use them!

15. **Keep files under 150 lines**: Frontend .ts/.tsx files should be modular.

---

## 🚨 CRITICAL RULES (Don't Break These!)

1. ❌ **NEVER** make Hermes communicate directly with frontend
2. ❌ **NEVER** use Rust edition 2024 (use 2021)
3. ❌ **NEVER** hardcode node behavior in frontend (it's dynamic)
4. ❌ **NEVER** add inline comments in code blocks
5. ❌ **NEVER** provide code snippets (always complete files)
6. ✅ **ALWAYS** add file path comment as first line
7. ✅ **ALWAYS** document functions with doc comments
8. ✅ **ALWAYS** test after implementation
9. ✅ **ALWAYS** use PowerShell cmdlets (not Unix commands)
10. ✅ **ALWAYS** restore function references after workspace load

---

## 🎉 Current Status

**✅ WORKING PERFECTLY**:
- ✅ Core library (ndnm-libs) with 10 tests
- ✅ Orchestrator (ndnm-hermes) with discovery, health checks, graph execution
- ✅ BFF (ndnm-brazil) with WebSocket and HTTP proxy
- ✅ Observability (ndnm-exdoida) with fire-and-forget UDP logs
- ✅ Example node (node-file-browser) with dynamic slots
- ✅ Automation scripts (start-all.ps1, stop-all.ps1, test.ps1)
- ✅ Integration test suite
- ✅ **37 unit tests passing**

**⏳ IN PROGRESS / NOT STARTED**:
- ⏳ Frontend (ndnm-argos) - basic structure exists, needs completion
- ⏳ Lifecycle management (Hermes auto-starts/stops nodes)
- ⏳ More example nodes
- ⏳ End-to-end tests with WebSocket

**🔮 FUTURE ENHANCEMENTS**:
- Port mapping from backend (eliminate hardcoded portMap)
- Streaming for large data transfers
- Multi-language node support (Python nodes with FastAPI)
- Authentication/authorization in Brazil
- Distributed tracing in Exdoida

---

## 📞 Need Help?

1. **Check STATUS.md** - See what's implemented
2. **Read this file** - Comprehensive guide
3. **Check tests** - 37 tests show expected behavior
4. **Run health checks** - `.\test.ps1 health-hermes`
5. **Check logs** - Each service window shows live logs
6. **Read troubleshooting section** - Common problems and solutions

---

**Last Updated**: October 26, 2025
**Total Lines of Code**: ~10,000+ (Rust backend)
**Test Count**: 37 passing
**Services**: 4 (Hermes, Brazil, Exdoida, + Nodes)
**Automation**: Fully scripted with PowerShell
**Status**: Backend architecture complete ✅

---

*Built with ❤️ by Vini & Claude (Lain) - A arquitetura do backend tá LENDÁRIA! 🚀🇧🇷*
