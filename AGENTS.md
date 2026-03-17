# AGENTS.md — Coding Agent Guidelines for cfdns

## Project Overview

`cfdns` is a synchronous Rust CLI tool for managing Cloudflare DNS records.
Binary name: `cfdns`. Package name: `cfdns`. Edition 2021, MSRV 1.93.1.
Uses `ureq` (sync HTTP), `clap` v4 (derive), `serde`, `thiserror`. No async runtime.

## Build / Lint / Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (opt-level=z, LTO, strip, panic=abort)
cargo check                          # Quick type-check without codegen
cargo fmt                            # Format code (default rustfmt settings)
cargo fmt --check                    # Check formatting without changes
cargo clippy -- -D warnings          # Lint (warnings are errors)
cargo clippy --all-targets --all-features -- -D warnings  # Lint everything
cargo test                           # Run all tests
cargo test -- --nocapture            # Tests with stdout visible
cargo test test_list_records_empty   # Run a single test by name
cargo test test_create              # Run tests matching a substring
cargo test --test api_test           # Run only the api_test integration test file
cargo doc --no-deps                  # Check documentation builds
```

Shorthand via Justfile: `just t` (test), `just l` (lint), `just f` (fmt), `just c` (check),
`just ci` (fmt-check + lint + test + doc-check), `just test-pattern PATTERN` (filtered tests).

## Project Structure

```
AGENTS.md            # Agent guidelines and coding instructions for this repository
Cargo.lock           # Lockfile for reproducible builds
Cargo.toml           # Rust crate manifest/config (dependencies, metadata)
Justfile             # Handy shortcuts for build, test, lint tasks
LICENSE              # Project license information
README.md            # Project description and user documentation
build.rs             # Build script (if needed for preprocessing)
src/
  api/
    mod.rs           # Cloudflare API client: HTTP request logic and endpoints
  cli/
    mod.rs           # CLI definitions, command-line argument parsing via clap
  commands/
    helpers.rs       # Helpers for command execution, formatting, etc.
    init.rs          # Command logic for project/init operations
    metadata.rs      # Command logic for metadata queries, info reporting
    mod.rs           # Root of commands module, re-exports
    query.rs         # Query DNS records: filtering, searching, etc.
    record.rs        # CRUD operations on DNS records
  config/
    mod.rs           # Configuration loading, env parsing, profile support
  lib.rs             # Re-export modules for integration tests
  main.rs            # Entry point: CLI dispatch, command invocation
  models/
    dns_record.rs    # All DNS record types, filters, and related structs
    error.rs         # Custom error types, AppError enum, Result alias
    mod.rs           # Module re-exports for models
  output/
    mod.rs           # Output formatters: table, JSON, quiet mode
tests/
  common.rs          # Shared test utilities and helpers
  crud_test.rs       # CRUD operation tests for DNS records
  list_test.rs       # Tests for listing, querying records
  model_test.rs      # Model serialization, deserialization, and type tests
```

- Modules use directory-based layout (`mod.rs` for each module).
- Main entry: `src/main.rs` (CLI, all command handlers).
- Commands broken out in `src/commands/` (helpers, CRUD, metadata, etc).
- Integration and model tests in `tests/`.
- Project configuration: `Cargo.toml`, `Justfile` (for task shorthand), `.env` (for Cloudflare credentials).


### Formatting

Default `rustfmt` — no `rustfmt.toml` exists. 4-space indentation. No special config.

### Import Ordering

Three groups separated by blank lines:

1. Local module declarations (`mod`) or crate imports (`use crate::...`)
2. External crate imports (`use clap::...`, `use serde::...`)
3. Standard library (`use std::...`)

Within `main.rs`, local imports use short paths (`use api::CloudflareClient`).
In submodules, use `crate::` prefix (`use crate::models::*`).
Wildcard `use crate::models::*` is the standard pattern for the models module.

### Naming Conventions

| Element            | Convention           | Examples                                      |
|--------------------|----------------------|-----------------------------------------------|
| Modules            | snake_case           | `dns_record`, `api`, `config`                 |
| Structs            | PascalCase           | `CloudflareClient`, `DnsRecord`, `ApiResponse`|
| Enums              | PascalCase           | `AppError`, `OutputFormat`, `RecordType`       |
| Functions          | snake_case           | `list_records`, `resolve_fqdn`                |
| Constants          | SCREAMING_SNAKE_CASE | `DEFAULT_BASE_URL`                            |
| CLI arg structs    | `{Name}Args`         | `ListArgs`, `CreateArgs`, `DeleteArgs`        |
| Request structs    | `{Name}Request`      | `CreateRecordRequest`, `BatchRequest`         |
| Response structs   | `{Name}Response`     | `ApiResponse<T>`, `BatchResponse`             |
| Command handlers   | `cmd_{name}`         | `cmd_list`, `cmd_create`, `cmd_delete`        |
| Test functions     | `test_{operation}`   | `test_list_records_empty`, `test_api_error`   |

### Derive Patterns

- **Error enums/structs**: `Debug, thiserror::Error`
- **API responses (read-only)**: `Debug, Deserialize` or `Debug, Clone, Deserialize`
- **Data models (read+write)**: `Debug, Clone, Serialize, Deserialize`
- **Request structs (write-only)**: `Debug, Clone, Serialize`
- **Structs with partial init**: add `Default` for `..Default::default()` usage
- **Simple enums**: `Debug, Clone, Copy, PartialEq, Eq`
- **Clap structs**: `Debug, Parser` / `Debug, Subcommand` / `Debug, clap::Args`
- **Output format enum**: `Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum`

`Debug` goes on everything. `Clone` when values are cloned. `Copy` only on small enums.
Serde derives are precise — only `Serialize` where needed, only `Deserialize` where needed.

### Serde Attributes

```rust
#[serde(rename = "type")]                          // Reserved word field
#[serde(default)]                                  // Use Default when missing
#[serde(default = "default_fn")]                   // Custom default function
#[serde(skip_serializing_if = "Option::is_none")]  // Omit None fields
#[serde(skip_serializing_if = "Vec::is_empty")]    // Omit empty vectors
```

### Error Handling

Custom error enum with `thiserror` in `models/error.rs`. Type alias: `pub type Result<T> = std::result::Result<T, AppError>;`

- Use `?` for automatic conversions (`std::io::Error`, `serde_json::Error` via `#[from]`)
- Use `.map_err(|e| AppError::Http(e.to_string()))?` for manual wrapping
- Use `Err(AppError::Config("...".into()))` for direct construction
- Use `.ok_or_else(|| AppError::Other(...))` to convert `None` to errors
- **Never** use `unwrap()` in business logic; only in non-critical display formatting
- **Never** use `panic!` for normal error flow
- Top-level `main()` catches errors with `if let Err(e) = run(cli)` and calls `process::exit(1)`

### Documentation

- Module-level: `//!` at top of every file (e.g., `//! Cloudflare DNS API client...`)
- Items: `///` with short one-line descriptions
- Section dividers in `main.rs`:
  ```rust
  // ---------------------------------------------------------------------------
  // Section Name
  // ---------------------------------------------------------------------------
  ```
- No `# Arguments` / `# Returns` / `# Examples` sections in practice

### Other Patterns

- `#![allow(dead_code)]` in `models/error.rs` and `models/dns_record.rs`
- `String` in struct fields, `&str` in function parameters
- `HashMap<String, serde_json::Value>` for dynamic JSON data
- `..Default::default()` for partial struct initialization
- Inline format syntax: `format!("{path}")`, `format!("Error: {e}")`
- No logging crate — errors via `eprintln!`, output via `println!`
- `r#type` and `r#match` for Rust keyword field names in clap

## Testing

Tests are in `tests/api_test.rs` — integration tests using `tiny_http` mock server.
No unit test modules (`#[cfg(test)]`) exist in source files.

Test pattern: spawn client call in a thread, respond from main thread:
```rust
let handle = std::thread::spawn(move || client.list_records(&ListOptions::default()));
respond_json(&server, r#"{"success":true,...}"#);
let result = handle.join().unwrap().expect("should succeed");
```

Helper functions: `mock_setup()`, `respond_json()`, `respond_text()`, `recv_request()`.

## Live Testing

To initialize credentials and configuration, run:

```bash
./cfdns init -i
```

## CI/CD

- CI/CD is handled via `.github/workflows/release.yml`.
- Workflow is triggered **manually** with `workflow_dispatch` and a version input (YYYY.M.D format).
- Jobs:
  - **validate-version**: Checks version input is valid.
  - **ci**: Runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo doc --no-deps`.
  - **windows**: Builds and packages Windows MSVC and GNU targets.
  - **linux-musl**: Builds and packages seven Linux musl targets using downloaded cross-toolchains.
  - **release**: Packages artifacts, generates checksums, and creates the GitHub release.
- Artifacts for each platform are zipped/tarred and uploaded as workflow artifacts.
- **No automated CI on push/PR:** Release is only triggered manually.
- **Environment:** Uses cache for dependencies, sets `CARGO_TERM_COLOR`, manages custom toolchain for musl.
- See workflow file for specific release and packaging logic.

