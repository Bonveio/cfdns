//! Metadata commands: comment, tag.

use crate::api::CloudflareClient;
use crate::cli;
use crate::models::*;
use crate::output::{print_record, OutputFormat};

use super::{parse_tags, resolve_record};

/// Update the comment on a DNS record.
pub fn cmd_comment(
    client: &CloudflareClient,
    args: cli::CommentArgs,
    fmt: OutputFormat,
) -> Result<()> {
    let existing = resolve_record(client, args.id.as_deref(), args.name.as_deref(), None)?;

    let req = UpdateRecordRequest {
        comment: Some(args.comment),
        ..Default::default()
    };

    let record = client.update_record(&existing.id, &req)?;
    if fmt == OutputFormat::Table {
        println!("Comment updated.");
    }
    print_record(&record, fmt);
    Ok(())
}

/// Update the tags on a DNS record.
pub fn cmd_tag(client: &CloudflareClient, args: cli::TagArgs, fmt: OutputFormat) -> Result<()> {
    let existing = resolve_record(client, args.id.as_deref(), args.name.as_deref(), None)?;

    let tags = parse_tags(Some(&args.tags));
    let req = UpdateRecordRequest {
        tags: Some(tags),
        ..Default::default()
    };

    let record = client.update_record(&existing.id, &req)?;
    if fmt == OutputFormat::Table {
        println!("Tags updated.");
    }
    print_record(&record, fmt);
    Ok(())
}
