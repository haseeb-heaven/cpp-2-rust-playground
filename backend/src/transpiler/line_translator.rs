use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};

static INCLUDE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^#\s*include\s*[<"]([^>"]+)[>"]"#).unwrap());
static DEFINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+)$"#).unwrap());
static TEMPLATE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^template\s*<\s*(?:typename|class)\s+([A-Za-z_][A-Za-z0-9_]*)\s*>$"#).unwrap()
});
static CLASS_OR_STRUCT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(class|struct)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{?\s*$"#).unwrap());
static FUNCTION_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:<>,\s*&]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:const\s*)?\{\s*$"#).unwrap()
});
static FOR_RANGE_UP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^for\s*\(\s*(int|size_t|long|auto)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);\s*[A-Za-z_][A-Za-z0-9_]*\s*(<=|<)\s*([^;]+);\s*[^)]+\)\s*\{\s*$"#).unwrap()
});
static FOR_RANGE_DOWN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^for\s*\(\s*(?:int|size_t|long|auto)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);\s*[A-Za-z_][A-Za-z0-9_]*\s*(>=|>)\s*([^;]+);\s*[^)]*--\s*\)\s*\{\s*$"#).unwrap()
});
static FOR_EACH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^for\s*\(\s*(const\s+auto&|auto&|auto|[A-Za-z_][A-Za-z0-9_:<>]*&?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^)]+)\)\s*\{\s*$"#).unwrap()
});
static WHILE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^while\s*\((.+)\)\s*\{\s*$"#).unwrap());
static IF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^if\s*\((.+)\)\s*\{\s*$"#).unwrap());
static ELSE_IF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\}\s*else\s+if\s*\((.+)\)\s*\{\s*$"#).unwrap());
static ELSE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\}\s*else\s*\{\s*$"#).unwrap());
static SWITCH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^switch\s*\((.+)\)\s*\{\s*$"#).unwrap());
static CASE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^case\s+([^:]+):\s*$"#).unwrap());
static PRINTF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^printf\s*\(\s*"((?:[^"\\]|\\.)*)"\s*(?:,\s*(.+))?\)\s*;\s*$"#).unwrap()
});
static SCANF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^scanf\s*\(\s*"([^"]*)"\s*,\s*&?\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*;\s*$"#)
        .unwrap()
});
static CIN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(?:std::)?cin\s*>>\s*([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap());
static VECTOR_CTOR_TWO_ARGS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:std::)?vector<(.+)>\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^,]+),\s*([^)]+)\)\s*;\s*$"#).unwrap()
});
static VECTOR_CTOR_ONE_ARG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:std::)?vector<(.+)>\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]+)\)\s*;\s*$"#)
        .unwrap()
});
static SORT_CALL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:std::)?sort\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\.begin\(\)\s*,\s*[A-Za-z_][A-Za-z0-9_]*\.end\(\)\s*\)\s*;\s*$"#).unwrap()
});
static REVERSE_CALL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:std::)?reverse\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\.begin\(\)\s*,\s*[A-Za-z_][A-Za-z0-9_]*\.end\(\)\s*\)\s*;\s*$"#).unwrap()
});
static MAP_PAIR_INSERT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\.insert\s*\(\s*\{\s*([^,]+)\s*,\s*([^}]+)\s*\}\s*\)\s*;\s*$"#).unwrap()
});
static MAP_BRACKET_ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\[(.+)\]\s*=\s*(.+);\s*$"#).unwrap()
});
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
static RETURN_BRACE_INIT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^return\s*\{(.*)\}\s*;\s*$"#).unwrap());
static RETURN_STMT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^return\s+(.+);\s*$"#).unwrap());
static FLOAT_SUFFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\b(\d+\.\d+)[fF]\b"#).unwrap());
static INDEX_ACCESS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"([A-Za-z_][A-Za-z0-9_]*)\[([^\[\]]+)\]"#).unwrap());

pub struct TranslatorState {
    pub required_imports: BTreeSet<String>,
    pub pending_generic: Option<String>,
    pub inside_main_function: bool,
    pub main_brace_depth: usize,
    pub current_brace_depth: usize,
    pub current_function_returns_option: bool,
    pub function_brace_depth: usize,
    pub inside_switch_block: bool,
    pub inside_struct_name: Option<String>,
    pub struct_generic: Option<String>,
    pub struct_fields: Vec<String>,
    pub struct_brace_depth: usize,
    pub inside_class_public: bool,
    pub collected_methods: Vec<String>,
    pub inside_inline_method: bool,
    pub inline_method_brace_depth: usize,
    pub method_parameter_names: HashSet<String>,
    pub declared_variables: HashSet<String>,
    pub hashmap_variables: HashSet<String>,
    pub hashset_variables: HashSet<String>,
    pub queue_variables: HashSet<String>,
    pub string_variables: HashSet<String>,
    pub float_vector_variables: HashSet<String>,
    pub option_variables: HashSet<String>,
    pub option_functions: HashSet<String>,
    pub function_ref_params: HashMap<String, Vec<Option<bool>>>,
}

impl TranslatorState {
    pub fn new() -> Self {
        Self {
            required_imports: BTreeSet::new(),
            pending_generic: None,
            inside_main_function: false,
            main_brace_depth: 0,
            current_brace_depth: 0,
            current_function_returns_option: false,
            function_brace_depth: 0,
            inside_switch_block: false,
            inside_struct_name: None,
            struct_generic: None,
            struct_fields: Vec::new(),
            struct_brace_depth: 0,
            inside_class_public: true,
            collected_methods: Vec::new(),
            inside_inline_method: false,
            inline_method_brace_depth: 0,
            method_parameter_names: HashSet::new(),
            declared_variables: HashSet::new(),
            hashmap_variables: HashSet::new(),
            hashset_variables: HashSet::new(),
            queue_variables: HashSet::new(),
            string_variables: HashSet::new(),
            float_vector_variables: HashSet::new(),
            option_variables: HashSet::new(),
            option_functions: HashSet::new(),
            function_ref_params: HashMap::new(),
        }
    }

    fn reset_function_local_tracking(&mut self) {
        self.method_parameter_names.clear();
        self.declared_variables.clear();
        self.hashmap_variables.clear();
        self.hashset_variables.clear();
        self.queue_variables.clear();
        self.string_variables.clear();
        self.float_vector_variables.clear();
        self.option_variables.clear();
    }

    fn register_variable_type(&mut self, variable_name: &str, rust_type: &str) {
        self.declared_variables.insert(variable_name.to_string());
        self.hashmap_variables.remove(variable_name);
        self.hashset_variables.remove(variable_name);
        self.queue_variables.remove(variable_name);
        self.string_variables.remove(variable_name);
        self.float_vector_variables.remove(variable_name);
        self.option_variables.remove(variable_name);

        if rust_type.starts_with("HashMap<") {
            self.hashmap_variables.insert(variable_name.to_string());
        } else if rust_type.starts_with("HashSet<") {
            self.hashset_variables.insert(variable_name.to_string());
        } else if rust_type.starts_with("VecDeque<") {
            self.queue_variables.insert(variable_name.to_string());
        } else if rust_type == "String" {
            self.string_variables.insert(variable_name.to_string());
        } else if rust_type == "Vec<f32>" || rust_type == "Vec<f64>" {
            self.float_vector_variables
                .insert(variable_name.to_string());
        } else if rust_type.starts_with("Option<") {
            self.option_variables.insert(variable_name.to_string());
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
        .or_else(|| extract_generic_inner(cleaned, "stack"))
        .or_else(|| extract_generic_inner(cleaned, "std::stack"))
    {
        return format!("Vec<{}>", map_cpp_type_to_rust(inner, state));
    }

    if let Some(inner) = extract_generic_inner(cleaned, "queue")
        .or_else(|| extract_generic_inner(cleaned, "std::queue"))
        .or_else(|| extract_generic_inner(cleaned, "deque"))
        .or_else(|| extract_generic_inner(cleaned, "std::deque"))
    {
        state
            .required_imports
            .insert("std::collections::VecDeque".to_string());
        return format!("VecDeque<{}>", map_cpp_type_to_rust(inner, state));
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
        other => {
            if let Some(open_pos) = other.find('<') {
                if other.ends_with('>') {
                    let base_name = &other[..open_pos];
                    let inner = &other[open_pos + 1..other.len() - 1];
                    let mapped_args = split_top_level_comma(inner)
                        .into_iter()
                        .map(|arg| map_cpp_type_to_rust(arg, state))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!("{}<{}>", base_name, mapped_args);
                }
            }
            other.to_string()
        }
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
    let mut in_quotes = false;
    let mut prev_escape = false;
    for (index, character) in input.char_indices() {
        if in_quotes {
            if character == '"' && !prev_escape {
                in_quotes = false;
            }
            prev_escape = character == '\\' && !prev_escape;
            continue;
        }
        match character {
            '"' => in_quotes = true,
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

fn transform_outside_string_literals<F>(input: &str, mut transform: F) -> String
where
    F: FnMut(&str) -> String,
{
    let mut result = String::with_capacity(input.len());
    let mut segment_start = 0usize;
    let mut in_quotes = false;
    let mut prev_escape = false;

    for (index, character) in input.char_indices() {
        if in_quotes {
            if character == '"' && !prev_escape {
                in_quotes = false;
                result.push_str(&input[segment_start..=index]);
                segment_start = index + 1;
            }
            prev_escape = character == '\\' && !prev_escape;
        } else if character == '"' {
            if index > segment_start {
                result.push_str(&transform(&input[segment_start..index]));
            }
            in_quotes = true;
            prev_escape = false;
            segment_start = index;
        }
    }

    if segment_start < input.len() {
        if in_quotes {
            result.push_str(&input[segment_start..]);
        } else {
            result.push_str(&transform(&input[segment_start..]));
        }
    }

    result
}

fn adjust_reference_calls(input: &str, function_ref_params: &HashMap<String, Vec<Option<bool>>>) -> String {
    if function_ref_params.is_empty() {
        return input.to_string();
    }

    let mut output = input.to_string();
    for (func_name, ref_modes) in function_ref_params {
        if ref_modes.iter().all(|mode| mode.is_none()) {
            continue;
        }
        let call_prefix_regex =
            Regex::new(&format!(r"\b{}\s*\(", regex::escape(func_name))).unwrap();
        let mut rebuilt = String::new();
        let mut cursor = 0usize;

        while let Some(found) = call_prefix_regex.find(&output[cursor..]) {
            let match_start = cursor + found.start();
            let open_paren_end = cursor + found.end();
            rebuilt.push_str(&output[cursor..open_paren_end]);

            let mut depth = 1usize;
            let mut in_quotes = false;
            let mut prev_escape = false;
            let mut close_paren_index = None;

            for (offset, character) in output[open_paren_end..].char_indices() {
                if in_quotes {
                    if character == '"' && !prev_escape {
                        in_quotes = false;
                    }
                    prev_escape = character == '\\' && !prev_escape;
                    continue;
                }
                match character {
                    '"' => in_quotes = true,
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            close_paren_index = Some(open_paren_end + offset);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if let Some(close_idx) = close_paren_index {
                let raw_args = &output[open_paren_end..close_idx];
                let args = split_top_level_comma(raw_args);
                let adjusted_args: Vec<String> = args
                    .into_iter()
                    .enumerate()
                    .map(|(index, arg)| {
                        let trimmed_arg = arg.trim();
                        if trimmed_arg.is_empty() {
                            return String::new();
                        }
                        match ref_modes.get(index).copied().flatten() {
                            Some(true) if !trimmed_arg.starts_with('&') => {
                                format!("&mut {}", trimmed_arg)
                            }
                            Some(false) if !trimmed_arg.starts_with('&') => {
                                format!("&{}", trimmed_arg)
                            }
                            _ => trimmed_arg.to_string(),
                        }
                    })
                    .collect();
                rebuilt.push_str(&adjusted_args.join(", "));
                rebuilt.push(')');
                cursor = close_idx + 1;
            } else {
                cursor = open_paren_end;
            }
            let _ = match_start;
        }
        rebuilt.push_str(&output[cursor..]);
        output = rebuilt;
    }

    output
}

pub fn translate_expression_with_state(expression: &str, state: &TranslatorState) -> String {
    let mut output = expression.trim().to_string();

    let make_unique_regex =
        Regex::new(r"(?:std::)?make_unique<[^>]+>\(([^)]*)\)").unwrap();
    output = make_unique_regex
        .replace_all(&output, "Box::new($1)")
        .to_string();

    for queue_var in &state.queue_variables {
        let escaped = regex::escape(queue_var);
        output = Regex::new(&format!(r"\b{}\.push_back\(", escaped))
            .unwrap()
            .replace_all(&output, format!("{}.__q_push_back__(", queue_var))
            .to_string();
        output = Regex::new(&format!(r"\b{}\.pop_back\(\)", escaped))
            .unwrap()
            .replace_all(&output, format!("{}.__q_pop_back__()", queue_var))
            .to_string();
        output = Regex::new(&format!(r"\b{}\.push\(", escaped))
            .unwrap()
            .replace_all(&output, format!("{}.__q_push_back__(", queue_var))
            .to_string();
        output = Regex::new(&format!(r"\b{}\.pop\(\)", escaped))
            .unwrap()
            .replace_all(&output, format!("{}.__q_pop_front__()", queue_var))
            .to_string();
        output = Regex::new(&format!(r"\b{}\.front\(\)", escaped))
            .unwrap()
            .replace_all(&output, format!("*{}.__q_front__().unwrap()", queue_var))
            .to_string();
        output = Regex::new(&format!(r"\b{}\.back\(\)", escaped))
            .unwrap()
            .replace_all(&output, format!("*{}.__q_back__().unwrap()", queue_var))
            .to_string();
    }

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
        (".has_value()", ".is_some()"),
        (".value()", ".unwrap()"),
        ("std::min(", "std::cmp::min("),
        ("std::max(", "std::cmp::max("),
        ("std::cout", ""),
        ("std::endl", "\"\\n\""),
        ("std::string(", "String::from("),
    ];

    for (from, to) in replacements {
        output = output.replace(from, to);
    }

    output = output
        .replace(".__q_push_back__(", ".push_back(")
        .replace(".__q_pop_back__()", ".pop_back()")
        .replace(".__q_pop_front__()", ".pop_front()")
        .replace(".__q_front__()", ".front()")
        .replace(".__q_back__()", ".back()");

    if output.starts_with('{') && output.ends_with('}') {
        let inner = &output[1..output.len() - 1];
        output = format!("vec![{}]", inner.trim());
    }

    let count_regex = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)\.count\(([^)]+)\)").unwrap();
    output = count_regex
        .replace_all(&output, |captures: &regex::Captures| {
            let receiver = captures.get(1).unwrap().as_str();
            let argument = captures.get(2).unwrap().as_str().trim();
            let arg_ref = if argument.starts_with('"') || argument.starts_with('&') {
                argument.to_string()
            } else {
                format!("&{}", argument)
            };
            if state.hashset_variables.contains(receiver) {
                format!("{}.contains({})", receiver, arg_ref)
            } else {
                format!("{}.contains_key({})", receiver, arg_ref)
            }
        })
        .to_string();

    if let Some(rest) = output.strip_prefix("new ") {
        output = format!("Box::new({})", rest.trim());
    }

    output = FLOAT_SUFFIX_REGEX
        .replace_all(&output, "${1}_f32")
        .to_string();

    output = transform_outside_string_literals(&output, |segment| {
        INDEX_ACCESS_REGEX
            .replace_all(segment, |captures: &regex::Captures| {
                let target = captures.get(1).unwrap().as_str();
                let index_expr = captures.get(2).unwrap().as_str().trim();
                if target == "vec" || index_expr.ends_with("usize") {
                    return format!("{}[{}]", target, index_expr);
                }
                if state.hashmap_variables.contains(target) {
                    if index_expr.starts_with('"') || index_expr.starts_with('&') {
                        return format!("{}[{}]", target, index_expr);
                    }
                    return format!("{}[&{}]", target, index_expr);
                }
                if index_expr.chars().all(|character| character.is_ascii_digit()) {
                    format!("{}[{}]", target, index_expr)
                } else {
                    format!("{}[({}) as usize]", target, index_expr)
                }
            })
            .to_string()
    });

    output = transform_outside_string_literals(&output, |segment| {
        adjust_reference_calls(segment, &state.function_ref_params)
    });

    output
}

pub fn translate_expression(expression: &str) -> String {
    let empty_state = TranslatorState::new();
    translate_expression_with_state(expression, &empty_state)
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
            let parameter_name = tokens
                .last()
                .unwrap()
                .trim_start_matches('&')
                .trim_start_matches('*');
            let raw_type = tokens[..tokens.len() - 1].join(" ");
            let rust_type = map_cpp_type_to_rust(&raw_type, state);
            state
                .method_parameter_names
                .insert(parameter_name.to_string());
            state.register_variable_type(parameter_name, &rust_type);

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

fn translate_cout_statement(trimmed: &str, state: &TranslatorState) -> Option<String> {
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
            format_arguments.push(translate_expression_with_state(part, state));
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

fn translate_printf_statement(
    format_literal: &str,
    raw_args: Option<&str>,
    state: &TranslatorState,
) -> String {
    let mut has_newline = false;
    let mut converted_format = format_literal.to_string();
    if let Some(stripped) = converted_format.strip_suffix("\\n") {
        has_newline = true;
        converted_format = stripped.to_string();
    }

    let specifier_regex = Regex::new(r"%[-+0-9.*]*[diuoxXfFeEgGcsp]").unwrap();
    converted_format = specifier_regex
        .replace_all(&converted_format, "{}")
        .to_string();
    let macro_name = if has_newline { "println!" } else { "print!" };

    match raw_args {
        Some(args) if !args.trim().is_empty() => {
            let translated_args = split_top_level_comma(args)
                .into_iter()
                .map(|arg| translate_expression_with_state(arg, state))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}(\"{}\", {});",
                macro_name, converted_format, translated_args
            )
        }
        _ => format!("{}(\"{}\");", macro_name, converted_format),
    }
}

fn qualify_struct_fields(statement: &str, state: &TranslatorState) -> String {
    transform_outside_string_literals(statement, |segment| {
        let mut output = segment.to_string();
        for field_name in &state.struct_fields {
            if state.method_parameter_names.contains(field_name) {
                continue;
            }
            let field_regex = Regex::new(&format!(r"\b{}\b", regex::escape(field_name))).unwrap();
            let snapshot = output.clone();
            output = field_regex
                .replace_all(&snapshot, |captures: &regex::Captures| {
                    let match_start = captures.get(0).unwrap().start();
                    let preceded_by_dot =
                        match_start > 0 && snapshot.as_bytes()[match_start - 1] == b'.';
                    let preceded_by_self =
                        match_start >= 5 && &snapshot[match_start - 5..match_start] == "self.";
                    if preceded_by_dot || preceded_by_self {
                        field_name.clone()
                    } else {
                        format!("self.{}", field_name)
                    }
                })
                .to_string();
        }
        output
    })
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
            "queue" | "deque" => {
                state
                    .required_imports
                    .insert("std::collections::VecDeque".to_string());
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
        return format!(
            "{}const {}: {} = {};",
            leading_whitespace, name, rust_type, value
        );
    }

    if trimmed.starts_with('#') {
        return format!("{}// [Preprocessor] {}", leading_whitespace, trimmed);
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
        state.struct_fields.clear();
        state.struct_brace_depth = state.current_brace_depth;
        state.inside_class_public = kind == "struct";
        state.current_brace_depth += 1;

        let generic_name = state.pending_generic.take();
        state.struct_generic = generic_name.clone();
        let generic_suffix = generic_name
            .map(|generic| format!("<{}>", generic))
            .unwrap_or_default();

        return format!(
            "{}#[derive(Debug, Clone, Default)]\n{}pub struct {}{} {{",
            leading_whitespace, leading_whitespace, name, generic_suffix
        );
    }

    if trimmed == "};" && state.inside_struct_name.is_some() {
        let struct_name = state.inside_struct_name.take().unwrap();
        let generic_type = state.struct_generic.take();
        let impl_header = match &generic_type {
            Some(generic) => format!("impl<{}: Clone> {}<{}>", generic, struct_name, generic),
            None => format!("impl {}", struct_name),
        };
        state.struct_fields.clear();
        state.current_brace_depth = state.current_brace_depth.saturating_sub(1);
        if state.collected_methods.is_empty() {
            return format!("{}}}", leading_whitespace);
        }
        let methods_block = state
            .collected_methods
            .drain(..)
            .collect::<Vec<_>>()
            .join("\n");
        return format!(
            "{}}}\n\n{}{} {{\n{}\n{}}}",
            leading_whitespace,
            leading_whitespace,
            impl_header,
            methods_block,
            leading_whitespace
        );
    }

    if state.inside_struct_name.is_some() && !state.inside_inline_method {
        if let Some(captures) = FUNCTION_DECL_REGEX.captures(trimmed) {
            let return_type_raw = captures.get(1).unwrap().as_str();
            let function_name = captures.get(2).unwrap().as_str();
            let params_raw = captures.get(3).unwrap().as_str();
            state.reset_function_local_tracking();
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
            state.collected_methods.push(format!(
                "    pub fn {}({}){} {{",
                function_name, self_param, return_clause
            ));
            return String::new();
        }

        if let Some(captures) = VAR_DECL_NO_INIT_REGEX.captures(trimmed) {
            let raw_type = captures.get(1).unwrap().as_str();
            let field_name = captures.get(2).unwrap().as_str();
            state.struct_fields.push(field_name.to_string());
            let rust_type = map_cpp_type_to_rust(raw_type, state);
            let visibility = if state.inside_class_public { "pub " } else { "" };
            return format!(
                "{}{}{}: {},",
                leading_whitespace, visibility, field_name, rust_type
            );
        }
    }

    if state.inside_inline_method {
        let open_count = trimmed.matches('{').count();
        let close_count = trimmed.matches('}').count();
        state.current_brace_depth =
            (state.current_brace_depth + open_count).saturating_sub(close_count);

        if trimmed == "}" && state.current_brace_depth <= state.inline_method_brace_depth {
            state.inside_inline_method = false;
            state.reset_function_local_tracking();
            state.collected_methods.push("    }".to_string());
            return String::new();
        }

        let translated_body = translate_statement_line(trimmed, state);
        let mut qualified_body = qualify_struct_fields(&translated_body, state);
        if state.struct_generic.is_some()
            && qualified_body.starts_with("return self.")
            && qualified_body.ends_with(';')
        {
            let expr = qualified_body
                .trim_start_matches("return ")
                .trim_end_matches(';');
            if !expr.contains(' ') {
                qualified_body = format!("return {}.clone();", expr);
            }
        }
        state
            .collected_methods
            .push(format!("        {}", qualified_body));
        return String::new();
    }

    let open_count = trimmed.matches('{').count();
    let close_count = trimmed.matches('}').count();
    let previous_depth = state.current_brace_depth;
    state.current_brace_depth =
        (state.current_brace_depth + open_count).saturating_sub(close_count);

    if state.inside_main_function
        && state.current_brace_depth <= state.main_brace_depth
        && trimmed == "}"
    {
        state.inside_main_function = false;
        state.reset_function_local_tracking();
        return format!("{}}}", leading_whitespace);
    }

    if state.current_function_returns_option
        && state.current_brace_depth <= state.function_brace_depth
        && trimmed == "}"
    {
        state.current_function_returns_option = false;
        state.reset_function_local_tracking();
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

        state.reset_function_local_tracking();

        if function_name == "main" {
            state.inside_main_function = true;
            state.main_brace_depth = previous_depth;
            return format!("{}fn main() {{", leading_whitespace);
        }

        let ref_modes: Vec<Option<bool>> =
            if params_raw.trim().is_empty() || params_raw.trim() == "void" {
                Vec::new()
            } else {
                split_top_level_comma(params_raw.trim())
                    .into_iter()
                    .map(|parameter| {
                        if parameter.contains('&') && !parameter.contains("const") {
                            Some(true)
                        } else if parameter.contains('&') {
                            Some(false)
                        } else {
                            None
                        }
                    })
                    .collect()
            };
        state
            .function_ref_params
            .insert(function_name.to_string(), ref_modes);
        let rust_params = translate_parameters(params_raw, state);
        let return_type = map_cpp_type_to_rust(return_type_raw, state);
        state.current_function_returns_option = return_type.starts_with("Option<");
        if state.current_function_returns_option {
            state.option_functions.insert(function_name.to_string());
        }
        state.function_brace_depth = previous_depth;
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

    if let Some(captures) = RETURN_BRACE_INIT_REGEX.captures(trimmed) {
        let inner = captures.get(1).unwrap().as_str().trim();
        if inner.is_empty() {
            return "return vec![];".to_string();
        }
        let translated_inner = split_top_level_comma(inner)
            .into_iter()
            .map(|item| translate_expression_with_state(item, state))
            .collect::<Vec<_>>()
            .join(", ");
        return format!("return vec![{}];", translated_inner);
    }

    if state.current_function_returns_option {
        if let Some(captures) = RETURN_STMT_REGEX.captures(trimmed) {
            let return_expr =
                translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
            let is_already_option = return_expr == "None"
                || return_expr.starts_with("Some(")
                || state.option_variables.contains(return_expr.as_str())
                || state.option_functions.iter().any(|func_name| {
                    return_expr.starts_with(&format!("{}(", func_name))
                });
            if is_already_option {
                return format!("return {};", return_expr);
            }
            return format!("return Some({});", return_expr);
        }
    }

    if let Some(cout_rust) = translate_cout_statement(trimmed, state) {
        return cout_rust;
    }

    if let Some(captures) = PRINTF_REGEX.captures(trimmed) {
        let format_literal = captures.get(1).unwrap().as_str();
        let raw_args = captures.get(2).map(|m| m.as_str());
        return translate_printf_statement(format_literal, raw_args, state);
    }

    if let Some(captures) = SCANF_REGEX.captures(trimmed) {
        let variable_name = captures.get(2).unwrap().as_str();
        if state.declared_variables.contains(variable_name) {
            return format!(
                "let mut _stdin_line = String::new(); std::io::stdin().read_line(&mut _stdin_line).unwrap(); {} = _stdin_line.trim().parse().unwrap_or(0);",
                variable_name
            );
        }
        state.declared_variables.insert(variable_name.to_string());
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

    if let Some(captures) = SORT_CALL_REGEX.captures(trimmed) {
        let vector_name = captures.get(1).unwrap().as_str();
        if state.float_vector_variables.contains(vector_name) {
            return format!(
                "{}.sort_by(|a, b| a.partial_cmp(b).unwrap());",
                vector_name
            );
        }
        return format!("{}.sort();", vector_name);
    }

    if let Some(captures) = REVERSE_CALL_REGEX.captures(trimmed) {
        let vector_name = captures.get(1).unwrap().as_str();
        return format!("{}.reverse();", vector_name);
    }

    if let Some(captures) = MAP_PAIR_INSERT_REGEX.captures(trimmed) {
        let map_name = captures.get(1).unwrap().as_str();
        let mut key_expr =
            translate_expression_with_state(captures.get(2).unwrap().as_str(), state);
        let value_expr = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        if key_expr.starts_with('"') {
            key_expr = format!("String::from({})", key_expr);
        }
        return format!("{}.entry({}).or_insert({});", map_name, key_expr, value_expr);
    }

    if let Some(captures) = MAP_BRACKET_ASSIGN_REGEX.captures(trimmed) {
        let target_name = captures.get(1).unwrap().as_str();
        if state.hashmap_variables.contains(target_name) {
            let mut key_expr =
                translate_expression_with_state(captures.get(2).unwrap().as_str(), state);
            let value_expr =
                translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
            if key_expr.starts_with('"') {
                key_expr = format!("String::from({})", key_expr);
            }
            return format!("{}.insert({}, {});", target_name, key_expr, value_expr);
        }
    }

    if let Some(captures) = VECTOR_CTOR_TWO_ARGS_REGEX.captures(trimmed) {
        let inner_type = map_cpp_type_to_rust(captures.get(1).unwrap().as_str(), state);
        let variable_name = captures.get(2).unwrap().as_str();
        let size_expr = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        let init_val = translate_expression_with_state(captures.get(4).unwrap().as_str(), state);
        state.register_variable_type(variable_name, &format!("Vec<{}>", inner_type));
        return format!(
            "let mut {}: Vec<{}> = vec![{}; ({}) as usize];",
            variable_name, inner_type, init_val, size_expr
        );
    }

    if let Some(captures) = VECTOR_CTOR_ONE_ARG_REGEX.captures(trimmed) {
        let inner_type = map_cpp_type_to_rust(captures.get(1).unwrap().as_str(), state);
        let variable_name = captures.get(2).unwrap().as_str();
        let size_expr = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        state.register_variable_type(variable_name, &format!("Vec<{}>", inner_type));
        return format!(
            "let mut {}: Vec<{}> = vec![Default::default(); ({}) as usize];",
            variable_name, inner_type, size_expr
        );
    }

    if let Some(captures) = FOR_RANGE_DOWN_REGEX.captures(trimmed) {
        let variable_name = captures.get(1).unwrap().as_str();
        let start_expr = translate_expression_with_state(captures.get(2).unwrap().as_str(), state);
        let operator = captures.get(3).unwrap().as_str();
        let end_expr = translate_expression_with_state(captures.get(4).unwrap().as_str(), state);
        state.declared_variables.insert(variable_name.to_string());
        if operator == ">=" {
            return format!(
                "for {} in ({}..={}).rev() {{",
                variable_name, end_expr, start_expr
            );
        }
        return format!(
            "for {} in (({} + 1)..={}).rev() {{",
            variable_name, end_expr, start_expr
        );
    }

    if let Some(captures) = FOR_RANGE_UP_REGEX.captures(trimmed) {
        let loop_type = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        let start_expr = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        let operator = captures.get(4).unwrap().as_str();
        let mut end_expr =
            translate_expression_with_state(captures.get(5).unwrap().as_str(), state);
        if end_expr.contains(".len()") {
            let target_cast = match loop_type {
                "size_t" => None,
                "long" => Some("i64"),
                _ => Some("i32"),
            };
            if let Some(cast_type) = target_cast {
                let len_regex = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*\.len\(\))").unwrap();
                end_expr = len_regex
                    .replace_all(&end_expr, format!("($1 as {})", cast_type))
                    .to_string();
            }
        }
        let range_operator = if operator == "<=" { "..=" } else { ".." };
        state.declared_variables.insert(variable_name.to_string());
        return format!(
            "for {} in {}{}{} {{",
            variable_name, start_expr, range_operator, end_expr
        );
    }

    if let Some(captures) = FOR_EACH_REGEX.captures(trimmed) {
        let binding_type = captures.get(1).unwrap().as_str().trim();
        let item_name = captures.get(2).unwrap().as_str();
        let collection_expr =
            translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        state.declared_variables.insert(item_name.to_string());
        if binding_type.contains("const") && binding_type.contains('&') {
            return format!("for {} in {}.iter().cloned() {{", item_name, collection_expr);
        } else if binding_type.contains('&') {
            return format!("for {} in &mut {} {{", item_name, collection_expr);
        } else {
            return format!("for {} in {}.iter().cloned() {{", item_name, collection_expr);
        }
    }

    if let Some(captures) = WHILE_REGEX.captures(trimmed) {
        let condition = translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
        return format!("while {} {{", condition);
    }

    if let Some(captures) = ELSE_IF_REGEX.captures(trimmed) {
        let condition = translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
        return format!("}} else if {} {{", condition);
    }

    if ELSE_REGEX.is_match(trimmed) {
        return "} else {".to_string();
    }

    if let Some(captures) = IF_REGEX.captures(trimmed) {
        let condition = translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
        return format!("if {} {{", condition);
    }

    if let Some(captures) = SWITCH_REGEX.captures(trimmed) {
        let switch_expr = translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
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
        let target = translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
        return format!("{} += 1;", target);
    }

    if let Some(captures) = DECREMENT_REGEX.captures(trimmed) {
        let target = translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
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

        state.register_variable_type(variable_name, &rust_type);

        let mut translated_value = translate_expression_with_state(raw_value, state);
        if rust_type == "String" && translated_value.starts_with('"') {
            translated_value = format!("String::from({})", translated_value);
        }
        if rust_type == "i32" && translated_value.contains(".len()") {
            let len_regex = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*\.len\(\))").unwrap();
            translated_value = len_regex
                .replace_all(&translated_value, "($1 as i32)")
                .to_string();
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
            state.register_variable_type(variable_name, &rust_type);
            if rust_type.starts_with("Vec<") {
                return format!("let mut {}: {} = Vec::new();", variable_name, rust_type);
            } else if rust_type.starts_with("VecDeque<") {
                return format!("let mut {}: {} = VecDeque::new();", variable_name, rust_type);
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

    translate_expression_with_state(trimmed, state)
}
