# Lineup — Developer Reference

This document covers the technical details of Lineup: project architecture, development setup, build pipeline, code organization, and the full Tauri command API.

For user-facing documentation see [README.md](README.md).

---

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri v2](https://tauri.app) |
| Frontend | [SolidJS](https://solidjs.com) + [Vite](https://vitejs.dev) |
| UI components | [Kobalte](https://kobalte.dev) |
| Styling | [Tailwind CSS v4](https://tailwindcss.com) |
| Routing | [@solidjs/router](https://github.com/solidjs/solid-router) |
| Linting / formatting | [Biome](https://biomejs.dev) |
| Backend language | Rust (2021 edition) |
| Persistence | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) (bundled) |
| Directory walking | [ignore](https://crates.io/crates/ignore) |
| Regex | [regex](https://crates.io/crates/regex) |
| TypeScript parsing | [oxc_parser](https://crates.io/crates/oxc_parser) 0.128 (pure Rust) |

---

## Prerequisites

| Tool | Minimum version | Notes |
|---|---|---|
| Node.js | 18+ | |
| pnpm | 9+ | `npm install -g pnpm` |
| Rust toolchain | stable | Install via [rustup](https://rustup.rs) |
| Tauri system deps | — | macOS: Xcode Command Line Tools; Linux: `libwebkit2gtk`, `libayatana-appindicator`; Windows: WebView2 |

Verify your environment with:

```bash
pnpm tauri info
```

---

## Setup

```bash
# Clone the repository
git clone <repo-url>
cd lineup

# Install JS dependencies
pnpm install
```

Rust dependencies are resolved automatically by Cargo the first time you build or run.

---

## Development Workflow

### Run the desktop app (recommended)

```bash
pnpm tauri:dev
```

This starts the Vite dev server and the Tauri window simultaneously. Hot-reload applies to frontend changes; Rust changes trigger a recompile.

### Run the web frontend only

```bash
pnpm dev
```

Opens the frontend at `http://localhost:1420`. Tauri backend commands are unavailable in this mode, but it is useful for rapid UI iteration.

---

## NPM Scripts

| Script | Command | Description |
|---|---|---|
| `dev` | `vite` | Start Vite dev server (frontend only) |
| `build` | `vite build` | Build frontend for production |
| `serve` | `vite preview` | Preview the production build locally |
| `tauri:dev` | `tauri dev` | Run desktop app in dev mode |
| `tauri:build` | `tauri build` | Build a production desktop bundle |
| `tauri:bundle` | `tauri bundle` | Bundle without compiling Rust (rarely needed) |
| `tauri:info` | `tauri info` | Print environment and dependency diagnostics |
| `tauri:signer` | `tauri signer` | Manage update signing keys |
| `tauri:permissions` | `tauri permission` | Manage Tauri permission files |
| `tauri:capabilities` | `tauri capabilities` | Manage Tauri capability files |
| `lint` | `biome check .` | Run Biome linter |
| `lint:fix` | `biome check --write .` | Run Biome linter and auto-fix |
| `format` | `biome format --write .` | Run Biome formatter |

---

## Building for Production

```bash
pnpm tauri:build
```

```bash
pnpm tauri build --bundles dmg -t aarch64-apple-darwin -v
```

> Bundle for distribution

The compiled installer and binary are placed in `src-tauri/target/release/bundle/`. The exact format depends on the host OS: `.dmg` / `.app` on macOS, `.msi` / `.exe` on Windows, `.deb` / `.rpm` / `.AppImage` on Linux.

The `tauri.conf.json` `bundle.targets` field is set to `"all"` so all applicable formats are produced per platform.

---

## Project Structure

```
lineup/
├── src/                        # SolidJS frontend
│   ├── index.tsx               # App entry point
│   ├── App.tsx                 # Router setup
│   ├── types.ts                # Shared TypeScript types
│   ├── index.css               # Global styles (Tailwind base)
│   ├── components/             # Reusable UI components
│   │   ├── AlertDialog.tsx     # Confirmation dialog (Kobalte)
│   │   ├── FileTree.tsx        # Left-pane file tree on results screen
│   │   ├── HistoryCard.tsx     # Scan history card on home screen
│   │   ├── ProgressModal.tsx   # Scan progress overlay with cancel
│   │   ├── ScanOptionsModal.tsx# Pre-scan config (language, arch, ignore patterns)
│   │   ├── StructCard.tsx      # Per-declaration summary card
│   │   └── StructDetail.tsx    # Right-pane detail view with copy button
│   ├── pages/
│   │   ├── Home.tsx            # Home / history screen
│   │   ├── ScanResults.tsx     # Three-pane results screen
│   │   └── Settings.tsx        # App settings page
│   ├── shared/
│   │   ├── Footer.tsx
│   │   └── Layout.tsx
│   └── store/
│       └── settings.ts         # Persisted app settings (SolidJS store)
├── src-tauri/                  # Tauri + Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json         # Tauri configuration
│   ├── build.rs
│   ├── capabilities/
│   │   └── default.json        # Tauri capability grants (dialog:open etc.)
│   └── src/
│       ├── main.rs             # Binary entry point
│       ├── lib.rs              # Tauri commands, managed state, app setup
│       ├── db.rs               # SQLite schema, queries, data types
│       ├── parser.rs           # Go source walker and struct parser
│       ├── ts_parser.rs        # TypeScript source walker and declaration parser
│       └── analyzer.rs         # Padding analyzer and optimal-order engine (Go + TS)
├── public/                     # Static assets
├── biome.json                  # Biome linter / formatter config
├── vite.config.ts
├── tsconfig.json
└── package.json
```

---

## Rust Modules

### `lib.rs`

Defines `AppState` (the Tauri managed state), all `#[tauri::command]` handlers, and the `tauri::Builder` setup. `AppState` holds a `Mutex<rusqlite::Connection>` and an `Arc<AtomicBool>` cancel flag.

`ScanOptions` carries a `language` field (`"GO"` or `"TS"`) that controls which walker and analyzer are invoked by `scan_repo`.

### `db.rs`

All database logic. Initializes the schema on first run and exposes typed functions for every persistence operation. The SQLite file is stored in the platform app-data directory resolved by `tauri::Manager::path().app_data_dir()`.

**Schema — `scans` table**

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | Auto-increment |
| `repo_path` | TEXT | Absolute path to the scanned directory |
| `scanned_at` | INTEGER | Unix timestamp (seconds) |
| `total_structs` | INTEGER | |
| `padded_structs` | INTEGER | Declarations with bytes_saved > 0 |
| `bytes_saved` | INTEGER | Sum across all declarations |
| `ignore_patterns` | TEXT | JSON array of regex strings |
| `target_arch` | TEXT | `"amd64"` or `"arm64"` |
| `language` | TEXT | `"GO"` or `"TS"` (added in 1.1.0; defaults to `"GO"` for existing rows) |

**Schema — `struct_results` table**

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | Auto-increment |
| `scan_id` | INTEGER FK | References `scans.id` (cascade delete) |
| `file_path` | TEXT | Repo-relative file path |
| `struct_name` | TEXT | |
| `line_number` | INTEGER | Line of the `type Name struct` declaration |
| `current_size` | INTEGER | Size in bytes of the current field order |
| `optimal_size` | INTEGER | Size in bytes of the optimal field order |
| `bytes_saved` | INTEGER | `current_size - optimal_size` |
| `current_def` | TEXT | Full struct definition as it appears in source |
| `optimized_def` | TEXT | Reordered definition with added comment header |
| `has_generics` | INTEGER | Boolean (0/1); sizes are approximate |
| `has_embedded` | INTEGER | Boolean (0/1) |
| `declaration_kind` | TEXT | `"struct"` for Go; `"class"`, `"interface"`, or `"type"` for TypeScript (added in 1.1.0; defaults to `"struct"` for existing rows) |

### `parser.rs`

Walks a **Go** repository using `ignore::Walk` — automatically respects `.gitignore` and skips `vendor/` and `testdata/`. User-supplied ignore patterns (regex strings) are compiled into a `RegexSet` before the walk begins and tested against each file's repo-relative path. Uses brace-counting (not pure regex) to reliably extract `type Name[TypeParams] struct { ... }` blocks including generic structs. Captures doc comments (`//`-prefixed lines immediately preceding `type Name struct`) and preserves inline field comments and struct tags.

### `ts_parser.rs`

Walks a **TypeScript** repository using the same `ignore::Walk` infrastructure. Hard-skips `node_modules/`, `dist/`, `build/`, `.next/`, `.nuxt/`, `coverage/`, and `.cache/` in addition to `.gitignore` exclusions. Accepts `.ts` and `.tsx` files.

Parsing is done with [`oxc_parser`](https://crates.io/crates/oxc_parser) (pure Rust, no C build step). The parser produces a typed AST from which the following top-level constructs are extracted:

- `class` declarations — instance `PropertyDefinition` nodes (static properties are skipped)
- `interface` declarations — `TSPropertySignature` nodes
- object `type` aliases (`type Foo = { ... }`) — `TSPropertySignature` nodes inside `TSTypeLiteral`

Exported and non-exported variants, including `export default class`, are all handled. For each property the raw source line is captured via `span` byte offsets for use in reconstructed definitions.

### `analyzer.rs`

Contains two independent analysis pipelines that share the same core layout primitives (`align_up`, `struct_layout`, optimal-sort by align desc / size desc / name asc).

**Go pipeline (`analyze_files`)**

Two-pass analysis:

1. **Pass 1** — parse all files and build a `HashMap<String, TypeInfo>` registry mapping type names to their computed size and alignment.
2. **Pass 2** — for each struct, resolve embedded types recursively from the registry, compute current size (with padding), compute optimal size, and generate the `optimized_def` string with a `// Reordered for optimal memory alignment` header.

The `Arch` enum (`Amd64` | `Arm64`) selects the type-size/alignment table. Generic type parameters and unresolved types default to size 8 / align 8 and set `approximate = true`.

**TypeScript pipeline (`analyze_ts_files`)**

Single-pass analysis using the V8 type-size model:

| TypeScript type | V8 representation | Size | Align |
|---|---|---|---|
| `number` | Unboxed Double | 8 B | 8 |
| `any`, `unknown`, `never`, `void` | Tagged pointer (conservative) | 4 B | 4 |
| Everything else | Tagged/compressed pointer | 4 B | 4 |

Types with generics (`has_generics`) or that extend/implement other types (`has_embedded`) are flagged `approximate = true`. The `declaration_kind` field on `AnalyzedStruct` is set to `"class"`, `"interface"`, or `"type"`. The `optimized_def` header is `// Reordered for optimal V8 memory layout`.

**Go type table (amd64 / arm64)**

| Go type(s) | Size (bytes) | Align (bytes) |
|---|---|---|
| `bool`, `byte`, `int8`, `uint8` | 1 | 1 |
| `int16`, `uint16` | 2 | 2 |
| `int32`, `uint32`, `float32`, `rune` | 4 | 4 |
| `int64`, `uint64`, `float64`, `int`, `uint`, `uintptr` | 8 | 8 |
| `*T`, `map[…]…`, `chan …`, `func(…)` | 8 | 8 |
| `string` | 16 | 8 |
| `interface{}` / `any` | 16 | 8 |
| `[]T` | 24 | 8 |
| `[N]T` | N × sizeof(T) | alignof(T) |
| Generic `T` / unresolved | 8 (conservative) | 8 |

---

## Data & Log Locations

The bundle identifier is **`com.capabletechnology.lineup`**.

### SQLite database

The database file is created on first launch at `{app_data_dir}/lineup.db`.

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.capabletechnology.lineup/lineup.db` |
| Linux | `$XDG_DATA_HOME/com.capabletechnology.lineup/lineup.db` (typically `~/.local/share/com.capabletechnology.lineup/lineup.db`) |
| Windows | `%APPDATA%\com.capabletechnology.lineup\lineup.db` (e.g. `C:\Users\<user>\AppData\Roaming\com.capabletechnology.lineup\lineup.db`) |

Deleting the file resets all scan history. The app will re-create the schema automatically on next launch.

### Logs

Lineup does not use `tauri-plugin-log` — no log files are written to disk.

- **Development** (`pnpm tauri:dev`): Rust `println!`/`eprintln!` output appears in the terminal that launched the dev command. Frontend `console.*` output appears in the WebView DevTools (right-click → Inspect, or `Ctrl+Shift+I` / `Cmd+Option+I`).
- **Production**: stdout/stderr is captured by the host OS. Use the platform system log viewer to inspect it:

| Platform | How to view |
|---|---|
| macOS | **Console.app** → filter by `com.capabletechnology.lineup`, or `log stream --predicate 'process == "Lineup"'` in Terminal |
| Linux | `journalctl -f` (if launched via systemd) or redirect stdout/stderr when launching from the command line |
| Windows | **Event Viewer** → Windows Logs → Application, or launch from a terminal to capture stdout/stderr directly | via `@tauri-apps/api/core` `invoke()`. Errors are returned as rejected promises (the Rust `Err(String)` maps to a JS string rejection).

---

### `open_folder_dialog`

Opens the native folder picker dialog.

```ts
invoke<string | null>('open_folder_dialog')
```

**Returns** — the absolute path of the selected folder, or `null` if the user cancelled.

---

### `scan_repo`

Walks and analyzes a Go repository. Emits `scan-progress` events while running.

```ts
invoke<ScanSummary>('scan_repo', {
  repo_path: string,   // absolute path returned by open_folder_dialog
  opts: ScanOptions,
})
```

**`ScanOptions`**

```ts
interface ScanOptions {
  ignore_patterns: string[];  // regex strings; matched against repo-relative file paths
  target_arch: string;        // "amd64" | "arm64" (used only when language is "go")
  language: string;           // "GO" | "TS"
}
```

**Returns** — `ScanSummary` for the completed scan.

**Emits** — `scan-progress` events of the shape:

```ts
interface ScanProgressEvent {
  stage: string;  // "walking" | "analyzing" | "done"
  file: string;   // current file path (empty for "walking" and "done")
  pct: number;    // 0–100
}
```

**Errors** — rejects with `"cancelled"` if `cancel_scan` was called during the scan.

---

### `cancel_scan`

Signals a running `scan_repo` to abort. The in-progress scan record is deleted from the database before the command returns an error.

```ts
invoke<void>('cancel_scan')
```

---

### `get_history`

Returns all scan records, newest first.

```ts
invoke<ScanSummary[]>('get_history')
```

**`ScanSummary`**

```ts
interface ScanSummary {
  id: number;
  repo_path: string;
  scanned_at: number;       // Unix timestamp (seconds)
  total_structs: number;
  padded_structs: number;
  bytes_saved: number;
  ignore_patterns: string[];
  target_arch: string;      // "amd64" | "arm64"
  language: string;         // "GO" | "TS"
}
```

---

### `delete_scan`

Deletes a single scan record and its associated struct results (cascade).

```ts
invoke<void>('delete_scan', { scan_id: number })
```

---

### `clear_history`

Deletes all scan records.

```ts
invoke<void>('clear_history')
```

---

### `get_scan_detail`

Returns all struct results for a scan, grouped by file.

```ts
invoke<FileScanResult[]>('get_scan_detail', { scan_id: number })
```

**`FileScanResult`**

```ts
interface FileScanResult {
  file_path: string;
  structs: StructSummary[];
}

interface StructSummary {
  id: number;
  struct_name: string;
  line_number: number;
  current_size: number;
  optimal_size: number;
  bytes_saved: number;
  has_generics: boolean;
  has_embedded: boolean;
  declaration_kind: string;  // "struct" | "class" | "interface" | "type"
}
```

---

### `get_struct_detail`

Returns the full detail for a single struct, including both source and optimized definitions.

```ts
invoke<StructDetail>('get_struct_detail', { struct_id: number })
```

**`StructDetail`**

```ts
interface StructDetail {
  id: number;
  scan_id: number;
  file_path: string;
  struct_name: string;
  line_number: number;
  current_size: number;
  optimal_size: number;
  bytes_saved: number;
  current_def: string;    // original source
  optimized_def: string;  // reordered source with comment header
  has_generics: boolean;
  has_embedded: boolean;
  declaration_kind: string;  // "struct" | "class" | "interface" | "type"
}
```

---

## Frontend Routes

| Path | Component | Description |
|---|---|---|
| `/` | `Home` | Scan history and "Open Repository" entry point |
| `/scan/:scanId` | `ScanResults` | Three-pane results view |

---

## Linting and Formatting

Biome is used for both linting and formatting. Configuration is in [biome.json](biome.json).

```bash
# Check everything
pnpm lint

# Auto-fix lint issues
pnpm lint:fix

# Format all files
pnpm format
```

Run `pnpm format` and `pnpm lint` before submitting a pull request.

---

## Contributing

1. Fork the repository and create a feature branch.
2. Make your changes. Keep commits focused and atomic.
3. Run `pnpm format` and `pnpm lint:fix` to ensure consistent style.
4. Test with `pnpm tauri:dev` and verify that the Rust backend compiles cleanly.
5. Open a pull request with a clear description of what changed and why.

---

## Troubleshooting

**`tauri:dev` or `tauri:build` fails at the Rust compile step**
- Confirm the Rust stable toolchain is installed: `rustup show`
- On macOS, ensure Xcode Command Line Tools are installed: `xcode-select --install`
- On Linux, install the required system libraries listed in the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

**Vite dev server fails to start**
- Check Node.js version: `node -v` (18+)
- Remove `node_modules` and re-run `pnpm install`

**`pnpm tauri info` shows missing tools**
- Follow the output's suggestions to install the missing dependencies before re-running any build

**Database errors on startup**
- The SQLite file lives at the path shown in the [Data & Log Locations](#data--log-locations) section above. Delete `lineup.db` in that directory to reset state (all scan history will be lost).
