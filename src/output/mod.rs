//! Output formatting: table, JSON, and quiet modes.

use crate::models::DnsRecord;

/// Output format selection.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table (default)
    #[default]
    Table,
    /// JSON output for scripting
    Json,
    /// IDs only, one per line
    Quiet,
}

/// Print a single DNS record.
pub fn print_record(record: &DnsRecord, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_record_table(record),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(record).unwrap_or_else(|_| "{}".into())
            );
        }
        OutputFormat::Quiet => println!("{}", record.id),
    }
}

/// Print a list of DNS records.
pub fn print_records(records: &[DnsRecord], format: OutputFormat) {
    match format {
        OutputFormat::Table => print_records_table(records),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(records).unwrap_or_else(|_| "[]".into())
            );
        }
        OutputFormat::Quiet => {
            for r in records {
                println!("{}", r.id);
            }
        }
    }
}

fn print_record_table(r: &DnsRecord) {
    println!("ID:       {}", r.id);
    println!("Name:     {}", r.name);
    println!("Type:     {}", r.record_type);
    println!("Content:  {}", r.content);
    println!("TTL:      {}", r.ttl);
    println!("Proxied:  {}", r.proxied);
    if let Some(p) = r.priority {
        println!("Priority: {p}");
    }
    if let Some(ref c) = r.comment {
        if !c.is_empty() {
            println!("Comment:  {c}");
        }
    }
    if !r.tags.is_empty() {
        println!("Tags:     {}", r.tags.join(", "));
    }
    if let Some(ref data) = r.data {
        if !data.is_empty() {
            println!(
                "Data:     {}",
                serde_json::to_string(data).unwrap_or_default()
            );
        }
    }
    if let Some(ref s) = r.settings {
        let mut parts = Vec::new();
        if let Some(v) = s.ipv4_only {
            parts.push(format!("ipv4_only={v}"));
        }
        if let Some(v) = s.ipv6_only {
            parts.push(format!("ipv6_only={v}"));
        }
        if !parts.is_empty() {
            println!("Settings: {}", parts.join(", "));
        }
    }
    if let Some(ref t) = r.created_on {
        println!("Created:  {t}");
    }
    if let Some(ref t) = r.modified_on {
        println!("Modified: {t}");
    }
}

fn print_records_table(records: &[DnsRecord]) {
    if records.is_empty() {
        println!("No DNS records found.");
        return;
    }

    // Calculate column widths
    let name_w = records
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(4)
        .clamp(4, 40);
    let type_w = 10;
    let content_w = records
        .iter()
        .map(|r| r.content.len())
        .max()
        .unwrap_or(7)
        .clamp(7, 45);

    println!(
        "{:<name_w$}  {:<type_w$}  {:<content_w$}  {:>5}  {:>7}  {:>3}",
        "NAME", "TYPE", "CONTENT", "TTL", "PROXIED", "PRI"
    );
    println!("{}", "-".repeat(name_w + type_w + content_w + 24));

    for r in records {
        let name_display = if r.name.len() > name_w {
            format!("{}...", &r.name[..name_w - 3])
        } else {
            r.name.clone()
        };
        let content_display = if r.content.len() > content_w {
            format!("{}...", &r.content[..content_w - 3])
        } else {
            r.content.clone()
        };
        let pri = r
            .priority
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".into());
        println!(
            "{:<name_w$}  {:<type_w$}  {:<content_w$}  {:>5}  {:>7}  {:>3}",
            name_display, r.record_type, content_display, r.ttl, r.proxied, pri,
        );
    }

    println!("\nTotal: {} record(s)", records.len());
}
