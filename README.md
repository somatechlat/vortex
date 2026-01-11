# VORTEX-GEN 3.0 "Centaur"

VORTEX is a production‑grade, agentic workflow engine for generative AI. It combines a Rust control plane, Python inference workers, and a vertical rack UI that compiles to a DAG for execution. The system is local‑first **and** cluster‑capable, with strong governance and auditability built in.

License: Apache 2.0.

## What This Project Is

- An MCP‑style tool box for humans and AI agents.
- A template‑driven workflow system (templates → runs → artifacts).
- A governance‑first platform with human‑in‑the‑loop approvals.

## Why It’s Different

- **Rack‑first UX**: vertical rack + signal bus (no infinite canvas).
- **Zero‑copy execution**: Apache Arrow shared memory.
- **Server‑enforced policy**: approvals and authZ are not UI‑only.
- **Cluster‑ready**: CPU control plane + GPU worker fabric.

## Architecture (High‑Level)

```
UI (Svelte 5)  <->  Core Engine (Rust)  <->  Worker Fabric (Python)
   |                      |                        |
   | HTTP/WS              | Protobuf + UDS/TLS      | PyTorch + Arrow
   v                      v                        v
Vertical Rack         Graph Compiler           GPU/CPU Execution
                      + Scheduler              + SHM (local to worker)
```

## Documentation (Source of Truth)

All requirements live in the SRS:
- `gemini/docs/specs/00_master_srs.md`
- `gemini/docs/specs/01_core_engine_srs.md`
- `gemini/docs/specs/02_frontend_ui_srs.md`
- `gemini/docs/specs/03_compute_fabric_srs.md`
- `gemini/docs/specs/04_registry_srs.md`
- `gemini/docs/specs/05_ui_screens_spec.md`
- `gemini/docs/specs/06_environment_config_spec.md`
- `gemini/docs/specs/06_enterprise_infrastructure_srs.md`
- `gemini/docs/specs/07_data_flow_spec.md`
- `gemini/docs/specs/08_workspace_tools_spec.md`

Project overview and rules:
- `gemini/agent.md`
- `gemini/rules.md`

## Status

SRS complete, implementation in progress.

## License

Apache License 2.0. See `LICENSE`.
