# NDNM Testing Script
# Usage: .\test.ps1 [command]

param(
    [Parameter(Position=0)]
    [string]$Command = "help"
)

$HermesUrl = "http://localhost:3000"
$NodeUrl = "http://localhost:3001"

function Show-Help {
    Write-Host ""
    Write-Host "NDNM Test Commands" -ForegroundColor Cyan
    Write-Host "==================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Health Checks:" -ForegroundColor Yellow
    Write-Host "  .\test.ps1 health-node     - Check node-file-browser health"
    Write-Host "  .\test.ps1 health-hermes   - Check hermes health"
    Write-Host ""
    Write-Host "Registry:" -ForegroundColor Yellow
    Write-Host "  .\test.ps1 registry        - Get all registered nodes"
    Write-Host "  .\test.ps1 node-config     - Get node config directly"
    Write-Host ""
    Write-Host "Node Operations:" -ForegroundColor Yellow
    Write-Host "  .\test.ps1 create-file     - Create a test file via node"
    Write-Host "  .\test.ps1 list-files      - List files in node directory"
    Write-Host ""
    Write-Host "Graph Execution:" -ForegroundColor Yellow
    Write-Host "  .\test.ps1 run-graph       - Execute a simple graph"
    Write-Host ""
    Write-Host "Workspace:" -ForegroundColor Yellow
    Write-Host "  .\test.ps1 save-workspace  - Save a test workspace"
    Write-Host "  .\test.ps1 list-workspaces - List all workspaces"
    Write-Host "  .\test.ps1 load-workspace  - Load test workspace"
    Write-Host ""
}

function Test-HealthNode {
    Write-Host "Checking node-file-browser health..." -ForegroundColor Cyan
    try {
        $response = Invoke-RestMethod -Uri "$NodeUrl/health" -Method Get
        Write-Host "Success - Node is healthy!" -ForegroundColor Green
    } catch {
        Write-Host "Error - Node is not responding" -ForegroundColor Red
        Write-Host "  Make sure it's running in ndnm-backend/nodes/node-file-browser" -ForegroundColor Yellow
    }
}

function Test-HealthHermes {
    Write-Host "Checking hermes health..." -ForegroundColor Cyan
    try {
        $response = Invoke-RestMethod -Uri "$HermesUrl/health" -Method Get
        Write-Host "Success - Hermes is healthy!" -ForegroundColor Green
    } catch {
        Write-Host "Error - Hermes is not responding" -ForegroundColor Red
        Write-Host "  Make sure it's running with: cargo run -p ndnm-hermes" -ForegroundColor Yellow
    }
}

function Get-Registry {
    Write-Host "Fetching node registry..." -ForegroundColor Cyan
    try {
        $response = Invoke-RestMethod -Uri "$HermesUrl/nodes/registry" -Method Get
        Write-Host ""
        Write-Host "Registered Nodes:" -ForegroundColor Green
        foreach ($node in $response.nodes) {
            Write-Host "  - $($node.config.label)" -ForegroundColor White
            Write-Host "    ID: $($node.node_id)" -ForegroundColor Gray
            Write-Host "    Port: $($node.port)" -ForegroundColor Gray
            Write-Host "    Sections: $($node.config.sections.Count)" -ForegroundColor Gray
        }
        Write-Host ""
        Write-Host "Full JSON:" -ForegroundColor Yellow
        $response | ConvertTo-Json -Depth 10
    } catch {
        Write-Host "Error - Failed to get registry" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
    }
}

function Get-NodeConfig {
    Write-Host "Fetching node config..." -ForegroundColor Cyan
    try {
        $response = Invoke-RestMethod -Uri "$NodeUrl/config" -Method Get
        $response | ConvertTo-Json -Depth 10
    } catch {
        Write-Host "Error - Failed to get config" -ForegroundColor Red
    }
}

function Create-TestFile {
    Write-Host "Creating test file..." -ForegroundColor Cyan
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $body = @{
        inputs = @{
            copy_input_0 = @{
                filename = "test_$timestamp.txt"
                content = "Hello from NDNM! Created at $(Get-Date)"
            }
        }
    } | ConvertTo-Json -Depth 10

    try {
        $response = Invoke-RestMethod -Uri "$NodeUrl/run" -Method Post -Body $body -ContentType "application/json"
        Write-Host "Success - File created!" -ForegroundColor Green
        $response | ConvertTo-Json -Depth 10
    } catch {
        Write-Host "Error - Failed to create file" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
    }
}

function List-Files {
    Write-Host "Listing files in node directory..." -ForegroundColor Cyan
    try {
        $response = Invoke-RestMethod -Uri "$NodeUrl/list" -Method Get
        Write-Host ""
        Write-Host "Files:" -ForegroundColor Green
        foreach ($file in $response.files) {
            Write-Host "  - $file" -ForegroundColor White
        }
    } catch {
        Write-Host "Error - Failed to list files" -ForegroundColor Red
    }
}

function Run-TestGraph {
    Write-Host "Executing test graph..." -ForegroundColor Cyan
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

    try {
        $response = Invoke-RestMethod -Uri "$HermesUrl/graphs/run" -Method Post -Body $body -ContentType "application/json"
        Write-Host "Success - Graph executed!" -ForegroundColor Green
        $response | ConvertTo-Json -Depth 10
    } catch {
        Write-Host "Error - Graph execution failed" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
    }
}

function Save-TestWorkspace {
    Write-Host "Saving test workspace..." -ForegroundColor Cyan
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $body = @{
        name = "test_workspace_$timestamp"
        data = @{
            graph = @{
                nodes = @(
                    @{
                        instance_id = "node1"
                        node_type_id = "hash_sha256_de_viniciusxpb_node-file-browser"
                        input_values = @{}
                    }
                )
                connections = @()
            }
            metadata = @{
                description = "Test workspace created by script"
                created_by = "test_script"
                created_at = (Get-Date -Format "yyyy-MM-ddTHH:mm:ss")
            }
        }
    } | ConvertTo-Json -Depth 10

    try {
        Invoke-RestMethod -Uri "$HermesUrl/nexus/save" -Method Post -Body $body -ContentType "application/json"
        Write-Host "Success - Workspace saved!" -ForegroundColor Green
    } catch {
        Write-Host "Error - Failed to save workspace" -ForegroundColor Red
    }
}

function List-Workspaces {
    Write-Host "Listing workspaces..." -ForegroundColor Cyan
    try {
        $response = Invoke-RestMethod -Uri "$HermesUrl/nexus/list" -Method Get
        Write-Host ""
        Write-Host "Workspaces:" -ForegroundColor Green
        foreach ($ws in $response.workspaces) {
            Write-Host "  - $ws" -ForegroundColor White
        }
    } catch {
        Write-Host "Error - Failed to list workspaces" -ForegroundColor Red
    }
}

function Load-TestWorkspace {
    Write-Host "Loading workspace..." -ForegroundColor Cyan

    try {
        $response = Invoke-RestMethod -Uri "$HermesUrl/nexus/list" -Method Get
        if ($response.workspaces.Count -gt 0) {
            $wsName = $response.workspaces[0]
            Write-Host "Loading: $wsName" -ForegroundColor Yellow

            $data = Invoke-RestMethod -Uri "$HermesUrl/nexus/load/$wsName" -Method Get
            Write-Host "Success - Workspace loaded!" -ForegroundColor Green
            $data | ConvertTo-Json -Depth 10
        } else {
            Write-Host "No workspaces found. Create one first with: .\test.ps1 save-workspace" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "Error - Failed to load workspace" -ForegroundColor Red
    }
}

# Execute command
switch ($Command.ToLower()) {
    "health-node" { Test-HealthNode }
    "health-hermes" { Test-HealthHermes }
    "registry" { Get-Registry }
    "node-config" { Get-NodeConfig }
    "create-file" { Create-TestFile }
    "list-files" { List-Files }
    "run-graph" { Run-TestGraph }
    "save-workspace" { Save-TestWorkspace }
    "list-workspaces" { List-Workspaces }
    "load-workspace" { Load-TestWorkspace }
    default { Show-Help }
}
