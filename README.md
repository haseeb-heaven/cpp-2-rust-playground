# cpp-2-rust-playground (`RustC-Playground`)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![CI/CD](https://github.com/haseeb-heaven/cpp-2-rust-playground/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/haseeb-heaven/cpp-2-rust-playground/actions/workflows/ci.yml)
[![C Engine: c2rust](https://img.shields.io/badge/C%20Engine-immunant%2Fc2rust-orange.svg)](https://github.com/immunant/c2rust)
[![C++ Engine: cpp2rust](https://img.shields.io/badge/C%2B%2B%20Engine-Cpp2Rust%2Fcpp2rust-purple.svg)](https://github.com/Cpp2Rust/cpp2rust)
[![AI Contributor: gemini-code-assist](https://img.shields.io/badge/AI%20Contributor-gemini--code--assist-4285F4.svg)](https://github.com/apps/gemini-code-assist)

Repository: https://github.com/haseeb-heaven/cpp-2-rust-playground

An interactive web IDE and compiler wrapper built with **Rust (Axum)** and **React + TypeScript + Vite** for translating **C** and **C++** code to Rust, inspecting diagnostics and line mappings, and compiling/running both source and generated Rust locally or online.

## Backbone Transpiler Engines & Fallback Transparency

`cpp-2-rust-playground` uses external AST-based C/C++ → Rust transpilers as its primary engines and falls back honestly to a built-in heuristic regex translator when the external binaries are not installed or fail:

| Source Language | Primary Engine (External CLI) | Fallback Engine (Built-in Heuristic) | Verification & Honest Labeling |
|---|---|---|---|
| **C (C99 / C11)** | [`immunant/c2rust`](https://github.com/immunant/c2rust) via `c2rust transpile compile_commands.json --output-dir <tmp>` | Built-in Regex C Fallback (`line_translator.rs`) | API returns `usedFallback`, `fallbackReason`, `unsupportedConstructs`, and an explicit `Built-in Regex C Fallback` engine badge when `c2rust` CLI did not execute |
| **C++ (C++17 / STL)** | [`Cpp2Rust/cpp2rust`](https://github.com/Cpp2Rust/cpp2rust) via `cpp2rust --file=<input.cpp> -o=<output.rs>` | Built-in Regex C++ Fallback (`line_translator.rs`) | Flags unsupported syntax (`try`/`catch`/`throw`, inheritance/`virtual`, operator overloads, variadic templates, `dynamic_cast`/`reinterpret_cast`, namespaces, `delete`, inline `asm`) and never claims unverified fallback code is equivalent |

### Output Modes

- **Faithful Rust (Raw Output)**: Preserves the uncleaned translator output 1:1.
- **Idiomatic Rust (Learning Mode)**: Optional educational view that strips boilerplate, removes redundant `mut` / casts / trivial `unsafe` blocks, formats with `rustfmt`, checks compilation via `rustc --emit=metadata`, and explains construct ownership & iteration differences (`std::vector<int>` → `Vec<i32>`, `&T` / `&mut T`, etc.). Any transform failing `rustc` verification rolls back automatically.

### Execution Isolation & Resource Limits

- Local compilation/execution runs in per-request temporary directories with `kill_on_drop(true)` process timeouts, scrubbed API credentials (`JDOODLE_CLIENT_ID`/`JDOODLE_CLIENT_SECRET`), Unix `ulimit` resource caps (CPU time, file size, core dumps, open file descriptors), and 64 KB stdout/stderr output truncation.
- For untrusted public multi-tenant hosting, deploy the backend inside an isolated container/VM sandbox (e.g., Docker/nsjail/Firecracker) or use the Online API provider.

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

See [CONTRIBUTORS.md](CONTRIBUTORS.md) for the full contributor list.

- **Creator & Lead Developer**: [Haseeb Mir (`@haseeb-heaven`)](https://github.com/haseeb-heaven)
- **C → Rust Backbone Transpiler**: [`immunant/c2rust`](https://github.com/immunant/c2rust) — C99-compliant C-to-Rust transpiler architecture
- **C++ → Rust Backbone Transpiler**: [`Cpp2Rust/cpp2rust`](https://github.com/Cpp2Rust/cpp2rust) — AST-driven C++ to Safe Rust translator and rule system
- **Testing & Review Tooling**: [`openqodex/openqodex`](https://github.com/openqodex/openqodex) and [Playwright](https://playwright.dev/)
- **AI Co-Contributor**: [**`gemini-code-assist[bot]`**](https://github.com/apps/gemini-code-assist) (Antigravity — Google DeepMind)

## License

This project is licensed under the **MIT License** — see the [LICENSE](./LICENSE) file for details and third-party notices for upstream backbone transpilers.
