#!/usr/bin/env bash
set -euo pipefail

export PATH="${HOME}/.cargo/bin:/opt/homebrew/bin:/opt/homebrew/opt/llvm/bin:/usr/local/bin:${PATH}"

SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_PORT="${BACKEND_PORT:-8080}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"

stop_port_if_running() {
  local port="$1"
  local existing_pids
  existing_pids="$(lsof -tiTCP:"${port}" -sTCP:LISTEN 2>/dev/null || true)"
  if [[ -n "${existing_pids}" ]]; then
    echo "Stopping existing process(es) on port ${port}: ${existing_pids}"
    echo "${existing_pids}" | xargs kill -9 2>/dev/null || true
    sleep 1
  fi
}

cleanup() {
  echo ""
  echo "Shutting down C/C++ to Rust Playground servers..."
  [[ -n "${BACKEND_PID:-}" ]] && kill "${BACKEND_PID}" 2>/dev/null || true
  [[ -n "${FRONTEND_PID:-}" ]] && kill "${FRONTEND_PID}" 2>/dev/null || true
}

trap cleanup EXIT INT TERM

stop_port_if_running "${BACKEND_PORT}"
stop_port_if_running "${FRONTEND_PORT}"

echo "Starting Rust backend on http://127.0.0.1:${BACKEND_PORT}..."
(cd "${SCRIPT_DIRECTORY}/backend" && cargo run) &
BACKEND_PID=$!

echo "Starting React frontend on http://localhost:${FRONTEND_PORT}..."
(cd "${SCRIPT_DIRECTORY}/frontend" && npm run dev) &
FRONTEND_PID=$!

wait "${BACKEND_PID}" "${FRONTEND_PID}"
