//! Generated end-to-end equivalence suite.
//!
//! Instead of hand-picking a few examples, this test deterministically generates
//! a large, varied set of C programs from composable templates (loops, arrays,
//! arithmetic, functions, structs, malloc, sizeof idioms, printf formats, ...)
//! and verifies for every single one that:
//!
//! 1. the C source compiles and runs locally,
//! 2. the conversion is produced by the real c2rust backbone (not the fallback),
//! 3. the Idiomatic projection passes rustc verification with no error
//!    diagnostics,
//! 4. the Rust translation compiles, runs, and prints byte-identical output.
//!
//! The case count is controlled by `PLAYGROUND_EQUIV_CASES` (default 40, so a
//! normal `cargo test` run stays reasonable). Run a large sweep with:
//!
//! ```sh
//! PLAYGROUND_EQUIV_CASES=1000 cargo test --manifest-path backend/Cargo.toml --test generated_equivalence_tests -- --nocapture
//! ```
//!
//! Every case is a pure function of its index (seeded RNG), so any failure is
//! exactly reproducible.

use rustc_playground_backend::config::ApplicationConfiguration;
use rustc_playground_backend::executor::{execute_program, find_local_compiler};
use rustc_playground_backend::models::{
    DiagnosticSeverity, ExecutionLanguage, ExecutionMode, ExecutionProvider, ExecutionRequest,
    SourceLanguage,
};
use rustc_playground_backend::transpiler::convert_source_to_rust_with_style;
use rustc_playground_backend::models::RustOutputStyle;

/// Deterministic xorshift64 RNG so each case is reproducible from its index.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound.max(1)
    }
}

fn generate_case(index: u64) -> String {
    let mut rng = Rng::new(0x9E37_79B9_7F4A_7C15u64.wrapping_mul(index + 1));
    let template = index % 15;
    let pick = |rng: &mut Rng, bound: u64| rng.below(bound);

    match template {
        0 => {
            // Fixed array, init list, %d loop printing.
            let n = 4 + pick(&mut rng, 5);
            let values: Vec<String> = (0..n).map(|i| format!("{}", (i as i64 * 7 + 3) % 97)).collect();
            format!(
                "#include <stdio.h>\nint main(void) {{\n    int arr[{n}] = {{{values}}};\n    for (int i = 0; i < {n}; i++) {{\n        printf(\"%d \", arr[i]);\n    }}\n    printf(\"\\n\");\n    return 0;\n}}",
                n = n,
                values = values.join(", ")
            )
        }
        1 => {
            // Accumulation loops: sum, product, min, max.
            let n = 3 + pick(&mut rng, 6);
            let base = 1 + pick(&mut rng, 9);
            format!(
                "#include <stdio.h>\nint main(void) {{\n    int sum = 0, prod = 1, mn = 100000, mx = -100000;\n    for (int i = {base}; i < {end}; i++) {{\n        sum += i;\n        prod *= (i % 7 + 1);\n        if (i < mn) mn = i;\n        if (i > mx) mx = i;\n    }}\n    printf(\"sum=%d prod=%d min=%d max=%d\\n\", sum, prod, mn, mx);\n    return 0;\n}}",
                base = base,
                end = base + n
            )
        }
        2 => {
            // Doubles and %f / %.Nf formatting.
            let a = 1 + pick(&mut rng, 50);
            let b = 1 + pick(&mut rng, 9);
            let digits = 1 + pick(&mut rng, 5);
            format!(
                "#include <stdio.h>\nint main(void) {{\n    double x = {a}.{b};\n    double y = x / {b}.0;\n    printf(\"x={x_f} y=%.{digits}f\\n\", x, y);\n    return 0;\n}}",
                a = a,
                b = b,
                digits = digits,
                x_f = format!("{a}.{b}")
            )
        }
        3 => {
            // Nested loops.
            let rows = 2 + pick(&mut rng, 4);
            let cols = 2 + pick(&mut rng, 4);
            format!(
                "#include <stdio.h>\nint main(void) {{\n    for (int i = 1; i <= {rows}; i++) {{\n        for (int j = 1; j <= {cols}; j++) {{\n            printf(\"%d \", i * j);\n        }}\n        printf(\"\\n\");\n    }}\n    return 0;\n}}",
                rows = rows,
                cols = cols
            )
        }
        4 => {
            // Helper functions and calls.
            let a = pick(&mut rng, 100);
            let b = pick(&mut rng, 100);
            format!(
                "#include <stdio.h>\nstatic int add(int a, int b) {{ return a + b; }}\nstatic int max_of(int a, int b) {{ if (a > b) return a; return b; }}\nint main(void) {{\n    printf(\"%d %d\\n\", add({a}, {b}), max_of({a}, {b}));\n    return 0;\n}}",
                a = a,
                b = b
            )
        }
        5 => {
            // Recursion.
            let n = 3 + pick(&mut rng, 8);
            format!(
                "#include <stdio.h>\nstatic unsigned long long fact(int n) {{\n    if (n <= 1) return 1;\n    return (unsigned long long)n * fact(n - 1);\n}}\nint main(void) {{\n    printf(\"fact=%llu\\n\", fact({n}));\n    return 0;\n}}",
                n = n
            )
        }
        6 => {
            // switch statement.
            let v = pick(&mut rng, 6);
            format!(
                "#include <stdio.h>\nint main(void) {{\n    int v = {v};\n    switch (v) {{\n        case 0: printf(\"zero\\n\"); break;\n        case 1: printf(\"one\\n\"); break;\n        case 2: printf(\"two\\n\"); break;\n        default: printf(\"many\\n\"); break;\n    }}\n    return 0;\n}}"
            )
        }
        7 => {
            // Bitwise ops on unsigned.
            let a = pick(&mut rng, 256);
            let b = pick(&mut rng, 256);
            format!(
                "#include <stdio.h>\nint main(void) {{\n    unsigned a = {a}u, b = {b}u;\n    printf(\"and=%u or=%u xor=%u shl=%u\\n\", a & b, a | b, a ^ b, (a << 1) & 1023u);\n    return 0;\n}}",
                a = a,
                b = b
            )
        }
        8 => {
            // typedef struct + field printing.
            let x = pick(&mut rng, 100);
            let y = pick(&mut rng, 100);
            format!(
                "#include <stdio.h>\ntypedef struct {{ int x; int y; }} Point;\nstatic int point_mag(Point p) {{ return p.x * p.x + p.y * p.y; }}\nint main(void) {{\n    Point p = {{{x}, {y}}};\n    printf(\"x=%d y=%d mag=%d\\n\", p.x, p.y, point_mag(p));\n    return 0;\n}}",
                x = x,
                y = y
            )
        }
        9 => {
            // sizeof(arr)/sizeof(arr[0]) array-length idiom.
            let n = 3 + pick(&mut rng, 6);
            let values: Vec<String> = (0..n).map(|i| format!("{}", (i as i64 * 13 + 5) % 89)).collect();
            format!(
                "#include <stdio.h>\nint main(void) {{\n    int data[{n}] = {{{values}}};\n    int total = 0;\n    for (size_t i = 0; i < sizeof(data)/sizeof(data[0]); i++) {{\n        total += (int)data[i];\n    }}\n    printf(\"total=%d count=%zu\\n\", total, sizeof(data)/sizeof(data[0]));\n    return 0;\n}}",
                n = n,
                values = values.join(", ")
            )
        }
        10 => {
            // malloc + write + read + free (the heap pattern from real programs).
            let n = 3 + pick(&mut rng, 6);
            let off = 1 + pick(&mut rng, 20);
            format!(
                "#include <stdio.h>\n#include <stdlib.h>\nint main(void) {{\n    int *arr = (int*)malloc(sizeof(int) * {n});\n    if (!arr) return 1;\n    for (int i = 0; i < {n}; i++) {{\n        arr[i] = i + {off};\n    }}\n    for (int i = 0; i < {n}; i++) {{\n        printf(\"%d\", arr[i]);\n    }}\n    printf(\"\\n\");\n    free(arr);\n    return 0;\n}}",
                n = n,
                off = off
            )
        }
        11 => {
            // 2D array.
            let rows = 2 + pick(&mut rng, 3);
            let cols = 2 + pick(&mut rng, 3);
            let fill = pick(&mut rng, 9);
            format!(
                "#include <stdio.h>\nint main(void) {{\n    int grid[{rows}][{cols}];\n    for (int i = 0; i < {rows}; i++) {{\n        for (int j = 0; j < {cols}; j++) {{\n            grid[i][j] = i * 10 + j + {fill};\n        }}\n    }}\n    int total = 0;\n    for (int i = 0; i < {rows}; i++) {{\n        for (int j = 0; j < {cols}; j++) {{\n            total += grid[i][j];\n        }}\n    }}\n    printf(\"total=%d\\n\", total);\n    return 0;\n}}",
                rows = rows,
                cols = cols,
                fill = fill
            )
        }
        13 => {
            // Line input via gets into a fixed char buffer, echoed with puts.
            let n = 8 + pick(&mut rng, 16);
            format!(
                "#include <stdio.h>\nint main() {{\n    char data[{n}];\n    gets(data);\n    puts(data);\n    return 0;\n}}",
                n = n
            )
        }
        14 => {
            // Prompt + gets + puts, the classic C console echo.
            format!(
                "#include <stdio.h>\nint main() {{\n    char data[32];\n    puts(\"Enter Data:\");\n    gets(data);\n    puts(data);\n    return 0;\n}}"
            )
        }
        _ => {
            // while loop + compound assignment + ternary.
            let n = 3 + pick(&mut rng, 10);
            format!(
                "#include <stdio.h>\nint main(void) {{\n    int i = 0, acc = 0;\n    while (i < {n}) {{\n        acc += (i % 2 == 0) ? i : -i;\n        i++;\n    }}\n    printf(\"acc=%d i=%d\\n\", acc, i);\n    return 0;\n}}",
                n = n
            )
        }
    }
}

fn generate_stdin(index: u64) -> String {
    if index % 15 == 13 || index % 15 == 14 {
        let mut rng = Rng::new(0x1234_5678u64.wrapping_mul(index + 3));
        let words = ["alpha", "bravo", "charlie", "delta", "echo"];
        return format!("{}\n", words[(rng.below(words.len() as u64)) as usize]);
    }
    String::new()
}

fn case_count() -> u64 {
    std::env::var("PLAYGROUND_EQUIV_CASES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(40)
}

#[tokio::test]
async fn generated_c_programs_convert_via_backbone_and_match_source_output() {
    if find_local_compiler(&ExecutionLanguage::C).is_none()
        || find_local_compiler(&ExecutionLanguage::Rust).is_none()
    {
        return;
    }
    if rustc_playground_backend::transpiler::detect_external_transpiler_binary(&SourceLanguage::C)
        .is_none()
    {
        return;
    }

    let configuration = ApplicationConfiguration {
        jdooodle_client_id: String::new(),
        jdooodle_client_secret: String::new(),
        jdooodle_api_url: "https://api.jdoodle.com/v1/execute".to_string(),
        backend_port: 8080,
        // Each case compiles C, runs several rustc checks, then runs Rust; a cold
        // toolchain can exceed the server's interactive default.
        request_timeout_seconds: 120,
    };

    let count = case_count();
    let mut converted = 0u64;

    for index in 0..count {
        let source = generate_case(index);
        let stdin_input = generate_stdin(index);

        let source_result = execute_program(
            &configuration,
            &ExecutionRequest {
                code: source.clone(),
                language: ExecutionLanguage::C,
                mode: ExecutionMode::Run,
                stdin_input: stdin_input.clone(),
                provider: Some(ExecutionProvider::Local),
                compiler_standard: None,
            },
        )
        .await;
        assert!(
            source_result.success,
            "case {index}: generated C failed to compile/run:\n{}\n{}",
            source,
            source_result.error_output
        );

        let conversion = convert_source_to_rust_with_style(
            &source,
            &SourceLanguage::C,
            RustOutputStyle::Idiomatic,
        );

        assert!(
            !conversion.used_fallback,
            "case {index}: the c2rust backbone must be used, got engine '{}' ({:?})",
            conversion.transpiler_engine,
            conversion.fallback_reason
        );
        assert!(
            conversion.transpiler_engine.contains("c2rust CLI"),
            "case {index}: unexpected engine '{}'",
            conversion.transpiler_engine
        );
        assert!(
            conversion.post_processing.compiler_verified,
            "case {index}: idiomatic projection must pass rustc verification: {:?}\nsource:\n{}",
            conversion.post_processing.verification_error,
            source
        );
        assert!(
            !conversion.generated_rust_code.contains("unsafe")
                && !conversion.generated_rust_code.contains("extern \"C\"")
                && !conversion.generated_rust_code.contains("::core::ffi"),
            "case {index}: Idiomatic Rust must be plain safe Rust (no unsafe/extern C):\n{}",
            conversion.generated_rust_code
        );

        let error_diagnostics: Vec<_> = conversion
            .diagnostics
            .iter()
            .filter(|diag| diag.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(
            error_diagnostics.is_empty(),
            "case {index}: conversion produced error diagnostics: {:?}",
            error_diagnostics
        );

        let rust_result = execute_program(
            &configuration,
            &ExecutionRequest {
                code: conversion.generated_rust_code.clone(),
                language: ExecutionLanguage::Rust,
                mode: ExecutionMode::Run,
                stdin_input: stdin_input.clone(),
                provider: Some(ExecutionProvider::Local),
                compiler_standard: None,
            },
        )
        .await;
        assert!(
            rust_result.success,
            "case {index}: idiomatic Rust failed to compile/run:\n{}\n--- Rust ---\n{}",
            rust_result.error_output,
            conversion.generated_rust_code
        );
        assert_eq!(
            source_result.output.trim(),
            rust_result.output.trim(),
            "case {index}: output mismatch\nC output:\n{}\nRust output:\n{}\n--- source ---\n{}\n--- Rust ---\n{}",
            source_result.output,
            rust_result.output,
            source,
            conversion.generated_rust_code
        );
        converted += 1;
    }

    println!("generated equivalence cases verified: {converted}/{count}");
    assert!(converted >= 1);
}


fn generate_cpp_case(index: u64) -> String {
    let mut rng = Rng::new(0xC0FFEEu64.wrapping_mul(index + 7));
    let pick = |rng: &mut Rng, bound: u64| rng.below(bound);
    match index % 5 {
        0 => {
            // Class with constructor, setter/getter and iostream output.
            let id = 1 + pick(&mut rng, 5);
            let score = 10 + pick(&mut rng, 90);
            format!(
                r#"#include <iostream>
class Person {{
private:
    int id;
    float score;
public:
    Person(int person_id) : id(person_id), score(0.0f) {{}}
    void set_score(float value) {{
        if (this->id > 0) {{
            this->score = value;
        }}
    }}
    float get_score() const {{
        if (this->id > 0) {{
            return this->score;
        }}
        return -1.0f;
    }}
}};
int main() {{
    Person p({id});
    p.set_score({score}.0f);
    std::cout << "score " << p.get_score() << std::endl;
    return 0;
}}"#,
                id = id,
                score = score
            )
        }
        1 => {
            // std::vector accumulation loop.
            let n = 3 + pick(&mut rng, 5);
            format!(
                r#"#include <iostream>
#include <vector>
int main() {{
    std::vector<int> values;
    for (int i = 0; i < {n}; i++) {{
        values.push_back(i * 3 + 1);
    }}
    int total = 0;
    for (int v : values) {{
        total += v;
    }}
    std::cout << "total " << total << std::endl;
    return 0;
}}"#,
                n = n
            )
        }
        2 => {
            // std::string concatenation and printing.
            let repeat = 1 + pick(&mut rng, 4);
            format!(
                r#"#include <iostream>
#include <string>
int main() {{
    std::string word = "rust";
    std::string line;
    for (int i = 0; i < {repeat}; i++) {{
        line += word;
    }}
    std::cout << line << std::endl;
    return 0;
}}"#,
                repeat = repeat
            )
        }
        3 => {
            // Struct with a free function.
            let a = pick(&mut rng, 40);
            let b = pick(&mut rng, 40);
            format!(
                r#"#include <iostream>
struct Point {{
    int x;
    int y;
}};
int manhattan(Point p) {{
    return (p.x < 0 ? -p.x : p.x) + (p.y < 0 ? -p.y : p.y);
}}
int main() {{
    Point p = {{ {a}, {b} }};
    std::cout << "manhattan " << manhattan(p) << std::endl;
    return 0;
}}"#,
                a = a,
                b = b
            )
        }
        _ => {
            // Nested loop with iostream formatting.
            let rows = 2 + pick(&mut rng, 3);
            let cols = 2 + pick(&mut rng, 3);
            format!(
                r#"#include <iostream>
int main() {{
    for (int i = 1; i <= {rows}; i++) {{
        for (int j = 1; j <= {cols}; j++) {{
            std::cout << i * j << " ";
        }}
        std::cout << std::endl;
    }}
    return 0;
}}"#,
                rows = rows,
                cols = cols
            )
        }
    }
}

fn cpp_case_count() -> u64 {
    std::env::var("PLAYGROUND_EQUIV_CPP_CASES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(10)
}

#[tokio::test]
async fn generated_cpp_programs_convert_via_backbone_and_match_source_output() {
    if find_local_compiler(&ExecutionLanguage::Cpp).is_none()
        || find_local_compiler(&ExecutionLanguage::Rust).is_none()
    {
        return;
    }
    if rustc_playground_backend::transpiler::detect_external_transpiler_binary(&SourceLanguage::Cpp)
        .is_none()
    {
        return;
    }

    let configuration = ApplicationConfiguration {
        jdooodle_client_id: String::new(),
        jdooodle_client_secret: String::new(),
        jdooodle_api_url: "https://api.jdoodle.com/v1/execute".to_string(),
        backend_port: 8080,
        request_timeout_seconds: 120,
    };

    let count = cpp_case_count();
    for index in 0..count {
        let source = generate_cpp_case(index);

        let source_result = execute_program(
            &configuration,
            &ExecutionRequest {
                code: source.clone(),
                language: ExecutionLanguage::Cpp,
                mode: ExecutionMode::Run,
                stdin_input: String::new(),
                provider: Some(ExecutionProvider::Local),
                compiler_standard: None,
            },
        )
        .await;
        assert!(
            source_result.success,
            "cpp case {index}: generated C++ failed to compile/run:\n{}\n{}",
            source,
            source_result.error_output
        );

        let conversion = convert_source_to_rust_with_style(
            &source,
            &SourceLanguage::Cpp,
            RustOutputStyle::Idiomatic,
        );
        assert!(
            !conversion.generated_rust_code.contains("unsafe")
                && !conversion.generated_rust_code.contains("extern \"C\"")
                && !conversion.generated_rust_code.contains("libcc2rs"),
            "cpp case {index}: Readable Rust must be plain safe Rust:\n{}",
            conversion.generated_rust_code
        );
        if !conversion.post_processing.compiler_verified {
            // Constructs with no safe projection (for example std::string) must say so
            // instead of showing the raw engine output as readable Rust.
            assert!(
                conversion
                    .generated_rust_code
                    .contains("Idiomatic Rust is unavailable"),
                "cpp case {index}: expected verified Rust or an explicit notice: {:?}\n--- C++ ---\n{}",
                conversion.post_processing.verification_error,
                source
            );
            println!("cpp case {index}: no safe projection (documented limitation)");
            continue;
        }

        let rust_result = execute_program(
            &configuration,
            &ExecutionRequest {
                code: conversion.generated_rust_code.clone(),
                language: ExecutionLanguage::Rust,
                mode: ExecutionMode::Run,
                stdin_input: String::new(),
                provider: Some(ExecutionProvider::Local),
                compiler_standard: None,
            },
        )
        .await;
        assert!(
            rust_result.success,
            "cpp case {index}: readable Rust failed to compile/run:\n{}\n--- Rust ---\n{}",
            rust_result.error_output, conversion.generated_rust_code
        );
        assert_eq!(
            source_result.output.trim(),
            rust_result.output.trim(),
            "cpp case {index}: output mismatch\nC++ output:\n{}\nRust output:\n{}\n--- Rust ---\n{}",
            source_result.output,
            rust_result.output,
            conversion.generated_rust_code
        );
    }
}

