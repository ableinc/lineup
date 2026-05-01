use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

mod analyzer;
mod db;
mod parsers;

use analyzer::Arch;
use db::{FileScanResult, ScanSummary, StructDetail};
use parsers::{go_parser, ts_parser};

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub cancel_flag: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanOptions {
    pub ignore_patterns: Vec<String>,
    pub target_arch: String,
    /// "go" or "typescript"
    pub language: String,
}

#[derive(Debug, Serialize, Clone)]
struct ScanProgress {
    stage: String,
    file: String,
    pct: u8,
}

#[tauri::command]
async fn open_folder_dialog(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string())
}

#[tauri::command]
async fn scan_repo(
    app: AppHandle,
    state: State<'_, AppState>,
    repo_path: String,
    opts: ScanOptions,
) -> Result<ScanSummary, String> {
    // Reset cancel flag
    state.cancel_flag.store(false, Ordering::SeqCst);
    let cancel = state.cancel_flag.clone();

    let arch: Arch = opts.target_arch.parse().unwrap_or(Arch::Amd64);
    let ignore_patterns = opts.ignore_patterns.clone();
    let language = opts.language.clone().to_uppercase();
    let is_typescript = language == "TS";

    let _ = app.emit(
        "scan-progress",
        ScanProgress {
            stage: "walking".to_string(),
            file: String::new(),
            pct: 0,
        },
    );

    let path = Path::new(&repo_path).to_path_buf();
    let patterns_clone = ignore_patterns.clone();

    let files_go;
    let files_ts;
    let total_files;

    let analyzed = if is_typescript {
        files_ts = ts_parser::walk_ts_repo(&path, &patterns_clone);
        total_files = files_ts.len().max(1);
        analyzer::analyze_ts_files(&files_ts)
    } else {
        files_go = go_parser::walk_repo(&path, &patterns_clone);
        total_files = files_go.len().max(1);
        analyzer::analyze_files(&files_go, arch)
    };
    let mut total_structs: i64 = 0;
    let mut padded_structs: i64 = 0;
    let mut total_bytes_saved: i64 = 0;

    let scanned_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    let scan_id = db::save_scan(
        &conn,
        &repo_path,
        scanned_at,
        0,
        0,
        0,
        &ignore_patterns,
        arch.as_str(),
        &language,
    )
    .map_err(|e| e.to_string())?;

    for (idx, (file_path, s)) in analyzed.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            let _ = db::delete_scan(&conn, scan_id);
            return Err("cancelled".to_string());
        }

        let pct = ((idx as f64 / total_files as f64) * 100.0) as u8;
        let _ = app.emit(
            "scan-progress",
            ScanProgress {
                stage: "analyzing".to_string(),
                file: file_path.clone(),
                pct,
            },
        );

        total_structs += 1;
        if s.bytes_saved > 0 {
            padded_structs += 1;
            total_bytes_saved += s.bytes_saved as i64;
        }

        db::save_struct_result(
            &conn,
            scan_id,
            file_path,
            &s.name,
            s.line_number as i64,
            s.current_size as i64,
            s.optimal_size as i64,
            s.bytes_saved as i64,
            &s.current_def,
            &s.optimized_def,
            s.has_generics,
            s.has_embedded,
            &s.declaration_kind,
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute(
        "UPDATE scans SET total_structs=?1, padded_structs=?2, bytes_saved=?3 WHERE id=?4",
        rusqlite::params![total_structs, padded_structs, total_bytes_saved, scan_id],
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "scan-progress",
        ScanProgress {
            stage: "done".to_string(),
            file: String::new(),
            pct: 100,
        },
    );

    Ok(ScanSummary {
        id: scan_id,
        repo_path,
        scanned_at,
        total_structs,
        padded_structs,
        bytes_saved: total_bytes_saved,
        ignore_patterns,
        target_arch: arch.as_str().to_string(),
        language,
    })
}

#[tauri::command]
fn cancel_scan(state: State<'_, AppState>) {
    state.cancel_flag.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Result<Vec<ScanSummary>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_history(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_scan(state: State<'_, AppState>, scan_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_scan(&conn, scan_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::clear_history(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scan_detail(state: State<'_, AppState>, scan_id: i64) -> Result<Vec<FileScanResult>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_scan_detail(&conn, scan_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_struct_detail(state: State<'_, AppState>, struct_id: i64) -> Result<StructDetail, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_struct_detail(&conn, struct_id).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("lineup.db");
            let conn = Connection::open(&db_path).expect("failed to open database");
            db::init_db(&conn).expect("failed to init database");
            let state = AppState {
                conn: Mutex::new(conn),
                cancel_flag: Arc::new(AtomicBool::new(false)),
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_folder_dialog,
            scan_repo,
            cancel_scan,
            get_history,
            delete_scan,
            clear_history,
            get_scan_detail,
            get_struct_detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
