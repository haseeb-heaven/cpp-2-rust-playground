# Converter Reliability and Library Examples

## Purpose

Make the C/C++ to Rust playground reliable and transparent for both Rust
output styles, including the mode controls in the browser. Add executable C
and C++ standard-library CRUD examples and regression coverage for the supplied
C11 slab-backed LRU cache with WAL persistence.

## Scope

- Treat C and C++ as separate supported source languages.
- Preserve **Faithful Rust** as raw output from the selected external engine or
  built-in fallback. It must never receive idiomatic cleanup passes.
- Produce **Idiomatic Rust** only as a verified post-processing result. If the
  result fails `rustc` verification, return the faithful code instead and add a
  diagnostic explaining that idiomatic processing was rejected.
- Ensure the selected output style reaches the conversion API for initial
  conversion, manual conversion, live conversion, and style changes.
- Preserve user-authored Rust: changing style may refresh generated output but
  must not overwrite divergent manual edits.
- Provide C and C++ CRUD examples using their standard libraries and file I/O,
  plus expected runtime output. Include create, lookup/read, update, delete,
  iteration, and persistence behavior.
- Add the supplied C11 LRU/WAL source as a regression fixture. It must compile
  and run as C. The converter must either emit compiler-verifiable Rust or
  explicitly diagnose unsupported constructs; it must not label unverified
  fallback output as equivalent.

## Architecture

The frontend owns selected output style and passes it with every conversion
request. The backend converts source once to faithful Rust, records engine and
fallback metadata, then conditionally post-processes that faithful output for
idiomatic Rust. The API returns both variants, the selected generated variant,
post-processing verification status, diagnostics, and line mappings.

The UI switches displayed generated Rust from API-returned variants only. A
mode switch triggers a conversion with the newly chosen style rather than
reusing stale state. Incremental merging continues to protect manual edits.

Examples are catalog entries with source language, source code, intended
runtime output, and language-specific compiler settings. Backend tests compile
and run sources where host tools are available and separately assert converter
contracts so missing third-party transpiler binaries do not create false test
successes.

## Error Handling and Honesty

- An unavailable or failed external engine is reported as fallback usage with a
  reason.
- Unsupported C/C++ constructs produce conversion diagnostics.
- A failed idiomatic transform is rejected atomically; raw faithful output
  remains available and its status is explicit.
- A request with an invalid output style is rejected as input validation error.
- Compilation and runtime tests compare exact normalized stdout and fail on a
  non-zero exit status.

## Test Plan

- Backend unit tests for both output styles, verification rollback, fallback
  metadata, and output-style request parsing.
- Backend integration tests covering C and C++ examples through conversion and
  execution endpoints.
- Frontend tests confirming the style selector dispatches the new style and
  displays its returned variant without overwriting manual Rust changes.
- End-to-end coverage for both styles and both source-language selectors.
- Regression tests for the supplied C11 cache source: C compile/run output and
  honest conversion diagnostics or verified Rust.
- Full repository verification: `cargo test`, frontend unit tests and build,
  and Playwright tests with the application running.

## Non-Goals

- Claiming that the built-in regex fallback fully translates arbitrary C11 or
  C++17.
- Replacing external AST-based engines with a new general-purpose parser.
- Overwriting a user’s manual Rust edits on mode changes.
