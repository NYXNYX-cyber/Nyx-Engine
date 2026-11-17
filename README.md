# Nyx Engine 🦀🎮

> **A deterministic, server-authoritative 2D & 3D game engine architecture built natively in Rust.**

Nyx Engine is designed for high-concurrency, low-latency networked simulation, zero garbage-collection jitter, and retro 3D visual authoring with embedded WebGL2 / GLSL shaders.

---

## 🏛️ Architectural Highlights

- **🦀 Pure Rust Core (`engine-core`):** Deterministic mathematical simulation, vector arithmetic, tick loop invariants with zero heap allocation churn.
- **⚡ Server-Authoritative Networking (`engine-server`):** Built with Tokio and Axum over high-throughput WebSockets running at a fixed 60Hz tick rate (< 1ms IPC latency).
- **🕹️ Unity 6 Standard Workstation IDE (`web/`):**
  - **Dual Viewports:** Free-fly Scene View with 3-axis interactive transform gizmos + Game View pointer-locked playtesting.
  - **Retro 3D & GLSL Pipeline:** Live shader compilation, affine texture mapping, Bayer matrix dithering, CRT scanlines, and volumetric deep-sea depth fog.
  - **DEEP: BELOW Prototype:** Psychological horror expedition prototype at 3,800m RMS Titanic wreck.
- **🔒 Persistence & Auth Vault:** Embedded SQLite with Argon2 password hashing and JWT token authentication.

---

## 📂 Project Structure

```text
nyx-engine/
├── Cargo.toml                  # Cargo workspace definition
├── crates/
│   ├── engine-core/            # Shared simulation math & state
│   │   └── src/lib.rs
│   └── engine-server/          # Axum HTTP + Tokio 60Hz WebSocket server
│       └── src/main.rs
├── data/                       # Embedded SQLite database storage
└── web/                        # 3D WebGL2 / Three.js Workstation IDE
    ├── assets/                 # Engine brand marks & emblems
    ├── libs/                   # Three.js & client dependencies
    └── index.html              # Full Studio Editor UI
```

---

## 🚀 Quick Start

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (v1.75+ recommended)

### Build & Run Engine Server
```bash
# Clone the repository
git clone https://github.com/NYXNYX-cyber/Nyx-Engine.git
cd Nyx-Engine

# Run the backend server
cargo run -p engine-server --release
```

Server listens on `http://127.0.0.1:4040`. Open your browser to access the Nyx Studio 3D IDE!

---

## 🗺️ Roadmap & Features
- [x] Workspace Cargo setup (`engine-core`, `engine-server`)
- [x] Deterministic 60Hz loop with Tokio
- [x] WebGL2 & Three.js 3D viewport with 3-axis transform gizmos
- [x] GLSL shader editor with live recompilation
- [x] Argon2 password hashing & JWT auth session
- [ ] Headless Rapier3D physics integration
- [ ] WebAssembly (`wasm32-unknown-unknown`) client standalone builds

---

## 📜 License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
