pub mod go_parser;
pub mod ts_parser;

/// Directories to always skip
const SKIP_DIRS: &[&str] = &[
    "vendor",
    "testdata",
    "node_modules",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "coverage",
    ".cache",
    ".github",
    ".git",
    "docs",
    "public",
    ".vscode",
    "assets",
    "configs",
    "scripts",
    "testdata",
    ".bin",
    ".local",
];