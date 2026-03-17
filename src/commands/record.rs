//! Record CRUD commands: create, update, overwrite, delete.

use crate::api::CloudflareClient;
use crate::cli;
use crate::models::*;
use crate::output::{print_record, OutputFormat};

use super::{parse_json_data, parse_settings, parse_tags, resolve_record};

/// Create a new DNS record.
pub fn cmd_create(
    client: &CloudflareClient,
    args: cli::CreateArgs,
    fmt: OutputFormat,
) -> Result<()> {
    let data = parse_json_data(args.data.as_deref())?;
    let settings = parse_settings(args.settings.as_deref())?;
    let tags = parse_tags(args.tags.as_deref());

    let proxied = if RecordType::supports_proxy(&args.r#type) {
        Some(args.proxied)
    } else {
        None
    };

    let req = CreateRecordRequest {
        name: args.name,
        record_type: args.r#type.to_uppercase(),
        content: args.content,
        ttl: args.ttl,
        proxied,
        priority: args.priority,
        comment: args.comment,
        tags,
        data,
        settings,
    };

    let record = client.create_record(&req)?;
    if fmt == OutputFormat::Table {
        println!("Record created successfully.");
    }
    print_record(&record, fmt);
    Ok(())
}

/// Update (PATCH) an existing DNS record.
pub fn cmd_update(
    client: &CloudflareClient,
    args: cli::UpdateArgs,
    fmt: OutputFormat,
) -> Result<()> {
    let existing = resolve_record(
        client,
        args.id.as_deref(),
        args.name.as_deref(),
        args.r#type.as_deref(),
    )?;

    let data = parse_json_data(args.data.as_deref())?;
    let settings = parse_settings(args.settings.as_deref())?;
    let tags = args.tags.as_deref().map(|t| parse_tags(Some(t)));

    let req = UpdateRecordRequest {
        name: args.new_name,
        record_type: None,
        content: args.content,
        ttl: args.ttl,
        proxied: args.proxied,
        priority: args.priority,
        comment: args.comment,
        tags,
        data,
        settings,
    };

    let record = client.update_record(&existing.id, &req)?;
    if fmt == OutputFormat::Table {
        println!("Record updated successfully.");
    }
    print_record(&record, fmt);
    Ok(())
}

/// Overwrite (PUT) an existing DNS record.
pub fn cmd_overwrite(
    client: &CloudflareClient,
    args: cli::OverwriteArgs,
    fmt: OutputFormat,
) -> Result<()> {
    let existing = resolve_record(
        client,
        args.id.as_deref(),
        args.name.as_deref(),
        Some(&args.r#type),
    )?;

    let data = parse_json_data(args.data.as_deref())?;
    let settings = parse_settings(args.settings.as_deref())?;
    let tags = parse_tags(args.tags.as_deref());

    let record_name = args.new_name.unwrap_or(existing.name);
    let proxied = if RecordType::supports_proxy(&args.r#type) {
        Some(args.proxied)
    } else {
        None
    };

    let req = OverwriteRecordRequest {
        name: record_name,
        record_type: args.r#type.to_uppercase(),
        content: args.content,
        ttl: args.ttl,
        proxied,
        priority: args.priority,
        comment: args.comment,
        tags,
        data,
        settings,
    };

    let record = client.overwrite_record(&existing.id, &req)?;
    if fmt == OutputFormat::Table {
        println!("Record overwritten successfully.");
    }
    print_record(&record, fmt);
    Ok(())
}

/// Delete a DNS record.
pub fn cmd_delete(
    client: &CloudflareClient,
    args: cli::DeleteArgs,
    fmt: OutputFormat,
) -> Result<()> {
    let existing = resolve_record(
        client,
        args.id.as_deref(),
        args.name.as_deref(),
        args.r#type.as_deref(),
    )?;

    let result = client.delete_record(&existing.id)?;
    match fmt {
        OutputFormat::Table => {
            println!(
                "Record deleted: {} ({} {})",
                existing.id, existing.record_type, existing.name
            );
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).unwrap_or_default()
            );
        }
        OutputFormat::Quiet => {
            println!("{}", existing.id);
        }
    }
    Ok(())
}
