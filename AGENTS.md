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
src/
  main.rs           # Entry point, CLI dispatch, all cmd_* handler functions (673 lines)
  lib.rs            # Re-exports modules for integration test access (5 lines)
  api/mod.rs        # CloudflareClient — all HTTP methods and API calls (372 lines)
  cli/mod.rs        # Clap derive CLI definitions, all subcommands and arg structs (509 lines)
  config/mod.rs     # Config loading: env vars > local .cfdns.toml > global config > profiles (190 lines)
  models/
    mod.rs          # Module re-exports (7 lines)
    error.rs        # AppError enum, ApiError, Result<T> type alias (64 lines)
    dns_record.rs   # All DNS record types, request/response structs, ListOptions/filters (555 lines)
  output/mod.rs     # OutputFormat enum, table/json/quiet formatting (151 lines)
tests/
  api_test.rs       # Integration tests using tiny_http mock server (841 lines, 38 tests)
```

Total source: ~3,362 lines (including tests).

Modules use directory-based layout (`module_name/mod.rs`), not file-based (`module_name.rs`).

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

A `.env` file exists with real Cloudflare credentials (zone `4b2271369c94651a4f56909c17b7f641`, domain `example.com`).
To load: `set -a && source .env && set +a` (must use `set -a` to export).
Env vars: `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ZONE_ID`, `CLOUDFLARE_DOMAIN_NAME`.

## CI/CD

Only `.github/workflows/release.yml` exists — manual `workflow_dispatch` trigger.
16 cross-compilation targets: Linux glibc/musl, Android API 19/28, macOS, Windows, FreeBSD.
No automated CI/test workflow on push/PR.

## Claude Code Commands

Custom slash commands are available in `.claude/commands/`:

| Command | Description |
|---------|-------------|
| `/cleanup-tech-debt` | Analyze and fix technical debt (dead code, dependencies, structure) |
| `/commit-changes` | Commit ad-hoc changes with auto-generated messages |
| `/commit-spec` | Commit spec implementations with proper messaging |
| `/coverage` | Analyze and improve test coverage with cargo-tarpaulin |
| `/create-spec` | Create detailed specification files for new features |
| `/debug` | Run comprehensive debugging analysis with error intelligence |
| `/implement-spec` | Implement a specification document into working code |
| `/lint` | Detect and fix linting issues with clippy and rustfmt |
| `/live-test` | Test against real Cloudflare API (requires .env credentials) |

All commands are tailored for Rust/Cargo workflows and cfdns-specific patterns.
