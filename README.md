<div align="center">
  <img src="web/assets/nyx-emblem.png" width="128" height="128" alt="Nyx Logo" />
  <h1>Nyx Engine 🦀🎮</h1>
  <p><strong>A deterministic, server-authoritative 2D & 3D game engine runtime engineered natively in Rust.</strong></p>
  
  <p>
    <a href="https://github.com/NYXNYX-cyber/Nyx-Engine/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square" alt="Build" /></a>
    <a href="https://crates.io"><img src="https://img.shields.io/badge/rust-1.80%2B-orange?style=flat-square&logo=rust" alt="Rust" /></a>
    <a href="https://github.com/dimforge/rapier"><img src="https://img.shields.io/badge/physics-Rapier3D_0.22-blue?style=flat-square" alt="Rapier3D" /></a>
    <a href="https://tokio.rs"><img src="https://img.shields.io/badge/async-Tokio_60Hz-purple?style=flat-square" alt="Tokio" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square" alt="License" /></a>
    <a href="BENCHMARKS.md"><img src="https://img.shields.io/badge/benchmarks-42M_entities%2Fsec-success?style=flat-square" alt="Benchmarks" /></a>
  </p>

  <p>
    <a href="https://nyxstudios.site">Web Portal</a> •
    <a href="BENCHMARKS.md">Benchmarks</a> •
    <a href="#-quick-start">Quick Start</a> •
    <a href="#-architectural-highlights">Architecture</a> •
    <a href="#-features--roadmap">Roadmap</a>
  </p>
</div>

---

## 🏛️ Architectural Highlights

- **🦀 Pure Rust Core (`crates/engine-core`):** Deterministic mathematical simulation, transform matrices, and tick loop invariants with zero heap allocation churn.
- **🎱 Headless Rapier3D Physics (`crates/engine-core/src/physics.rs`):** 60Hz server-side physics pipeline with **Continuous Collision Detection (CCD)**, rigid bodies, and static/dynamic colliders.
- **⚡ Server-Authoritative Networking (`crates/engine-server`):** Built with Tokio and Axum over high-throughput WebSockets running at a fixed 60Hz tick rate (< 1ms IPC latency).
- **🕹️ Unity 6 Standard Workstation IDE (`web/`):**
  - **Dual Viewports:** Free-fly Scene View with 3-axis interactive transform gizmos + Game View pointer-locked playtesting.
  - **Retro 3D & GLSL Pipeline:** Live shader compilation, affine texture mapping, Bayer matrix dithering, CRT scanlines, and volumetric deep-sea depth fog.
  - **DEEP: BELOW Prototype:** Psychological horror expedition prototype at 3,800m RMS Titanic wreck.
- **🔒 Persistence & Auth Vault:** Embedded SQLite with Argon2 password hashing and JWT token authentication.

---

## 📂 Project Structure

```text
Nyx-Engine/
├── Cargo.toml                  # Cargo workspace definition
├── crates/
│   ├── engine-core/            # Shared simulation math & Rapier3D physics
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── physics.rs      # Rapier3D continuous collision detection
│   └── engine-server/          # Axum HTTP + Tokio 60Hz WebSocket server
│       ├── Cargo.toml
│       └── src/
│           ├── auth.rs         # Argon2 & JWT authentication
│           ├── db.rs           # SQLite embedded persistence
│           └── main.rs
├── data/                       # Embedded SQLite storage (gitignored)
└── web/                        # 3D WebGL2 / Three.js Workstation IDE
    ├── assets/                 # Engine brand marks & emblems
    ├── libs/                   # Three.js & client dependencies
    └── index.html              # Full Studio Editor UI
```

---


## 📊 Verified Benchmarks & Stress Tests

Empirical performance metrics benchmarked natively on an Intel Core i5-3470 (4 Cores @ 3.20GHz, 16GB RAM) running Debian 13 Linux:

| Metric Category | Stress Test Result | Budget / Target | Status |
|:----------------|:------------------|:---------------|:------:|
| **Simulation Throughput** | **42,166,907 Entities / sec** | High-throughput 2D/3D | ✔ Verified |
| **Rapier3D Physics Scale** | **8.30 ms @ 5,000 Rigidbodies** | 16.66 ms frame budget | **49.8% Util (50% headroom)** |
| **Active Player Capacity** | **4.74 ms @ 100,000 Entities** | 60Hz tick rate | **71.5% Headroom** |
| **WebSocket Concurrency** | **500 Concurrent Clients** | 0.00% Packet Loss | ✔ Rock Solid |
| **Runtime Memory Footprint** | **< 42 MB RAM under load** | Embedded & Server host | ✔ Zero GC Leaks |

> 📖 **Read the full benchmark report & methodology in [BENCHMARKS.md](BENCHMARKS.md).**

---

## 🚀 Quick Start

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (v1.75+ recommended)

### Build & Run Engine Server
```bash
# Clone the repository
git clone https://github.com/NYXNYX-cyber/Nyx-Engine.git
cd Nyx-Engine

# Run unit tests (including Rapier3D physics validation)
cargo test -p engine-core

# Launch the backend engine server
cargo run -p engine-server --release
```

The engine server listens on `http://127.0.0.1:4040`. Open your browser to access the Nyx Studio 3D IDE!

---

## 🗺️ Features & Roadmap
- [x] Workspace Cargo modular setup (`engine-core`, `engine-server`)
- [x] Deterministic 60Hz loop with Tokio WebSockets
- [x] **Headless Rapier3D physics pipeline with Continuous Collision Detection (CCD)**
- [x] WebGL2 & Three.js 3D viewport with 3-axis transform gizmos
- [x] GLSL shader editor with live recompilation
- [x] Argon2 password hashing & JWT auth session
- [ ] Headless physics joint constraints & character controller
- [ ] WebAssembly (`wasm32-unknown-unknown`) client standalone builds

---

## 📜 License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
