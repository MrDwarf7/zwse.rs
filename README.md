<!-- PROJECT LOGO / BANNER -->
<p align="center">
  <img src="assets/README-header.png" alt="zwse" width="100%">
</p>

<p align="center">
  <img src="assets/icon-128.png" alt="zwse icon" width="64">
</p>

<p align="center">
  <strong>zwse.rs</strong> -- Export Zen Browser tabs and workspaces
  <br>
  <a href="https://crates.io/crates/zwse"><img src="https://img.shields.io/crates/v/zwse" alt="crates.io"></a>
  <a href="https://github.com/MrDwarf7/zwse.rs/actions/workflows/build.yml"><img src="https://github.com/MrDwarf7/zwse.rs/actions/workflows/build.yml/badge.svg" alt="build"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT-blue" alt="license"></a>
  <a href="https://github.com/MrDwarf7/zwse.rs/releases"><img src="https://img.shields.io/github/v/release/MrDwarf7/zwse.rs" alt="release"></a>
</p>

# zwse.rs

Export tabs and workspaces from a Zen Browser sessionstore.jsonlz4

zwse : Session exporter for Zen Browser

A CLI tool that reads Firefox/Zen Browser session files (sessionstore.jsonlz4), decompresses them with lz4_flex, and exports tab/workspace data as a pipe-delimited table. Supports automatic Zen profile discovery across Linux, macOS, and Windows, or explicit session file paths for CI/fixtures.

## Installation

### Quick install (Linux/macOS)

```sh
curl -fsSL https://github.com/MrDwarf7/zwse.rs/raw/main/build/install.sh | sh
```

### Quick install (Windows PowerShell)

```powershell
iwr https://github.com/MrDwarf7/zwse.rs/raw/main/build/install.ps1 | iex
```

### Cargo install

```sh
cargo install --git https://github.com/MrDwarf7/zwse.rs
```

### From source

```sh
git clone https://github.com/MrDwarf7/zwse.rs
cd zwse.rs
cargo build --release
# Binary at ./target/release/zwse
```

## Usage

```sh
# Auto-discover Zen profile, export to data/workspace_export_rs.txt
zwse

# Explicit session file (useful for CI/fixtures)
zwse -i data/sessionstore_operations.jsonlz4 -o data/export.txt

# Custom profile directory + session name
zwse -p ~/.zen/fg7mv5ii.Default\ \(beta\) -s sessionstore.jsonlz4 -o export.txt

# Verbosity
zwse -l DEBUG -i sessionstore.jsonlz4 -o export.txt
```

Options:
- `-i, --input <PATH>` — Sessionstore file path. `auto` (default) = discover via Zen profile
- `-p, --profile <DIR>` — Zen profile directory. `auto` (default) = first profile found
- `-s, --session-name <NAME>` — Session filename inside profile (default: sessionstore.jsonlz4)
- `-o, --output <PATH>` — Output file (default: data/workspace_export_rs.txt)
- `-l, --level_verbosity <LEVEL>` — ERROR|WARN|INFO|DEBUG|TRACE or 0-4 (default: INFO)
- `-v, --version` — Print version and exit

## Output Format

Pipe-delimited table:
```
| id | title | workspace | pinned | pinned_entry | pinned_id | last_accessed | url |
```

Example:
```
| 1 | "GitHub - Home" | | false |None | | 1704067200 | "https://github.com" |
```

## Build

```bash
# Release binary
cargo build --release

# Run tests
cargo test

# Format check
cargo fmt --all --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings
```

Or via `cargo make`:
```bash
cargo make build      # cargo build
cargo make build_release  # cargo build --release
cargo make test       # cargo test
cargo make format     # cargo fmt
```

## CI / GitHub Actions

- **build.yml** — fmt, check, clippy (Linux + Windows, nightly)
- **test.yml** — cargo test (Linux + Windows)
- **format.yml** — fmt + build check
- **docs.yml** — cargo doc with warnings as errors
- **draft.yml** — Tag push: builds release artifacts (zip + sha256), uploads as draft release

Artifacts: `zwse-<target>-<tag>.zip` and `.sha256`

## License

MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).