# cpp-2-rust-playground (`RustC-Playground`)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![CI/CD](https://github.com/haseeb-heaven/cpp-2-rust-playground/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/haseeb-heaven/cpp-2-rust-playground/actions/workflows/ci.yml)
[![C Engine: c2rust](https://img.shields.io/badge/C%20Engine-immunant%2Fc2rust-orange.svg)](https://github.com/immunant/c2rust)
[![C++ Engine: cpp2rust](https://img.shields.io/badge/C%2B%2B%20Engine-Cpp2Rust%2Fcpp2rust-purple.svg)](https://github.com/Cpp2Rust/cpp2rust)
[![Built with Gemini Argon & Carbon](https://img.shields.io/badge/Built%20With-Gemini%20Argon%20%26%20Carbon-4285F4.svg)](#credits--acknowledgments)

Repository: https://github.com/haseeb-heaven/cpp-2-rust-playground

An interactive web IDE and compiler wrapper built with **Rust (Axum)** and **React + TypeScript + Vite** for translating **C** and **C++** code to Rust, inspecting diagnostics and line mappings, and compiling/running both source and generated Rust locally or online.

## Backbone Transpiler Engines

`cpp-2-rust-playground` acts as a web wrapper around two language-specific backbone transpilers:

| Source Language | Backbone Engine | CLI Dispatch | Fallback Engine |
|---|---|---|---|
| **C (C99 / C11)** | [`immunant/c2rust`](https://github.com/immunant/c2rust) | Invokes `c2rust transpile compile_commands.json --output-dir <tmp>` when `C2RUST_BIN` or `c2rust` is installed | Built-in C2Rust-aligned C99/C11 -> Safe Rust transpiler |
| **C++ (C++17 / STL)** | [`Cpp2Rust/cpp2rust`](https://github.com/Cpp2Rust/cpp2rust) | Invokes `cpp2rust --file=<input.cpp> -o=<output.rs>` when `CPP2RUST_BIN` or `cpp2rust` is installed | Built-in Cpp2Rust-aligned C++ -> Safe Rust transpiler |

### Installing External Backbone CLIs (Optional)

By default, the backend works out-of-the-box with zero external transpiler setup via the built-in safe transpiler engines. To also wire up the external `c2rust` and `cpp2rust` CLI binaries:

```bash
./scripts/setup-engines.sh
# Or point explicitly to built binaries:
export C2RUST_BIN="/path/to/c2rust"
export CPP2RUST_BIN="/path/to/cpp2rust"
```

## Features

- **Dual Conversion Modes**: **Live Mode** (debounced line-by-line transpilation skipping incomplete cursor lines) and **Manual Mode** (on-demand via **Convert**), preserving manual Rust edits via incremental merging.
- **Dual Execution Providers**: **Local** (`gcc`/`clang`, `g++`/`clang++`, `rustc`) and **Online** (JDoodle Live API via `.env`), with separate **Compile** and **Run** buttons for C, C++, and Rust, plus `stdin` support, exit codes, memory, CPU time, and diagnostics.
- **Pre-Made Examples**: 31 executable examples spanning C/C++ basics, STL (`Vec`, `HashMap`, `HashSet`, `VecDeque`, `Option`, generics), C structs/unions/linked lists, game-engine snippets (Quake III, Wolfenstein 3D, DOOM, Quake, SM64), `<float.h>` limits, and pure ISO C99/C11 constructs (`_Generic`, flexible array members, VLAs, compound literals, `restrict`/`[static N]`).
- **Interactive UI**: Resizable/swappable CodeMirror 6 split-screen panels, Diff View, Rust formatter, `.c`/`.cpp`/`.rs` import/export, local storage workspace persistence, themes (`Dark`, `Midnight`, `Light`), and keyboard shortcuts.

## Branching Model

- **`master`**: Stable release branch — updated only when new versions are released.
- **`develop`**: Default active development and integration branch.
- **`feature`**: Active feature branch for staging incremental work before merging into `develop`.

## Setup & Run

1. Copy `.env.example` to `.env` and configure JDoodle credentials (optional if using Local mode):
   ```bash
   cp .env.example .env
   ```
2. Start backend (`http://127.0.0.1:8080`) and frontend (`http://localhost:5173`):
   ```bash
   ./start.sh
   ```

## Testing & CI/CD

- **Backend unit, integration, and 31-example equivalence tests**:
  ```bash
  cd backend && cargo test
  ```
- **Frontend unit/component tests & build**:
  ```bash
  cd frontend && npm test && npm run build
  ```
- **Playwright End-to-End tests** (with `./start.sh` running):
  ```bash
  cd frontend && npm run test:e2e
  ```

## Credits & Acknowledgments

- **Creator & Lead Developer**: [Haseeb Mir (`@haseeb-heaven`)](https://github.com/haseeb-heaven)
- **C → Rust Backbone Transpiler**: [`immunant/c2rust`](https://github.com/immunant/c2rust) — C99-compliant C-to-Rust transpiler architecture
- **C++ → Rust Backbone Transpiler**: [`Cpp2Rust/cpp2rust`](https://github.com/Cpp2Rust/cpp2rust) — AST-driven C++ to Safe Rust translator and rule system
- **Testing & Review Tooling**: [`openqodex/openqodex`](https://github.com/openqodex/openqodex) and [Playwright](https://playwright.dev/)
- **AI Co-Contributors**: **Antigravity** (Google DeepMind) powered by **Gemini Argon** (Planning & Architecture) and **Gemini Carbon** (Execution & Verification)

## License

This project is licensed under the **MIT License** — see the [LICENSE](./LICENSE) file for details and third-party notices for upstream backbone transpilers.
