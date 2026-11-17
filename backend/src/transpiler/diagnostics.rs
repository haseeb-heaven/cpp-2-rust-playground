use crate::models::{ConversionDiagnostic, DiagnosticSeverity, ErrorCategory};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static PREPROCESSOR_KEYWORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^#\s*([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static VAR_DECL_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(?:const\s+)?(?:std::)?[A-Za-z_][A-Za-z0-9_:<>,\s]*(?:\s*\*|\s*&)?\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:=|;|\()",
    )
    .unwrap()
});
static FOR_EACH_TARGET_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^for\s*\(\s*[^:]+:\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*\{?\s*$").unwrap()
});
static FOR_LOOP_VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^for\s*\(\s*(?:int|size_t|long|auto)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap()
});
static FUNCTION_PARAMS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z_][A-Za-z0-9_:<>,\s*&]*?\s+[A-Za-z_][A-Za-z0-9_]*\s*\(([^)]*)\)\s*(?:const\s*)?\{\s*$").unwrap()
});
static UNSUPPORTED_EXCEPTIONS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\btry\s*\{|\bcatch\s*\(|\bthrow\b)").unwrap()
});
static UNSUPPORTED_INHERITANCE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\bclass\s+[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:public|protected|private)\s+[A-Za-z_]|\bvirtual\s+|\boverride\b)").unwrap()
});
static UNSUPPORTED_OPERATOR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\boperator\s*(?:\+\+|--|<<|>>|==|!=|<=|>=|\+=|-=|\*=|/=|\[\]|\(\)|[+\-*/%<>&|^!~=])").unwrap()
});
static UNSUPPORTED_TEMPLATE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\b(?:typename|class)\s*\.\.\.|\btemplate\s*<\s*>)").unwrap()
});
static UNSUPPORTED_CAST_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:dynamic_cast|reinterpret_cast|const_cast)\s*<").unwrap()
});
static UNSUPPORTED_NAMESPACE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*namespace\s+[A-Za-z_][A-Za-z0-9_:]*\s*\{?\s*$").unwrap()
});
static UNSUPPORTED_DELETE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*delete(?:\s*\[\s*\])?\s+[A-Za-z_*(&]").unwrap()
});
static UNSUPPORTED_ASM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\b__asm__\b|\basm\s*\()").unwrap()
});

fn strip_string_literals_for_scan(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut in_quotes = false;
    let mut prev_escape = false;
    for ch in line.chars() {
        if in_quotes {
            if ch == '"' && !prev_escape {
                in_quotes = false;
            }
            prev_escape = ch == '\\' && !prev_escape;
            output.push(' ');
            continue;
        }
        if ch == '"' {
            in_quotes = true;
            output.push(' ');
        } else {
            output.push(ch);
        }
    }
    output
}

pub fn classify_unsupported_construct(
    trimmed_line: &str,
) -> Option<(&'static str, &'static str, &'static str)> {
    if trimmed_line.is_empty()
        || trimmed_line.starts_with("//")
        || trimmed_line.starts_with("/*")
        || trimmed_line.starts_with('#')
    {
        return None;
    }

    let code_only = strip_string_literals_for_scan(trimmed_line);
    let sanitized = code_only.trim();

    if UNSUPPORTED_EXCEPTIONS_REGEX.is_match(sanitized) {
        return Some((
            "C++ exceptions (try/catch/throw)",
            "C++ exceptions ('try'/'catch'/'throw') are not supported by the fallback translator; Rust uses Result<T, E>.",
            "Replace exceptions with Result<T, E> return types and '?' propagation in Rust.",
        ));
    }
    if UNSUPPORTED_INHERITANCE_REGEX.is_match(sanitized) {
        return Some((
            "C++ inheritance / virtual dispatch",
            "C++ inheritance and virtual methods are not supported by the fallback translator; Rust uses traits and composition.",
            "Model shared interfaces with Rust 'trait' definitions and struct composition.",
        ));
    }
    if UNSUPPORTED_OPERATOR_REGEX.is_match(sanitized) {
        return Some((
            "C++ operator overloading",
            "C++ operator overloading is not supported by the fallback translator; Rust uses std::ops traits.",
            "Implement Rust traits such as std::ops::Add, PartialEq, or Display instead of operator overloads.",
        ));
    }
    if UNSUPPORTED_TEMPLATE_REGEX.is_match(sanitized) {
        return Some((
            "Variadic templates / template specialization",
            "C++ variadic templates and template specializations are not supported by the fallback translator.",
            "Install Cpp2Rust CLI for full Clang AST template support, or rewrite using slices/tuples/macros.",
        ));
    }
    if UNSUPPORTED_CAST_REGEX.is_match(sanitized) {
        return Some((
            "C++ RTTI / low-level casts",
            "C++ RTTI and low-level casts ('dynamic_cast'/'reinterpret_cast'/'const_cast') are not supported by the fallback translator.",
            "Use Rust enums/traits for safe dispatch, or explicit unsafe casts where required.",
        ));
    }
    if UNSUPPORTED_NAMESPACE_REGEX.is_match(sanitized) {
        return Some((
            "C++ namespace block",
            "C++ namespace blocks are not supported by the fallback translator; Rust uses 'mod' blocks.",
            "Replace 'namespace name { ... }' with Rust 'mod name { ... }'.",
        ));
    }
    if UNSUPPORTED_DELETE_REGEX.is_match(sanitized) {
        return Some((
            "Manual delete / delete[]",
            "Manual 'delete'/'delete[]' is not supported in safe Rust; memory is freed via Drop.",
            "Remove manual 'delete' calls and rely on Box<T> / Vec<T> RAII ownership in Rust.",
        ));
    }
    if UNSUPPORTED_ASM_REGEX.is_match(sanitized) {
        return Some((
            "Inline assembly",
            "Inline assembly is not supported by the fallback translator.",
            "Use Rust's std::arch::asm! macro for target-specific assembly.",
        ));
    }

    None
}

pub fn detect_unsupported_constructs(
    source_lines: &[&str],
) -> (Vec<ConversionDiagnostic>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut construct_names = Vec::new();
    let mut seen = HashSet::new();

    for (idx, raw_line) in source_lines.iter().enumerate() {
        let trimmed = raw_line.trim();
        if let Some((construct_name, message, fix)) = classify_unsupported_construct(trimmed) {
            if seen.insert(construct_name.to_string()) {
                construct_names.push(construct_name.to_string());
            }
            diagnostics.push(ConversionDiagnostic {
                line_number: idx + 1,
                severity: DiagnosticSeverity::Error,
                category: ErrorCategory::Conversion,
                message: message.to_string(),
                suggested_fix: Some(fix.to_string()),
            });
        }
    }

    (diagnostics, construct_names)
}

const VALID_PREPROCESSOR_DIRECTIVES: &[&str] = &[
    "include", "define", "undef", "if", "ifdef", "ifndef", "else", "elif", "endif", "pragma",
    "error", "warning", "line",
];

fn levenshtein_distance(first: &str, second: &str) -> usize {
    let first_chars: Vec<char> = first.chars().collect();
    let second_chars: Vec<char> = second.chars().collect();
    let mut previous_row: Vec<usize> = (0..=second_chars.len()).collect();

    for (first_index, first_char) in first_chars.iter().enumerate() {
        let mut current_row = vec![first_index + 1];
        for (second_index, second_char) in second_chars.iter().enumerate() {
            let cost = if first_char == second_char { 0 } else { 1 };
            let deletion = previous_row[second_index + 1] + 1;
            let insertion = current_row[second_index] + 1;
            let substitution = previous_row[second_index] + cost;
            current_row.push(deletion.min(insertion).min(substitution));
        }
        previous_row = current_row;
    }

    previous_row[second_chars.len()]
}

fn find_closest_match<'a>(target: &str, candidates: &'a HashSet<String>) -> Option<&'a str> {
    candidates
        .iter()
        .filter_map(|candidate| {
            let distance = levenshtein_distance(target, candidate);
            if distance <= 2 {
                Some((distance, candidate.as_str()))
            } else {
                None
            }
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

pub fn analyze_source_diagnostics(source_lines: &[&str]) -> Vec<ConversionDiagnostic> {
    let mut diagnostics = analyze_base_source_diagnostics(source_lines);
    let (unsupported_diagnostics, _) = detect_unsupported_constructs(source_lines);
    diagnostics.extend(unsupported_diagnostics);
    diagnostics
}

pub fn analyze_base_source_diagnostics(source_lines: &[&str]) -> Vec<ConversionDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut brace_stack: Vec<usize> = Vec::new();
    let mut declared_identifiers: HashSet<String> = HashSet::new();

    for (zero_based_index, raw_line) in source_lines.iter().enumerate() {
        let line_number = zero_based_index + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with('#') {
            if let Some(captures) = PREPROCESSOR_KEYWORD_REGEX.captures(trimmed) {
                let directive = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                if !VALID_PREPROCESSOR_DIRECTIVES.contains(&directive) {
                    let suggestion = if levenshtein_distance(directive, "include") <= 2 {
                        Some(trimmed.replacen(directive, "include", 1))
                    } else {
                        Some("Use a valid C/C++ preprocessor directive such as '#include' or '#define'.".to_string())
                    };
                    diagnostics.push(ConversionDiagnostic {
                        line_number,
                        severity: DiagnosticSeverity::Error,
                        category: ErrorCategory::Syntax,
                        message: format!("Unknown preprocessor directive '#{}'.", directive),
                        suggested_fix: suggestion.map(|fix| format!("Did you mean '{}'?", fix)),
                    });
                }
            }
            continue;
        }

        for character in trimmed.chars() {
            if character == '{' {
                brace_stack.push(line_number);
            } else if character == '}' {
                if brace_stack.pop().is_none() {
                    diagnostics.push(ConversionDiagnostic {
                        line_number,
                        severity: DiagnosticSeverity::Error,
                        category: ErrorCategory::Syntax,
                        message: "Unexpected closing brace '}' with no matching opening brace."
                            .to_string(),
                        suggested_fix: Some("Remove the extra '}' or add a matching '{'.".to_string()),
                    });
                }
            }
        }

        if let Some(captures) = FUNCTION_PARAMS_REGEX.captures(trimmed) {
            if let Some(params_match) = captures.get(1) {
                for param_chunk in params_match.as_str().split(',') {
                    if let Some(param_name) = param_chunk.split_whitespace().last() {
                        let cleaned_name = param_name.trim_start_matches('&').trim_start_matches('*');
                        if !cleaned_name.is_empty() {
                            declared_identifiers.insert(cleaned_name.to_string());
                        }
                    }
                }
            }
        } else if let Some(captures) = FOR_LOOP_VAR_REGEX.captures(trimmed) {
            if let Some(loop_var) = captures.get(1) {
                declared_identifiers.insert(loop_var.as_str().to_string());
            }
        } else if let Some(captures) = VAR_DECL_NAME_REGEX.captures(trimmed) {
            if let Some(identifier) = captures.get(1) {
                let name = identifier.as_str();
                if !matches!(name, "main" | "if" | "for" | "while" | "switch" | "return") {
                    declared_identifiers.insert(name.to_string());
                }
            }
        }

        if let Some(captures) = FOR_EACH_TARGET_REGEX.captures(trimmed) {
            if let Some(collection_match) = captures.get(1) {
                let collection_name = collection_match.as_str();
                if !declared_identifiers.is_empty()
                    && !declared_identifiers.contains(collection_name)
                {
                    let suggestion = find_closest_match(collection_name, &declared_identifiers)
                        .map(|closest| format!("Did you mean '{}'?", closest))
                        .unwrap_or_else(|| {
                            format!("Declare '{}' before iterating over it.", collection_name)
                        });
                    diagnostics.push(ConversionDiagnostic {
                        line_number,
                        severity: DiagnosticSeverity::Error,
                        category: ErrorCategory::Compilation,
                        message: format!(
                            "Undeclared identifier '{}' in range-based for loop.",
                            collection_name
                        ),
                        suggested_fix: Some(suggestion),
                    });
                }
            }
        }

        if trimmed.contains("malloc(") || trimmed.contains("calloc(") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Memory,
                message: "Manual heap allocation detected (malloc/calloc).".to_string(),
                suggested_fix: Some(
                    "Use Rust's Vec::with_capacity() or Box::new() for safe heap allocation."
                        .to_string(),
                ),
            });
        }

        if trimmed.contains("gets(") || trimmed.contains("strcpy(") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Memory,
                message: "Unsafe C string function detected (gets/strcpy).".to_string(),
                suggested_fix: Some(
                    "Use String::from() or std::io::stdin().read_line() in Rust.".to_string(),
                ),
            });
        }

        if trimmed.starts_with("goto ") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "'goto' statements are not supported in Rust.".to_string(),
                suggested_fix: Some(
                    "Refactor using a labeled loop ('label: loop { ... break 'label; })."
                        .to_string(),
                ),
            });
        }

        if trimmed.contains("long double") || trimmed.contains("LDBL_") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C/C++ 'long double' and LDBL_* macros have no distinct stable Rust primitive type (f128 is nightly-only); approximated using 'f64'.".to_string(),
                suggested_fix: Some(
                    "Use 'f64' (f64::MANTISSA_DIGITS, f64::DIGITS, f64::MAX) in stable Rust, or the 'f128' / 'rug' crate if extended precision is required.".to_string(),
                ),
            });
        }

        if trimmed.starts_with("union ") || trimmed.starts_with("typedef union") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C/C++ untagged 'union' has no safe Rust equivalent (reading a Rust 'union' field requires 'unsafe'); converted to a safe Rust 'struct'.".to_string(),
                suggested_fix: Some(
                    "Use a Rust tagged 'enum' for type-safe variants, or '#[repr(C)] union' with 'unsafe' blocks for low-level memory overlap.".to_string(),
                ),
            });
        }

        if trimmed.contains("void*") || trimmed.contains("void *") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C/C++ untyped 'void*' pointer has no safe Rust equivalent.".to_string(),
                suggested_fix: Some(
                    "Use generics '<T>', trait objects 'Box<dyn Any>', or an enum instead of 'void*'.".to_string(),
                ),
            });
        }

        if trimmed.contains("volatile ") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C/C++ 'volatile' qualifier has no variable-level equivalent in Rust.".to_string(),
                suggested_fix: Some(
                    "Use 'std::ptr::read_volatile' and 'std::ptr::write_volatile' for hardware register access.".to_string(),
                ),
            });
        }

        if trimmed.contains("va_list")
            || trimmed.contains("va_start")
            || trimmed.contains("setjmp")
            || trimmed.contains("longjmp")
        {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C variadic functions ('va_list') and non-local jumps ('setjmp'/'longjmp') have no direct Rust equivalent.".to_string(),
                suggested_fix: Some(
                    "Use slices '&[T]' or 'macro_rules!' instead of 'va_list', and 'Result<T, E>' with '?' instead of 'setjmp'/'longjmp'.".to_string(),
                ),
            });
        }

        if trimmed.ends_with("[];") && !trimmed.contains('=') {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C99 Flexible Array Member ('[]' at end of struct) is exclusive to ISO C (forbidden in C++) and has no fixed-size struct layout in safe Rust; modeled as 'Vec<T>'.".to_string(),
                suggested_fix: Some(
                    "Use a 'Vec<T>' field inside the Rust struct instead of manual 'malloc(sizeof(S) + n * sizeof(T))'.".to_string(),
                ),
            });
        }

        if trimmed.contains("_Generic(") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C11 '_Generic' compile-time type selection is exclusive to ISO C (not supported in C++); dispatched via static type resolution in Rust.".to_string(),
                suggested_fix: Some(
                    "Use Rust traits (e.g., 'trait TypeName { fn type_name() -> &\\'static str; }') for extensible type dispatch.".to_string(),
                ),
            });
        }

        if trimmed.contains("&(") && trimmed.contains("){") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C99 addressable compound literal ('&(Type){...}') is exclusive to ISO C (illegal in C++); bound to a mutable stack variable in Rust.".to_string(),
                suggested_fix: Some(
                    "Bind a local 'let mut v = Type { ... };' and pass '&mut v' in Rust.".to_string(),
                ),
            });
        }

        if trimmed.contains(" restrict ") || trimmed.contains("[static ") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "C99 'restrict' pointer qualifier and '[static N]' array parameter are exclusive to ISO C; mapped to Rust '&[T]' and '&mut T' (which enforces non-aliasing by default).".to_string(),
                suggested_fix: Some(
                    "In Rust, '&mut T' guarantees exclusive non-aliased mutable access and '&[T]' / '&[T; N]' provides bounds-checked slices.".to_string(),
                ),
            });
        }
    }

    for unclosed_line in brace_stack {
        diagnostics.push(ConversionDiagnostic {
            line_number: unclosed_line,
            severity: DiagnosticSeverity::Error,
            category: ErrorCategory::Syntax,
            message: "Unclosed opening brace '{'.".to_string(),
            suggested_fix: Some("Add a matching closing brace '}'.".to_string()),
        });
    }

    diagnostics
}
