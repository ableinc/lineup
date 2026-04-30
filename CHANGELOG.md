# Changelog

All notable changes to Lineup are documented here.

---

## [1.1.0] — 2026-04-30

### Added

#### TypeScript support
- Lineup can now scan TypeScript projects (`.ts` and `.tsx` files) for V8 JIT memory layout inefficiencies. V8 stores `number` properties as unboxed 8-byte doubles while all other property types are stored as 4-byte tagged compressed pointers (Node.js 14+, 64-bit). Interleaving `number` and non-`number` properties creates the same hidden padding waste as Go struct misalignment.
- Analyzes three TypeScript constructs: `class` declarations (instance properties only; static properties are skipped), `interface` declarations, and object `type` aliases (`type Foo = { ... }`).
- Supports both exported and non-exported declarations, including `export default class`.
- New `ts_parser.rs` Rust module: walks `.ts`/`.tsx` files using `oxc_parser` 0.128 (pure Rust, no C build step). Auto-skips `node_modules/`, `dist/`, `build/`, `.next/`, `.nuxt/`, `coverage/`, and `.cache/` in addition to `.gitignore` exclusions.
- New TypeScript analysis pipeline in `analyzer.rs`: `ts_type_info()` V8 size model, `analyze_ts_files()`, and `build_ts_def()`. Shares the same `align_up`, `struct_layout`, and optimal-sort primitives as the Go pipeline.
- TypeScript declarations that `extend` or `implement` other types are flagged `~approximate` because parent properties affect V8 hidden-class layout but are not resolved across files.

#### Language selector
- New **Language** toggle (`Go` / `TypeScript`) in the Scan Options modal. Arch selector is hidden when TypeScript is selected (V8 always uses 64-bit pointer compression).
- New **Default Language** setting in the Settings page (`go` by default).
- `ScanOptions` now carries a `language` field passed to the Tauri `scan_repo` command.

#### Declaration kind
- `declaration_kind` field added to all result types: `"struct"` for Go, `"class"` / `"interface"` / `"type"` for TypeScript.
- Kind badge shown next to the type name in `StructCard` and `StructDetail` for non-Go results.
- Toolbar on the Scan Results screen shows `"declarations"` instead of `"structs"` for TypeScript scans.

### Changed

- **Database schema** — two additive `ALTER TABLE` migrations applied at startup:
  - `scans.language TEXT NOT NULL DEFAULT 'go'`
  - `struct_results.declaration_kind TEXT NOT NULL DEFAULT 'struct'`
  - Both migrations are no-ops on existing databases (duplicate-column errors are silently ignored).
- `save_scan` and `save_struct_result` in `db.rs` now accept and store `language` / `declaration_kind` respectively.
- `get_history`, `get_scan_detail`, and `get_struct_detail` queries updated to select and map the new columns.
- `ScanSummary`, `StructSummary`, and `StructDetail` Rust and TypeScript types updated with new fields.
- `AppSettings` TypeScript interface gains `defaultLanguage: string`.
- Re-scan pre-fills the `language` of the original scan (same as it does for arch and ignore patterns).

### Dependencies

- Added `oxc_allocator`, `oxc_parser`, `oxc_ast`, `oxc_span` at version `0.128.0` (pure Rust TypeScript/JSX parser).

---

## [1.0.0] — initial release

- Go struct padding analysis for `amd64` and `arm64` targets.
- Three-pane results UI (file tree / declaration list / detail panel).
- Scan history persisted to local SQLite database.
- Re-scan with pre-filled options.
- Configurable ignore patterns (regex).
- Copy optimized definition to clipboard.
- Dark / light theme.
