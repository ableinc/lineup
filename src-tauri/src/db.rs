use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanSummary {
    pub id: i64,
    pub repo_path: String,
    pub scanned_at: i64,
    pub total_structs: i64,
    pub padded_structs: i64,
    pub bytes_saved: i64,
    pub ignore_patterns: Vec<String>,
    pub target_arch: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileScanResult {
    pub file_path: String,
    pub structs: Vec<StructSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StructSummary {
    pub id: i64,
    pub struct_name: String,
    pub line_number: i64,
    pub current_size: i64,
    pub optimal_size: i64,
    pub bytes_saved: i64,
    pub has_generics: bool,
    pub has_embedded: bool,
    pub declaration_kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StructDetail {
    pub id: i64,
    pub scan_id: i64,
    pub file_path: String,
    pub struct_name: String,
    pub line_number: i64,
    pub current_size: i64,
    pub optimal_size: i64,
    pub bytes_saved: i64,
    pub current_def: String,
    pub optimized_def: String,
    pub has_generics: bool,
    pub has_embedded: bool,
    pub declaration_kind: String,
}

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;

         CREATE TABLE IF NOT EXISTS scans (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             repo_path TEXT NOT NULL,
             scanned_at INTEGER NOT NULL,
             total_structs INTEGER NOT NULL DEFAULT 0,
             padded_structs INTEGER NOT NULL DEFAULT 0,
             bytes_saved INTEGER NOT NULL DEFAULT 0,
             ignore_patterns TEXT NOT NULL DEFAULT '[]',
             target_arch TEXT NOT NULL DEFAULT 'amd64'
         );

         CREATE TABLE IF NOT EXISTS struct_results (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             scan_id INTEGER NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
             file_path TEXT NOT NULL,
             struct_name TEXT NOT NULL,
             line_number INTEGER NOT NULL DEFAULT 0,
             current_size INTEGER NOT NULL DEFAULT 0,
             optimal_size INTEGER NOT NULL DEFAULT 0,
             bytes_saved INTEGER NOT NULL DEFAULT 0,
             current_def TEXT NOT NULL DEFAULT '',
             optimized_def TEXT NOT NULL DEFAULT '',
             has_generics INTEGER NOT NULL DEFAULT 0,
             has_embedded INTEGER NOT NULL DEFAULT 0
         );",
    )?;
    // Additive migrations — silently ignored on existing databases
    let _ = conn.execute("ALTER TABLE scans ADD COLUMN language TEXT NOT NULL DEFAULT 'GO'", []);
    let _ = conn.execute("ALTER TABLE struct_results ADD COLUMN declaration_kind TEXT NOT NULL DEFAULT 'struct'", []);
    Ok(())
}

pub fn save_scan(
    conn: &Connection,
    repo_path: &str,
    scanned_at: i64,
    total_structs: i64,
    padded_structs: i64,
    bytes_saved: i64,
    ignore_patterns: &[String],
    target_arch: &str,
    language: &str,
) -> Result<i64> {
    let patterns_json = serde_json::to_string(ignore_patterns).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO scans (repo_path, scanned_at, total_structs, padded_structs, bytes_saved, ignore_patterns, target_arch, language)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![repo_path, scanned_at, total_structs, padded_structs, bytes_saved, patterns_json, target_arch, language.to_uppercase()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn save_struct_result(
    conn: &Connection,
    scan_id: i64,
    file_path: &str,
    struct_name: &str,
    line_number: i64,
    current_size: i64,
    optimal_size: i64,
    bytes_saved: i64,
    current_def: &str,
    optimized_def: &str,
    has_generics: bool,
    has_embedded: bool,
    declaration_kind: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO struct_results
         (scan_id, file_path, struct_name, line_number, current_size, optimal_size, bytes_saved, current_def, optimized_def, has_generics, has_embedded, declaration_kind)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            scan_id, file_path, struct_name, line_number,
            current_size, optimal_size, bytes_saved,
            current_def, optimized_def,
            has_generics as i64, has_embedded as i64,
            declaration_kind
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_history(conn: &Connection) -> Result<Vec<ScanSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, repo_path, scanned_at, total_structs, padded_structs, bytes_saved, ignore_patterns, target_arch, language
         FROM scans ORDER BY scanned_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let patterns_json: String = row.get(6)?;
        let ignore_patterns: Vec<String> =
            serde_json::from_str(&patterns_json).unwrap_or_default();
        Ok(ScanSummary {
            id: row.get(0)?,
            repo_path: row.get(1)?,
            scanned_at: row.get(2)?,
            total_structs: row.get(3)?,
            padded_structs: row.get(4)?,
            bytes_saved: row.get(5)?,
            ignore_patterns,
            target_arch: row.get(7)?,
            language: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "GO".to_string().to_uppercase()),
        })
    })?;
    rows.collect()
}

pub fn get_scan_detail(conn: &Connection, scan_id: i64) -> Result<Vec<FileScanResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, struct_name, line_number, current_size, optimal_size, bytes_saved, has_generics, has_embedded, declaration_kind
         FROM struct_results WHERE scan_id = ?1 ORDER BY file_path, line_number",
    )?;
    let mut file_map: std::collections::BTreeMap<String, Vec<StructSummary>> =
        std::collections::BTreeMap::new();
    let rows = stmt.query_map([scan_id], |row| {
        Ok((
            row.get::<_, String>(1)?,
            StructSummary {
                id: row.get(0)?,
                struct_name: row.get(2)?,
                line_number: row.get(3)?,
                current_size: row.get(4)?,
                optimal_size: row.get(5)?,
                bytes_saved: row.get(6)?,
                has_generics: row.get::<_, i64>(7)? != 0,
                has_embedded: row.get::<_, i64>(8)? != 0,
                declaration_kind: row.get::<_, Option<String>>(9)?.unwrap_or_else(|| "struct".to_string()),
            },
        ))
    })?;
    for row in rows {
        let (file_path, s) = row?;
        file_map.entry(file_path).or_default().push(s);
    }
    Ok(file_map
        .into_iter()
        .map(|(file_path, structs)| FileScanResult { file_path, structs })
        .collect())
}

pub fn get_struct_detail(conn: &Connection, struct_id: i64) -> Result<StructDetail> {
    conn.query_row(
        "SELECT id, scan_id, file_path, struct_name, line_number, current_size, optimal_size, bytes_saved, current_def, optimized_def, has_generics, has_embedded, declaration_kind
         FROM struct_results WHERE id = ?1",
        [struct_id],
        |row| {
            Ok(StructDetail {
                id: row.get(0)?,
                scan_id: row.get(1)?,
                file_path: row.get(2)?,
                struct_name: row.get(3)?,
                line_number: row.get(4)?,
                current_size: row.get(5)?,
                optimal_size: row.get(6)?,
                bytes_saved: row.get(7)?,
                current_def: row.get(8)?,
                optimized_def: row.get(9)?,
                has_generics: row.get::<_, i64>(10)? != 0,
                has_embedded: row.get::<_, i64>(11)? != 0,
                declaration_kind: row.get::<_, Option<String>>(12)?.unwrap_or_else(|| "struct".to_string()),
            })
        },
    )
}

pub fn delete_scan(conn: &Connection, scan_id: i64) -> Result<()> {
    conn.execute("DELETE FROM scans WHERE id = ?1", [scan_id])?;
    Ok(())
}

pub fn clear_history(conn: &Connection) -> Result<()> {
    conn.execute_batch("DELETE FROM scans;")?;
    Ok(())
}
