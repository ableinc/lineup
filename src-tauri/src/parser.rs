use std::path::Path;
use ignore::Walk;
use regex::RegexSet;

#[derive(Debug, Clone)]
pub struct GoField {
    pub name: String,
    pub type_str: String,
    pub tag: Option<String>,
    pub inline_comment: Option<String>,
    pub line_number: usize,
    pub raw_line: String,
}

#[derive(Debug, Clone)]
pub struct GoStruct {
    pub name: String,
    pub type_params: Option<String>,
    pub fields: Vec<GoField>,
    pub doc_comments: Vec<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct GoFile {
    pub path: String,
    pub structs: Vec<GoStruct>,
}

pub fn walk_repo(repo_path: &Path, ignore_patterns: &[String]) -> Vec<GoFile> {
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
        if ext != "go" {
            continue;
        }
        let rel = path
            .strip_prefix(repo_path)
            .unwrap_or(path)
            .to_string_lossy();

        // Skip vendor and testdata directories
        let rel_str = rel.as_ref();
        if rel_str.starts_with("vendor/")
            || rel_str.contains("/vendor/")
            || rel_str.starts_with("testdata/")
            || rel_str.contains("/testdata/")
        {
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

        let structs = parse_structs(&content);
        if !structs.is_empty() {
            files.push(GoFile {
                path: path.to_string_lossy().into_owned(),
                structs,
            });
        }
    }
    files
}

fn parse_structs(content: &str) -> Vec<GoStruct> {
    let lines: Vec<&str> = content.lines().collect();
    let mut structs = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Collect doc comments immediately before the struct declaration
        // We look backwards from the current line
        let mut doc_comments: Vec<String> = Vec::new();

        // Check if this line starts a struct declaration
        if let Some((name, type_params, body_start)) = try_parse_struct_header(trimmed) {
            // Gather doc comments by looking backwards from i
            let mut doc_idx = i;
            while doc_idx > 0 {
                doc_idx -= 1;
                let prev = lines[doc_idx].trim();
                if prev.starts_with("//") {
                    doc_comments.insert(0, prev.to_string());
                } else {
                    break;
                }
            }

            let struct_line = i + 1; // 1-based

            // Find the end of the struct body using brace counting
            let fields;
            let end_line;

            if body_start {
                // Brace is on the same line
                let (f, end) = extract_fields(&lines, i, struct_line);
                fields = f;
                end_line = end;
            } else {
                // Brace might be on next line
                if i + 1 < lines.len() && lines[i + 1].trim() == "{" {
                    i += 1;
                    let (f, end) = extract_fields(&lines, i, struct_line);
                    fields = f;
                    end_line = end;
                } else {
                    i += 1;
                    continue;
                }
            }

            structs.push(GoStruct {
                name,
                type_params,
                fields,
                doc_comments,
                line_number: struct_line,
            });
            i = end_line + 1;
        } else {
            i += 1;
        }
    }

    structs
}

/// Returns (name, type_params, brace_on_same_line) if the line is a struct header.
fn try_parse_struct_header(line: &str) -> Option<(String, Option<String>, bool)> {
    // Match: type Name[TypeParams] struct {
    // or:    type Name[TypeParams] struct
    let line = line.trim();
    if !line.starts_with("type ") {
        return None;
    }
    let rest = &line[5..];

    // Find "struct" keyword
    let struct_pos = rest.find(" struct")?;
    let name_part = &rest[..struct_pos];

    // Extract name and optional type params
    let (name, type_params) = if let Some(bracket_pos) = name_part.find('[') {
        let name = name_part[..bracket_pos].trim().to_string();
        // Find matching close bracket
        let after_bracket = &name_part[bracket_pos..];
        let mut depth = 0;
        let mut close = after_bracket.len();
        for (idx, ch) in after_bracket.char_indices() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        close = idx;
                        break;
                    }
                }
                _ => {}
            }
        }
        let tp = after_bracket[1..close].trim().to_string();
        (name, Some(tp))
    } else {
        (name_part.trim().to_string(), None)
    };

    if name.is_empty() || !name.chars().next().map_or(false, |c| c.is_uppercase()) {
        return None;
    }

    let after_struct = &rest[struct_pos + 7..]; // after " struct"
    let trimmed_after = after_struct.trim();
    let brace_on_same_line = trimmed_after.starts_with('{');

    Some((name, type_params, brace_on_same_line))
}

/// Extract fields starting at the line containing '{', returns (fields, end_line_index).
fn extract_fields(lines: &[&str], open_brace_line: usize, struct_line: usize) -> (Vec<GoField>, usize) {
    let mut fields = Vec::new();
    let mut depth = 0;
    let mut i = open_brace_line;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();

        for ch in trimmed.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }

        if depth == 0 {
            return (fields, i);
        }

        // Parse field if we're at depth 1 (inside the struct body, not nested)
        if depth == 1 && i != open_brace_line {
            if let Some(field) = parse_field(trimmed, raw, i + 1) {
                fields.push(field);
            }
        }

        i += 1;
    }

    (fields, i.saturating_sub(1))
}

fn parse_field(trimmed: &str, raw_line: &str, line_number: usize) -> Option<GoField> {
    // Skip blank lines, closing brace, and comments-only lines
    if trimmed.is_empty() || trimmed == "}" || trimmed.starts_with("//") {
        return None;
    }

    // Split off inline comment
    let (code_part, inline_comment) = split_inline_comment(trimmed);
    let code_part = code_part.trim();

    if code_part.is_empty() {
        return None;
    }

    // Split off struct tag (backtick-quoted)
    let (code_no_tag, tag) = split_tag(code_part);
    let code_no_tag = code_no_tag.trim();

    // Parse "Name Type" or embedded "Type"
    let tokens: Vec<&str> = code_no_tag.splitn(2, char::is_whitespace).collect();
    if tokens.is_empty() {
        return None;
    }

    let (name, type_str) = if tokens.len() == 1 {
        // Embedded field — name is the type (strip package prefix and pointer)
        let t = tokens[0].trim_start_matches('*');
        let short = t.rsplit('.').next().unwrap_or(t);
        (short.to_string(), tokens[0].to_string())
    } else {
        let n = tokens[0].to_string();
        let t = tokens[1..].join(" ").trim().to_string();
        if n.starts_with('(') || t.is_empty() {
            return None;
        }
        (n, t)
    };

    Some(GoField {
        name,
        type_str,
        tag,
        inline_comment,
        line_number,
        raw_line: raw_line.to_string(),
    })
}

/// Split `// comment` from the end of a code snippet (respects backtick strings).
fn split_inline_comment(s: &str) -> (&str, Option<String>) {
    let mut in_backtick = false;
    let mut in_double_quote = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '`' if !in_double_quote => in_backtick = !in_backtick,
            '"' if !in_backtick => in_double_quote = !in_double_quote,
            '/' if !in_backtick && !in_double_quote && i + 1 < chars.len() && chars[i + 1] == '/' => {
                let comment = s[i..].to_string();
                return (&s[..i], Some(comment));
            }
            _ => {}
        }
        i += 1;
    }
    (s, None)
}

/// Split backtick struct tag from the end of a field declaration.
fn split_tag(s: &str) -> (&str, Option<String>) {
    if let Some(start) = s.rfind('`') {
        if let Some(end) = s[..start].rfind('`') {
            if end < start {
                let tag = s[end..=start].to_string();
                return (&s[..end].trim_end(), Some(tag));
            }
        }
        // Only one backtick found — malformed
    }
    (s, None)
}
