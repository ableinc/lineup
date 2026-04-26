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
│   │   ├── ScanOptionsModal.tsx# Pre-scan config (arch, ignore patterns)
│   │   ├── StructCard.tsx      # Per-struct summary card
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
│       └── analyzer.rs         # Padding analyzer and optimal-order engine
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

### `db.rs`

All database logic. Initializes the schema on first run and exposes typed functions for every persistence operation. The SQLite file is stored in the platform app-data directory resolved by `tauri::Manager::path().app_data_dir()`.

**Schema — `scans` table**

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | Auto-increment |
| `repo_path` | TEXT | Absolute path to the scanned directory |
| `scanned_at` | INTEGER | Unix timestamp (seconds) |
| `total_structs` | INTEGER | |
| `padded_structs` | INTEGER | Structs with bytes_saved > 0 |
| `bytes_saved` | INTEGER | Sum across all structs |
| `ignore_patterns` | TEXT | JSON array of regex strings |
| `target_arch` | TEXT | `"amd64"` or `"arm64"` |

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

### `parser.rs`

Walks a repository using `ignore::Walk` — automatically respects `.gitignore` and skips `vendor/`. User-supplied ignore patterns (regex strings) are compiled into a `RegexSet` before the walk begins and tested against each file's repo-relative path. Uses brace-counting (not pure regex) to reliably extract `type Name[TypeParams] struct { ... }` blocks including generic structs. Captures doc comments (`//`-prefixed lines immediately preceding `type Name struct`) and preserves inline field comments and struct tags.

### `analyzer.rs`

Two-pass analysis:

1. **Pass 1** — parse all files and build a `HashMap<String, StructInfo>` registry mapping type names to their computed size and alignment.
2. **Pass 2** — for each struct, resolve embedded types recursively from the registry, compute current size (with padding), compute optimal size (fields sorted by alignment descending, ties broken by size then name), and generate the `optimized_def` string.

The `Arch` enum (`Amd64` | `Arm64`) selects the type-size/alignment table. Current tables are identical for both 64-bit targets. Generic type parameters and unresolved types default to size 8 / align 8 and are flagged `has_generics = true`.

**Type table (amd64 / arm64)**

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

## Tauri Command API

All commands are invoked from the frontend via `@tauri-apps/api/core` `invoke()`. Errors are returned as rejected promises (the Rust `Err(String)` maps to a JS string rejection).

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
  target_arch: string;        // "amd64" | "arm64"
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
  current_def: string;    // original struct source
  optimized_def: string;  // reordered struct with "// Reordered for optimal memory alignment" header
  has_generics: boolean;
  has_embedded: boolean;
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
- The SQLite file lives in the platform app-data directory. Delete `lineup.db` in that directory to reset state (all scan history will be lost).
