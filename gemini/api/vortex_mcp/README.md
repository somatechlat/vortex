# VORTEX MCP Server

Model Context Protocol server for VORTEX AI Workflow Engine.

## Setup

```bash
cd api/vortex_mcp
pip install -r requirements.txt
```

## Run

```bash
# Development
uvicorn vortex_mcp.asgi:application --host 0.0.0.0 --port 11190 --reload

# Production
uvicorn vortex_mcp.asgi:application --host 0.0.0.0 --port 11190 --workers 4
```

## MCP Inspector

```bash
python manage.py mcp_inspector http://localhost:11190/mcp/sse
```

## Available Tools

- `create_workflow` - Create workflow from template
- `execute_workflow` - Execute workflow with parameters
- `get_execution_status` - Check execution progress
- `list_templates` - List available templates
- `approve_gate` - Approve/reject HITL gates
- `cancel_execution` - Cancel running execution
- `get_artifact` - Get artifact URLs

## Environment Variables

- `VORTEX_CORE_API_URL` - VORTEX Core API URL (default: http://localhost:11188)
- `POSTGRES_HOST` - PostgreSQL host
- `POSTGRES_DB` - Database name
- `POSTGRES_USER` - Database user
- `POSTGRES_PASSWORD` - Database password
