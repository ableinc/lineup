use std::path::Path;

use ignore::Walk;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ClassElement, Declaration, PropertyKey, Statement, TSSignature, TSType};
use oxc_parser::Parser;
use oxc_span::SourceType;
use regex::RegexSet;

use crate::parsers::SKIP_DIRS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TsDeclKind {
    Class,
    Interface,
    TypeAlias,
}

impl TsDeclKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TsDeclKind::Class => "class",
            TsDeclKind::Interface => "interface",
            TsDeclKind::TypeAlias => "type",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TsField {
    pub name: String,
    pub type_str: String,
    pub optional: bool,
    pub line_number: usize,
    pub raw_line: String,
}

#[derive(Debug, Clone)]
pub struct TsDeclaration {
    pub name: String,
    pub kind: TsDeclKind,
    pub type_params: Option<String>,
    pub fields: Vec<TsField>,
    pub doc_comments: Vec<String>,
    pub line_number: usize,
    pub has_extends: bool,
}

#[derive(Debug, Clone)]
pub struct TsFile {
    pub path: String,
    pub declarations: Vec<TsDeclaration>,
}

fn should_skip(rel_str: &str) -> bool {
    for dir in SKIP_DIRS {
        if rel_str.starts_with(&format!("{dir}/")) || rel_str.contains(&format!("/{dir}/")) {
            return true;
        }
    }
    false
}

pub fn walk_ts_repo(repo_path: &Path, ignore_patterns: &[String]) -> Vec<TsFile> {
    let pattern_set = if ignore_patterns.is_empty() {
        None
    } else {
        RegexSet::new(ignore_patterns).ok()
    };

    let mut files = Vec::new();
    for entry in Walk::new(repo_path).flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ts" && ext != "tsx" {
            continue;
        }
        let rel = path.strip_prefix(repo_path).unwrap_or(path).to_string_lossy();
        let rel_str = rel.as_ref();

        if should_skip(rel_str) {
            continue;
        }

        if let Some(ref set) = pattern_set {
            if set.is_match(rel_str) {
                continue;
            }
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let is_tsx = ext == "tsx";
        let declarations = parse_ts_declarations(&content, is_tsx);
        if !declarations.is_empty() {
            files.push(TsFile {
                path: path.to_string_lossy().into_owned(),
                declarations,
            });
        }
    }
    files
}

/// Convert a byte offset into a 1-based line number.
fn byte_to_line(src: &str, offset: usize) -> usize {
    let safe = offset.min(src.len());
    src[..safe].bytes().filter(|&b| b == b'\n').count() + 1
}

/// Extract the full source line that contains `offset`.
fn line_at(src: &str, offset: usize) -> String {
    let safe = offset.min(src.len());
    let line_start = src[..safe].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = src[safe..].find('\n').map(|i| i + safe).unwrap_or(src.len());
    src[line_start..line_end].to_string()
}

/// Walk backwards through lines immediately before `decl_start` collecting `//` comments.
fn doc_comments_before(src: &str, decl_start: usize) -> Vec<String> {
    let before = &src[..decl_start.min(src.len())];
    let lines: Vec<&str> = before.lines().collect();
    let mut comments = Vec::new();
    let mut idx = lines.len();
    while idx > 0 {
        idx -= 1;
        let trimmed = lines[idx].trim();
        if trimmed.starts_with("//") {
            comments.insert(0, lines[idx].to_string());
        } else {
            break;
        }
    }
    comments
}

/// Strip a leading `: ` from a type-annotation span slice.
fn strip_colon(s: &str) -> &str {
    s.trim_start_matches(':').trim_start_matches('?').trim_start_matches(':').trim()
}

// ---------------------------------------------------------------------------
// Field extraction helpers
// ---------------------------------------------------------------------------

fn field_name_from_key(key: &PropertyKey) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str().to_string()),
        PropertyKey::StringLiteral(lit) => Some(lit.value.as_str().to_string()),
        // Skip private (#field) and computed ([expr]) keys
        _ => None,
    }
}

fn type_str_from_annotation(ann_span_start: u32, ann_span_end: u32, src: &str) -> String {
    let text = &src[ann_span_start as usize..ann_span_end as usize];
    strip_colon(text).to_string()
}

// ---------------------------------------------------------------------------
// Per-declaration handlers
// ---------------------------------------------------------------------------

fn handle_class(cls: &oxc_ast::ast::Class, src: &str, results: &mut Vec<TsDeclaration>) {
    let Some(ref id) = cls.id else { return };
    let name = id.name.as_str().to_string();
    let line_number = byte_to_line(src, cls.span.start as usize);
    let has_extends = cls.super_class.is_some();
    let type_params = cls.type_parameters.as_ref().map(|tp| {
        src[tp.span.start as usize..tp.span.end as usize].to_string()
    });
    let doc_comments = doc_comments_before(src, cls.span.start as usize);

    let fields = cls
        .body
        .body
        .iter()
        .filter_map(|element| {
            let ClassElement::PropertyDefinition(prop) = element else { return None };
            // Skip static properties — they don't live in the instance shape
            if prop.r#static {
                return None;
            }
            let field_name = field_name_from_key(&prop.key)?;
            let type_str = if let Some(ann) = &prop.type_annotation {
                type_str_from_annotation(ann.span.start, ann.span.end, src)
            } else {
                "any".to_string()
            };
            let lnum = byte_to_line(src, prop.span.start as usize);
            let raw = line_at(src, prop.span.start as usize);
            Some(TsField { name: field_name, type_str, optional: prop.optional, line_number: lnum, raw_line: raw })
        })
        .collect();

    results.push(TsDeclaration { name, kind: TsDeclKind::Class, type_params, fields, doc_comments, line_number, has_extends });
}

fn handle_interface(iface: &oxc_ast::ast::TSInterfaceDeclaration, src: &str, results: &mut Vec<TsDeclaration>) {
    let name = iface.id.name.as_str().to_string();
    let line_number = byte_to_line(src, iface.span.start as usize);
    let has_extends = !iface.extends.is_empty();
    let type_params = iface.type_parameters.as_ref().map(|tp| {
        src[tp.span.start as usize..tp.span.end as usize].to_string()
    });
    let doc_comments = doc_comments_before(src, iface.span.start as usize);

    let fields = iface
        .body
        .body
        .iter()
        .filter_map(|sig| {
            let TSSignature::TSPropertySignature(prop) = sig else { return None };
            let field_name = field_name_from_key(&prop.key)?;
            let type_str = if let Some(ann) = &prop.type_annotation {
                type_str_from_annotation(ann.span.start, ann.span.end, src)
            } else {
                "any".to_string()
            };
            let lnum = byte_to_line(src, prop.span.start as usize);
            let raw = line_at(src, prop.span.start as usize);
            Some(TsField { name: field_name, type_str, optional: prop.optional, line_number: lnum, raw_line: raw })
        })
        .collect();

    results.push(TsDeclaration { name, kind: TsDeclKind::Interface, type_params, fields, doc_comments, line_number, has_extends });
}

fn handle_type_alias(alias: &oxc_ast::ast::TSTypeAliasDeclaration, src: &str, results: &mut Vec<TsDeclaration>) {
    // Only handle object type literals — `type Foo = { ... }`
    let TSType::TSTypeLiteral(lit) = &alias.type_annotation else { return };

    let name = alias.id.name.as_str().to_string();
    let line_number = byte_to_line(src, alias.span.start as usize);
    let type_params = alias.type_parameters.as_ref().map(|tp| {
        src[tp.span.start as usize..tp.span.end as usize].to_string()
    });
    let doc_comments = doc_comments_before(src, alias.span.start as usize);

    let fields = lit
        .members
        .iter()
        .filter_map(|sig| {
            let TSSignature::TSPropertySignature(prop) = sig else { return None };
            let field_name = field_name_from_key(&prop.key)?;
            let type_str = if let Some(ann) = &prop.type_annotation {
                type_str_from_annotation(ann.span.start, ann.span.end, src)
            } else {
                "any".to_string()
            };
            let lnum = byte_to_line(src, prop.span.start as usize);
            let raw = line_at(src, prop.span.start as usize);
            Some(TsField { name: field_name, type_str, optional: prop.optional, line_number: lnum, raw_line: raw })
        })
        .collect();

    results.push(TsDeclaration { name, kind: TsDeclKind::TypeAlias, type_params, fields, doc_comments, line_number, has_extends: false });
}

fn handle_declaration(decl: &Declaration, src: &str, results: &mut Vec<TsDeclaration>) {
    match decl {
        Declaration::ClassDeclaration(cls) => handle_class(cls, src, results),
        Declaration::TSInterfaceDeclaration(iface) => handle_interface(iface, src, results),
        Declaration::TSTypeAliasDeclaration(alias) => handle_type_alias(alias, src, results),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn parse_ts_declarations(src: &str, is_tsx: bool) -> Vec<TsDeclaration> {
    let allocator = Allocator::default();
    let source_type = if is_tsx { SourceType::tsx() } else { SourceType::ts() };
    let ret = Parser::new(&allocator, src, source_type).parse();

    let mut results = Vec::new();
    for stmt in &ret.program.body {
        match stmt {
            // Non-exported declarations (direct variants)
            Statement::ClassDeclaration(cls) => handle_class(cls, src, &mut results),
            Statement::TSInterfaceDeclaration(iface) => handle_interface(iface, src, &mut results),
            Statement::TSTypeAliasDeclaration(alias) => handle_type_alias(alias, src, &mut results),
            // `export class/interface/type ...`
            Statement::ExportNamedDeclaration(exp) => {
                if let Some(decl) = &exp.declaration {
                    handle_declaration(decl, src, &mut results);
                }
            }
            // `export default class Foo { ... }`
            Statement::ExportDefaultDeclaration(exp) => {
                use oxc_ast::ast::ExportDefaultDeclarationKind;
                if let ExportDefaultDeclarationKind::ClassDeclaration(cls) = &exp.declaration {
                    handle_class(cls, src, &mut results);
                }
            }
            _ => {}
        }
    }
    results
}
