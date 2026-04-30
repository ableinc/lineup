use std::collections::HashMap;
use crate::go_parser::{GoField, GoFile, GoStruct};
use crate::ts_parser::{TsDeclaration, TsField, TsFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    Amd64,
    Arm64,
}

impl Arch {
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::Amd64 => "amd64",
            Arch::Arm64 => "arm64",
        }
    }
}

impl std::str::FromStr for Arch {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arm64" => Ok(Arch::Arm64),
            _ => Ok(Arch::Amd64),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub size: u64,
    pub align: u64,
    pub approximate: bool,
}

/// Returns (size, align, approximate) for a Go type string.
pub fn type_info(type_str: &str, arch: Arch, registry: &HashMap<String, TypeInfo>) -> TypeInfo {
    let t = type_str.trim();

    // Pointer
    if t.starts_with('*') {
        return TypeInfo { size: 8, align: 8, approximate: false };
    }

    // Slice: []T
    if t.starts_with("[]") {
        return TypeInfo { size: 24, align: 8, approximate: false };
    }

    // Array: [N]T
    if t.starts_with('[') {
        if let Some(close) = t.find(']') {
            let n_str = &t[1..close];
            let elem_type = &t[close + 1..];
            if let Ok(n) = n_str.parse::<u64>() {
                let elem = type_info(elem_type, arch, registry);
                return TypeInfo {
                    size: n * elem.size,
                    align: elem.align,
                    approximate: elem.approximate,
                };
            }
        }
    }

    // Map, chan, func
    if t.starts_with("map[") || t.starts_with("chan ") || t == "chan" || t.starts_with("func(") || t.starts_with("func (") {
        return TypeInfo { size: 8, align: 8, approximate: false };
    }

    // Primitive types
    match t {
        "bool" | "byte" | "int8" | "uint8" => TypeInfo { size: 1, align: 1, approximate: false },
        "int16" | "uint16" => TypeInfo { size: 2, align: 2, approximate: false },
        "int32" | "uint32" | "float32" | "rune" => TypeInfo { size: 4, align: 4, approximate: false },
        "int64" | "uint64" | "float64" | "complex64" | "uintptr" => {
            TypeInfo { size: 8, align: 8, approximate: false }
        }
        "complex128" => TypeInfo { size: 16, align: 8, approximate: false },
        "int" | "uint" => TypeInfo { size: 8, align: 8, approximate: false },
        "string" => TypeInfo { size: 16, align: 8, approximate: false },
        "error" | "interface{}" | "any" => TypeInfo { size: 16, align: 8, approximate: false },
        _ => {
            // Strip package qualifier: pkg.Type → Type
            let short = t.rsplit('.').next().unwrap_or(t);

            if let Some(info) = registry.get(short).or_else(|| registry.get(t)) {
                return info.clone();
            }

            // Generic / unresolved — conservative 8 bytes, flagged
            TypeInfo { size: 8, align: 8, approximate: true }
        }
    }
}

fn align_up(offset: u64, align: u64) -> u64 {
    if align == 0 {
        return offset;
    }
    (offset + align - 1) & !(align - 1)
}

fn struct_layout(fields: &[(TypeInfo, String)]) -> (u64, u64) {
    // Returns (total_size, max_align)
    let mut offset: u64 = 0;
    let mut max_align: u64 = 1;

    for (info, _) in fields {
        offset = align_up(offset, info.align);
        offset += info.size;
        if info.align > max_align {
            max_align = info.align;
        }
    }
    // Trailing pad
    offset = align_up(offset, max_align);
    (offset, max_align)
}

#[derive(Debug, Clone)]
pub struct AnalyzedStruct {
    pub name: String,
    pub type_params: Option<String>,
    pub line_number: usize,
    pub current_size: u64,
    pub optimal_size: u64,
    pub bytes_saved: u64,
    pub current_def: String,
    pub optimized_def: String,
    pub has_generics: bool,
    pub has_embedded: bool,
    pub approximate: bool,
    /// "struct" for Go, "class" / "interface" / "type" for TypeScript.
    pub declaration_kind: String,
}

pub fn analyze_files(files: &[GoFile], arch: Arch) -> Vec<(String, AnalyzedStruct)> {
    // Pass 1: Build registry of all struct sizes
    let mut registry: HashMap<String, TypeInfo> = HashMap::new();

    // First pass with an empty registry to get basic sizes
    for file in files {
        for s in &file.structs {
            let (size, align, approx) = compute_struct_size(s, arch, &registry);
            registry.insert(
                s.name.clone(),
                TypeInfo { size, align, approximate: approx },
            );
        }
    }

    // Second pass with the registry populated
    for file in files {
        for s in &file.structs {
            let (size, align, approx) = compute_struct_size(s, arch, &registry);
            registry.insert(
                s.name.clone(),
                TypeInfo { size, align, approximate: approx },
            );
        }
    }

    // Pass 2: Analyze each struct for padding
    let mut results = Vec::new();
    for file in files {
        for s in &file.structs {
            if let Some(analyzed) = analyze_struct(s, arch, &registry) {
                results.push((file.path.clone(), analyzed));
            }
        }
    }
    results
}

fn compute_struct_size(s: &GoStruct, arch: Arch, registry: &HashMap<String, TypeInfo>) -> (u64, u64, bool) {
    let mut approximate = s.type_params.is_some();
    let field_infos: Vec<(TypeInfo, String)> = s
        .fields
        .iter()
        .map(|f| {
            let info = type_info(&f.type_str, arch, registry);
            if info.approximate {
                approximate = true;
            }
            (info, f.name.clone())
        })
        .collect();

    if field_infos.is_empty() {
        return (0, 1, approximate);
    }

    let (size, align) = struct_layout(&field_infos);
    (size, align, approximate)
}

fn analyze_struct(
    s: &GoStruct,
    arch: Arch,
    registry: &HashMap<String, TypeInfo>,
) -> Option<AnalyzedStruct> {
    if s.fields.is_empty() {
        return None;
    }

    let has_generics = s.type_params.is_some();
    let has_embedded = s.fields.iter().any(|f| f.name == f.type_str.trim_start_matches('*').rsplit('.').next().unwrap_or(&f.type_str));

    let mut approximate = has_generics;
    let field_infos: Vec<(TypeInfo, &GoField)> = s
        .fields
        .iter()
        .map(|f| {
            let info = type_info(&f.type_str, arch, registry);
            if info.approximate {
                approximate = true;
            }
            (info, f)
        })
        .collect();

    // Current layout
    let current_fields: Vec<(TypeInfo, String)> = field_infos
        .iter()
        .map(|(info, f)| (info.clone(), f.name.clone()))
        .collect();
    let (current_size, _) = struct_layout(&current_fields);

    // Optimal layout: sort by align desc, then size desc, then name
    let mut sorted = field_infos.clone();
    sorted.sort_by(|(a_info, a_field), (b_info, b_field)| {
        b_info
            .align
            .cmp(&a_info.align)
            .then(b_info.size.cmp(&a_info.size))
            .then(a_field.name.cmp(&b_field.name))
    });
    let optimal_fields: Vec<(TypeInfo, String)> = sorted
        .iter()
        .map(|(info, f)| (info.clone(), f.name.clone()))
        .collect();
    let (optimal_size, _) = struct_layout(&optimal_fields);

    let bytes_saved = current_size.saturating_sub(optimal_size);

    // Generate current_def
    let current_refs: Vec<&GoField> = s.fields.iter().collect();
    let current_def = build_def(s, &current_refs, false);
    // Generate optimized_def with reordered fields
    let sorted_fields: Vec<&GoField> = sorted.iter().map(|(_, f)| *f).collect();
    let optimized_def = build_def(s, &sorted_fields, true);

    Some(AnalyzedStruct {
        name: s.name.clone(),
        type_params: s.type_params.clone(),
        line_number: s.line_number,
        current_size,
        optimal_size,
        bytes_saved,
        current_def,
        optimized_def,
        has_generics,
        has_embedded,
        approximate,
        declaration_kind: "struct".to_string(),
    })
}

// ---------------------------------------------------------------------------
// TypeScript / V8 analysis
// ---------------------------------------------------------------------------

/// V8 JIT type-size model (64-bit, pointer compression enabled).
/// `number` fields are stored as unboxed 8-byte doubles.
/// Everything else is a 4-byte tagged/compressed pointer.
pub fn ts_type_info(type_str: &str) -> TypeInfo {
    match type_str.trim() {
        "number" => TypeInfo { size: 8, align: 8, approximate: false },
        "any" | "unknown" | "never" | "void" => TypeInfo { size: 4, align: 4, approximate: true },
        _ => TypeInfo { size: 4, align: 4, approximate: false },
    }
}

pub fn analyze_ts_files(files: &[TsFile]) -> Vec<(String, AnalyzedStruct)> {
    let mut results = Vec::new();
    for file in files {
        for decl in &file.declarations {
            if let Some(analyzed) = analyze_ts_decl(decl) {
                results.push((file.path.clone(), analyzed));
            }
        }
    }
    results
}

fn analyze_ts_decl(decl: &TsDeclaration) -> Option<AnalyzedStruct> {
    if decl.fields.is_empty() {
        return None;
    }

    let has_generics = decl.type_params.is_some();
    let has_embedded = decl.has_extends;
    let mut approximate = has_generics || has_embedded;

    let field_infos: Vec<(TypeInfo, &TsField)> = decl
        .fields
        .iter()
        .map(|f| {
            let info = ts_type_info(&f.type_str);
            if info.approximate {
                approximate = true;
            }
            (info, f)
        })
        .collect();

    let current_fields: Vec<(TypeInfo, String)> = field_infos
        .iter()
        .map(|(info, f)| (info.clone(), f.name.clone()))
        .collect();
    let (current_size, _) = struct_layout(&current_fields);

    // Optimal: numbers (8B) first, then everything else, stable by name within each group.
    let mut sorted = field_infos.clone();
    sorted.sort_by(|(a_info, a_field), (b_info, b_field)| {
        b_info
            .align
            .cmp(&a_info.align)
            .then(b_info.size.cmp(&a_info.size))
            .then(a_field.name.cmp(&b_field.name))
    });
    let optimal_fields: Vec<(TypeInfo, String)> = sorted
        .iter()
        .map(|(info, f)| (info.clone(), f.name.clone()))
        .collect();
    let (optimal_size, _) = struct_layout(&optimal_fields);

    let bytes_saved = current_size.saturating_sub(optimal_size);

    let current_refs: Vec<&TsField> = decl.fields.iter().collect();
    let current_def = build_ts_def(decl, &current_refs, false);
    let sorted_fields: Vec<&TsField> = sorted.iter().map(|(_, f)| *f).collect();
    let optimized_def = build_ts_def(decl, &sorted_fields, true);

    Some(AnalyzedStruct {
        name: decl.name.clone(),
        type_params: decl.type_params.clone(),
        line_number: decl.line_number,
        current_size,
        optimal_size,
        bytes_saved,
        current_def,
        optimized_def,
        has_generics,
        has_embedded,
        approximate,
        declaration_kind: decl.kind.as_str().to_string(),
    })
}

fn build_ts_def(decl: &TsDeclaration, fields: &[&TsField], optimized: bool) -> String {
    use crate::ts_parser::TsDeclKind;
    let mut lines: Vec<String> = Vec::new();

    for doc in &decl.doc_comments {
        lines.push(doc.clone());
    }
    if optimized {
        lines.push("// Reordered for optimal V8 memory layout".to_string());
    }

    let tp = decl.type_params.as_deref().unwrap_or("");
    let header = match decl.kind {
        TsDeclKind::Class => format!("class {}{} {{", decl.name, tp),
        TsDeclKind::Interface => format!("interface {}{} {{", decl.name, tp),
        TsDeclKind::TypeAlias => format!("type {}{} = {{", decl.name, tp),
    };
    lines.push(header);

    for field in fields {
        lines.push(format!("\t{}", field.raw_line.trim()));
    }

    let footer = if decl.kind == TsDeclKind::TypeAlias { "};" } else { "}" };
    lines.push(footer.to_string());
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Go analysis (unchanged)
// ---------------------------------------------------------------------------

fn build_def(s: &GoStruct, fields: &[&GoField], optimized: bool) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Prepend doc comments verbatim
    for doc in &s.doc_comments {
        lines.push(doc.clone());
    }

    if optimized {
        lines.push("// Reordered for optimal memory alignment".to_string());
    }

    // Struct header
    let header = if let Some(ref tp) = s.type_params {
        format!("type {}[{}] struct {{", s.name, tp)
    } else {
        format!("type {} struct {{", s.name)
    };
    lines.push(header);

    for field in fields {
        lines.push(format!("\t{}", field.raw_line.trim()));
    }

    lines.push("}".to_string());
    lines.join("\n")
}
