<p align="center">
  <img src="docs/assets/vortex-logo.svg" alt="VORTEX Logo" width="200"/>
</p>

<h1 align="center">VORTEX</h1>

<p align="center">
  <strong>Next-Generation AI Workflow Engine</strong><br>
  <em>A revolutionary node-based interface for generative AI</em>
</p>

<p align="center">
  <a href="https://www.somatech.dev">
    <img src="https://img.shields.io/badge/Developer-SOMATECH-blue?style=for-the-badge" alt="Developer">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-Apache%202.0-green?style=for-the-badge" alt="License">
  </a>
  <a href="#">
    <img src="https://img.shields.io/badge/Status-In%20Development-orange?style=for-the-badge" alt="Status">
  </a>
</p>

---

## 🚀 What is VORTEX?

**VORTEX** is an open-source, high-performance AI workflow execution engine designed to revolutionize how users interact with generative AI models. Built on the innovative **Centaur Architecture** (Rust control plane + Python compute fabric), VORTEX delivers unprecedented performance, security, and flexibility.

### Why VORTEX?

| Problem | VORTEX Solution |
|---------|----------------|
| **Slow Python GUIs** | Rust async control plane (10× faster) |
| **Memory copying overhead** | Zero-copy transport via Apache Arrow |
| **No security for custom nodes** | Seccomp sandbox + AST scanning |
| **Single-user only** | Real-time CRDT collaboration |
| **Unscalable node graphs** | WebGL rendering (10,000 nodes @ 60fps) |
| **"Works on my machine"** | Reproducible environments with lockfiles |

---

## ✨ Key Features

### 🎨 Visual Node-Based Workflow
- Intuitive drag-and-drop interface
- Smart auto-routing (no spaghetti wires)
- Semantic zoom (LOD rendering)
- Workflow embedding in PNG metadata

### ⚡ Blazing Fast Performance
- **Zero-copy memory transport** via 64GB shared memory arena
- **Incremental computation** - only re-execute changed nodes
- **Predictive VRAM management** - prevent OOM before it happens
- **LFU cache eviction** with future-use prediction

### 🔒 Enterprise-Grade Security
- **Seccomp sandboxing** for Python workers
- **AST security scanning** blocks malicious code
- **Network isolation** by default
- **Signed package verification**

### 👥 Real-Time Collaboration
- **CRDT-based sync** (Yjs) - conflict-free editing
- **Multi-cursor presence** - see collaborators in real-time
- **Shared execution results** - everyone sees outputs

### 📦 Modern Package Management
- **PubGrub resolver** (same as Dart/Cargo)
- **Lockfile reproducibility** - identical environments everywhere
- **Environment forking** - handle conflicting dependencies

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        VORTEX                               │
├─────────────────────────────────────────────────────────────┤
│   ┌─────────────┐                ┌─────────────────────┐   │
│   │  Frontend   │◄──────────────►│    Core Engine      │   │
│   │  (Svelte 5) │    WebSocket   │      (Rust)         │   │
│   └─────────────┘                └──────────┬──────────┘   │
│                                             │              │
│                                    Protobuf │ UDS          │
│                                             ▼              │
│   ┌─────────────────────────────────────────────────────┐  │
│   │              Python Compute Workers                 │  │
│   │   ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐   │  │
│   │   │Worker 1│  │Worker 2│  │Worker 3│  │Worker N│   │  │
│   │   └────┬───┘  └────┬───┘  └────┬───┘  └────┬───┘   │  │
│   └────────┼───────────┼───────────┼───────────┼────────┘  │
│            └───────────┴─────┬─────┴───────────┘           │
│                              ▼                             │
│   ┌─────────────────────────────────────────────────────┐  │
│   │          Shared Memory Arena (64GB Arrow)           │  │
│   └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| **Control Plane** | Rust, Tokio, Salsa |
| **Compute Fabric** | Python, PyTorch, CUDA |
| **Memory** | Apache Arrow, mmap |
| **Frontend** | Svelte 5, WebGL, Yjs |
| **IPC** | Protobuf, Unix Domain Sockets |
| **Database** | SQLite (SQLx) |
| **Package Manager** | PubGrub solver |

---

## 📦 Installation

> **Note**: VORTEX is currently in active development. Installation instructions will be updated upon first release.

### Requirements
- **OS**: Linux (kernel 5.15+) or macOS (13.0+)
- **RAM**: 16GB minimum, 32GB recommended
- **GPU**: NVIDIA (CUDA 12+) or Apple Silicon
- **Python**: 3.10+
- **Rust**: 1.75+

### Quick Start
```bash
# Clone the repository
git clone https://github.com/somatechlat/vortex.git
cd vortex

# Build the Rust core
cargo build --release

# Install Python dependencies
pip install -r requirements.txt

# Run VORTEX
./target/release/vortex
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [Agent Context](agent.md) | Quick-start for developers |
| [VIBE Rules](rules.md) | Coding standards |
| [Master SRS](docs/specs/00_master_srs.md) | System architecture |
| [Core Engine SRS](docs/specs/01_core_engine_srs.md) | Rust engine specs |
| [Frontend SRS](docs/specs/02_frontend_ui_srs.md) | UI specifications |
| [Compute Fabric SRS](docs/specs/03_compute_fabric_srs.md) | Worker specs |
| [Registry SRS](docs/specs/04_registry_srs.md) | Package manager specs |

---

## 🗺️ Roadmap

- [x] **Phase 0**: Complete SRS documentation (8,694 lines)
- [ ] **Phase 1**: Core engine implementation
- [ ] **Phase 2**: Frontend UI development
- [ ] **Phase 3**: Package registry
- [ ] **Phase 4**: Public beta release

---

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) before submitting PRs.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Follow the [VIBE Coding Rules](rules.md)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

---

## 📄 License

This project is licensed under the **Apache License 2.0** - see the [LICENSE](LICENSE) file for details.

```
Copyright 2026 SOMATECH (www.somatech.dev)

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

---

## 🔗 Links

- **Website**: [www.somatech.dev](https://www.somatech.dev)
- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/somatechlat/vortex/issues)

---

<p align="center">
  Made with ❤️ by <a href="https://www.somatech.dev">SOMATECH</a>
</p>
