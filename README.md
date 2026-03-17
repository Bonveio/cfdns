# cfdns

A lightweight CLI tool for managing Cloudflare DNS records.

## Features

- **Common DNS record types**: A, AAAA, CAA, CNAME, HTTPS, MX, NS, PTR, SRV, SVCB, TXT (other types via raw strings)
- **Full CRUD operations**: list, get, create, update, overwrite, delete
- **Convenience commands**: search, count, comment, tag
- **Output formats**: table (human-readable), JSON (scripting), quiet (IDs only)
- **Config profiles**: environment variables, global config file, per-project config
- **Lightweight**: ~3MB static binary, pure Rust, no runtime dependencies

## Installation

Download the latest release for your platform from the [Releases](https://github.com/Bonveio/cfdns/releases) page.

### From source

```sh
cargo install --path .
```

## Configuration

### Environment variables (recommended)

```sh
export CLOUDFLARE_API_TOKEN="your-api-token"
export CLOUDFLARE_ZONE_ID="your-zone-id"
export CLOUDFLARE_DOMAIN_NAME="example.com"  # optional, enables short names
```

### Config file

```sh
cfdns init  # creates ~/.config/cfdns/config.toml
```

```toml
default_profile = "default"

[profiles.default]
api_token = "your-api-token"
zone_id = "your-zone-id"
domain_name = "example.com"
```

### Per-project config

Place a `.cfdns.toml` file in your project root. It takes priority over the global config.

### Profiles

Select a profile via `--profile` flag or `CFDNS_PROFILE` environment variable.

## Usage

### List records

```sh
cfdns list
cfdns list -t A
cfdns list --name-startswith www
cfdns list --content-contains 192.168
cfdns list --comment-present
cfdns list --tag env:prod
cfdns list --output json
```

### Get a record

```sh
cfdns get www                  # by subdomain
cfdns get www.example.com      # by FQDN
cfdns get --id <record-id>     # by ID
cfdns get www -t A             # by name + type
```

### Create a record

```sh
cfdns create www -t A -c 1.2.3.4
cfdns create www -t A -c 1.2.3.4 --proxied --comment "web server"
cfdns create mail -t MX -c mail.example.com -p 10
cfdns create example.com -t CAA --data '{"flags":0,"tag":"issue","value":"letsencrypt.org"}'
```

### Update a record (PATCH)

```sh
cfdns update www -t A -c 5.6.7.8
cfdns update --id <record-id> -c 5.6.7.8
cfdns update www -t A --comment "updated"
cfdns update www -t A --new-name www2
```

### Overwrite a record (PUT)

```sh
cfdns overwrite www -t A -c 9.8.7.6 --ttl 300
```

### Delete a record

```sh
cfdns delete www -t A
cfdns delete --id <record-id>
```

### Search

```sh
cfdns search dev
cfdns search mail -t MX
```

### Count

```sh
cfdns count
cfdns count -t A
```

### Update comment/tags

```sh
cfdns comment www -c "production web server"
cfdns tag www -t "env:prod,tier:1"
```

### Output formats

```sh
cfdns list --output table   # default, human-readable
cfdns list --output json    # JSON for scripting
cfdns list --output quiet   # IDs only, one per line
```

## Supported platforms

| Platform | Architectures | Notes |
|----------|---------------|-------|
| Linux (glibc) | x86_64, aarch64 | GLIBC 2.35+ (Ubuntu 22.04+) |
| Linux (musl) | x86_64, aarch64 | Fully static, runs anywhere |
| Android API 19 (KitKat 4.4) | arm32 | NDK r25c, `cfdns init -i` unavailable |
| Android API 33 (Android 13) | arm32, arm64, x86_64 | NDK r25c |
| macOS | x86_64, aarch64 | macOS 11+ |
| Windows | x86_64 (MSVC), x86_64 (GNU) | Windows 10+ |

## Building

### Development

This project uses [Just](https://github.com/casey/just) as a command runner. Install it with:

```sh
cargo install just
```

Common commands:

```sh
just              # Show available commands
just build        # Debug build
just build-release # Release build
just t            # Run tests
just l            # Run clippy linter
just f            # Format code
just c            # Quick type-check
just ci           # Run all CI checks (fmt-check + lint + test + doc-check)
just coverage     # Generate test coverage report (requires cargo-tarpaulin)
```

### Install development tools

```sh
just install-tools
```

This installs: rustfmt, clippy, cargo-watch, cargo-tarpaulin, cargo-audit, cargo-outdated, cargo-zigbuild

### Release build

```sh
cargo build --release
```

The release binary is optimized for size (opt-level=z, LTO enabled, symbols stripped, panic=abort).
