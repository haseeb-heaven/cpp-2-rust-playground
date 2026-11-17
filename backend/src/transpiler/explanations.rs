use crate::models::{
    ConversionExplanation, PostProcessingSummary, RustOutputStyle, SourceLanguage,
};
use once_cell::sync::Lazy;
use regex::Regex;

static VECTOR_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\b((?:std::)?vector\s*<\s*((?:std::)?vector\s*<\s*[A-Za-z0-9_: ]+?\s*>|[A-Za-z0-9_:\s*&]+?)\s*>)",
    )
    .unwrap()
});
static MAP_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b((?:std::)?(?:unordered_)?map\s*<\s*([^,>]+?)\s*,\s*([^>]+?)\s*>)").unwrap()
});
static SET_DECL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b((?:std::)?(?:unordered_)?set\s*<\s*([^>]+?)\s*>)").unwrap());
static QUEUE_DECL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b((?:std::)?(?:queue|deque)\s*<\s*([^>]+?)\s*>)").unwrap());
static OPTIONAL_DECL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b((?:std::)?optional\s*<\s*([^>]+?)\s*>)").unwrap());
static POINTER_OR_REF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?:\b(?:void|char|int|float|double|long|size_t|bool|auto|[A-Z][A-Za-z0-9_]*)\s*[*&]\s*[A-Za-z_*]|\brestrict\b|\[\s*static\s+\d+\s*\])",
    )
    .unwrap()
});
static STRING_TYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\b(?:std::)?string\b|\bchar\s*\*|\bchar\s+[A-Za-z_][A-Za-z0-9_]*\s*\[)")
        .unwrap()
});
static STRUCT_OR_CLASS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?:class|typedef\s+struct|struct|union)\s+[A-Za-z_]").unwrap());
static LINKED_LIST_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:\*\s*next\b|->\s*next\b)").unwrap());
static FLEXIBLE_ARRAY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[\s*\]\s*;").unwrap());
static COMPOUND_LITERAL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&\s*\(\s*[A-Za-z_][A-Za-z0-9_]*\s*\)\s*\{").unwrap());
static VLA_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\s*[a-zA-Z_][a-zA-Z0-9_]*\s*\](?:\[\s*[a-zA-Z_][a-zA-Z0-9_]*\s*\])?\s*;")
        .unwrap()
});

fn map_cpp_element_to_rust(raw_type: &str) -> String {
    let normalized = raw_type
        .trim()
        .trim_start_matches("const ")
        .trim()
        .replace("std::", "");
    if let Some(inner) = normalized
        .strip_prefix("vector<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return format!("Vec<{}>", map_cpp_element_to_rust(inner));
    }
    match normalized.as_str() {
        "int" | "int32_t" | "signed" => "i32".to_string(),
        "unsigned int" | "unsigned" | "uint32_t" => "u32".to_string(),
        "long" | "long long" | "int64_t" => "i64".to_string(),
        "unsigned long" | "unsigned long long" | "uint64_t" => "u64".to_string(),
        "size_t" => "usize".to_string(),
        "float" => "f32".to_string(),
        "double" | "long double" => "f64".to_string(),
        "bool" => "bool".to_string(),
        "char" => "char".to_string(),
        "string" => "String".to_string(),
        other => other.to_string(),
    }
}

fn push_vector_explanation(
    source_code: &str,
    rust_code: &str,
    out: &mut Vec<ConversionExplanation>,
) {
    if let Some(cap) = VECTOR_DECL_REGEX.captures(source_code) {
        let cpp_pattern = cap.get(1).unwrap().as_str().trim().to_string();
        let elem_rust = map_cpp_element_to_rust(cap.get(2).unwrap().as_str());
        let rust_equivalent = format!("Vec<{}>", elem_rust);
        let elem_raw = cap.get(2).unwrap().as_str().trim();

        out.push(ConversionExplanation {
            construct: "Dynamic Array (std::vector → Vec)".to_string(),
            cpp_pattern: cpp_pattern.clone(),
            rust_equivalent: rust_equivalent.clone(),
            why_chosen: format!(
                "{} is Rust's contiguous, growable heap-allocated array matching {} with explicit element sizing ({} → {}). Unlike C++, where vector reallocation can silently invalidate raw pointers and iterators at runtime, Rust's borrow checker forbids holding element references across push/resize mutations at compile time.",
                rust_equivalent, cpp_pattern, elem_raw, elem_rust
            ),
            ownership_and_iteration: format!(
                "Ownership: {} owns its heap buffer and frees elements deterministically on scope exit (RAII Drop). Passing {} by value moves ownership instead of silently deep-copying like C++; use .clone() for a copy or &[{}] to borrow a read-only slice. Iteration: `for x in &vec` borrows immutably (&{}), `for x in &mut vec` borrows mutably (&mut {}), and `for x in vec` (.into_iter()) moves and consumes the vector.",
                rust_equivalent, rust_equivalent, elem_rust, elem_rust, elem_rust
            ),
        });
    } else if rust_code.contains("Vec<") || rust_code.contains("Vec::new") || rust_code.contains("vec![") {
        out.push(ConversionExplanation {
            construct: "Dynamic Array (Vec<T>)".to_string(),
            cpp_pattern: "std::vector<T>".to_string(),
            rust_equivalent: "Vec<T>".to_string(),
            why_chosen: "Vec<T> provides heap-allocated contiguous storage with O(1) indexed access, amortized O(1) push(), and compile-time bounds/aliasing safety.".to_string(),
            ownership_and_iteration: "Ownership: Vec<T> owns its heap allocation and drops elements automatically. Iteration: `for x in &vec` borrows immutably, `for x in &mut vec` mutates in place, and `for x in vec` consumes the vector.".to_string(),
        });
    }
}

fn push_associative_collection_explanations(
    source_code: &str,
    rust_code: &str,
    out: &mut Vec<ConversionExplanation>,
) {
    if let Some(cap) = MAP_DECL_REGEX.captures(source_code) {
        let cpp_pattern = cap.get(1).unwrap().as_str().trim().to_string();
        let k_rust = map_cpp_element_to_rust(cap.get(2).unwrap().as_str());
        let v_rust = map_cpp_element_to_rust(cap.get(3).unwrap().as_str());
        let rust_equivalent = format!("HashMap<{}, {}>", k_rust, v_rust);
        out.push(ConversionExplanation {
            construct: "Associative Map (std::unordered_map / map → HashMap)".to_string(),
            cpp_pattern: cpp_pattern.clone(),
            rust_equivalent: rust_equivalent.clone(),
            why_chosen: format!(
                "{} provides O(1) average key-value lookup with DoS-resistant SipHash hashing, replacing {} without C++'s hazard of default-inserting missing keys on read.",
                rust_equivalent, cpp_pattern
            ),
            ownership_and_iteration: "Ownership: .insert(k, v) or .entry(k).or_insert(v) transfers key/value ownership into the map; lookups like .get(&k) and .contains_key(&k) only borrow the key by reference. Iteration: `for (k, v) in &map` borrows each pair immutably, `for (_, v) in &mut map` updates values in place, and `for (k, v) in map` consumes the map.".to_string(),
        });
    } else if rust_code.contains("HashMap") {
        out.push(ConversionExplanation {
            construct: "Hash Map (HashMap<K, V>)".to_string(),
            cpp_pattern: "std::unordered_map<K, V>".to_string(),
            rust_equivalent: "HashMap<K, V>".to_string(),
            why_chosen: "std::collections::HashMap gives O(1) expected lookup while forcing explicit handling when a key might be absent via Option<&V>.".to_string(),
            ownership_and_iteration: "Ownership: keys are moved on insert and borrowed on get(&k). Iterate with `for (k, v) in &map` or mutate values via `&mut map`.".to_string(),
        });
    }

    if let Some(cap) = SET_DECL_REGEX.captures(source_code) {
        let cpp_pattern = cap.get(1).unwrap().as_str().trim().to_string();
        let t_rust = map_cpp_element_to_rust(cap.get(2).unwrap().as_str());
        let rust_equivalent = format!("HashSet<{}>", t_rust);
        out.push(ConversionExplanation {
            construct: "Unique Set (std::unordered_set / set → HashSet)".to_string(),
            cpp_pattern,
            rust_equivalent: rust_equivalent.clone(),
            why_chosen: format!(
                "{} stores unique owned values with O(1) membership tests via .contains(&val), guaranteeing no uninitialized sentinel elements.",
                rust_equivalent
            ),
            ownership_and_iteration: "Ownership: .insert(val) moves val into the set and returns bool indicating insertion. Iteration: `for item in &set` yields shared references (&T) without copying elements.".to_string(),
        });
    }

    if let Some(cap) = QUEUE_DECL_REGEX.captures(source_code) {
        let cpp_pattern = cap.get(1).unwrap().as_str().trim().to_string();
        let t_rust = map_cpp_element_to_rust(cap.get(2).unwrap().as_str());
        let rust_equivalent = format!("VecDeque<{}>", t_rust);
        out.push(ConversionExplanation {
            construct: "FIFO Queue / Deque (std::queue → VecDeque)".to_string(),
            cpp_pattern,
            rust_equivalent: rust_equivalent.clone(),
            why_chosen: format!(
                "{} (std::collections::VecDeque) uses a growable ring buffer supporting O(1) push_back() and pop_front(). Unlike C++ std::queue where .front() and .pop() are separate error-prone calls, Rust's .pop_front() atomically pops and returns Option<{}>.",
                rust_equivalent, t_rust
            ),
            ownership_and_iteration: "Ownership & Iteration: .pop_front() transfers ownership of the front item out in Some(val) or returns None when empty; `while let Some(node) = queue.pop_front()` safely drives BFS/queue loops without manual empty() + front() + pop() boilerplate.".to_string(),
        });
    }

    if let Some(cap) = OPTIONAL_DECL_REGEX.captures(source_code) {
        let cpp_pattern = cap.get(1).unwrap().as_str().trim().to_string();
        let t_rust = map_cpp_element_to_rust(cap.get(2).unwrap().as_str());
        let rust_equivalent = format!("Option<{}>", t_rust);
        out.push(ConversionExplanation {
            construct: "Nullable Value (std::optional → Option)".to_string(),
            cpp_pattern,
            rust_equivalent: rust_equivalent.clone(),
            why_chosen: format!(
                "{} encodes presence (Some) or absence (None) directly in Rust's type system, eliminating null-pointer dereferences and uninitialized std::nullopt UB.",
                rust_equivalent
            ),
            ownership_and_iteration: "Ownership & Iteration: Pattern matching (`if let Some(val) = opt`) moves or borrows the inner value safely only when present; Option<T> also implements IntoIterator (0 or 1 item).".to_string(),
        });
    }
}

fn push_memory_and_pointer_explanations(
    source_code: &str,
    rust_code: &str,
    out: &mut Vec<ConversionExplanation>,
) {
    let has_heap_alloc = source_code.contains("std::unique_ptr")
        || source_code.contains("malloc(")
        || source_code.contains("calloc(")
        || source_code.contains("free(")
        || rust_code.contains("Box::new");
    if has_heap_alloc && !FLEXIBLE_ARRAY_REGEX.is_match(source_code) {
        out.push(ConversionExplanation {
            construct: "Single-Owner Heap Allocation (malloc / unique_ptr → Box)".to_string(),
            cpp_pattern: if source_code.contains("unique_ptr") {
                "std::unique_ptr<T>".to_string()
            } else {
                "T* = malloc(sizeof(T)) / free()".to_string()
            },
            rust_equivalent: "Box<T>".to_string(),
            why_chosen: "Box<T> provides sole heap ownership with zero runtime reference-counting overhead, replacing manual malloc/free calls that risk leaks, double-frees, or null pointer dereferences.".to_string(),
            ownership_and_iteration: "Ownership: Only one binding owns a Box<T> at a time; assigning or passing it moves ownership and automatically calls Drop::drop when the owner goes out of scope (so free() is never needed). Borrow inner fields via &T or &mut T.".to_string(),
        });
    }

    if POINTER_OR_REF_REGEX.is_match(source_code)
        || rust_code.contains("&mut ")
        || rust_code.contains(": &[")
    {
        let (cpp_pat, rust_eq) = if source_code.contains("restrict")
            || source_code.contains("[static ")
        {
            (
                "const double[static 4], double * restrict".to_string(),
                "&[f64], &mut f64".to_string(),
            )
        } else if source_code.contains("auto&") {
            ("auto& / T&".to_string(), "&mut T / &T".to_string())
        } else {
            ("T* / const T* / T&".to_string(), "&mut T / &T / &[T]".to_string())
        };
        out.push(ConversionExplanation {
            construct: "Pointers, Borrowing & Aliasing (&T / &mut T / &[T])".to_string(),
            cpp_pattern: cpp_pat,
            rust_equivalent: rust_eq,
            why_chosen: "Rust replaces unchecked C/C++ raw pointers and references with lifetime-checked borrows: `&T` for shared read-only access, `&mut T` for exclusive mutation (making C99 `restrict` non-aliasing a compiler-enforced rule), and slices `&[T]` bundling pointer + bounds length (making C99 `[static N]` verifiable at runtime/compile time).".to_string(),
            ownership_and_iteration: "Ownership & Iteration: At any point in a scope, Rust permits EITHER any number of shared `&T` borrows OR exactly one exclusive `&mut T` borrow, eliminating aliasing bugs, dangling pointers, and iterator invalidation.".to_string(),
        });
    }

    if LINKED_LIST_REGEX.is_match(source_code) {
        out.push(ConversionExplanation {
            construct: "Self-Referential List (struct Node *next → Vec<Node> / Option<Box<Node>>)".to_string(),
            cpp_pattern: "struct Node *next / while (curr != NULL)".to_string(),
            rust_equivalent: "Vec<Node> / for curr in &list".to_string(),
            why_chosen: "Intrusive raw `*next` pointers in C require unsafe dereferencing and manual traversal cleanup in Rust. Flattening acyclic lists into owned `Vec<Node>` (or `Option<Box<Node>>`) removes raw pointer dereferences and eliminates memory leaks.".to_string(),
            ownership_and_iteration: "Ownership & Iteration: The container owns every node and drops them all together without recursive Drop stack overflow. Iterating uses `for node in &list` instead of chasing `curr = curr->next`.".to_string(),
        });
    }
}

fn push_string_struct_and_c_explanations(
    source_code: &str,
    source_language: &SourceLanguage,
    rust_code: &str,
    out: &mut Vec<ConversionExplanation>,
) {
    if STRING_TYPE_REGEX.is_match(source_code)
        || rust_code.contains("String::")
        || rust_code.contains(": &str")
    {
        out.push(ConversionExplanation {
            construct: "Strings (std::string / char* → String & &str)".to_string(),
            cpp_pattern: if source_code.contains("string") {
                "std::string / const char*".to_string()
            } else {
                "char* / char buf[]".to_string()
            },
            rust_equivalent: "String / &str".to_string(),
            why_chosen: "Rust separates owned, heap-allocated UTF-8 strings (`String`) from zero-copy string slice views (`&str`). Read-only parameters take `&str` so literals and `String` buffers can be passed without copying.".to_string(),
            ownership_and_iteration: "Ownership & Iteration: Assigning a `String` moves ownership (unless cloned); borrowing `&str` does not allocate. Unlike C's `while (*s++)` NUL-terminated char scans, Rust strings track length and iterate cleanly with `.chars()` (Unicode scalars) or `.as_bytes()` (raw u8 bytes).".to_string(),
        });
    }

    if STRUCT_OR_CLASS_REGEX.is_match(source_code) || rust_code.contains("pub struct ") {
        out.push(ConversionExplanation {
            construct: "Data Structures & Methods (struct / class → struct + impl)".to_string(),
            cpp_pattern: "typedef struct / class".to_string(),
            rust_equivalent: "pub struct + impl (with #[derive(Default, Clone)])".to_string(),
            why_chosen: "Rust decouples memory layout (`pub struct`) from behavior (`impl` blocks) with explicit field visibility and auto-derived `Clone`, `Debug`, and safe zero-initialization via `Default`.".to_string(),
            ownership_and_iteration: "Ownership: Methods explicitly declare how they borrow the instance: `&self` (read-only), `&mut self` (mutating), or `self` (consuming ownership).".to_string(),
        });
    }

    if source_code.contains("long double") || source_code.contains("LDBL_") {
        out.push(ConversionExplanation {
            construct: "Extended Precision Floating-Point (<float.h> long double → f64)".to_string(),
            cpp_pattern: "long double / LDBL_MAX / LDBL_MANT_DIG".to_string(),
            rust_equivalent: "f64 / f64::MAX / f64::MANTISSA_DIGITS".to_string(),
            why_chosen: "Stable Rust provides IEEE-754 `f32` and `f64` (`f128` is unstable/nightly-only), so C `long double` and `<float.h>` limit macros map cleanly to `f64` and its associated constants (`f64::MANTISSA_DIGITS`, `f64::DIGITS`, `f64::MAX`, `std::mem::size_of::<f64>()`).".to_string(),
            ownership_and_iteration: "Ownership: `f64` implements `Copy`, so passing or indexing floating-point scalars copies bitwise on the stack without move invalidation.".to_string(),
        });
    }

    let has_c99_exclusive = *source_language == SourceLanguage::C
        && (FLEXIBLE_ARRAY_REGEX.is_match(source_code)
            || source_code.contains("_Generic(")
            || COMPOUND_LITERAL_REGEX.is_match(source_code)
            || VLA_DECL_REGEX.is_match(source_code));
    if has_c99_exclusive {
        out.push(ConversionExplanation {
            construct: "Pure ISO C99/C11 Idioms → Safe Rust Equivalents".to_string(),
            cpp_pattern: "C99 FAM[] / VLA / Compound Literal / C11 _Generic".to_string(),
            rust_equivalent: "Vec<T> / &mut Struct {...} / Static Type Dispatch".to_string(),
            why_chosen: "C99 flexible array members and runtime-sized VLAs have no direct stack-unsized C++/Rust equivalent, so Rust lowers them to heap-backed `Vec<T>` (`vec![vec![...; cols]; rows]`); C99 compound literals become `&mut Type { ... }`, and C11 `_Generic` compiles to type-safe static dispatch.".to_string(),
            ownership_and_iteration: "Ownership & Iteration: `Vec<T>` tracks capacity and length safely across runtime sizes, freeing nested buffers automatically and eliminating stack overflow and uninitialized trailing-array UB.".to_string(),
        });
    }
}

fn push_loop_and_style_explanations(
    source_code: &str,
    rust_code: &str,
    output_style: &RustOutputStyle,
    post_processing: &PostProcessingSummary,
    used_fallback: bool,
    transpiler_engine: &str,
    out: &mut Vec<ConversionExplanation>,
) {
    if source_code.contains("for (") || source_code.contains("for(") || rust_code.contains("for ") {
        out.push(ConversionExplanation {
            construct: "Loops & Range Iteration (for (...) → Rust Iterator)".to_string(),
            cpp_pattern: "for (int i = 0; i < n; i++) / for (auto& x : c)".to_string(),
            rust_equivalent: "for i in 0..n / for x in &mut c".to_string(),
            why_chosen: "Rust replaces C-style three-clause loops and raw iterators with half-open ranges (`0..n`) and the `IntoIterator` trait, removing off-by-one conditions and accidental loop-counter mutation.".to_string(),
            ownership_and_iteration: "Ownership & Iteration: `&collection` iterates by shared reference without moving elements; `&mut collection` iterates by mutable reference for in-place edits; bare `collection` consumes and yields owned values.".to_string(),
        });
    }

    let engine_note = if used_fallback {
        format!(
            "Translated by the built-in regex fallback ({}); fallback rules are heuristic and are not guaranteed equivalent for arbitrary C++ templates, overloads, or ownership unless compiled and tested.",
            transpiler_engine
        )
    } else {
        format!("Translated by primary transpiler engine ({}).", transpiler_engine)
    };

    let (style_label, style_why) = match output_style {
        RustOutputStyle::Idiomatic => {
            let summary_details = format!(
                "applied [{}], removed {} unnecessary `mut`, removed {} redundant casts, removed {} trivial unsafe wrappers, formatted with rustfmt: {}, rustc verified: {}",
                if post_processing.applied_passes.is_empty() {
                    "direct clean output".to_string()
                } else {
                    post_processing.applied_passes.join(", ")
                },
                post_processing.redundant_mut_removed,
                post_processing.redundant_casts_removed,
                post_processing.unsafe_blocks_removed,
                post_processing.formatted_with_rustfmt,
                post_processing.compiler_verified
            );
            (
                "Idiomatic Rust (Optional Learning View)",
                format!(
                    "{} Idiomatic mode is an optional learning view alongside Faithful Rust that removes boilerplate comments, redundant `mut`, unnecessary casts, and trivial `unsafe` blocks, then checks compilation with `rustc` ({}).",
                    engine_note, summary_details
                ),
            )
        }
        RustOutputStyle::Faithful => (
            "Faithful Rust (Raw Output)",
            format!(
                "{} Faithful mode preserves the uncleaned translator output 1:1 before optional idiomatic learning passes are applied.",
                engine_note
            ),
        ),
    };

    let verification_note = if post_processing.compiler_verified {
        "Verification: `rustc --emit=metadata` succeeded on the displayed Rust code. Click Run C/C++ and Run Rust to compare runtime behavior.".to_string()
    } else {
        format!(
            "Verification: NOT compiler-verified ({}). Review diagnostics before running or relying on this translation.",
            post_processing
                .verification_error
                .as_deref()
                .unwrap_or("rustc verification failed or was skipped")
        )
    };

    out.push(ConversionExplanation {
        construct: format!("Output Mode ({})", style_label),
        cpp_pattern: transpiler_engine.to_string(),
        rust_equivalent: match output_style {
            RustOutputStyle::Idiomatic => "Idiomatic Rust (Learning Mode)".to_string(),
            RustOutputStyle::Faithful => "Faithful Rust (Raw Output)".to_string(),
        },
        why_chosen: style_why,
        ownership_and_iteration: verification_note,
    });
}

pub fn generate_conversion_explanations(
    source_code: &str,
    source_language: &SourceLanguage,
    faithful_rust: &str,
    idiomatic_rust: &str,
    output_style: &RustOutputStyle,
    post_processing: &PostProcessingSummary,
    used_fallback: bool,
    transpiler_engine: &str,
) -> Vec<ConversionExplanation> {
    let active_rust = match output_style {
        RustOutputStyle::Faithful => faithful_rust,
        RustOutputStyle::Idiomatic => idiomatic_rust,
    };
    let mut explanations = Vec::new();

    push_vector_explanation(source_code, active_rust, &mut explanations);
    push_associative_collection_explanations(source_code, active_rust, &mut explanations);
    push_memory_and_pointer_explanations(source_code, active_rust, &mut explanations);
    push_string_struct_and_c_explanations(source_code, source_language, active_rust, &mut explanations);
    push_loop_and_style_explanations(
        source_code,
        active_rust,
        output_style,
        post_processing,
        used_fallback,
        transpiler_engine,
        &mut explanations,
    );

    explanations
}
