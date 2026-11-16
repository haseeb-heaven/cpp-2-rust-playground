use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeSet;

static INCLUDE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^#\s*include\s*[<"]([^>"]+)[>"]"#).unwrap());
static DEFINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+)$"#).unwrap());
static TEMPLATE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^template\s*<\s*(?:typename|class)\s+([A-Za-z_][A-Za-z0-9_]*)\s*>$"#).unwrap());
static CLASS_OR_STRUCT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(class|struct)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{?\s*$"#).unwrap());
static FUNCTION_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:<>\s*&]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:const\s*)?\{\s*$"#).unwrap()
});
static FOR_RANGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^for\s*\(\s*(?:int|size_t|long|auto)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);\s*[A-Za-z_][A-Za-z0-9_]*\s*(<=|<)\s*([^;]+);\s*[^)]+\)\s*\{\s*$"#).unwrap()
});
static FOR_EACH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^for\s*\(\s*(const\s+auto&|auto&|auto|[A-Za-z_][A-Za-z0-9_:<>]*&?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^)]+)\)\s*\{\s*$"#).unwrap()
});
static WHILE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^while\s*\((.+)\)\s*\{\s*$"#).unwrap());
static IF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^if\s*\((.+)\)\s*\{\s*$"#).unwrap());
static ELSE_IF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^}\s*else\s+if\s*\((.+)\)\s*\{\s*$"#).unwrap());
static SWITCH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^switch\s*\((.+)\)\s*\{\s*$"#).unwrap());
static CASE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^case\s+([^:]+):\s*$"#).unwrap());
static PRINTF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^printf\s*\(\s*"((?:[^"\\]|\\.)*)"\s*(?:,\s*(.+))?\)\s*;\s*$"#).unwrap());
static SCANF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^scanf\s*\(\s*"([^"]*)"\s*,\s*&?\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*;\s*$"#).unwrap());
static CIN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(?:std::)?cin\s*>>\s*([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap());
static VAR_DECL_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(const\s+)?([A-Za-z_][A-Za-z0-9_:<>,\s]*(?:\s*\*|\s*&)?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+);\s*$"#).unwrap()
});
static VAR_DECL_NO_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:<>,\s]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap()
});
static INCREMENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^([A-Za-z_][A-Za-z0-9_.\[\]]*)\s*\+\+\s*;\s*$"#).unwrap());
static DECREMENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^([A-Za-z_][A-Za-z0-9_.\[\]]*)\s*--\s*;\s*$"#).unwrap());

pub struct TranslatorState {
    pub required_imports: BTreeSet<String>,
    pub pending_generic: Option<String>,
    pub inside_main_function: bool,
    pub main_brace_depth: usize,
    pub current_brace_depth: usize,
    pub inside_switch_block: bool,
    pub inside_struct_name: Option<String>,
    pub struct_brace_depth: usize,
    pub inside_class_public: bool,
    pub collected_methods: Vec<String>,
    pub inside_inline_method: bool,
    pub inline_method_brace_depth: usize,
}

impl TranslatorState {
    pub fn new() -> Self {
        Self {
            required_imports: BTreeSet::new(),
            pending_generic: None,
            inside_main_function: false,
            main_brace_depth: 0,
            current_brace_depth: 0,
            inside_switch_block: false,
            inside_struct_name: None,
            struct_brace_depth: 0,
            inside_class_public: true,
            collected_methods: Vec::new(),
            inside_inline_method: false,
            inline_method_brace_depth: 0,
        }
    }
}

pub fn map_cpp_type_to_rust(raw_type: &str, state: &mut TranslatorState) -> String {
    let cleaned = raw_type
        .trim()
        .trim_start_matches("const ")
        .trim_end_matches('&')
        .trim();

    if let Some(inner) = cleaned.strip_suffix('*').map(|value| value.trim()) {
        let mapped_inner = map_cpp_type_to_rust(inner, state);
        return format!("Box<{}>", mapped_inner);
    }

    if let Some(inner) = extract_generic_inner(cleaned, "vector")
        .or_else(|| extract_generic_inner(cleaned, "std::vector"))
    {
        return format!("Vec<{}>", map_cpp_type_to_rust(inner, state));
    }

    if let Some(inner) = extract_generic_inner(cleaned, "unordered_map")
        .or_else(|| extract_generic_inner(cleaned, "std::unordered_map"))
        .or_else(|| extract_generic_inner(cleaned, "map"))
        .or_else(|| extract_generic_inner(cleaned, "std::map"))
    {
        state
            .required_imports
            .insert("std::collections::HashMap".to_string());
        let parts: Vec<&str> = split_top_level_comma(inner);
        if parts.len() == 2 {
            return format!(
                "HashMap<{}, {}>",
                map_cpp_type_to_rust(parts[0], state),
                map_cpp_type_to_rust(parts[1], state)
            );
        }
    }

    if let Some(inner) = extract_generic_inner(cleaned, "unordered_set")
        .or_else(|| extract_generic_inner(cleaned, "std::unordered_set"))
        .or_else(|| extract_generic_inner(cleaned, "set"))
        .or_else(|| extract_generic_inner(cleaned, "std::set"))
    {
        state
            .required_imports
            .insert("std::collections::HashSet".to_string());
        return format!("HashSet<{}>", map_cpp_type_to_rust(inner, state));
    }

    if let Some(inner) = extract_generic_inner(cleaned, "optional")
        .or_else(|| extract_generic_inner(cleaned, "std::optional"))
    {
        return format!("Option<{}>", map_cpp_type_to_rust(inner, state));
    }

    if let Some(inner) = extract_generic_inner(cleaned, "unique_ptr")
        .or_else(|| extract_generic_inner(cleaned, "std::unique_ptr"))
    {
        return format!("Box<{}>", map_cpp_type_to_rust(inner, state));
    }

    match cleaned {
        "int" | "signed" | "int32_t" => "i32".to_string(),
        "unsigned int" | "unsigned" | "uint32_t" => "u32".to_string(),
        "long" | "long long" | "int64_t" => "i64".to_string(),
        "unsigned long" | "unsigned long long" | "uint64_t" | "size_t" => "usize".to_string(),
        "short" | "int16_t" => "i16".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "bool" => "bool".to_string(),
        "char" => "char".to_string(),
        "void" => "()".to_string(),
        "string" | "std::string" => "String".to_string(),
        "auto" => "_".to_string(),
        other => other.to_string(),
    }
}

fn extract_generic_inner<'a>(input: &'a str, container_name: &str) -> Option<&'a str> {
    let prefix = format!("{}<", container_name);
    if input.starts_with(&prefix) && input.ends_with('>') {
        Some(&input[prefix.len()..input.len() - 1])
    } else {
        None
    }
}

fn split_top_level_comma(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, character) in input.char_indices() {
        match character {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(input[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(input[start..].trim());
    parts
}

pub fn translate_expression(expression: &str) -> String {
    let mut output = expression.trim().to_string();

    let replacements = [
        ("std::nullopt", "None"),
        ("nullptr", "None"),
        ("NULL", "None"),
        ("->", "."),
        (".push_back(", ".push("),
        (".pop_back()", ".pop()"),
        (".emplace_back(", ".push("),
        (".size()", ".len()"),
        (".length()", ".len()"),
        (".empty()", ".is_empty()"),
        (".front()", ".first().unwrap()"),
        (".back()", ".last().unwrap()"),
        ("std::cout", ""),
        ("std::endl", "\"\\n\""),
        ("std::string(", "String::from("),
    ];

    for (from, to) in replacements {
        output = output.replace(from, to);
    }

    if output.starts_with('{') && output.ends_with('}') {
        let inner = &output[1..output.len() - 1];
        output = format!("vec![{}]", inner.trim());
    }

    if output.contains(".count(") {
        let count_regex = Regex::new(r"\.count\(([^)]+)\)").unwrap();
        output = count_regex
            .replace_all(&output, ".contains_key(&$1)")
            .to_string();
    }

    if let Some(rest) = output.strip_prefix("new ") {
        output = format!("Box::new({})", rest.trim());
    }

    output
}

fn translate_parameters(raw_parameters: &str, state: &mut TranslatorState) -> String {
    let trimmed = raw_parameters.trim();
    if trimmed.is_empty() || trimmed == "void" {
        return String::new();
    }

    split_top_level_comma(trimmed)
        .into_iter()
        .filter_map(|parameter| {
            let tokens: Vec<&str> = parameter.split_whitespace().collect();
            if tokens.len() < 2 {
                return None;
            }
            let parameter_name = tokens.last().unwrap().trim_start_matches('&').trim_start_matches('*');
            let raw_type = tokens[..tokens.len() - 1].join(" ");
            let rust_type = map_cpp_type_to_rust(&raw_type, state);
            if parameter.contains('&') && !parameter.contains("const") {
                Some(format!("{}: &mut {}", parameter_name, rust_type))
            } else if parameter.contains('&') {
                Some(format!("{}: &{}", parameter_name, rust_type))
            } else {
                Some(format!("{}: {}", parameter_name, rust_type))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn translate_cout_statement(trimmed: &str) -> Option<String> {
    let without_semicolon = trimmed.trim_end_matches(';').trim();
    let rest = without_semicolon
        .strip_prefix("std::cout")
        .or_else(|| without_semicolon.strip_prefix("cout"))?
        .trim();

    if !rest.starts_with("<<") {
        return None;
    }

    let parts: Vec<&str> = rest
        .split("<<")
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect();

    let mut has_newline = false;
    let mut format_placeholders = Vec::new();
    let mut format_arguments = Vec::new();

    for part in parts {
        if part == "endl" || part == "std::endl" || part == "\"\\n\"" {
            has_newline = true;
        } else if part.starts_with('"') && part.ends_with('"') && part.len() >= 2 {
            let literal = &part[1..part.len() - 1];
            if let Some(stripped) = literal.strip_suffix("\\n") {
                has_newline = true;
                format_placeholders.push(stripped.to_string());
            } else {
                format_placeholders.push(literal.to_string());
            }
        } else {
            format_placeholders.push("{}".to_string());
            format_arguments.push(translate_expression(part));
        }
    }

    let macro_name = if has_newline { "println!" } else { "print!" };
    let format_string = format_placeholders.join("");
    if format_arguments.is_empty() {
        Some(format!("{}(\"{}\");", macro_name, format_string))
    } else {
        Some(format!(
            "{}(\"{}\", {});",
            macro_name,
            format_string,
            format_arguments.join(", ")
        ))
    }
}

fn translate_printf_statement(format_literal: &str, raw_args: Option<&str>) -> String {
    let mut has_newline = false;
    let mut converted_format = format_literal.to_string();
    if let Some(stripped) = converted_format.strip_suffix("\\n") {
        has_newline = true;
        converted_format = stripped.to_string();
    }

    let specifier_regex = Regex::new(r"%[-+0-9.*]*[diuoxXfFeEgGcsp]").unwrap();
    converted_format = specifier_regex.replace_all(&converted_format, "{}").to_string();
    let macro_name = if has_newline { "println!" } else { "print!" };

    match raw_args {
        Some(args) if !args.trim().is_empty() => {
            let translated_args = split_top_level_comma(args)
                .into_iter()
                .map(translate_expression)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}(\"{}\", {});", macro_name, converted_format, translated_args)
        }
        _ => format!("{}(\"{}\");", macro_name, converted_format),
    }
}

pub fn translate_single_line(raw_line: &str, state: &mut TranslatorState) -> String {
    let leading_whitespace: String = raw_line.chars().take_while(|c| c.is_whitespace()).collect();
    let trimmed = raw_line.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return raw_line.to_string();
    }

    if let Some(captures) = INCLUDE_REGEX.captures(trimmed) {
        let header = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        match header {
            "map" | "unordered_map" => {
                state
                    .required_imports
                    .insert("std::collections::HashMap".to_string());
            }
            "set" | "unordered_set" => {
                state
                    .required_imports
                    .insert("std::collections::HashSet".to_string());
            }
            _ => {}
        }
        return format!("{}// #include <{}>", leading_whitespace, header);
    }

    if trimmed == "using namespace std;" {
        return format!("{}// using namespace std;", leading_whitespace);
    }

    if let Some(captures) = DEFINE_REGEX.captures(trimmed) {
        let name = captures.get(1).unwrap().as_str();
        let value = captures.get(2).unwrap().as_str().trim();
        let rust_type = if value.starts_with('"') {
            "&str"
        } else if value.contains('.') {
            "f64"
        } else {
            "i64"
        };
        return format!("{}const {}: {} = {};", leading_whitespace, name, rust_type, value);
    }

    if let Some(captures) = TEMPLATE_REGEX.captures(trimmed) {
        let generic_name = captures.get(1).unwrap().as_str().to_string();
        state.pending_generic = Some(generic_name);
        return String::new();
    }

    if trimmed == "public:" {
        state.inside_class_public = true;
        return String::new();
    }
    if trimmed == "private:" || trimmed == "protected:" {
        state.inside_class_public = false;
        return String::new();
    }

    if let Some(captures) = CLASS_OR_STRUCT_REGEX.captures(trimmed) {
        let kind = captures.get(1).unwrap().as_str();
        let name = captures.get(2).unwrap().as_str();
        state.inside_struct_name = Some(name.to_string());
        state.struct_brace_depth = state.current_brace_depth;
        state.inside_class_public = kind == "struct";
        state.current_brace_depth += 1;

        let generic_suffix = state
            .pending_generic
            .take()
            .map(|generic| format!("<{}>", generic))
            .unwrap_or_default();

        return format!(
            "{}#[derive(Debug, Clone)]\n{}pub struct {}{} {{",
            leading_whitespace, leading_whitespace, name, generic_suffix
        );
    }

    if trimmed == "};" && state.inside_struct_name.is_some() {
        let struct_name = state.inside_struct_name.take().unwrap();
        state.current_brace_depth = state.current_brace_depth.saturating_sub(1);
        if state.collected_methods.is_empty() {
            return format!("{}}}", leading_whitespace);
        }
        let methods_block = state.collected_methods.drain(..).collect::<Vec<_>>().join("\n");
        return format!(
            "{}}}\n\n{}impl {} {{\n{}\n{}}}",
            leading_whitespace, leading_whitespace, struct_name, methods_block, leading_whitespace
        );
    }

    if state.inside_struct_name.is_some() && !state.inside_inline_method {
        if let Some(captures) = FUNCTION_DECL_REGEX.captures(trimmed) {
            let return_type_raw = captures.get(1).unwrap().as_str();
            let function_name = captures.get(2).unwrap().as_str();
            let params_raw = captures.get(3).unwrap().as_str();
            let rust_params = translate_parameters(params_raw, state);
            let self_param = if rust_params.is_empty() {
                "&self".to_string()
            } else {
                format!("&self, {}", rust_params)
            };
            let return_type = map_cpp_type_to_rust(return_type_raw, state);
            let return_clause = if return_type == "()" {
                String::new()
            } else {
                format!(" -> {}", return_type)
            };
            state.inside_inline_method = true;
            state.inline_method_brace_depth = state.current_brace_depth;
            state.current_brace_depth += 1;
            state
                .collected_methods
                .push(format!("    pub fn {}({}){} {{", function_name, self_param, return_clause));
            return String::new();
        }

        if let Some(captures) = VAR_DECL_NO_INIT_REGEX.captures(trimmed) {
            let raw_type = captures.get(1).unwrap().as_str();
            let field_name = captures.get(2).unwrap().as_str();
            let rust_type = map_cpp_type_to_rust(raw_type, state);
            let visibility = if state.inside_class_public { "pub " } else { "" };
            return format!("{}{}{}: {},", leading_whitespace, visibility, field_name, rust_type);
        }
    }

    if state.inside_inline_method {
        let open_count = trimmed.matches('{').count();
        let close_count = trimmed.matches('}').count();
        state.current_brace_depth = state.current_brace_depth + open_count - close_count.min(state.current_brace_depth + open_count);

        if trimmed == "}" && state.current_brace_depth <= state.inline_method_brace_depth {
            state.inside_inline_method = false;
            state.collected_methods.push("    }".to_string());
            return String::new();
        }

        let mut temp_state = TranslatorState::new();
        let translated_body = translate_statement_line(trimmed, &mut temp_state);
        for import in temp_state.required_imports {
            state.required_imports.insert(import);
        }
        state
            .collected_methods
            .push(format!("        {}", translated_body));
        return String::new();
    }

    let open_count = trimmed.matches('{').count();
    let close_count = trimmed.matches('}').count();
    let previous_depth = state.current_brace_depth;
    state.current_brace_depth =
        (state.current_brace_depth + open_count).saturating_sub(close_count);

    if state.inside_main_function && state.current_brace_depth <= state.main_brace_depth && trimmed == "}" {
        state.inside_main_function = false;
        return format!("{}}}", leading_whitespace);
    }

    if state.inside_switch_block && trimmed == "}" {
        state.inside_switch_block = false;
        return format!("{}}}", leading_whitespace);
    }

    if let Some(captures) = FUNCTION_DECL_REGEX.captures(trimmed) {
        let return_type_raw = captures.get(1).unwrap().as_str();
        let function_name = captures.get(2).unwrap().as_str();
        let params_raw = captures.get(3).unwrap().as_str();

        let generic_clause = state
            .pending_generic
            .take()
            .map(|generic| format!("<{}>", generic))
            .unwrap_or_default();

        if function_name == "main" {
            state.inside_main_function = true;
            state.main_brace_depth = previous_depth;
            return format!("{}fn main() {{", leading_whitespace);
        }

        let rust_params = translate_parameters(params_raw, state);
        let return_type = map_cpp_type_to_rust(return_type_raw, state);
        let return_clause = if return_type == "()" {
            String::new()
        } else {
            format!(" -> {}", return_type)
        };
        return format!(
            "{}fn {}{}({}){} {{",
            leading_whitespace, function_name, generic_clause, rust_params, return_clause
        );
    }

    format!(
        "{}{}",
        leading_whitespace,
        translate_statement_line(trimmed, state)
    )
}

fn translate_statement_line(trimmed: &str, state: &mut TranslatorState) -> String {
    if state.inside_main_function && trimmed == "return 0;" {
        return "// return 0;".to_string();
    }

    if let Some(cout_rust) = translate_cout_statement(trimmed) {
        return cout_rust;
    }

    if let Some(captures) = PRINTF_REGEX.captures(trimmed) {
        let format_literal = captures.get(1).unwrap().as_str();
        let raw_args = captures.get(2).map(|m| m.as_str());
        return translate_printf_statement(format_literal, raw_args);
    }

    if let Some(captures) = SCANF_REGEX.captures(trimmed) {
        let variable_name = captures.get(2).unwrap().as_str();
        return format!(
            "let mut _stdin_line = String::new(); std::io::stdin().read_line(&mut _stdin_line).unwrap(); let {}: i32 = _stdin_line.trim().parse().unwrap_or(0);",
            variable_name
        );
    }

    if let Some(captures) = CIN_REGEX.captures(trimmed) {
        let variable_name = captures.get(1).unwrap().as_str();
        return format!(
            "let mut _stdin_line = String::new(); std::io::stdin().read_line(&mut _stdin_line).unwrap(); {} = _stdin_line.trim().parse().unwrap_or_default();",
            variable_name
        );
    }

    if let Some(captures) = FOR_RANGE_REGEX.captures(trimmed) {
        let variable_name = captures.get(1).unwrap().as_str();
        let start_expr = translate_expression(captures.get(2).unwrap().as_str());
        let operator = captures.get(3).unwrap().as_str();
        let end_expr = translate_expression(captures.get(4).unwrap().as_str());
        let range_operator = if operator == "<=" { "..=" } else { ".." };
        return format!("for {} in {}{}{} {{", variable_name, start_expr, range_operator, end_expr);
    }

    if let Some(captures) = FOR_EACH_REGEX.captures(trimmed) {
        let binding_type = captures.get(1).unwrap().as_str().trim();
        let item_name = captures.get(2).unwrap().as_str();
        let collection_expr = translate_expression(captures.get(3).unwrap().as_str());
        if binding_type.contains("const") && binding_type.contains('&') {
            return format!("for {} in &{} {{", item_name, collection_expr);
        } else if binding_type.contains('&') {
            return format!("for {} in &mut {} {{", item_name, collection_expr);
        } else {
            return format!("for {} in {}.iter() {{", item_name, collection_expr);
        }
    }

    if let Some(captures) = WHILE_REGEX.captures(trimmed) {
        let condition = translate_expression(captures.get(1).unwrap().as_str());
        return format!("while {} {{", condition);
    }

    if let Some(captures) = ELSE_IF_REGEX.captures(trimmed) {
        let condition = translate_expression(captures.get(1).unwrap().as_str());
        return format!("}} else if {} {{", condition);
    }

    if let Some(captures) = IF_REGEX.captures(trimmed) {
        let condition = translate_expression(captures.get(1).unwrap().as_str());
        return format!("if {} {{", condition);
    }

    if let Some(captures) = SWITCH_REGEX.captures(trimmed) {
        let switch_expr = translate_expression(captures.get(1).unwrap().as_str());
        state.inside_switch_block = true;
        return format!("match {} {{", switch_expr);
    }

    if state.inside_switch_block {
        if let Some(captures) = CASE_REGEX.captures(trimmed) {
            let case_val = captures.get(1).unwrap().as_str().trim();
            return format!("{} => {{", case_val);
        }
        if trimmed == "default:" {
            return "_ => {".to_string();
        }
        if trimmed == "break;" {
            return "}".to_string();
        }
    }

    if let Some(captures) = INCREMENT_REGEX.captures(trimmed) {
        let target = captures.get(1).unwrap().as_str();
        return format!("{} += 1;", target);
    }

    if let Some(captures) = DECREMENT_REGEX.captures(trimmed) {
        let target = captures.get(1).unwrap().as_str();
        return format!("{} -= 1;", target);
    }

    if let Some(rest) = trimmed.strip_prefix("delete ") {
        let target = rest.trim_end_matches(';').trim();
        return format!("drop({});", target);
    }

    if let Some(captures) = VAR_DECL_INIT_REGEX.captures(trimmed) {
        let is_const = captures.get(1).is_some();
        let raw_type = captures.get(2).unwrap().as_str();
        let variable_name = captures.get(3).unwrap().as_str();
        let raw_value = captures.get(4).unwrap().as_str();
        let rust_type = map_cpp_type_to_rust(raw_type, state);
        let mut translated_value = translate_expression(raw_value);
        if rust_type == "String" && translated_value.starts_with('"') {
            translated_value = format!("String::from({})", translated_value);
        }
        let mutability = if is_const { "" } else { "mut " };
        if rust_type == "_" {
            return format!("let {}{} = {};", mutability, variable_name, translated_value);
        }
        return format!(
            "let {}{}: {} = {};",
            mutability, variable_name, rust_type, translated_value
        );
    }

    if let Some(captures) = VAR_DECL_NO_INIT_REGEX.captures(trimmed) {
        let raw_type = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        if !matches!(raw_type, "return" | "break" | "continue" | "else") {
            let rust_type = map_cpp_type_to_rust(raw_type, state);
            if rust_type.starts_with("Vec<") {
                return format!("let mut {}: {} = Vec::new();", variable_name, rust_type);
            } else if rust_type.starts_with("HashMap<") {
                return format!("let mut {}: {} = HashMap::new();", variable_name, rust_type);
            } else if rust_type.starts_with("HashSet<") {
                return format!("let mut {}: {} = HashSet::new();", variable_name, rust_type);
            } else if rust_type == "String" {
                return format!("let mut {}: String = String::new();", variable_name);
            } else {
                return format!("let mut {}: {} = Default::default();", variable_name, rust_type);
            }
        }
    }

    translate_expression(trimmed)
}
