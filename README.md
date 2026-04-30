# Lineup

Lineup is a desktop app that scans **Go** and **TypeScript** repositories and finds types that waste memory due to field padding. It shows you exactly which types can be improved, how many bytes can be saved, and what the optimized field order looks like — without touching your source files.

---

## What does it do?

### Go

Go aligns struct fields in memory according to their type's alignment requirements. When fields are ordered carelessly, the compiler inserts hidden padding bytes between them to satisfy alignment rules. Lineup detects this, computes the optimal field order, and tells you how much memory each struct is wasting.

### TypeScript

V8's JIT compiler stores `number` properties as unboxed 8-byte doubles, while all other property types are stored as 4-byte tagged compressed pointers (Node.js 14+, 64-bit). Interleaving `number` and non-`number` properties in a class, interface, or object type creates the same hidden padding waste as Go struct misalignment. Lineup models this layout, computes the optimal property order, and reports how many bytes per instance can be recovered.

Lineup analyzes `class` declarations, `interface` declarations, and object `type` aliases in `.ts` and `.tsx` files.

No changes are made to your code. Lineup only reads and reports.

---

## Screenshots

**Home screen** — open a repository or revisit a previous scan from your history.

![Home screen](screenshots/home.png)

**Scan results** — three-pane view: file tree on the left, struct list in the center, and field-level detail on the right.

![Scan results](screenshots/scan.png)

---

## Using the App

### 1. Open a repository

Click **Open Repository** on the home screen and select the root folder of any Go or TypeScript project. Lineup will walk the directory tree, skipping files excluded by `.gitignore` and language-specific noise directories.

### 2. Configure scan options

Before the scan starts you can set options:

| Option | Description |
|---|---|
| **Language** | `Go` (default) or `TypeScript`. Selects which files are scanned and which size model is used. |
| **Architecture** | `x86_64` (default) or `ARM64`. Only shown for Go scans. Selects the type-size table. |
| **Ignore patterns** | One regex per line. Any file whose path matches a pattern is skipped (e.g. `.*\.pb\.go$` to skip generated protobuf files). |

### 3. Read the results

The results screen has three panes:

- **Left — File tree.** Every file that contained at least one analyzed type. A badge shows the number of types with fixable padding. Click a file to filter the center list.
- **Center — Declaration list.** Each card shows the type name, its kind (`class`, `interface`, or `type` for TypeScript), current size, optimal size, and bytes that can be saved. Types with no padding waste show `0 B saveable`.
- **Right — Detail panel.** Select a type to see its current field order and the suggested optimized order side by side. A "Copy Optimized" button puts the reordered definition on your clipboard.

### 4. Understand the numbers

| Label | Meaning |
|---|---|
| **structs / declarations** | Total number of types found (`structs` for Go, `declarations` for TypeScript). |
| **padded** | How many of those types currently waste at least 1 byte to padding. |
| **B saveable** | Bytes per instance that would be recovered by reordering the fields shown. |
| **class / interface / type** badge | TypeScript declaration kind, shown next to the name in results. |
| **~approximate** badge | The type uses generics, or extends/implements another type; sizes are estimated conservatively. |
| **embedded** badge | The struct embeds another Go type; sizes account for the embedded layout. |

### 5. Act on the suggestions

Lineup never modifies your source files. To apply a suggestion:

1. Click **Copy Optimized** in the detail panel.
2. Open the source file and replace the type definition with the copied version.
3. For **Go**: verify the new order does not break anything that depends on field position — for example, `encoding/binary` reads, `unsafe.Offsetof` calls, or cgo-shared types.
4. For **TypeScript**: verify that no code depends on property declaration order (e.g. class field initializer execution order or serialization key order). Compiled TypeScript is unaffected — this optimization targets runtime V8 memory layout only.
5. Compile and test as normal.

### 6. Re-scan

Use the **Re-scan** button on the results screen to re-analyze the same repository (for example, after you've applied some fixes). Re-scan pre-fills the original architecture and ignore patterns so you can adjust them if needed. Each re-scan is saved as a separate history entry.

---

## Scan History

Every scan is saved locally. From the home screen you can:

- Click **View** on any history card to return to those results.
- Click the delete icon to remove a single scan record.
- Click **Clear All History** to remove all saved scans (a confirmation dialog will appear).

Scan data is stored in a SQLite database inside the app's local data directory. It never leaves your machine.

---

## Practical notes

- **Go:** Lineup targets the `amd64` (x86-64) memory model by default. Switch to `ARM64` in scan options if you are building for Apple Silicon or another 64-bit ARM target.
- **TypeScript:** The V8 model assumes 64-bit pointer compression (the default in Node.js 14+ and all modern Chromium-based runtimes). `number` is always modelled as an unboxed 8-byte Double (worst-case). If a property is always an integer, V8 may use a Smi (4 bytes) at runtime — the reported savings are a conservative upper bound.
- Not every suggested reordering is appropriate. Review the proposed order before applying it, especially for types that are serialized, passed over a network boundary, or shared with C code.
- Go structs that use type parameters (generics) are flagged as `~approximate` because their concrete field sizes depend on how the type is instantiated. TypeScript types that `extend` or `implement` other types are also flagged `~approximate` because parent properties affect the V8 hidden-class layout but are not resolved across files.

---

## License

MIT.

---

> For developer setup, build instructions, project architecture, and the full Tauri command API reference, see [DEVELOPER.md](DEVELOPER.md).

