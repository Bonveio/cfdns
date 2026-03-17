//! CLI command definitions using clap derive.

use crate::output::OutputFormat;
use clap::{Parser, Subcommand};

/// cfdns - Cloudflare DNS record manager
///
/// A lightweight CLI tool for managing Cloudflare DNS records.
/// Supports all 21 DNS record types and the full Cloudflare DNS API.
///
/// Configuration via environment variables or config file (~/.config/cfdns/config.toml):
///   CLOUDFLARE_API_TOKEN   - Your Cloudflare API token
///   CLOUDFLARE_ZONE_ID     - The zone ID for your domain
///   CLOUDFLARE_DOMAIN_NAME - Your domain name (optional, for short names)
#[derive(Debug, Parser)]
#[command(name = "cfdns", version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(long, global = true, default_value = "table", value_enum)]
    pub output: OutputFormat,

    /// Configuration profile to use
    #[arg(long, global = true)]
    pub profile: Option<String>,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// List DNS records with optional filters
    List(ListArgs),

    /// Get a single DNS record by ID or name
    Get(GetArgs),

    /// Create a new DNS record
    Create(CreateArgs),

    /// Update (patch) an existing DNS record
    Update(UpdateArgs),

    /// Overwrite (replace) an existing DNS record
    Overwrite(OverwriteArgs),

    /// Delete a DNS record
    Delete(DeleteArgs),

    /// Search DNS records by name or content
    Search(SearchArgs),

    /// Count DNS records in the zone
    Count(CountArgs),

    /// Update the comment on a DNS record
    Comment(CommentArgs),

    /// Update the tags on a DNS record
    Tag(TagArgs),

    /// Initialize a config file with example values
    Init(InitArgs),
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct ListArgs {
    /// Filter by record type (A, AAAA, CNAME, MX, TXT, etc.)
    #[arg(short = 't', long)]
    pub r#type: Option<String>,

    /// Filter by exact name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Filter: name contains substring
    #[arg(long)]
    pub name_contains: Option<String>,

    /// Filter: name starts with prefix
    #[arg(long)]
    pub name_startswith: Option<String>,

    /// Filter: name ends with suffix
    #[arg(long)]
    pub name_endswith: Option<String>,

    /// Filter by exact content
    #[arg(short, long)]
    pub content: Option<String>,

    /// Filter: content contains substring
    #[arg(long)]
    pub content_contains: Option<String>,

    /// Filter: content starts with prefix
    #[arg(long)]
    pub content_startswith: Option<String>,

    /// Filter: content ends with suffix
    #[arg(long)]
    pub content_endswith: Option<String>,

    /// Filter by exact comment
    #[arg(long)]
    pub comment: Option<String>,

    /// Filter: comment contains substring
    #[arg(long)]
    pub comment_contains: Option<String>,

    /// Filter: comment starts with prefix
    #[arg(long)]
    pub comment_startswith: Option<String>,

    /// Filter: comment ends with suffix
    #[arg(long)]
    pub comment_endswith: Option<String>,

    /// Filter: only records without a comment
    #[arg(long)]
    pub comment_absent: bool,

    /// Filter: only records with a comment
    #[arg(long)]
    pub comment_present: bool,

    /// Filter by exact tag (name:value format)
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter: tag contains substring
    #[arg(long)]
    pub tag_contains: Option<String>,

    /// Filter: tag starts with prefix
    #[arg(long)]
    pub tag_startswith: Option<String>,

    /// Filter: tag ends with suffix
    #[arg(long)]
    pub tag_endswith: Option<String>,

    /// Filter: records without this tag name
    #[arg(long)]
    pub tag_absent: Option<String>,

    /// Filter: records with this tag name
    #[arg(long)]
    pub tag_present: Option<String>,

    /// Filter by proxied status
    #[arg(long)]
    pub proxied: Option<bool>,

    /// Full-text search across multiple fields
    #[arg(short, long)]
    pub search: Option<String>,

    /// Order by field (type, name, content, ttl, proxied)
    #[arg(long)]
    pub order: Option<String>,

    /// Sort direction (asc, desc)
    #[arg(short, long)]
    pub direction: Option<String>,

    /// Page number
    #[arg(long)]
    pub page: Option<u64>,

    /// Records per page (max 5000000)
    #[arg(long)]
    pub per_page: Option<u64>,

    /// Match mode: all (AND) or any (OR)
    #[arg(long, default_value = "all")]
    pub r#match: Option<String>,

    /// Tag match mode: all (AND) or any (OR)
    #[arg(long)]
    pub tag_match: Option<String>,
}

// ---------------------------------------------------------------------------
// Get
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct GetArgs {
    /// Subdomain or FQDN to look up
    pub name: Option<String>,

    /// Look up by record ID instead
    #[arg(short, long)]
    pub id: Option<String>,

    /// Filter by record type when looking up by name
    #[arg(short = 't', long)]
    pub r#type: Option<String>,
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct CreateArgs {
    /// Subdomain or FQDN for the record
    pub name: String,

    /// DNS record type (A, AAAA, CNAME, MX, TXT, SRV, etc.)
    #[arg(short = 't', long, default_value = "A")]
    pub r#type: String,

    /// Record content (IP address, domain, text, etc.)
    #[arg(short, long)]
    pub content: Option<String>,

    /// TTL in seconds (1 = auto)
    #[arg(long, default_value = "1")]
    pub ttl: u32,

    /// Priority (for MX, SRV, URI records)
    #[arg(short, long)]
    pub priority: Option<u16>,

    /// Enable Cloudflare proxy (A/AAAA/CNAME only)
    #[arg(long)]
    pub proxied: bool,

    /// Comment for the record
    #[arg(long)]
    pub comment: Option<String>,

    /// Tags (comma-separated, name:value format)
    #[arg(long)]
    pub tags: Option<String>,

    /// Type-specific data as JSON (for SRV, CAA, LOC, etc.)
    #[arg(long)]
    pub data: Option<String>,

    /// Record settings as JSON (e.g. '{"ipv4_only":true}')
    #[arg(long)]
    pub settings: Option<String>,
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct UpdateArgs {
    /// Subdomain or FQDN to update
    pub name: Option<String>,

    /// Update by record ID instead
    #[arg(short, long)]
    pub id: Option<String>,

    /// Filter by record type when looking up by name
    #[arg(short = 't', long)]
    pub r#type: Option<String>,

    /// New content
    #[arg(short, long)]
    pub content: Option<String>,

    /// New TTL
    #[arg(long)]
    pub ttl: Option<u32>,

    /// New priority
    #[arg(short, long)]
    pub priority: Option<u16>,

    /// Set proxied status
    #[arg(long)]
    pub proxied: Option<bool>,

    /// New comment
    #[arg(long)]
    pub comment: Option<String>,

    /// New tags (comma-separated)
    #[arg(long)]
    pub tags: Option<String>,

    /// Type-specific data as JSON
    #[arg(long)]
    pub data: Option<String>,

    /// Record settings as JSON
    #[arg(long)]
    pub settings: Option<String>,

    /// New name (rename the record)
    #[arg(long)]
    pub new_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Overwrite
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct OverwriteArgs {
    /// Subdomain or FQDN to overwrite
    pub name: Option<String>,

    /// Overwrite by record ID
    #[arg(short, long)]
    pub id: Option<String>,

    /// Filter by record type when looking up by name
    #[arg(short = 't', long)]
    pub r#type: String,

    /// Record content
    #[arg(short, long)]
    pub content: Option<String>,

    /// TTL in seconds (1 = auto)
    #[arg(long, default_value = "1")]
    pub ttl: u32,

    /// Priority
    #[arg(short, long)]
    pub priority: Option<u16>,

    /// Enable Cloudflare proxy
    #[arg(long)]
    pub proxied: bool,

    /// Comment
    #[arg(long)]
    pub comment: Option<String>,

    /// Tags (comma-separated)
    #[arg(long)]
    pub tags: Option<String>,

    /// Type-specific data as JSON
    #[arg(long)]
    pub data: Option<String>,

    /// Record settings as JSON
    #[arg(long)]
    pub settings: Option<String>,

    /// New name for the record
    #[arg(long)]
    pub new_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct DeleteArgs {
    /// Subdomain or FQDN to delete
    pub name: Option<String>,

    /// Delete by record ID
    #[arg(short, long)]
    pub id: Option<String>,

    /// Record type for verification when deleting by name
    #[arg(short = 't', long)]
    pub r#type: Option<String>,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct SearchArgs {
    /// Search query (matches across name, content, and other fields)
    pub query: String,

    /// Filter by record type
    #[arg(short = 't', long)]
    pub r#type: Option<String>,
}

// ---------------------------------------------------------------------------
// Count
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct CountArgs {
    /// Count only records of this type
    #[arg(short = 't', long)]
    pub r#type: Option<String>,
}

// ---------------------------------------------------------------------------
// Comment
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct CommentArgs {
    /// Subdomain or FQDN
    pub name: Option<String>,

    /// Record ID
    #[arg(short, long)]
    pub id: Option<String>,

    /// New comment text (empty string to clear)
    #[arg(short, long)]
    pub comment: String,
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct TagArgs {
    /// Subdomain or FQDN
    pub name: Option<String>,

    /// Record ID
    #[arg(short, long)]
    pub id: Option<String>,

    /// New tags (comma-separated, name:value format). Empty string to clear.
    #[arg(short, long)]
    pub tags: String,
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Path where the config file will be created (default: ~/.config/cfdns)
    #[arg(short, long)]
    pub path: Option<String>,

    /// Interactively prompt for config values
    #[arg(short, long)]
    pub interactive: bool,
}
