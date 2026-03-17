# cfdns Justfile - Cloudflare DNS CLI development commands

# Default recipe - show available commands
default:
    @just --list

# Aliases for common operations
alias d := dev
alias r := run
alias t := test
alias c := check
alias f := fmt
alias l := lint

# === DEVELOPMENT ===

# Run the CLI in development mode (shows help)
dev:
    cargo run -- --help

# Run cfdns with arguments
run *ARGS:
    cargo run -- {{ARGS}}

# Run with release optimizations
run-release *ARGS:
    cargo run --release -- {{ARGS}}

# === BUILDING ===

# Build debug version
build:
    cargo build

# Build release version (optimized for size)
build-release:
    cargo build --release

# Quick type-check without codegen
check:
    cargo check

# Check all targets and features
check-all:
    cargo check --all-targets --all-features

# Clean build artifacts
clean:
    cargo clean

# === TESTING ===

# Run all tests
test:
    cargo test

# Run tests with output visible
test-verbose:
    cargo test -- --nocapture

# Run tests matching a pattern
test-pattern PATTERN:
    cargo test {{PATTERN}}

# Run only integration tests
test-integration:
    cargo test --test list_test --test crud_test --test model_test

# Watch for changes and run tests
test-watch:
    cargo watch -x test

# Run tests with coverage (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html --output-dir coverage/

# === CODE QUALITY ===

# Format code
fmt:
    cargo fmt

# Check formatting without changes
fmt-check:
    cargo fmt --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run clippy with all targets
lint-all:
    cargo clippy --all-targets --all-features -- -D warnings

# Fix automatically fixable lints
fix:
    cargo fix --allow-dirty

# Fix clippy suggestions automatically
fix-clippy:
    cargo clippy --fix --allow-dirty

# === DOCUMENTATION ===

# Generate and open documentation
doc:
    cargo doc --open

# Check documentation for errors
doc-check:
    cargo doc --no-deps

# === DEPENDENCIES ===

# Update dependencies
update:
    cargo update

# Audit dependencies for security vulnerabilities
audit:
    cargo audit

# Check for outdated dependencies
outdated:
    cargo outdated

# Add a new dependency
add CRATE:
    cargo add {{CRATE}}

# Add a development dependency
add-dev CRATE:
    cargo add --dev {{CRATE}}

# Remove a dependency
remove CRATE:
    cargo remove {{CRATE}}

# Show dependency tree
tree:
    cargo tree

# Find duplicate dependencies
duplicate-deps:
    cargo tree --duplicates

# Find unused dependencies (requires cargo-machete)
unused-deps:
    cargo machete

# === CI/CD SIMULATION ===

# Run all CI checks locally
ci: fmt-check lint test doc-check
    @echo "All CI checks passed!"

# Full development cycle check
full-check: clean build test lint-all doc-check
    @echo "Full development cycle completed!"

# === LIVE TESTING ===

# Load environment and list records
live-list:
    @echo "Loading .env and listing DNS records..."
    @bash -c 'set -a && source .env && set +a && cargo run -- list'

# Load environment and count records
live-count:
    @echo "Loading .env and counting DNS records..."
    @bash -c 'set -a && source .env && set +a && cargo run -- count'

# Load environment and run any cfdns command
live *ARGS:
    @bash -c 'set -a && source .env && set +a && cargo run -- {{ARGS}}'

# === UTILITY ===

# Show project tree structure
project-tree:
    tree -I 'target|coverage'

# Show git status
status:
    git status

# === INSTALLATION ===

# Install development tools
install-tools:
    rustup component add rustfmt clippy
    cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated cargo-zigbuild

# Install additional development tools
install-extras:
    cargo install cargo-expand cargo-machete cargo-deny cargo-udeps

# === RELEASE ===

# Prepare for release (dry run)
release-check:
    cargo publish --dry-run

# Create a new release (requires manual version bump)
release:
    cargo publish

# === HELP ===

# Show detailed help for cargo commands
help:
    @echo "cfdns development commands:"
    @echo ""
    @echo "  Build & Run:"
    @echo "    just build         - Build debug version"
    @echo "    just build-release - Build release version"
    @echo "    just run <args>    - Run cfdns with arguments"
    @echo ""
    @echo "  Testing:"
    @echo "    just t / just test - Run all tests"
    @echo "    just test-verbose  - Run tests with output"
    @echo "    just coverage      - Generate coverage report"
    @echo ""
    @echo "  Code Quality:"
    @echo "    just f / just fmt  - Format code"
    @echo "    just l / just lint - Run clippy"
    @echo "    just c / just check- Quick type-check"
    @echo "    just ci            - Run all CI checks"
    @echo ""
    @echo "  Live Testing (requires .env):"
    @echo "    just live-list     - List DNS records"
    @echo "    just live-count    - Count DNS records"
    @echo "    just live <cmd>    - Run any cfdns command"
    @echo ""
    @echo "  Dependencies:"
    @echo "    just update        - Update dependencies"
    @echo "    just audit         - Security audit"
    @echo "    just outdated      - Check for updates"
