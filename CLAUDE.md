# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NDNM is a node-based data manipulation system built in Rust, following a modular architecture with three core backend services and a dynamic node execution system. The project is organized as a monorepo with:
- `ndnm-backend/`: Rust workspace containing all backend services
- `ndnm-argos/`: Frontend (currently empty)

## Architecture

### Core Services (Backend)

The backend follows a strict layered architecture where each service has specific responsibilities:

#### 1. `ndnm-libs` - Shared Library Foundation
- **Type**: Rust library crate (`--lib`)
- **Purpose**: Provides core abstractions and types used across all services
- **Key exports**:
  - `trait Node`: Interface that all executable nodes must implement (defines `validate` and `process` methods)
  - `AppError` enum: Standardized error handling across the system
  - Configuration structures for `config.yaml` parsing:
    - `NodeConfig`: Root configuration (contains `node_id_hash`, `label`, `node_type`, `sections`, `input_fields`)
    - `Section`: Defines I/O slot groups with behaviors (`auto_increment`, `dynamic_per_file`)
    - `SlotTemplate`: Describes paired input/output handles with connection constraints
    - `InputFieldConfig`: Internal node UI controls
  - `load_config`: Function to parse and validate node YAML configurations
- **Usage**: All other crates depend on this via `ndnm-libs = { path = "../ndnm-libs" }`

#### 2. `ndnm-hermes` - Control Plane & Orchestrator
- **Type**: Executable service
- **Purpose**: The "maestro" that discovers, manages, and orchestrates node execution
- **Key responsibilities**:
  - **Node discovery**: Scans `nodes/` directory, parses `config.yaml` files with full structure validation
  - **Dynamic handle management**: Interprets section behaviors (e.g., `dynamic_per_file` creates slots per file found)
  - **Graph execution**: Analyzes graph definitions, resolves dependencies, executes nodes sequentially/parallel
  - **Data flow**: Maps output handles to input handles, manages potentially large data transfers (tensors, files)
  - **Port management**: Assigns and manages communication ports for nodes (nodes no longer define their own ports)
  - **API**: Exposes internal API for `ndnm-brazil` (e.g., `POST /graphs/run`, `GET /nodes/registry`, `POST /nexus/save`)
- **Critical rule**: NEVER communicates directly with frontend. All communication goes through `ndnm-brazil`
- **Tech**: Axum, Tokio, Serde (with `serde_yaml`), Reqwest

#### 3. `ndnm-brazil` - Backend-for-Frontend (BFF)
- **Type**: Executable service
- **Purpose**: Translation layer between frontend (`ndnm-argos`) and backend (`ndnm-hermes`)
- **Key responsibilities**:
  - **WebSocket server**: Maintains persistent connection with `ndnm-argos`
  - **Config distribution**: Fetches rich node structures from `ndnm-hermes` and sends to frontend for dynamic UI rendering
  - **Command translation**: Converts frontend actions to `ndnm-hermes` API calls
  - **State relay**: Forwards granular execution updates from `ndnm-hermes` to frontend
  - **Authentication/Authorization**: Designated location for future auth logic
- **Tech**: Axum, Tokio, Tokio-Tungstenite (WebSocket), Reqwest

#### 4. `ndnm-exdoida` - Observability Service
- **Type**: Executable service
- **Purpose**: Independent observer for logs, metrics, and traces
- **Key principle**: **Highly decoupled and resilient** - system continues if this service fails
- **Responsibilities**: Collects observability data via fire-and-forget mechanisms (UDP, log files)
- **NOT responsible for**: System operation (other services don't depend on it)
- **Tech**: Axum (optional API), Tokio, Tracing/Log crates

### Nodes System

Individual nodes live in `ndnm-backend/nodes/` and are discovered dynamically by `ndnm-hermes`.

#### Node Configuration (`config.yaml`)
Each node has a `config.yaml` defining its interface structure:
- **No port definition**: `ndnm-hermes` manages all ports
- **Sections**: Groups of I/O slots with specific behaviors:
  - `auto_increment`: UI adds more slots as connections are made
  - `dynamic_per_file`: System generates slot pairs per file found
- **Slot templates**: Define paired input/output handles with:
  - `name`: Base name (e.g., `copy_input`, gets numbered as `copy_input_0`, `copy_input_1`)
  - `label`: UI display (supports placeholders like `{filename}`)
  - `type`: Data type (`FILE_CONTENT`, `STRING`, `NUMBER`)
  - `connections`: `1` for single connection, `"n"` for unlimited
- **Input fields**: Internal node controls (text inputs, buttons)

Example node: `node-file-browser` manages a directory with two sections:
1. `copy_here`: Dynamic input slots to copy files into managed directory
2. `internal_files`: Dynamic slots per existing file for read/overwrite operations

## Development Commands

### Building
```bash
cd ndnm-backend
cargo build                    # Build all workspace members
cargo build --release          # Production build
cargo build -p ndnm-hermes     # Build specific crate
```

### Testing
```bash
cd ndnm-backend
cargo test                     # Run all tests
cargo test -p ndnm-libs        # Test specific crate
cargo test --lib               # Test library code only
```

### Running
```bash
cd ndnm-backend
cargo run -p ndnm-hermes       # Run the orchestrator
cargo run -p ndnm-brazil       # Run the BFF
cargo run -p ndnm-exdoida      # Run observability service
```

### Linting and Formatting
```bash
cd ndnm-backend
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting without changes
cargo clippy                   # Run linter
cargo clippy -- -D warnings    # Treat warnings as errors
```

### Workspace Management
```bash
cd ndnm-backend
cargo check                    # Fast check without building
cargo clean                    # Clean build artifacts
cargo update                   # Update dependencies
```

## Key Architectural Principles

1. **Strict Service Boundaries**:
   - `ndnm-hermes` never communicates with frontend
   - `ndnm-brazil` is the ONLY frontend-facing service
   - Nodes never orchestrate themselves
   - `ndnm-exdoida` is passive and decoupled

2. **Configuration-Driven Dynamic UI**:
   - Nodes define their interface declaratively in `config.yaml`
   - Frontend renders UI based on config structure
   - No hardcoded node-specific UI logic in frontend
   - Section behaviors enable dynamic slot generation

3. **Data Flow**:
   - Large data transfers (files, tensors) may use streaming or temp file references
   - Output handles map directly to connected input handles
   - Dynamic handle naming follows pattern: `{base_name}_{index}` or `{base_name}_{filename}`

4. **Workspace Persistence**:
   - Saved in `ndnm-backend/nexus/` directory
   - JSON format
   - Managed by `ndnm-hermes`

## Rust Workspace Structure

This is a Cargo workspace (`ndnm-backend/Cargo.toml`) with resolver "2":
- Workspace members share `Cargo.lock`
- `ndnm-libs` is a common dependency for all other crates
- Use path dependencies: `ndnm-libs = { path = "../ndnm-libs" }`
- Nodes in `nodes/*` are automatically included via `"nodes/*"` glob

## Toolchain

- Rust Edition: 2024
- Cargo: 1.89.0+
- Rustc: 1.89.0+
