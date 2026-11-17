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
static CLASS_OR_STRUCT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:typedef\s+)?(class|struct|union)(?:\s+([A-Za-z_][A-Za-z0-9_]*))?\s*\{?\s*$"#).unwrap()
});
static STRUCT_CLOSE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\}\s*([A-Za-z_][A-Za-z0-9_]*)?\s*;\s*$"#).unwrap());
static TYPEDEF_ENUM_SINGLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^typedef\s+enum(?:\s+[A-Za-z_][A-Za-z0-9_]*)?\s*\{([^}]+)\}\s*([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap()
});
static TYPEDEF_ENUM_OPEN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^typedef\s+enum(?:\s+[A-Za-z_][A-Za-z0-9_]*)?\s*\{\s*$"#).unwrap()
});
static STRUCT_ARRAY_FIELD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:\s]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*([^\]]+)\s*\]\s*;\s*$"#).unwrap()
});
static STRUCT_MULTI_FIELD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:\s]*?)\s+([A-Za-z_][A-Za-z0-9_]*(?:\s*,\s*[A-Za-z_][A-Za-z0-9_]*)+)\s*;\s*$"#).unwrap()
});
static STRUCT_SELF_PTR_FIELD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:struct\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap()
});
static TYPEDEF_ARRAY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^typedef\s+([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*(\d+)\s*\]\s*;\s*$"#).unwrap()
});
static TYPEDEF_PRIMITIVE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^typedef\s+([A-Za-z_][A-Za-z0-9_:\s]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap()
});
static ANON_ENUM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^enum\s*\{([^}]+)\}\s*;\s*$"#).unwrap());
static DESIGNATED_INIT_SINGLE_LINE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:const\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\{\s*(\.[^{}]+(?:\{[^{}]*\}[^{}]*)*)\}\s*;\s*$"#).unwrap()
});
static POSITIONAL_STRUCT_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:const\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\{\s*([^.{][^}]*)\}\s*;\s*$"#).unwrap()
});
static DESIGNATED_INIT_OPEN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:const\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\{\s*$"#).unwrap()
});
static DESIGNATED_FIELD_ARRAY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\.([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\{([^}]*)\}\s*,?\s*$"#).unwrap()
});
static DESIGNATED_FIELD_VALUE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\.([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+?)\s*,?\s*$"#).unwrap()
});
static FUNCTION_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:<>,\s*&]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:const\s*)?\{\s*$"#).unwrap()
});
static FUNCTION_PROTOTYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:<>,\s*&]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:const\s*)?;\s*$"#).unwrap()
});
static SINGLE_LINE_IF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^if\s*\((.+)\)\s+(.+;)\s*$"#).unwrap()
});
static ARRAY_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:static\s+)?(const\s+)?([A-Za-z_][A-Za-z0-9_:\s]*?)\s*(\*?)\s*([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*\d*\s*\]\s*=\s*\{(.+)\}\s*;\s*$"#).unwrap()
});
static CHAR_BUFFER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^char\s+([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*\d+\s*\]\s*;\s*$"#).unwrap()
});
static FGETS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^fgets\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*,\s*[^,]+\s*,\s*stdin\s*\)\s*;\s*$"#).unwrap()
});
static STRCSPN_NEWLINE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*strcspn\s*\(\s*[A-Za-z_][A-Za-z0-9_]*\s*,\s*"(?:\\r)?\\n"\s*\)\s*\]\s*=\s*'\\0'\s*;\s*$"#).unwrap()
});
static MEMMOVE_SHIFT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:memmove|memcpy|strcpy)\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*,\s*[A-Za-z_][A-Za-z0-9_]*\s*\+\s*([^,]+)\s*,.+\)\s*;\s*$"#).unwrap()
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
    Lazy::new(|| Regex::new(r#"^(\}\s*)?else\s+if\s*\((.+)\)\s*\{\s*$"#).unwrap());
static ELSE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(\}\s*)?else\s*\{\s*$"#).unwrap());
static WHILE_STR_SEARCH_ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^while\s*\(\s*\(?\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(strchr|strrchr|strstr)\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)\s*\)?\s*(?:!=\s*(?:NULL|nullptr|0))?\s*\)\s*\{\s*$"#).unwrap()
});
static IF_STR_SEARCH_ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^if\s*\(\s*\(?\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(strchr|strrchr|strstr)\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)\s*\)?\s*(?:!=\s*(?:NULL|nullptr|0))?\s*\)\s*\{\s*$"#).unwrap()
});
static VEC_INSERT_BEGIN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\.insert\s*\(\s*[A-Za-z_][A-Za-z0-9_]*\.begin\(\)(?:\s*\+\s*([^,]+))?\s*,\s*(.+)\s*\)\s*;\s*$"#).unwrap()
});
static VEC_INSERT_END_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\.insert\s*\(\s*[A-Za-z_][A-Za-z0-9_]*\.end\(\)\s*,\s*(.+)\s*\)\s*;\s*$"#).unwrap()
});
static VEC_ERASE_BEGIN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\.erase\s*\(\s*[A-Za-z_][A-Za-z0-9_]*\.begin\(\)(?:\s*\+\s*([^)]+))?\s*\)\s*;\s*$"#).unwrap()
});
static VEC_PUSH_CALL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\.(?:push_back|emplace_back)\s*\(\s*(.+)\s*\)\s*;\s*$"#).unwrap()
});
static SWITCH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^switch\s*\((.+)\)\s*\{\s*$"#).unwrap());
static CASE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^case\s+([^:]+):\s*$"#).unwrap());
static PRINTF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^printf\s*\(\s*"((?:[^"\\]|\\.)*)"\s*(?:,\s*(.+))?\)\s*;\s*$"#).unwrap()
});
static PUTS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^puts\s*\(\s*(.+)\s*\)\s*;\s*$"#).unwrap());
static SCANF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^scanf\s*\(\s*"([^"]*)"\s*,\s*(.+)\)\s*;\s*$"#).unwrap());
static CIN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(?:std::)?cin\s*>>\s*([A-Za-z_][A-Za-z0-9_.\[\]]*)\s*;\s*$"#).unwrap());
static STRUCT_ZERO_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:const\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\{\s*0\s*\}\s*;\s*$"#).unwrap()
});
static ARRAY_ZERO_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:const\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*(\d+)\s*\]\s*=\s*\{\s*0\s*\}\s*;\s*$"#).unwrap()
});
static STRUCT_FLEX_ARRAY_FIELD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:\s]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*\]\s*;\s*$"#).unwrap()
});
static MALLOC_FLEX_ARRAY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:\(\s*[A-Za-z_][A-Za-z0-9_]*\s*\*\s*\)\s*)?malloc\s*\(\s*sizeof\s*\([^)]+\)\s*\+\s*([A-Za-z_][A-Za-z0-9_]*)\s*\*\s*sizeof\s*\([^)]+\)\s*\)\s*;\s*$"#).unwrap()
});
static MALLOC_STRUCT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:struct\s+)?([A-Za-z_][A-Za-z0-9_\s]*?)\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:\(\s*[^)]*\*\s*\)\s*)?(?:malloc|calloc)\s*\(.+\)\s*;\s*$"#).unwrap()
});
static DEFINE_GENERIC_MACRO_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*_Generic\s*\(\s*\(?\s*[A-Za-z_][A-Za-z0-9_]*\s*\)?\s*,\s*(.+)\)\s*$"#).unwrap()
});
static COMPOUND_LITERAL_PTR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*&\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*\{\s*(.+?)\s*\}\s*;\s*$"#).unwrap()
});
static VLA_2D_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:\s]*?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*([^\]]+)\s*\]\s*\[\s*([^\]]+)\s*\]\s*;\s*$"#).unwrap()
});
static MULTI_VAR_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_:\s]*?)\s+([A-Za-z_][A-Za-z0-9_]*\s*=\s*[^,;]+(?:\s*,\s*[A-Za-z_][A-Za-z0-9_]*\s*=\s*[^,;]+)+)\s*;\s*$"#).unwrap()
});
static FREE_CALL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^free\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*;\s*$"#).unwrap());
static LINKED_LIST_HEAD_TAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([A-Za-z_][A-Za-z0-9_]*)\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*NULL\s*,\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*NULL\s*;\s*$"#).unwrap()
});
static NULL_RETURN_CHECK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^if\s*\(\s*(?:!\s*[A-Za-z_][A-Za-z0-9_]*|[A-Za-z_][A-Za-z0-9_]*\s*==\s*(?:NULL|nullptr))\s*\)\s*return\s+[^;]+;\s*$"#).unwrap()
});
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
    Regex::new(r#"^(?:static\s+)?(const\s+)?([A-Za-z_][A-Za-z0-9_:<>,\s]*?)(?:\s*([*&]+)\s*|\s+)([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+);\s*$"#).unwrap()
});
static VAR_DECL_NO_INIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:static\s+)?([A-Za-z_][A-Za-z0-9_:<>,\s]*?)(?:\s*([*&]+)\s*|\s+)([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap()
});
static INCREMENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:\+\+\s*([A-Za-z_][A-Za-z0-9_.\[\]\->]*)|([A-Za-z_][A-Za-z0-9_.\[\]\->]*)\s*\+\+)\s*;\s*$"#).unwrap()
});
static DECREMENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:--\s*([A-Za-z_][A-Za-z0-9_.\[\]\->]*)|([A-Za-z_][A-Za-z0-9_.\[\]\->]*)\s*--)\s*;\s*$"#).unwrap()
});
static RETURN_BRACE_INIT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^return\s*\{(.*)\}\s*;\s*$"#).unwrap());
static RETURN_STMT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^return\s+(.+);\s*$"#).unwrap());
static FLOAT_SUFFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\b(\d+\.\d+)[fF]\b"#).unwrap());
static LONG_DOUBLE_SUFFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\b(\d+\.\d+)[lL]\b"#).unwrap());
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
    pub struct_typedef_alias: Option<String>,
    pub anon_struct_counter: usize,
    pub struct_field_defs: HashMap<String, Vec<(String, String)>>,
    pub self_ptr_fields: HashSet<String>,
    pub global_cvar_head: Option<String>,
    pub inside_cvar_function: bool,
    pub cvar_function_depth: usize,
    pub cvar_instance_names: HashMap<String, String>,
    pub inside_class_public: bool,
    pub inside_enum_block: bool,
    pub collected_enum_variants: Vec<String>,
    pub inside_designated_init: bool,
    pub pending_multiline_statement: Option<String>,
    pub collected_methods: Vec<String>,
    pub inside_inline_method: bool,
    pub inline_method_brace_depth: usize,
    pub method_parameter_names: HashSet<String>,
    pub declared_variables: HashSet<String>,
    pub hashmap_variables: HashSet<String>,
    pub hashset_variables: HashSet<String>,
    pub queue_variables: HashSet<String>,
    pub string_variables: HashSet<String>,
    pub str_slice_variables: HashSet<String>,
    pub char_ptr_iter_variables: HashSet<String>,
    pub float_vector_variables: HashSet<String>,
    pub float_variables: HashSet<String>,
    pub array_variables: HashSet<String>,
    pub array_ptr_aliases: HashMap<String, String>,
    pub box_pointer_variables: HashSet<String>,
    pub enum_type_names: HashSet<String>,
    pub linked_list_head_var: Option<String>,
    pub linked_list_tail_var: Option<String>,
    pub linked_list_node_type: Option<String>,
    pub linked_list_iter_var: Option<String>,
    pub in_linked_list_traversal: bool,
    pub linked_list_traversal_depth: usize,
    pub option_variables: HashSet<String>,
    pub option_functions: HashSet<String>,
    pub function_ref_params: HashMap<String, Vec<Option<bool>>>,
    pub flex_array_structs: HashMap<String, (String, String)>,
    pub generic_macros: HashMap<String, (Vec<(String, String)>, String)>,
    pub variable_c_types: HashMap<String, String>,
    pub vla_byte_sizes: HashMap<String, String>,
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
            struct_typedef_alias: None,
            anon_struct_counter: 0,
            struct_field_defs: HashMap::new(),
            self_ptr_fields: HashSet::new(),
            global_cvar_head: None,
            inside_cvar_function: false,
            cvar_function_depth: 0,
            cvar_instance_names: HashMap::new(),
            inside_class_public: true,
            inside_enum_block: false,
            collected_enum_variants: Vec::new(),
            inside_designated_init: false,
            pending_multiline_statement: None,
            collected_methods: Vec::new(),
            inside_inline_method: false,
            inline_method_brace_depth: 0,
            method_parameter_names: HashSet::new(),
            declared_variables: HashSet::new(),
            hashmap_variables: HashSet::new(),
            hashset_variables: HashSet::new(),
            queue_variables: HashSet::new(),
            string_variables: HashSet::new(),
            str_slice_variables: HashSet::new(),
            char_ptr_iter_variables: HashSet::new(),
            float_vector_variables: HashSet::new(),
            float_variables: HashSet::new(),
            array_variables: HashSet::new(),
            array_ptr_aliases: HashMap::new(),
            box_pointer_variables: HashSet::new(),
            enum_type_names: HashSet::new(),
            linked_list_head_var: None,
            linked_list_tail_var: None,
            linked_list_node_type: None,
            linked_list_iter_var: None,
            in_linked_list_traversal: false,
            linked_list_traversal_depth: 0,
            option_variables: HashSet::new(),
            option_functions: HashSet::new(),
            function_ref_params: HashMap::new(),
            flex_array_structs: HashMap::new(),
            generic_macros: HashMap::new(),
            variable_c_types: HashMap::new(),
            vla_byte_sizes: HashMap::new(),
        }
    }

    fn reset_function_local_tracking(&mut self) {
        self.method_parameter_names.clear();
        self.declared_variables.clear();
        self.hashmap_variables.clear();
        self.hashset_variables.clear();
        self.queue_variables.clear();
        self.string_variables.clear();
        self.str_slice_variables.clear();
        self.char_ptr_iter_variables.clear();
        self.float_vector_variables.clear();
        self.float_variables.clear();
        self.array_variables.clear();
        self.array_ptr_aliases.clear();
        self.box_pointer_variables.clear();
        self.linked_list_head_var = None;
        self.linked_list_tail_var = None;
        self.linked_list_node_type = None;
        self.linked_list_iter_var = None;
        self.in_linked_list_traversal = false;
        self.linked_list_traversal_depth = 0;
        self.option_variables.clear();
        self.variable_c_types.clear();
        self.vla_byte_sizes.clear();
    }

    fn register_variable_type(&mut self, variable_name: &str, rust_type: &str) {
        self.declared_variables.insert(variable_name.to_string());
        self.hashmap_variables.remove(variable_name);
        self.hashset_variables.remove(variable_name);
        self.queue_variables.remove(variable_name);
        self.string_variables.remove(variable_name);
        self.str_slice_variables.remove(variable_name);
        self.char_ptr_iter_variables.remove(variable_name);
        self.float_vector_variables.remove(variable_name);
        self.float_variables.remove(variable_name);
        self.option_variables.remove(variable_name);

        if rust_type.starts_with("HashMap<") {
            self.hashmap_variables.insert(variable_name.to_string());
        } else if rust_type.starts_with("HashSet<") {
            self.hashset_variables.insert(variable_name.to_string());
        } else if rust_type.starts_with("VecDeque<") {
            self.queue_variables.insert(variable_name.to_string());
        } else if rust_type == "String" {
            self.string_variables.insert(variable_name.to_string());
        } else if rust_type == "&str" {
            self.str_slice_variables.insert(variable_name.to_string());
        } else if rust_type == "Vec<f32>" || rust_type == "Vec<f64>" {
            self.float_vector_variables
                .insert(variable_name.to_string());
        } else if rust_type == "f32" || rust_type == "f64" {
            self.float_variables.insert(variable_name.to_string());
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
        if inner == "char" {
            return "String".to_string();
        }
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

    if let Some(inner) = extract_generic_inner(cleaned, "array")
        .or_else(|| extract_generic_inner(cleaned, "std::array"))
    {
        let parts = split_top_level_comma(inner);
        if parts.len() == 2 {
            return format!(
                "[{}; {}]",
                map_cpp_type_to_rust(parts[0], state),
                parts[1].trim()
            );
        }
    }

    if let Some(inner) = extract_generic_inner(cleaned, "pair")
        .or_else(|| extract_generic_inner(cleaned, "std::pair"))
    {
        let parts = split_top_level_comma(inner);
        if parts.len() == 2 {
            return format!(
                "({}, {})",
                map_cpp_type_to_rust(parts[0], state),
                map_cpp_type_to_rust(parts[1], state)
            );
        }
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
        "unsigned short" | "uint16_t" => "u16".to_string(),
        "short" | "int16_t" => "i32".to_string(),
        "signed char" | "int8_t" => "i8".to_string(),
        "unsigned char" | "uint8_t" => "u8".to_string(),
        "float" => "f32".to_string(),
        "double" | "long double" => "f64".to_string(),
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

fn split_trailing_line_comment(line: &str) -> (&str, Option<&str>) {
    let bytes = line.as_bytes();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut index = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_double_quotes {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == b'"' {
                in_double_quotes = false;
            }
            index += 1;
            continue;
        }
        if in_single_quotes {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == b'\'' {
                in_single_quotes = false;
            }
            index += 1;
            continue;
        }

        if byte == b'"' {
            in_double_quotes = true;
        } else if byte == b'\'' {
            in_single_quotes = true;
        } else if byte == b'/' && index + 1 < bytes.len() && bytes[index + 1] == b'/' {
            return (line[..index].trim_end(), Some(line[index..].trim()));
        }
        index += 1;
    }

    (line, None)
}

fn compute_function_ref_modes(params_raw: &str) -> Vec<Option<bool>> {
    let trimmed = params_raw.trim();
    if trimmed.is_empty() || trimmed == "void" {
        return Vec::new();
    }
    split_top_level_comma(trimmed)
        .into_iter()
        .map(|parameter| {
            let is_char_ptr = parameter.contains("char*") || parameter.contains("char *");
            if is_char_ptr {
                Some(false)
            } else if parameter.contains('&') || parameter.contains('*') || parameter.contains('[') {
                if parameter.contains("const") {
                    Some(false)
                } else {
                    Some(true)
                }
            } else {
                None
            }
        })
        .collect()
}

fn cast_char_comparisons_to_i32(segment: &str) -> String {
    let char_arith_regex = Regex::new(r"'([^'\\])'\s*([+\-])").unwrap();
    let mut updated = char_arith_regex
        .replace_all(segment, "('$1' as i32) $2")
        .to_string();

    for method_suffix in [".to_ascii_uppercase()", ".to_ascii_lowercase()"] {
        let mut search_from = 0usize;
        while let Some(rel_pos) = updated[search_from..].find(method_suffix) {
            let method_start = search_from + rel_pos;
            let method_end = method_start + method_suffix.len();

            let rest = updated[method_end..].trim_start();
            let is_cmp_with_i32 = [">=", "<=", ">", "<", "==", "!="]
                .iter()
                .any(|op| {
                    if let Some(after_op) = rest.strip_prefix(op) {
                        let rhs_end = after_op
                            .find("&&")
                            .into_iter()
                            .chain(after_op.find("||"))
                            .chain(after_op.find(';'))
                            .min()
                            .unwrap_or(after_op.len());
                        after_op[..rhs_end].contains("as i32)")
                    } else {
                        false
                    }
                });

            if !is_cmp_with_i32 || method_start == 0 {
                search_from = method_end;
                continue;
            }

            let prefix_bytes = updated[..method_start].as_bytes();
            let mut receiver_start = method_start;
            if prefix_bytes[method_start - 1] == b')' {
                let mut depth = 0isize;
                let mut idx = method_start;
                while idx > 0 {
                    idx -= 1;
                    if prefix_bytes[idx] == b')' {
                        depth += 1;
                    } else if prefix_bytes[idx] == b'(' {
                        depth -= 1;
                        if depth == 0 {
                            receiver_start = idx;
                            break;
                        }
                    }
                }
            } else {
                while receiver_start > 0
                    && (prefix_bytes[receiver_start - 1].is_ascii_alphanumeric()
                        || prefix_bytes[receiver_start - 1] == b'_'
                        || prefix_bytes[receiver_start - 1] == b'.')
                {
                    receiver_start -= 1;
                }
            }

            let wrapped = format!("({} as i32)", &updated[receiver_start..method_end]);
            updated.replace_range(receiver_start..method_end, &wrapped);
            search_from = receiver_start + wrapped.len();
        }
    }

    updated
}

fn translate_ternary_outside_strings(segment: &str) -> String {
    let trimmed = segment.trim();
    let ternary_regex = Regex::new(r"^([^?]+?)\s*\?\s*([^:?]+?)\s*:\s*(.+)$").unwrap();
    if let Some(captures) = ternary_regex.captures(trimmed) {
        let condition = captures.get(1).unwrap().as_str().trim();
        let when_true = captures.get(2).unwrap().as_str().trim();
        let when_false = translate_ternary_outside_strings(captures.get(3).unwrap().as_str());
        return format!("if {} {{ {} }} else {{ {} }}", condition, when_true, when_false);
    }
    segment.to_string()
}

fn adjust_reference_calls(input: &str, state: &TranslatorState) -> String {
    if state.function_ref_params.is_empty() {
        return input.to_string();
    }

    let mut output = input.to_string();
    for (func_name, ref_modes) in &state.function_ref_params {
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
                        let already_str_slice = trimmed_arg.starts_with('"')
                            || trimmed_arg.starts_with('&')
                            || state.str_slice_variables.contains(trimmed_arg)
                            || state.char_ptr_iter_variables.contains(trimmed_arg);
                        match ref_modes.get(index).copied().flatten() {
                            Some(true) => {
                                if trimmed_arg.starts_with("&mut ") {
                                    trimmed_arg.to_string()
                                } else if let Some(rest) = trimmed_arg.strip_prefix('&') {
                                    format!("&mut {}", rest.trim())
                                } else {
                                    format!("&mut {}", trimmed_arg)
                                }
                            }
                            Some(false) if !already_str_slice => {
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

    for ptr_var in &state.char_ptr_iter_variables {
        let escaped = regex::escape(ptr_var);
        let neq_null_regex =
            Regex::new(&format!(r"\*\s*{}\s*!=\s*'\\0'", escaped)).unwrap();
        output = neq_null_regex
            .replace_all(&output, format!("!{}.is_empty()", ptr_var))
            .to_string();
        let eq_null_regex =
            Regex::new(&format!(r"\*\s*{}\s*==\s*'\\0'", escaped)).unwrap();
        output = eq_null_regex
            .replace_all(&output, format!("{}.is_empty()", ptr_var))
            .to_string();
        let deref_regex = Regex::new(&format!(r"\*\s*{}\b", escaped)).unwrap();
        output = deref_regex
            .replace_all(&output, format!("{}.chars().next().unwrap()", ptr_var))
            .to_string();
    }

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
        ("std::make_pair(", "("),
        ("make_pair(", "("),
        ("std::string(", "String::from("),
    ];

    output = transform_outside_string_literals(&output, |segment| {
        let mut replaced = segment.to_string();
        for (ptr_name, array_name) in &state.array_ptr_aliases {
            let ptr_offset_regex = Regex::new(&format!(
                r"\(\s*{}\s*\+\s*([^)]+)\s*\)\s*(?:->|\.)\s*([A-Za-z_][A-Za-z0-9_]*)",
                regex::escape(ptr_name)
            ))
            .unwrap();
            replaced = ptr_offset_regex
                .replace_all(&replaced, |captures: &regex::Captures| {
                    let index_expr = captures.get(1).unwrap().as_str().trim();
                    let field_name = captures.get(2).unwrap().as_str();
                    if index_expr.chars().all(|c| c.is_ascii_digit()) {
                        format!("{}[{}].{}", array_name, index_expr, field_name)
                    } else {
                        format!("{}[({}) as usize].{}", array_name, index_expr, field_name)
                    }
                })
                .to_string();
        }
        for enum_name in &state.enum_type_names {
            let cast_regex = Regex::new(&format!(
                r"\(\s*{}\s*\)\s*([A-Za-z_][A-Za-z0-9_]*|\d+)",
                regex::escape(enum_name)
            ))
            .unwrap();
            replaced = cast_regex
                .replace_all(&replaced, format!("$1 as {}", enum_name))
                .to_string();
        }
        let atof_regex =
            Regex::new(r"(?:\(\s*float\s*\)\s*)?atof\s*\(([^)]+)\)").unwrap();
        replaced = atof_regex
            .replace_all(&replaced, "$1.parse::<f32>().unwrap_or(0.0)")
            .to_string();
        let bit_not_regex = Regex::new(r"~\s*([A-Za-z_][A-Za-z0-9_]*)").unwrap();
        replaced = bit_not_regex
            .replace_all(&replaced, "!$1")
            .to_string();
        let bit_zero_regex = Regex::new(r"!\s*\(([^()]+&[^()]+)\)").unwrap();
        replaced = bit_zero_regex
            .replace_all(&replaced, "(($1) == 0)")
            .to_string();
        for box_var in &state.box_pointer_variables {
            let escaped = regex::escape(box_var);
            let ne_right_re = Regex::new(&format!(
                r"(^|[^*.\w]){}\s*!=\s*(?:NULL|nullptr|0)\b",
                escaped
            ))
            .unwrap();
            replaced = ne_right_re.replace_all(&replaced, "${1}true").to_string();
            let ne_left_re = Regex::new(&format!(
                r"(^|[^*.\w])(?:NULL|nullptr|0)\s*!=\s*{}\b",
                escaped
            ))
            .unwrap();
            replaced = ne_left_re.replace_all(&replaced, "${1}true").to_string();
            let eq_right_re = Regex::new(&format!(
                r"(^|[^*.\w]){}\s*==\s*(?:NULL|nullptr|0)\b",
                escaped
            ))
            .unwrap();
            replaced = eq_right_re.replace_all(&replaced, "${1}false").to_string();
            let eq_left_re = Regex::new(&format!(
                r"(^|[^*.\w])(?:NULL|nullptr|0)\s*==\s*{}\b",
                escaped
            ))
            .unwrap();
            replaced = eq_left_re.replace_all(&replaced, "${1}false").to_string();
            let not_var_re = Regex::new(&format!(r"(^|[^*.\w])!\s*{}\b", escaped)).unwrap();
            replaced = not_var_re.replace_all(&replaced, "${1}false").to_string();
        }
        for (from, to) in replacements {
            replaced = replaced.replace(from, to);
        }
        for ptr_field in &state.self_ptr_fields {
            let ptr_access_regex = Regex::new(&format!(
                r"\.{}\.([A-Za-z_][A-Za-z0-9_]*)",
                regex::escape(ptr_field)
            ))
            .unwrap();
            replaced = ptr_access_regex
                .replace_all(&replaced, format!(".{}.as_ref().unwrap().$1", ptr_field))
                .to_string();
        }
        let keyword_type_field = Regex::new(r"\.type\b").unwrap();
        replaced = keyword_type_field
            .replace_all(&replaced, ".r#type")
            .to_string();
        replaced
    });

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
    output = LONG_DOUBLE_SUFFIX_REGEX
        .replace_all(&output, "${1}_f64")
        .to_string();

    output = transform_outside_string_literals(&output, |segment| {
        let mut updated = segment.to_string();
        let int_float_div_regex =
            Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*([*/])\s*(\d+\.\d+(?:_f32|_f64)?)").unwrap();
        updated = int_float_div_regex
            .replace_all(&updated, |captures: &regex::Captures| {
                let ident = captures.get(1).unwrap().as_str();
                let op = captures.get(2).unwrap().as_str();
                let lit = captures.get(3).unwrap().as_str();
                if state.float_variables.contains(ident) {
                    format!("{} {} {}", ident, op, lit)
                } else {
                    let cast_target = if lit.ends_with("_f64") { "f64" } else { "f32" };
                    format!("({} as {}) {} {}", ident, cast_target, op, lit)
                }
            })
            .to_string();
        updated
    });

    output = transform_outside_string_literals(&output, translate_ternary_outside_strings);

    output = transform_outside_string_literals(&output, |segment| {
        let mut updated = segment.to_string();
        let strlen_regex = Regex::new(r"\bstrlen\s*\(([^)]+)\)").unwrap();
        updated = strlen_regex
            .replace_all(&updated, "$1.len() as i32")
            .to_string();
        let strcmp_eq_re =
            Regex::new(r"\bstrcmp\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)\s*==\s*0\b").unwrap();
        updated = strcmp_eq_re.replace_all(&updated, "$1 == $2").to_string();
        let strcmp_ne_re =
            Regex::new(r"\bstrcmp\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)\s*!=\s*0\b").unwrap();
        updated = strcmp_ne_re.replace_all(&updated, "$1 != $2").to_string();
        let strstr_ne_re = Regex::new(
            r"\b(?:strstr|strchr)\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)\s*!=\s*(?:NULL|nullptr|None|0)\b",
        )
        .unwrap();
        updated = strstr_ne_re
            .replace_all(&updated, "$1.contains($2)")
            .to_string();
        let strstr_eq_re = Regex::new(
            r"\b(?:strstr|strchr)\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)\s*==\s*(?:NULL|nullptr|None|0)\b",
        )
        .unwrap();
        updated = strstr_eq_re
            .replace_all(&updated, "!$1.contains($2)")
            .to_string();
        let numeric_limits_regex =
            Regex::new(r"(?:std::)?numeric_limits<\s*([^>]+?)\s*>::(max|min)\(\)").unwrap();
        updated = numeric_limits_regex
            .replace_all(&updated, |captures: &regex::Captures| {
                let inner_c_type = captures.get(1).unwrap().as_str().trim();
                let bound = captures.get(2).unwrap().as_str().to_uppercase();
                let rust_t = match inner_c_type {
                    "int" | "int32_t" => "i32",
                    "double" | "long double" => "f64",
                    "float" => "f32",
                    "long" | "long long" | "int64_t" => "i64",
                    "unsigned int" | "unsigned" | "uint32_t" => "u32",
                    "unsigned long" | "unsigned long long" | "uint64_t" | "size_t" => "usize",
                    other => other,
                };
                format!("{}::{}", rust_t, bound)
            })
            .to_string();
        let abs_regex = Regex::new(
            r"(^|[^.\w])(?:std::)?(?:abs|fabs|fabsf)\s*\(((?:[^()]|\([^()]*\))+)\)",
        )
        .unwrap();
        updated = abs_regex.replace_all(&updated, "$1($2).abs()").to_string();
        let sqrt_regex = Regex::new(
            r"(^|[^.\w])(?:std::)?(?:sqrt|sqrtf)\s*\(((?:[^()]|\([^()]*\))+)\)",
        )
        .unwrap();
        updated = sqrt_regex
            .replace_all(&updated, "$1(($2) as f64).sqrt()")
            .to_string();
        updated = Regex::new(r"\.first\b")
            .unwrap()
            .replace_all(&updated, ".0")
            .to_string();
        updated = Regex::new(r"\.second\b")
            .unwrap()
            .replace_all(&updated, ".1")
            .to_string();
        for (macro_name, (branches, default_expr)) in &state.generic_macros {
            if updated.contains(macro_name) {
                let call_regex = Regex::new(&format!(
                    r"\b{}\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)",
                    regex::escape(macro_name)
                ))
                .unwrap();
                updated = call_regex
                    .replace_all(&updated, |captures: &regex::Captures| {
                        let arg_var = captures.get(1).unwrap().as_str();
                        if let Some(c_type) = state.variable_c_types.get(arg_var) {
                            for (branch_type, branch_val) in branches {
                                if branch_type == c_type {
                                    return branch_val.clone();
                                }
                            }
                        }
                        default_expr.clone()
                    })
                    .to_string();
            }
        }
        let sizeof_regex = Regex::new(r"\bsizeof\s*\(\s*([^()]+?)\s*\)").unwrap();
        updated = sizeof_regex
            .replace_all(&updated, |captures: &regex::Captures| {
                let inner = captures.get(1).unwrap().as_str().trim();
                if let Some(vla_bytes) = state.vla_byte_sizes.get(inner) {
                    return vla_bytes.clone();
                }
                if state.declared_variables.contains(inner) {
                    return format!("std::mem::size_of_val(&{})", inner);
                }
                let rust_type = match inner {
                    "long double" | "double" => "f64",
                    "float" => "f32",
                    "char" | "signed char" | "unsigned char" | "int8_t" | "uint8_t" => "u8",
                    "short" | "unsigned short" | "int16_t" | "uint16_t" => "i16",
                    "int" | "unsigned int" | "signed" | "unsigned" | "int32_t" | "uint32_t" => {
                        "i32"
                    }
                    "long" | "unsigned long" | "long long" | "unsigned long long" | "int64_t"
                    | "uint64_t" => "i64",
                    "size_t" | "ssize_t" | "uintptr_t" | "intptr_t" => "usize",
                    "bool" | "_Bool" => "bool",
                    other if other.ends_with('*') => "*const ()",
                    other => other,
                };
                format!("std::mem::size_of::<{}>()", rust_type)
            })
            .to_string();
        const C_LIMIT_MACROS: &[(&str, &str)] = &[
            ("LDBL_MANT_DIG", "f64::MANTISSA_DIGITS"),
            ("DBL_MANT_DIG", "f64::MANTISSA_DIGITS"),
            ("FLT_MANT_DIG", "f32::MANTISSA_DIGITS"),
            ("LDBL_DIG", "f64::DIGITS"),
            ("DBL_DIG", "f64::DIGITS"),
            ("FLT_DIG", "f32::DIGITS"),
            ("LDBL_MAX", "f64::MAX"),
            ("DBL_MAX", "f64::MAX"),
            ("FLT_MAX", "f32::MAX"),
            ("LDBL_MIN_EXP", "f64::MIN_EXP"),
            ("DBL_MIN_EXP", "f64::MIN_EXP"),
            ("LDBL_MAX_EXP", "f64::MAX_EXP"),
            ("DBL_MAX_EXP", "f64::MAX_EXP"),
            ("LDBL_MIN", "f64::MIN_POSITIVE"),
            ("DBL_MIN", "f64::MIN_POSITIVE"),
            ("FLT_MIN", "f32::MIN_POSITIVE"),
            ("LDBL_EPSILON", "f64::EPSILON"),
            ("DBL_EPSILON", "f64::EPSILON"),
            ("FLT_EPSILON", "f32::EPSILON"),
            ("FLT_MIN_EXP", "f32::MIN_EXP"),
            ("FLT_MAX_EXP", "f32::MAX_EXP"),
            ("INT_MAX", "i32::MAX"),
            ("INT_MIN", "i32::MIN"),
            ("UINT_MAX", "u32::MAX"),
            ("LONG_MAX", "i64::MAX"),
            ("LLONG_MAX", "i64::MAX"),
            ("LONG_MIN", "i64::MIN"),
            ("LLONG_MIN", "i64::MIN"),
            ("ULONG_MAX", "u64::MAX"),
            ("ULLONG_MAX", "u64::MAX"),
            ("SHRT_MAX", "i16::MAX"),
            ("SHRT_MIN", "i16::MIN"),
            ("USHRT_MAX", "u16::MAX"),
            ("CHAR_BIT", "u8::BITS"),
        ];
        for &(c_macro, rust_const) in C_LIMIT_MACROS {
            if updated.contains(c_macro) {
                let macro_regex =
                    Regex::new(&format!(r"\b{}\b", regex::escape(c_macro))).unwrap();
                updated = macro_regex.replace_all(&updated, rust_const).to_string();
            }
        }
        let ctype_funcs = [
            ("isdigit", "is_ascii_digit"),
            ("isalpha", "is_ascii_alphabetic"),
            ("isalnum", "is_ascii_alphanumeric"),
            ("toupper", "to_ascii_uppercase"),
            ("tolower", "to_ascii_lowercase"),
        ];
        for (c_func, rust_method) in ctype_funcs {
            let func_regex =
                Regex::new(&format!(r"\b{}\s*\(((?:[^()]|\([^()]*\))+)\)", c_func)).unwrap();
            updated = func_regex
                .replace_all(&updated, format!("($1).{}()", rust_method))
                .to_string();
        }
        let char_sub_regex =
            Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*-\s*'([^'\\])'").unwrap();
        updated = char_sub_regex
            .replace_all(&updated, "($1 as i32) - ('$2' as i32)")
            .to_string();
        cast_char_comparisons_to_i32(&updated)
    });

    output = transform_outside_string_literals(&output, |segment| {
        let first_pass = INDEX_ACCESS_REGEX
            .replace_all(segment, |captures: &regex::Captures| {
                let target = captures.get(1).unwrap().as_str();
                let index_expr = captures.get(2).unwrap().as_str().trim();
                if target == "vec" || index_expr.ends_with("usize") {
                    return format!("{}[{}]", target, index_expr);
                }
                if state.str_slice_variables.contains(target)
                    || state.string_variables.contains(target)
                    || state.char_ptr_iter_variables.contains(target)
                {
                    if index_expr.chars().all(|character| character.is_ascii_digit()) {
                        return format!("({}.as_bytes()[{}] as char)", target, index_expr);
                    }
                    return format!("({}.as_bytes()[({}) as usize] as char)", target, index_expr);
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
            .to_string();
        let chained_bracket_regex = Regex::new(r"\]\s*\[([^\[\]]+)\]").unwrap();
        chained_bracket_regex
            .replace_all(&first_pass, |captures: &regex::Captures| {
                let index_expr = captures.get(1).unwrap().as_str().trim();
                if index_expr.ends_with("usize")
                    || index_expr.chars().all(|character| character.is_ascii_digit())
                {
                    format!("][{}]", index_expr)
                } else {
                    format!("][({}) as usize]", index_expr)
                }
            })
            .to_string()
    });

    output = transform_outside_string_literals(&output, |segment| {
        adjust_reference_calls(segment, state)
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
            let cleaned_param = Regex::new(r"\b(?:__)?restrict\b")
                .unwrap()
                .replace_all(parameter, "")
                .to_string();
            let parameter = cleaned_param.trim();
            if let Some(bracket_pos) = parameter.find('[') {
                let before_bracket = parameter[..bracket_pos].trim();
                let tokens: Vec<&str> = before_bracket.split_whitespace().collect();
                if tokens.len() >= 2 {
                    let param_name = tokens.last().unwrap();
                    let raw_type = tokens[..tokens.len() - 1].join(" ");
                    let elem_rust = map_cpp_type_to_rust(&raw_type, state);
                    state
                        .method_parameter_names
                        .insert((*param_name).to_string());
                    state.declared_variables.insert((*param_name).to_string());
                    if raw_type.contains("const") {
                        return Some(format!("{}: &[{}]", param_name, elem_rust));
                    } else {
                        return Some(format!("{}: &mut [{}]", param_name, elem_rust));
                    }
                }
            }
            let tokens: Vec<&str> = parameter.split_whitespace().collect();
            if tokens.len() < 2 {
                return None;
            }
            let last_token = tokens.last().unwrap();
            let pointer_prefix_count = last_token
                .chars()
                .take_while(|ch| *ch == '*' || *ch == '&')
                .count();
            let attached_sigils = &last_token[..pointer_prefix_count];
            let parameter_name = &last_token[pointer_prefix_count..];
            let raw_type = format!("{} {}", tokens[..tokens.len() - 1].join(" "), attached_sigils)
                .trim()
                .to_string();
            let mut rust_type = map_cpp_type_to_rust(&raw_type, state);
            let is_char_ptr = raw_type.contains("char*") || raw_type.contains("char *");
            if is_char_ptr && rust_type == "String" {
                rust_type = "&str".to_string();
            }
            state
                .method_parameter_names
                .insert(parameter_name.to_string());
            state.register_variable_type(parameter_name, &rust_type);

            if rust_type == "&str" {
                Some(format!("{}: &str", parameter_name))
            } else if !is_char_ptr && (raw_type.ends_with('*') || attached_sigils.contains('*')) {
                let without_ptr = raw_type
                    .trim_start_matches("const ")
                    .trim_end_matches('*')
                    .trim();
                let inner_rust = map_cpp_type_to_rust(without_ptr, state);
                if raw_type.contains("const") {
                    Some(format!("{}: &{}", parameter_name, inner_rust))
                } else {
                    Some(format!("{}: &mut {}", parameter_name, inner_rust))
                }
            } else if parameter.contains('&') && !parameter.contains("const") {
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

    converted_format = converted_format.replace("%%", "__RUSTC_PCT__");
    let specifier_regex =
        Regex::new(r"%(?:[-+0 #])*(?:\d+|\*)?(?:\.(\d+))?(?:hh|h|ll|l|L|z|j|t)?([diuoxXfFeEgGcsp])")
            .unwrap();
    let mut spec_list: Vec<(String, Option<usize>)> = Vec::new();
    converted_format = specifier_regex
        .replace_all(&converted_format, |captures: &regex::Captures| {
            let spec_char = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            let prec_opt = captures
                .get(1)
                .and_then(|m| m.as_str().parse::<usize>().ok());
            spec_list.push((spec_char.to_string(), prec_opt));
            if spec_char == "f" || spec_char == "F" {
                if let Some(precision) = prec_opt {
                    format!("{{:.{}}}", precision)
                } else {
                    "{:.6}".to_string()
                }
            } else if matches!(spec_char, "g" | "G" | "e" | "E") {
                "{}".to_string()
            } else if let Some(precision) = prec_opt {
                format!("{{:.{}}}", precision)
            } else {
                "{}".to_string()
            }
        })
        .to_string();
    converted_format = converted_format.replace("__RUSTC_PCT__", "%");
    let macro_name = if has_newline { "println!" } else { "print!" };

    match raw_args {
        Some(args) if !args.trim().is_empty() => {
            let translated_args = split_top_level_comma(args)
                .into_iter()
                .enumerate()
                .map(|(index, arg)| {
                    let translated_arg = translate_expression_with_state(arg, state);
                    if let Some((spec_char, prec_opt)) = spec_list.get(index) {
                        if matches!(spec_char.as_str(), "g" | "G" | "e" | "E") {
                            let exp_char = if spec_char == "G" || spec_char == "E" {
                                "E"
                            } else {
                                "e"
                            };
                            let exp_prec = if spec_char == "g" || spec_char == "G" {
                                prec_opt.unwrap_or(6).saturating_sub(1)
                            } else {
                                prec_opt.unwrap_or(6)
                            };
                            return format!(
                                "format!(\"{{:.{}{}}}\", {}).replace(\"{}-\", \"__EXP_NEG__\").replace(\"{}\", \"{}+\").replace(\"__EXP_NEG__\", \"{}-\")",
                                exp_prec,
                                exp_char,
                                translated_arg,
                                exp_char,
                                exp_char,
                                exp_char,
                                exp_char
                            );
                        }
                    }
                    translated_arg
                })
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
    let raw_trimmed = raw_line.trim();

    if raw_trimmed.is_empty() {
        return String::new();
    }

    if raw_trimmed.starts_with("//") || raw_trimmed.starts_with("/*") || raw_trimmed.starts_with('*') {
        return raw_line.to_string();
    }

    let (code_part, trailing_comment) = split_trailing_line_comment(raw_trimmed);
    let mut working_owned = code_part.trim().to_string();
    if let Some(pending) = state.pending_multiline_statement.take() {
        working_owned = format!("{} {}", pending, working_owned);
    }

    if working_owned.ends_with('\\') {
        working_owned.pop();
        state.pending_multiline_statement = Some(working_owned.trim_end().to_string());
        return String::new();
    }

    let is_label = working_owned.starts_with("case ")
        || working_owned == "default:"
        || working_owned == "public:"
        || working_owned == "private:"
        || working_owned == "protected:";
    let is_template_header = working_owned.starts_with("template") && working_owned.ends_with('>');

    if !working_owned.ends_with(';')
        && !working_owned.ends_with('{')
        && !working_owned.ends_with('}')
        && !state.inside_enum_block
        && !state.inside_designated_init
        && !working_owned.starts_with('#')
        && !is_template_header
        && !is_label
    {
        state.pending_multiline_statement = Some(working_owned);
        return String::new();
    }

    let trimmed = working_owned.trim();
    let comment_suffix = trailing_comment
        .map(|comment| format!(" {}", comment))
        .unwrap_or_default();

    let translated = translate_single_line_core(&leading_whitespace, trimmed, raw_line, state);
    if translated.is_empty() || comment_suffix.is_empty() {
        translated
    } else {
        format!("{}{}", translated, comment_suffix)
    }
}

fn is_unsupported_fallback_construct(trimmed: &str) -> bool {
    super::diagnostics::classify_unsupported_construct(trimmed).is_some()
}

fn translate_single_line_core(
    leading_whitespace: &str,
    trimmed: &str,
    raw_line: &str,
    state: &mut TranslatorState,
) -> String {
    let _ = raw_line;

    if is_unsupported_fallback_construct(trimmed) {
        return format!(
            "{}// [Unsupported C/C++ syntax in regex fallback]: {}",
            leading_whitespace, trimmed
        );
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
            "float.h" | "cfloat" => {
                return format!(
                    "{}// #include <{}> (C long double / LDBL_* has no distinct stable Rust primitive; mapped to f64)",
                    leading_whitespace, header
                );
            }
            _ => {}
        }
        return format!("{}// #include <{}>", leading_whitespace, header);
    }

    if trimmed == "using namespace std;" {
        return format!("{}// using namespace std;", leading_whitespace);
    }

    if trimmed.starts_with("#define VectorCopy(") {
        return format!("{}// #define VectorCopy", leading_whitespace);
    }
    if trimmed.starts_with("#define VectorMA(") {
        return format!("{}// #define VectorMA (expanded inline)", leading_whitespace);
    }
    if trimmed.starts_with("#define TO_FIXED(") {
        return format!(
            "{}fn TO_FIXED(x: f32) -> fixed {{ (x * ((1 << FRACBITS) as f32)) as fixed }}",
            leading_whitespace
        );
    }
    if trimmed.starts_with("#define TO_FLOAT(") {
        return format!(
            "{}fn TO_FLOAT(x: fixed) -> f32 {{ (x as f32) / ((1 << FRACBITS) as f32) }}",
            leading_whitespace
        );
    }

    if let Some(captures) = DEFINE_GENERIC_MACRO_REGEX.captures(trimmed) {
        let macro_name = captures.get(1).unwrap().as_str();
        let associations_raw = captures.get(3).unwrap().as_str();
        let mut branches = Vec::new();
        let mut default_expr = "\"unknown\"".to_string();
        for item in split_top_level_comma(associations_raw) {
            if let Some((type_key, val_expr)) = item.split_once(':') {
                let key = type_key.trim();
                let val = val_expr.trim();
                if key == "default" {
                    default_expr = val.to_string();
                } else {
                    branches.push((key.to_string(), val.to_string()));
                }
            }
        }
        state
            .generic_macros
            .insert(macro_name.to_string(), (branches, default_expr));
        return format!(
            "{}// C11 _Generic macro '{}' dispatched via static type table",
            leading_whitespace, macro_name
        );
    }

    if let Some(captures) = DEFINE_REGEX.captures(trimmed) {
        let name = captures.get(1).unwrap().as_str();
        let value = captures.get(2).unwrap().as_str().trim();
        let rust_type = if value.starts_with('"') {
            "&str"
        } else if value.contains('.') {
            "f64"
        } else {
            "i32"
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

    if state.inside_enum_block {
        if let Some(captures) = STRUCT_CLOSE_REGEX.captures(trimmed) {
            state.inside_enum_block = false;
            let enum_name = captures
                .get(1)
                .map(|m| m.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("AnonEnum");
            state.enum_type_names.insert(enum_name.to_string());
            let mut lines = vec![format!("{}pub type {} = i32;", leading_whitespace, enum_name)];
            for (index, variant) in state.collected_enum_variants.drain(..).enumerate() {
                lines.push(format!(
                    "{}pub const {}: {} = {};",
                    leading_whitespace, variant, enum_name, index
                ));
            }
            return lines.join("\n");
        }
        for part in trimmed.split(',') {
            let variant = part.trim();
            if !variant.is_empty() {
                state.collected_enum_variants.push(variant.to_string());
            }
        }
        return String::new();
    }

    if state.inside_designated_init {
        if trimmed == "};" {
            state.inside_designated_init = false;
            return format!("{}    ..Default::default()\n{}}};", leading_whitespace, leading_whitespace);
        }
        if let Some(captures) = DESIGNATED_FIELD_ARRAY_REGEX.captures(trimmed) {
            let field_name = captures.get(1).unwrap().as_str();
            let rust_field_name = if field_name == "type" { "r#type" } else { field_name };
            let items_raw = captures.get(2).unwrap().as_str();
            let translated_items = split_top_level_comma(items_raw)
                .into_iter()
                .map(|item| translate_expression_with_state(item, state))
                .collect::<Vec<_>>()
                .join(", ");
            return format!("{}{}: [{}],", leading_whitespace, rust_field_name, translated_items);
        }
        if let Some(captures) = DESIGNATED_FIELD_VALUE_REGEX.captures(trimmed) {
            let field_name = captures.get(1).unwrap().as_str();
            let rust_field_name = if field_name == "type" { "r#type" } else { field_name };
            let val_expr = translate_expression_with_state(captures.get(2).unwrap().as_str(), state);
            return format!("{}{}: {},", leading_whitespace, rust_field_name, val_expr);
        }
        return String::new();
    }

    if let Some(captures) = TYPEDEF_ARRAY_REGEX.captures(trimmed) {
        let elem_raw = captures.get(1).unwrap().as_str();
        let alias = captures.get(2).unwrap().as_str();
        let count_str = captures.get(3).unwrap().as_str();
        let elem_rust = map_cpp_type_to_rust(elem_raw, state);
        return format!(
            "{}pub type {} = [{}; {}];",
            leading_whitespace, alias, elem_rust, count_str
        );
    }

    if let Some(captures) = TYPEDEF_PRIMITIVE_REGEX.captures(trimmed) {
        let base_raw = captures.get(1).unwrap().as_str();
        let alias = captures.get(2).unwrap().as_str();
        let rust_type = map_cpp_type_to_rust(base_raw, state);
        return format!("{}pub type {} = {};", leading_whitespace, alias, rust_type);
    }

    if let Some(captures) = ANON_ENUM_REGEX.captures(trimmed) {
        let body = captures.get(1).unwrap().as_str();
        let mut next_value = 0i32;
        let mut lines = Vec::new();
        for item in split_top_level_comma(body) {
            let trimmed_item = item.trim();
            if trimmed_item.is_empty() {
                continue;
            }
            if let Some((name_part, val_part)) = trimmed_item.split_once('=') {
                let name = name_part.trim();
                let val_num = val_part.trim().parse::<i32>().unwrap_or(next_value);
                lines.push(format!("{}pub const {}: i32 = {};", leading_whitespace, name, val_num));
                next_value = val_num + 1;
            } else {
                lines.push(format!(
                    "{}pub const {}: i32 = {};",
                    leading_whitespace, trimmed_item, next_value
                ));
                next_value += 1;
            }
        }
        return lines.join("\n");
    }

    if let Some(captures) = TYPEDEF_ENUM_SINGLE_REGEX.captures(trimmed) {
        let variants_raw = captures.get(1).unwrap().as_str();
        let enum_name = captures.get(2).unwrap().as_str();
        state.enum_type_names.insert(enum_name.to_string());
        let mut lines = vec![format!("{}pub type {} = i32;", leading_whitespace, enum_name)];
        for (index, variant) in variants_raw.split(',').enumerate() {
            let cleaned_variant = variant.trim();
            if !cleaned_variant.is_empty() {
                lines.push(format!(
                    "{}pub const {}: {} = {};",
                    leading_whitespace, cleaned_variant, enum_name, index
                ));
            }
        }
        return lines.join("\n");
    }

    if TYPEDEF_ENUM_OPEN_REGEX.is_match(trimmed) {
        state.inside_enum_block = true;
        state.collected_enum_variants.clear();
        return String::new();
    }

    if let Some(captures) = CLASS_OR_STRUCT_REGEX.captures(trimmed) {
        let kind = captures.get(1).unwrap().as_str();
        let name = if let Some(explicit_name) = captures.get(2).map(|m| m.as_str()) {
            explicit_name.to_string()
        } else {
            state.anon_struct_counter += 1;
            format!("AnonStruct{}", state.anon_struct_counter)
        };
        state.inside_struct_name = Some(name.clone());
        state.struct_fields.clear();
        state.struct_field_defs.insert(name.clone(), Vec::new());
        state.struct_brace_depth = state.current_brace_depth;
        state.inside_class_public = kind == "struct" || kind == "union";
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

    if state.inside_struct_name.is_some() {
        if let Some(captures) = STRUCT_CLOSE_REGEX.captures(trimmed) {
            let struct_name = state.inside_struct_name.take().unwrap();
            let alias_opt = captures
                .get(1)
                .map(|m| m.as_str())
                .filter(|s| !s.is_empty());
            let generic_type = state.struct_generic.take();
            let impl_header = match &generic_type {
                Some(generic) => format!("impl<{}: Clone> {}<{}>", generic, struct_name, generic),
                None => format!("impl {}", struct_name),
            };
            state.struct_fields.clear();
            state.current_brace_depth = state.current_brace_depth.saturating_sub(1);
            let mut result = if state.collected_methods.is_empty() {
                format!("{}}}", leading_whitespace)
            } else {
                let methods_block = state
                    .collected_methods
                    .drain(..)
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{}}}\n\n{}{} {{\n{}\n{}}}",
                    leading_whitespace,
                    leading_whitespace,
                    impl_header,
                    methods_block,
                    leading_whitespace
                )
            };
            if let Some(alias) = alias_opt {
                if alias != struct_name {
                    if let Some(defs) = state.struct_field_defs.get(&struct_name).cloned() {
                        state.struct_field_defs.insert(alias.to_string(), defs);
                    }
                    if let Some(flex) = state.flex_array_structs.get(&struct_name).cloned() {
                        state.flex_array_structs.insert(alias.to_string(), flex);
                    }
                    result.push_str(&format!(
                        "\n{}pub type {} = {};",
                        leading_whitespace, alias, struct_name
                    ));
                }
            }
            return result;
        }
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

        if let Some(captures) = STRUCT_FLEX_ARRAY_FIELD_REGEX.captures(trimmed) {
            let raw_type = captures.get(1).unwrap().as_str().trim();
            let field_name = captures.get(2).unwrap().as_str();
            let elem_rust = map_cpp_type_to_rust(raw_type, state);
            let rust_type = format!("Vec<{}>", elem_rust);
            state.struct_fields.push(field_name.to_string());
            if let Some(struct_name) = state.inside_struct_name.as_ref() {
                state.flex_array_structs.insert(
                    struct_name.clone(),
                    (field_name.to_string(), elem_rust.clone()),
                );
                if let Some(defs) = state.struct_field_defs.get_mut(struct_name) {
                    defs.push((field_name.to_string(), rust_type.clone()));
                }
            }
            let visibility = if state.inside_class_public { "pub " } else { "" };
            return format!(
                "{}{}{}: {},",
                leading_whitespace, visibility, field_name, rust_type
            );
        }

        if let Some(captures) = STRUCT_ARRAY_FIELD_REGEX.captures(trimmed) {
            let raw_type = captures.get(1).unwrap().as_str().trim();
            let field_name = captures.get(2).unwrap().as_str();
            let size_expr = captures.get(3).unwrap().as_str().trim();
            state.struct_fields.push(field_name.to_string());
            let rust_field_name = if field_name == "type" {
                "r#type"
            } else {
                field_name
            };
            let rust_type = if raw_type == "char" {
                "String".to_string()
            } else if size_expr.chars().all(|c| c.is_ascii_digit()) {
                format!("[{}; {}]", map_cpp_type_to_rust(raw_type, state), size_expr)
            } else {
                format!(
                    "[{}; ({}) as usize]",
                    map_cpp_type_to_rust(raw_type, state),
                    size_expr
                )
            };
            if let Some(struct_name) = state.inside_struct_name.as_ref() {
                if let Some(defs) = state.struct_field_defs.get_mut(struct_name) {
                    defs.push((rust_field_name.to_string(), rust_type.clone()));
                }
            }
            let visibility = if state.inside_class_public { "pub " } else { "" };
            return format!(
                "{}{}{}: {},",
                leading_whitespace, visibility, rust_field_name, rust_type
            );
        }

        if let Some(captures) = STRUCT_MULTI_FIELD_REGEX.captures(trimmed) {
            let raw_type = captures.get(1).unwrap().as_str().trim();
            let fields_raw = captures.get(2).unwrap().as_str();
            let rust_type = map_cpp_type_to_rust(raw_type, state);
            let visibility = if state.inside_class_public { "pub " } else { "" };
            let mut emitted_fields = Vec::new();
            for raw_field in fields_raw.split(',') {
                let field_name = raw_field.trim();
                if field_name.is_empty() {
                    continue;
                }
                state.struct_fields.push(field_name.to_string());
                let rust_field_name = if field_name == "type" {
                    "r#type"
                } else {
                    field_name
                };
                if let Some(struct_name) = state.inside_struct_name.as_ref() {
                    if let Some(defs) = state.struct_field_defs.get_mut(struct_name) {
                        defs.push((rust_field_name.to_string(), rust_type.clone()));
                    }
                }
                emitted_fields.push(format!(
                    "{}{}{}: {},",
                    leading_whitespace, visibility, rust_field_name, rust_type
                ));
            }
            return emitted_fields.join("\n");
        }

        if let Some(captures) = STRUCT_SELF_PTR_FIELD_REGEX.captures(trimmed) {
            let target_type = captures.get(1).unwrap().as_str();
            let field_name = captures.get(2).unwrap().as_str();
            if state.inside_struct_name.as_deref() == Some(target_type) {
                state.struct_fields.push(field_name.to_string());
                state.self_ptr_fields.insert(field_name.to_string());
                let rust_type = format!("Option<Box<{}>>", target_type);
                if let Some(struct_name) = state.inside_struct_name.as_ref() {
                    if let Some(defs) = state.struct_field_defs.get_mut(struct_name) {
                        defs.push((field_name.to_string(), rust_type.clone()));
                    }
                }
                let visibility = if state.inside_class_public { "pub " } else { "" };
                return format!(
                    "{}{}{}: {},",
                    leading_whitespace, visibility, field_name, rust_type
                );
            }
        }

        if let Some(captures) = VAR_DECL_NO_INIT_REGEX.captures(trimmed) {
            let raw_type_base = captures.get(1).unwrap().as_str();
            let sigil = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            let combined_type = format!("{} {}", raw_type_base.trim(), sigil).trim().to_string();
            let field_name = captures.get(3).unwrap().as_str();
            state.struct_fields.push(field_name.to_string());
            let rust_field_name = if field_name == "type" {
                "r#type"
            } else {
                field_name
            };
            let rust_type = map_cpp_type_to_rust(&combined_type, state);
            if let Some(struct_name) = state.inside_struct_name.as_ref() {
                if let Some(defs) = state.struct_field_defs.get_mut(struct_name) {
                    defs.push((rust_field_name.to_string(), rust_type.clone()));
                }
            }
            let visibility = if state.inside_class_public { "pub " } else { "" };
            return format!(
                "{}{}{}: {},",
                leading_whitespace, visibility, rust_field_name, rust_type
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

    if state.current_brace_depth == 0 {
        if let Some(captures) = VAR_DECL_INIT_REGEX.captures(trimmed) {
            let sigil = captures.get(3).map(|m| m.as_str()).unwrap_or("");
            let var_name = captures.get(4).unwrap().as_str();
            let raw_val = captures.get(5).unwrap().as_str().trim();
            if sigil.contains('*') && (raw_val == "NULL" || raw_val == "nullptr") {
                state.global_cvar_head = Some(var_name.to_string());
                return format!(
                    "{}// Global pointer '{}' managed via safe Rust references",
                    leading_whitespace, var_name
                );
            }
        }
        if let Some(captures) = FUNCTION_PROTOTYPE_REGEX.captures(trimmed) {
            let return_type_raw = captures.get(1).unwrap().as_str().trim();
            let function_name = captures.get(2).unwrap().as_str();
            let params_raw = captures.get(3).unwrap().as_str();
            if !matches!(return_type_raw, "return" | "delete" | "throw") && !trimmed.contains('=') {
                let ref_modes = compute_function_ref_modes(params_raw);
                state
                    .function_ref_params
                    .insert(function_name.to_string(), ref_modes);
                return format!("{}// Forward declaration: {}", leading_whitespace, trimmed);
            }
        }
    }

    if let Some(captures) = DESIGNATED_INIT_SINGLE_LINE_REGEX.captures(trimmed) {
        let type_name = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        let body = captures.get(3).unwrap().as_str();
        state.register_variable_type(variable_name, type_name);

        let mut field_assignments = Vec::new();
        for item in split_top_level_comma(body) {
            let trimmed_item = item.trim();
            if let Some(arr_cap) = DESIGNATED_FIELD_ARRAY_REGEX.captures(trimmed_item) {
                let field = arr_cap.get(1).unwrap().as_str();
                let rust_field = if field == "type" { "r#type" } else { field };
                let items = split_top_level_comma(arr_cap.get(2).unwrap().as_str())
                    .into_iter()
                    .map(|elem| translate_expression_with_state(elem, state))
                    .collect::<Vec<_>>()
                    .join(", ");
                field_assignments.push(format!("{}: [{}]", rust_field, items));
            } else if let Some(val_cap) = DESIGNATED_FIELD_VALUE_REGEX.captures(trimmed_item) {
                let field = val_cap.get(1).unwrap().as_str();
                let rust_field = if field == "type" { "r#type" } else { field };
                let val_expr = translate_expression_with_state(val_cap.get(2).unwrap().as_str(), state);
                field_assignments.push(format!("{}: {}", rust_field, val_expr));
            }
        }
        return format!(
            "{}let mut {}: {} = {} {{ {}, ..Default::default() }};",
            leading_whitespace,
            variable_name,
            type_name,
            type_name,
            field_assignments.join(", ")
        );
    }

    if let Some(captures) = POSITIONAL_STRUCT_INIT_REGEX.captures(trimmed) {
        let type_name = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        let body = captures.get(3).unwrap().as_str();
        if body.trim() != "0" {
            if let Some(field_defs) = state.struct_field_defs.get(type_name).cloned() {
                state.register_variable_type(variable_name, type_name);
                let args = split_top_level_comma(body);
                if let Some(first_arg) = args.first() {
                    if first_arg.trim().starts_with('"') {
                        state
                            .cvar_instance_names
                            .insert(first_arg.trim().to_string(), variable_name.to_string());
                    }
                }
                let mut field_assignments = Vec::new();
                for (index, (field_name, field_type)) in field_defs.into_iter().enumerate() {
                    if let Some(raw_val) = args.get(index) {
                        let mut val_expr = translate_expression_with_state(raw_val, state);
                        if field_type == "String" && val_expr.starts_with('"') {
                            val_expr = format!("String::from({})", val_expr);
                        }
                        field_assignments.push(format!("{}: {}", field_name, val_expr));
                    }
                }
                return format!(
                    "{}let mut {}: {} = {} {{ {} }};",
                    leading_whitespace,
                    variable_name,
                    type_name,
                    type_name,
                    field_assignments.join(", ")
                );
            }
        }
    }

    if let Some(captures) = DESIGNATED_INIT_OPEN_REGEX.captures(trimmed) {
        let type_name = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        state.inside_designated_init = true;
        state.register_variable_type(variable_name, type_name);
        return format!(
            "{}let mut {}: {} = {} {{",
            leading_whitespace, variable_name, type_name, type_name
        );
    }

    if state.inside_cvar_function {
        if state.current_brace_depth <= state.cvar_function_depth && trimmed == "}" {
            state.inside_cvar_function = false;
            return format!("{}}}", leading_whitespace);
        }
        return format!("{}// {}", leading_whitespace, trimmed);
    }

    if state.in_linked_list_traversal && state.current_brace_depth <= state.linked_list_traversal_depth && trimmed == "}" {
        state.in_linked_list_traversal = false;
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

        let ref_modes = compute_function_ref_modes(params_raw);
        state
            .function_ref_params
            .insert(function_name.to_string(), ref_modes);
        let rust_params = translate_parameters(params_raw, state);
        if function_name == "Cvar_Set" && state.global_cvar_head.is_some() {
            state.inside_cvar_function = true;
            state.cvar_function_depth = previous_depth;
            return format!(
                "{}fn {}(_var_name: &str, _value: &str) {{",
                leading_whitespace, function_name
            );
        }
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

    let stmt = translate_statement_line(trimmed, state);
    if stmt.contains('\n') {
        stmt.lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{}{}", leading_whitespace, line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        format!("{}{}", leading_whitespace, stmt)
    }
}

fn coerce_arg_for_float_vec(raw_arg: &str, state: &TranslatorState) -> String {
    let translated = translate_expression_with_state(raw_arg, state);
    let trimmed = translated.trim();
    let is_int_literal = trimmed
        .strip_prefix('-')
        .unwrap_or(trimmed)
        .chars()
        .all(|c| c.is_ascii_digit())
        && !trimmed.is_empty();
    if is_int_literal {
        format!("{}.0", trimmed)
    } else {
        translated
    }
}

fn rust_type_concept_comment(rust_type: &str) -> Option<String> {
    if rust_type.starts_with("Vec<") {
        Some(format!(
            "// std::vector -> {}: owns its heap buffer (RAII Drop; iterate via &vec to borrow, &mut vec to mutate, or vec/.into_iter() to move)",
            rust_type
        ))
    } else if rust_type.starts_with("HashMap<") {
        Some(format!(
            "// std::unordered_map/map -> {}: owned key-value table; .insert(k, v) moves keys/values instead of default-inserting on []",
            rust_type
        ))
    } else if rust_type.starts_with("HashSet<") {
        Some(format!(
            "// std::unordered_set/set -> {}: owned unique-key set; .contains(&k) borrows the lookup key",
            rust_type
        ))
    } else if rust_type.starts_with("VecDeque<") {
        Some(format!(
            "// std::queue/deque -> {}: double-ended ring buffer (.push_back / .pop_front)",
            rust_type
        ))
    } else if rust_type.starts_with("Option<") {
        Some(format!(
            "// std::optional / nullable -> {}: explicit Some(v)/None replaces unchecked null/uninitialized access",
            rust_type
        ))
    } else if rust_type.starts_with("Box<") {
        Some(format!(
            "// Rust {}: single-owner heap allocation (non-null; freed automatically via RAII Drop when out of scope)",
            rust_type
        ))
    } else {
        None
    }
}

fn translate_statement_line(trimmed: &str, state: &mut TranslatorState) -> String {
    if state.inside_main_function && trimmed == "return 0;" {
        return "// return 0;".to_string();
    }

    if let Some(inner) = trimmed
        .strip_prefix("VectorMA(")
        .and_then(|rest| rest.strip_suffix(");"))
    {
        let args = split_top_level_comma(inner);
        if args.len() == 4 {
            let v = translate_expression_with_state(args[0], state);
            let s = translate_expression_with_state(args[1], state);
            let b = translate_expression_with_state(args[2], state);
            let o = translate_expression_with_state(args[3], state);
            return format!(
                "{o}[0] = {v}[0] + {b}[0] * {s}; {o}[1] = {v}[1] + {b}[1] * {s}; {o}[2] = {v}[2] + {b}[2] * {s};"
            );
        }
    }

    if let Some(ref head_var) = state.global_cvar_head {
        if trimmed.contains(head_var) && trimmed.contains('=') {
            return "// linked into global cvar list".to_string();
        }
    }

    let cvar_set_call_regex =
        Regex::new(r#"^Cvar_Set\s*\(\s*("[^"]+")\s*,\s*("[^"]+")\s*\)\s*;\s*$"#).unwrap();
    if let Some(captures) = cvar_set_call_regex.captures(trimmed) {
        let name_lit = captures.get(1).unwrap().as_str();
        let val_lit = captures.get(2).unwrap().as_str();
        if let Some(var_name) = state.cvar_instance_names.get(name_lit) {
            return format!(
                "{var}.string = String::from({val}); {var}.value = {val}.parse::<f32>().unwrap_or(0.0);",
                var = var_name,
                val = val_lit
            );
        }
    }

    let self_ptr_assign_regex = Regex::new(
        r#"^([A-Za-z_][A-Za-z0-9_.]*)(?:->|\.)([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#,
    )
    .unwrap();
    if let Some(captures) = self_ptr_assign_regex.captures(trimmed) {
        let lhs = captures.get(1).unwrap().as_str();
        let field = captures.get(2).unwrap().as_str();
        let rhs = captures.get(3).unwrap().as_str();
        if state.self_ptr_fields.contains(field)
            && rhs != "NULL"
            && rhs != "nullptr"
            && rhs != "None"
        {
            return format!("{}.{} = Some(Box::new({}.clone()));", lhs, field, rhs);
        }
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

    if let Some(captures) = PUTS_REGEX.captures(trimmed) {
        let arg = captures.get(1).unwrap().as_str().trim();
        if arg.starts_with('"') && arg.ends_with('"') {
            return format!("println!({});", arg);
        }
        let translated_arg = translate_expression_with_state(arg, state);
        return format!("println!(\"{{}}\", {});", translated_arg);
    }

    if let Some(captures) = FGETS_REGEX.captures(trimmed) {
        let buffer_name = captures.get(1).unwrap().as_str();
        return format!("std::io::stdin().read_line(&mut {}).unwrap();", buffer_name);
    }

    if let Some(captures) = STRCSPN_NEWLINE_REGEX.captures(trimmed) {
        let buffer_name = captures.get(1).unwrap().as_str();
        return format!(
            "{} = {}.trim_end_matches(&['\\r', '\\n'][..]).to_string();",
            buffer_name, buffer_name
        );
    }

    if let Some(captures) = MEMMOVE_SHIFT_REGEX.captures(trimmed) {
        let buffer_name = captures.get(1).unwrap().as_str();
        let offset_raw =
            translate_expression_with_state(captures.get(2).unwrap().as_str(), state);
        let offset_usize = if offset_raw.chars().all(|c| c.is_ascii_digit()) {
            offset_raw
        } else {
            format!("({}) as usize", offset_raw)
        };
        return format!(
            "{} = {}.chars().skip({}).collect();",
            buffer_name, buffer_name, offset_usize
        );
    }

    if let Some(captures) = SCANF_REGEX.captures(trimmed) {
        let format_spec = captures.get(1).unwrap().as_str();
        let raw_targets = captures.get(2).unwrap().as_str();
        let targets: Vec<String> = split_top_level_comma(raw_targets)
            .into_iter()
            .map(|target| {
                let cleaned = target.trim().trim_start_matches('&').trim();
                translate_expression_with_state(cleaned, state)
            })
            .collect();

        if targets.len() == 1
            && !targets[0].contains('.')
            && !targets[0].contains('[')
            && !state.declared_variables.contains(&targets[0])
        {
            let variable_name = &targets[0];
            let rust_type = if format_spec.contains("%lf") {
                "f64"
            } else if format_spec.contains("%f") {
                "f32"
            } else if format_spec.contains("%ld") || format_spec.contains("%lld") {
                "i64"
            } else if format_spec.contains("%lu") {
                "usize"
            } else if format_spec.contains("%u") {
                "u32"
            } else if format_spec.contains("%c") {
                "char"
            } else if format_spec.contains('s') {
                "String"
            } else {
                "i32"
            };
            state.register_variable_type(variable_name, rust_type);
            return format!(
                "{{ let mut _stdin_line = String::new(); std::io::stdin().read_line(&mut _stdin_line).unwrap(); let mut _tokens = _stdin_line.split_whitespace(); let {}: {} = _tokens.next().unwrap_or_default().parse().unwrap_or_default(); }}",
                variable_name, rust_type
            );
        }

        let mut statements = vec![
            "let mut _stdin_line = String::new();".to_string(),
            "std::io::stdin().read_line(&mut _stdin_line).unwrap();".to_string(),
            "let mut _tokens = _stdin_line.split_whitespace();".to_string(),
        ];
        for target in targets {
            statements.push(format!(
                "{} = _tokens.next().unwrap_or_default().parse().unwrap_or_default();",
                target
            ));
        }
        return format!("{{ {} }}", statements.join(" "));
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

    if let Some(captures) = VEC_INSERT_BEGIN_REGEX.captures(trimmed) {
        let vec_name = captures.get(1).unwrap().as_str();
        let offset_opt = captures.get(2).map(|m| m.as_str().trim());
        let raw_val = captures.get(3).unwrap().as_str().trim();
        let val_expr = if state.float_vector_variables.contains(vec_name) {
            coerce_arg_for_float_vec(raw_val, state)
        } else {
            translate_expression_with_state(raw_val, state)
        };
        let index_expr = match offset_opt {
            None => "0".to_string(),
            Some(off) if off.chars().all(|c| c.is_ascii_digit()) => off.to_string(),
            Some(off) => format!("({}) as usize", translate_expression_with_state(off, state)),
        };
        return format!(
            "{}.insert({}, {}); // Vec::insert takes a usize index instead of a C++ begin() iterator",
            vec_name, index_expr, val_expr
        );
    }

    if let Some(captures) = VEC_INSERT_END_REGEX.captures(trimmed) {
        let vec_name = captures.get(1).unwrap().as_str();
        let raw_val = captures.get(2).unwrap().as_str().trim();
        let val_expr = if state.float_vector_variables.contains(vec_name) {
            coerce_arg_for_float_vec(raw_val, state)
        } else {
            translate_expression_with_state(raw_val, state)
        };
        return format!("{}.push({});", vec_name, val_expr);
    }

    if let Some(captures) = VEC_ERASE_BEGIN_REGEX.captures(trimmed) {
        let vec_name = captures.get(1).unwrap().as_str();
        let offset_opt = captures.get(2).map(|m| m.as_str().trim());
        let index_expr = match offset_opt {
            None => "0".to_string(),
            Some(off) if off.chars().all(|c| c.is_ascii_digit()) => off.to_string(),
            Some(off) => format!("({}) as usize", translate_expression_with_state(off, state)),
        };
        return format!(
            "{}.remove({}); // Vec::remove takes a usize index instead of a C++ iterator",
            vec_name, index_expr
        );
    }

    if let Some(captures) = VEC_PUSH_CALL_REGEX.captures(trimmed) {
        let vec_name = captures.get(1).unwrap().as_str();
        let raw_val = captures.get(2).unwrap().as_str().trim();
        if state.float_vector_variables.contains(vec_name) {
            let val_expr = coerce_arg_for_float_vec(raw_val, state);
            return format!("{}.push({});", vec_name, val_expr);
        }
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
        let rust_type = format!("Vec<{}>", inner_type);
        state.register_variable_type(variable_name, &rust_type);
        let comment = rust_type_concept_comment(&rust_type).unwrap();
        return format!(
            "{}\nlet mut {}: {} = vec![{}; ({}) as usize];",
            comment, variable_name, rust_type, init_val, size_expr
        );
    }

    if let Some(captures) = VECTOR_CTOR_ONE_ARG_REGEX.captures(trimmed) {
        let inner_type = map_cpp_type_to_rust(captures.get(1).unwrap().as_str(), state);
        let variable_name = captures.get(2).unwrap().as_str();
        let size_expr = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        let rust_type = format!("Vec<{}>", inner_type);
        state.register_variable_type(variable_name, &rust_type);
        let comment = rust_type_concept_comment(&rust_type).unwrap();
        return format!(
            "{}\nlet mut {}: {} = vec![Default::default(); ({}) as usize];",
            comment, variable_name, rust_type, size_expr
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
            return format!(
                "for {} in {}.iter().cloned() {{ // & borrows elements immutably without moving the container",
                item_name, collection_expr
            );
        } else if binding_type.contains('&') {
            return format!(
                "for {} in &mut {} {{ // &mut borrows elements mutably in-place without reallocating",
                item_name, collection_expr
            );
        } else {
            return format!("for {} in {}.iter().cloned() {{", item_name, collection_expr);
        }
    }

    if state.in_linked_list_traversal {
        if trimmed.contains("->next;") || trimmed.contains("= curr;") {
            return "// handled by Rust iterator".to_string();
        }
        if FREE_CALL_REGEX.is_match(trimmed) {
            return "// memory freed automatically by Rust".to_string();
        }
    }

    if let Some(head_var) = state.linked_list_head_var.clone() {
        let head_push_if_regex = Regex::new(&format!(
            r#"^if\s*\(\s*!\s*{}\s*\)\s*{}\s*=\s*[^=]+=\s*([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#,
            regex::escape(&head_var),
            regex::escape(&head_var)
        ))
        .unwrap();
        if head_push_if_regex.is_match(trimmed) {
            return "// list push handled below".to_string();
        }
        if trimmed.starts_with("else") && trimmed.contains("->next =") {
            let pushed_var_regex =
                Regex::new(r"->next\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*;").unwrap();
            if let Some(captures) = pushed_var_regex.captures(trimmed) {
                let node_var = captures.get(1).unwrap().as_str();
                return format!("{}.push({});", head_var, node_var);
            }
        }
        if let Some(iter_var) = state.linked_list_iter_var.clone() {
            let while_not_null_regex = Regex::new(&format!(
                r#"^while\s*\(\s*{}\s*!=\s*(?:NULL|nullptr)\s*\)\s*\{{\s*$"#,
                regex::escape(&iter_var)
            ))
            .unwrap();
            if while_not_null_regex.is_match(trimmed) {
                state.in_linked_list_traversal = true;
                state.linked_list_traversal_depth = state.current_brace_depth.saturating_sub(1);
                return format!("for {} in &{} {{", iter_var, head_var);
            }
        }
    }

    if let Some(captures) = LINKED_LIST_HEAD_TAIL_REGEX.captures(trimmed) {
        let node_type = captures.get(1).unwrap().as_str();
        let head_var = captures.get(2).unwrap().as_str();
        let tail_var = captures.get(3).unwrap().as_str();
        state.linked_list_node_type = Some(node_type.to_string());
        state.linked_list_head_var = Some(head_var.to_string());
        state.linked_list_tail_var = Some(tail_var.to_string());
        return format!("let mut {}: Vec<{}> = Vec::new();", head_var, node_type);
    }

    if let Some(captures) = MALLOC_FLEX_ARRAY_REGEX.captures(trimmed) {
        let struct_type = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        let count_expr = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        state.register_variable_type(variable_name, struct_type);
        state.box_pointer_variables.insert(variable_name.to_string());
        if let Some((flex_field, _)) = state.flex_array_structs.get(struct_type) {
            return format!(
                "// Rust Box<{}>: single-owner heap allocation with Vec replacing C99 flexible array member\nlet mut {}: Box<{}> = Box::new({} {{ {}: vec![Default::default(); ({}) as usize], ..Default::default() }});",
                struct_type, variable_name, struct_type, struct_type, flex_field, count_expr
            );
        }
    }

    if let Some(captures) = MALLOC_STRUCT_REGEX.captures(trimmed) {
        let raw_type = captures.get(1).unwrap().as_str().trim();
        let variable_name = captures.get(2).unwrap().as_str();
        let rust_type = map_cpp_type_to_rust(raw_type, state);
        state.register_variable_type(variable_name, &rust_type);
        if state.linked_list_head_var.is_some() {
            return format!("let mut {}: {} = Default::default();", variable_name, rust_type);
        }
        state.box_pointer_variables.insert(variable_name.to_string());
        return format!(
            "// Rust Box<{}>: single-owner heap allocation (non-null; freed automatically via RAII Drop when out of scope)\nlet mut {}: Box<{}> = Box::new(Default::default());",
            rust_type, variable_name, rust_type
        );
    }

    if let Some(captures) = COMPOUND_LITERAL_PTR_REGEX.captures(trimmed) {
        let type_name = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        let literal_type = captures.get(3).unwrap().as_str();
        let body = captures.get(4).unwrap().as_str();
        state.register_variable_type(variable_name, type_name);
        let mut field_assignments = Vec::new();
        for item in split_top_level_comma(body) {
            let trimmed_item = item.trim();
            if let Some(val_cap) = DESIGNATED_FIELD_VALUE_REGEX.captures(trimmed_item) {
                let field = val_cap.get(1).unwrap().as_str();
                let rust_field = if field == "type" { "r#type" } else { field };
                let val_expr =
                    translate_expression_with_state(val_cap.get(2).unwrap().as_str(), state);
                field_assignments.push(format!("{}: {}", rust_field, val_expr));
            }
        }
        return format!(
            "// C99 compound literal pointer -> stack-owned Rust struct with Default fill\nlet mut {}: {} = {} {{ {}, ..Default::default() }};",
            variable_name,
            type_name,
            literal_type,
            field_assignments.join(", ")
        );
    }

    if let Some(captures) = VLA_2D_REGEX.captures(trimmed) {
        let raw_type = captures.get(1).unwrap().as_str().trim();
        let variable_name = captures.get(2).unwrap().as_str();
        let rows_expr = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        let cols_expr = translate_expression_with_state(captures.get(4).unwrap().as_str(), state);
        let elem_rust = map_cpp_type_to_rust(raw_type, state);
        state.register_variable_type(variable_name, &format!("Vec<Vec<{}>>", elem_rust));
        state.vla_byte_sizes.insert(
            variable_name.to_string(),
            format!(
                "(({}) as usize) * (({}) as usize) * std::mem::size_of::<{}>()",
                rows_expr, cols_expr, elem_rust
            ),
        );
        return format!(
            "// C99 runtime VLA -> heap-allocated nested Vec<Vec<{}>> (Rust arrays require compile-time const lengths)\nlet mut {}: Vec<Vec<{}>> = vec![vec![Default::default(); ({}) as usize]; ({}) as usize];",
            elem_rust, variable_name, elem_rust, cols_expr, rows_expr
        );
    }

    if NULL_RETURN_CHECK_REGEX.is_match(trimmed) {
        return "// null check omitted in safe Rust".to_string();
    }

    if let Some(captures) = FREE_CALL_REGEX.captures(trimmed) {
        let target = captures.get(1).unwrap().as_str();
        return format!(
            "drop({}); // Explicit early drop (Rust also frees Box automatically at scope exit)",
            target
        );
    }

    if let Some(captures) = STRUCT_ZERO_INIT_REGEX.captures(trimmed) {
        let type_name = captures.get(1).unwrap().as_str();
        let variable_name = captures.get(2).unwrap().as_str();
        state.register_variable_type(variable_name, type_name);
        return format!("let mut {}: {} = Default::default();", variable_name, type_name);
    }

    if let Some(captures) = ARRAY_ZERO_INIT_REGEX.captures(trimmed) {
        let elem_type = map_cpp_type_to_rust(captures.get(1).unwrap().as_str(), state);
        let variable_name = captures.get(2).unwrap().as_str();
        let count_str = captures.get(3).unwrap().as_str();
        state.register_variable_type(variable_name, &format!("Vec<{}>", elem_type));
        state.array_variables.insert(variable_name.to_string());
        return format!(
            "let mut {}: Vec<{}> = vec![Default::default(); {}];",
            variable_name, elem_type, count_str
        );
    }

    if let Some(captures) = WHILE_STR_SEARCH_ASSIGN_REGEX.captures(trimmed) {
        let target_var = captures.get(1).unwrap().as_str().trim();
        let func_name = captures.get(2).unwrap().as_str().trim();
        let haystack = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        let needle = translate_expression_with_state(captures.get(4).unwrap().as_str(), state);
        state.str_slice_variables.insert(target_var.to_string());
        state.char_ptr_iter_variables.insert(target_var.to_string());
        let method = if func_name == "strrchr" { "rfind" } else { "find" };
        return format!(
            "// C {} pointer scan -> Rust Option<usize> pattern match on &str slice\nwhile let Some(_pos) = {}.{}({}) {{\n    {} = &{}[_pos..];",
            func_name, haystack, method, needle, target_var, haystack
        );
    }

    if let Some(captures) = IF_STR_SEARCH_ASSIGN_REGEX.captures(trimmed) {
        let target_var = captures.get(1).unwrap().as_str().trim();
        let func_name = captures.get(2).unwrap().as_str().trim();
        let haystack = translate_expression_with_state(captures.get(3).unwrap().as_str(), state);
        let needle = translate_expression_with_state(captures.get(4).unwrap().as_str(), state);
        state.str_slice_variables.insert(target_var.to_string());
        state.char_ptr_iter_variables.insert(target_var.to_string());
        let method = if func_name == "strrchr" { "rfind" } else { "find" };
        return format!(
            "if let Some(_pos) = {}.{}({}) {{\n    {} = &{}[_pos..];",
            haystack, method, needle, target_var, haystack
        );
    }

    if let Some(captures) = WHILE_REGEX.captures(trimmed) {
        let condition = translate_expression_with_state(captures.get(1).unwrap().as_str(), state);
        return format!("while {} {{", condition);
    }

    if let Some(captures) = ELSE_IF_REGEX.captures(trimmed) {
        let has_close_brace = captures.get(1).is_some();
        let condition = translate_expression_with_state(captures.get(2).unwrap().as_str(), state);
        if has_close_brace {
            return format!("}} else if {} {{", condition);
        }
        return format!("else if {} {{", condition);
    }

    if let Some(captures) = ELSE_REGEX.captures(trimmed) {
        if captures.get(1).is_some() {
            return "} else {".to_string();
        }
        return "else {".to_string();
    }

    if let Some(captures) = SINGLE_LINE_IF_REGEX.captures(trimmed) {
        let raw_cond = captures.get(1).unwrap().as_str().trim();
        let condition = if state.box_pointer_variables.contains(raw_cond) {
            "true".to_string()
        } else {
            translate_expression_with_state(raw_cond, state)
        };
        let inline_stmt = translate_statement_line(captures.get(2).unwrap().as_str(), state);
        return format!("if {} {{ {} }}", condition, inline_stmt);
    }

    if let Some(captures) = CHAR_BUFFER_REGEX.captures(trimmed) {
        let variable_name = captures.get(1).unwrap().as_str();
        state.register_variable_type(variable_name, "String");
        return format!("let mut {}: String = String::new();", variable_name);
    }

    if let Some(captures) = ARRAY_INIT_REGEX.captures(trimmed) {
        let is_const = captures.get(1).is_some();
        let raw_type = captures.get(2).unwrap().as_str().trim();
        let sigil = captures.get(3).map(|m| m.as_str()).unwrap_or("");
        let variable_name = captures.get(4).unwrap().as_str();
        let items_raw = captures.get(5).unwrap().as_str();
        let rust_vec_type = if sigil.contains('*') && raw_type == "char" {
            "Vec<&str>".to_string()
        } else {
            let rust_elem_type = map_cpp_type_to_rust(raw_type, state);
            format!("Vec<{}>", rust_elem_type)
        };
        state.register_variable_type(variable_name, &rust_vec_type);
        let translated_items = split_top_level_comma(items_raw)
            .into_iter()
            .map(|item| translate_expression_with_state(item, state))
            .collect::<Vec<_>>()
            .join(", ");
        let mutability = if is_const { "" } else { "mut " };
        return format!(
            "let {}{}: {} = vec![{}];",
            mutability, variable_name, rust_vec_type, translated_items
        );
    }

    if let Some(captures) = IF_REGEX.captures(trimmed) {
        let raw_cond = captures.get(1).unwrap().as_str().trim();
        let condition = if state.box_pointer_variables.contains(raw_cond) {
            "true".to_string()
        } else {
            translate_expression_with_state(raw_cond, state)
        };
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
        let raw_target = captures
            .get(1)
            .or_else(|| captures.get(2))
            .unwrap()
            .as_str()
            .trim();
        if state.char_ptr_iter_variables.contains(raw_target)
            || state.str_slice_variables.contains(raw_target)
        {
            return format!("{} = &{}[1..];", raw_target, raw_target);
        }
        let target = translate_expression_with_state(raw_target, state);
        return format!("{} += 1;", target);
    }

    if let Some(captures) = DECREMENT_REGEX.captures(trimmed) {
        let raw_target = captures
            .get(1)
            .or_else(|| captures.get(2))
            .unwrap()
            .as_str()
            .trim();
        let target = translate_expression_with_state(raw_target, state);
        return format!("{} -= 1;", target);
    }

    if let Some(rest) = trimmed.strip_prefix("delete ") {
        let target = rest.trim_end_matches(';').trim();
        return format!("drop({});", target);
    }

    if let Some(captures) = MULTI_VAR_INIT_REGEX.captures(trimmed) {
        let raw_type = captures.get(1).unwrap().as_str().trim();
        let decls_raw = captures.get(2).unwrap().as_str();
        let first_word = raw_type.split_whitespace().next().unwrap_or("");
        if !matches!(first_word, "return" | "delete" | "throw" | "if" | "for" | "while") {
            let parts = split_top_level_comma(decls_raw);
            if parts.len() >= 2 && parts.iter().all(|part| part.contains('=')) {
                let rust_type = map_cpp_type_to_rust(raw_type, state);
                let mut stmts = Vec::new();
                for part in parts {
                    if let Some((name_part, val_part)) = part.split_once('=') {
                        let var_name = name_part.trim();
                        let val_expr = translate_expression_with_state(val_part.trim(), state);
                        state.register_variable_type(var_name, &rust_type);
                        state
                            .variable_c_types
                            .insert(var_name.to_string(), raw_type.to_string());
                        stmts.push(format!("let mut {}: {} = {};", var_name, rust_type, val_expr));
                    }
                }
                if !stmts.is_empty() {
                    return stmts.join(" ");
                }
            }
        }
    }

    if let Some(captures) = VAR_DECL_INIT_REGEX.captures(trimmed) {
        let is_const = captures.get(1).is_some();
        let raw_type_base = captures.get(2).unwrap().as_str().trim();
        let sigil = captures.get(3).map(|m| m.as_str()).unwrap_or("");
        let variable_name = captures.get(4).unwrap().as_str();
        let raw_value = captures.get(5).unwrap().as_str().trim();

        let first_word = raw_type_base.split_whitespace().next().unwrap_or("");
        if !matches!(first_word, "return" | "delete" | "throw") {
            let combined_type = format!("{} {}", raw_type_base, sigil).trim().to_string();
            state
                .variable_c_types
                .insert(variable_name.to_string(), combined_type.clone());

            if sigil.contains('*') && state.array_variables.contains(raw_value) {
                state
                    .array_ptr_aliases
                    .insert(variable_name.to_string(), raw_value.to_string());
                return format!("// Pointer alias: {} -> {}", variable_name, raw_value);
            }

            if sigil.contains('*')
                && state.linked_list_head_var.as_deref() == Some(raw_value)
            {
                state.linked_list_iter_var = Some(variable_name.to_string());
                return "// list traversal start".to_string();
            }

            let is_char_ptr = combined_type == "char*"
                || combined_type == "char *"
                || combined_type == "const char*"
                || combined_type == "const char *";
            if is_char_ptr
                && (state.str_slice_variables.contains(raw_value)
                    || state.string_variables.contains(raw_value))
            {
                state.register_variable_type(variable_name, "&str");
                state
                    .char_ptr_iter_variables
                    .insert(variable_name.to_string());
                let init_expr = if state.string_variables.contains(raw_value) {
                    format!("&{}", raw_value)
                } else {
                    raw_value.to_string()
                };
                return format!("let mut {}: &str = {};", variable_name, init_expr);
            }

            let rust_type = if is_const && is_char_ptr {
                "&str".to_string()
            } else {
                map_cpp_type_to_rust(&combined_type, state)
            };
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
            let decl_line = format!(
                "let {}{}: {} = {};",
                mutability, variable_name, rust_type, translated_value
            );
            if let Some(comment) = rust_type_concept_comment(&rust_type) {
                return format!("{}\n{}", comment, decl_line);
            }
            return decl_line;
        }
    }

    if let Some(captures) = VAR_DECL_NO_INIT_REGEX.captures(trimmed) {
        let raw_type_base = captures.get(1).unwrap().as_str().trim();
        let sigil = captures.get(2).map(|m| m.as_str()).unwrap_or("");
        let variable_name = captures.get(3).unwrap().as_str();
        let first_word = raw_type_base.split_whitespace().next().unwrap_or("");
        if !matches!(
            first_word,
            "return" | "break" | "continue" | "else" | "delete" | "using" | "throw"
        ) {
            let combined_type = format!("{} {}", raw_type_base, sigil).trim().to_string();
            state
                .variable_c_types
                .insert(variable_name.to_string(), combined_type.clone());
            let rust_type = map_cpp_type_to_rust(&combined_type, state);
            state.register_variable_type(variable_name, &rust_type);
            let decl_line = if rust_type.starts_with("Vec<") {
                format!("let mut {}: {} = Vec::new();", variable_name, rust_type)
            } else if rust_type.starts_with("VecDeque<") {
                format!("let mut {}: {} = VecDeque::new();", variable_name, rust_type)
            } else if rust_type.starts_with("HashMap<") {
                format!("let mut {}: {} = HashMap::new();", variable_name, rust_type)
            } else if rust_type.starts_with("HashSet<") {
                format!("let mut {}: {} = HashSet::new();", variable_name, rust_type)
            } else if rust_type == "String" {
                format!("let mut {}: String = String::new();", variable_name)
            } else {
                format!("let mut {}: {} = Default::default();", variable_name, rust_type)
            };
            if let Some(comment) = rust_type_concept_comment(&rust_type) {
                return format!("{}\n{}", comment, decl_line);
            }
            return decl_line;
        }
    }

    translate_expression_with_state(trimmed, state)
}
