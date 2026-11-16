# RustC-Playground

An online C/C++ to Rust line-by-line transpiler and code-execution platform built with **Rust (Axum)** and **React + TypeScript + Vite**.

## Features

- **Real-time Debounced Line-by-Line Transpilation**: Translates C/C++ to idiomatic Rust (`Vec`, `HashMap`, `HashSet`, `Option`, `Box`, `println!`, `struct`/`impl`, generics) while preserving manual Rust edits via incremental merging.
- **Separate Compile & Run**: Compile and Run C, C++, and Rust via the JDoodle Live API with `stdin` support, exit codes, memory, CPU time, and timeout handling.
- **Interactive Split-Screen UI**: Resizable and swappable panels, syntax highlighting (CodeMirror 6), line-to-line mapping highlights, Diff View, Rust code formatter, pre-made examples, local storage persistence, `.c`/`.cpp`/`.rs` import/export, themes (`Dark`, `Midnight`, `Light`), and keyboard shortcuts.

## Setup

1. Copy `.env.example` to `.env` and set your JDoodle credentials:
   ```bash
   cp .env.example .env
   ```
2. Start the Rust backend:
   ```bash
   cd backend
   cargo run
   ```
3. Start the React frontend:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

## Testing

- **Backend unit & integration tests**:
  ```bash
  cd backend && cargo test
  ```
- **Frontend unit & component tests**:
  ```bash
  cd frontend && npm test
  ```
- **End-to-End tests (Playwright)** (with backend and frontend running):
  ```bash
  cd frontend && npm run test:e2e
  ```
