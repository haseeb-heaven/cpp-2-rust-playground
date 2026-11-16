#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_DIR="${ROOT_DIR}/.tools"
mkdir -p "${TOOLS_DIR}/bin"

echo "==> Installing backbone engines into ${TOOLS_DIR}"
echo "  1) C -> Rust   : https://github.com/immunant/c2rust"
echo "  2) C++ -> Rust : https://github.com/Cpp2Rust/cpp2rust"

if command -v cargo >/dev/null 2>&1; then
  if ! command -v c2rust >/dev/null 2>&1; then
    echo "Installing c2rust via cargo (requires LLVM/clang/cmake/openssl)..."
    cargo install c2rust || {
      echo "cargo install c2rust did not succeed; cloning source to ${TOOLS_DIR}/c2rust instead..."
      if [ ! -d "${TOOLS_DIR}/c2rust/.git" ]; then
        git clone --depth 1 https://github.com/immunant/c2rust.git "${TOOLS_DIR}/c2rust"
      fi
    }
  else
    echo "c2rust already available at: $(command -v c2rust)"
  fi
fi

if [ ! -d "${TOOLS_DIR}/cpp2rust/.git" ]; then
  git clone --depth 1 https://github.com/Cpp2Rust/cpp2rust.git "${TOOLS_DIR}/cpp2rust"
else
  echo "cpp2rust repository already cloned at ${TOOLS_DIR}/cpp2rust"
fi

if command -v cmake >/dev/null 2>&1 && command -v ninja >/dev/null 2>&1; then
  mkdir -p "${TOOLS_DIR}/cpp2rust/build"
  if (cd "${TOOLS_DIR}/cpp2rust/build" && cmake -GNinja .. && ninja); then
    echo "Built cpp2rust at ${TOOLS_DIR}/cpp2rust/build/cpp2rust/cpp2rust"
  else
    echo "Notice: Cpp2Rust/cpp2rust requires Clang/libclang-dev headers matching its CMake setup; built-in Safe C++ fallback remains active."
  fi
else
  echo "Notice: cmake + ninja required to build Cpp2Rust/cpp2rust from source."
fi

echo ""
echo "Done. Optional environment overrides:"
echo "  export C2RUST_BIN=\"\$(command -v c2rust || true)\""
echo "  export CPP2RUST_BIN=\"${TOOLS_DIR}/cpp2rust/build/cpp2rust/cpp2rust\""
