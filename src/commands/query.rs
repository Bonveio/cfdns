//! Query commands: list, get, search, count.

use crate::api::CloudflareClient;
use crate::cli;
use crate::models::*;
use crate::output::{print_record, print_records, OutputFormat};

use super::resolve_record;

/// List DNS records with optional filters.
pub fn cmd_list(client: &CloudflareClient, args: cli::ListArgs, fmt: OutputFormat) -> Result<()> {
    let opts = build_list_options(&args);
    let (records, info) = client.list_records(&opts)?;
    print_records(&records, fmt);

    if fmt == OutputFormat::Table {
        if let Some(ri) = info {
            if let (Some(page), Some(total_pages)) = (ri.page, ri.total_pages) {
                if total_pages > 1 {
                    println!("Page {page} of {total_pages}");
                }
            }
        }
    }
    Ok(())
}

fn build_list_options(args: &cli::ListArgs) -> ListOptions {
    let name_filter = if args.name.is_some()
        || args.name_contains.is_some()
        || args.name_startswith.is_some()
        || args.name_endswith.is_some()
    {
        Some(ListFilter {
            exact: args.name.clone(),
            contains: args.name_contains.clone(),
            startswith: args.name_startswith.clone(),
            endswith: args.name_endswith.clone(),
        })
    } else {
        None
    };

    let content_filter = if args.content.is_some()
        || args.content_contains.is_some()
        || args.content_startswith.is_some()
        || args.content_endswith.is_some()
    {
        Some(ListFilter {
            exact: args.content.clone(),
            contains: args.content_contains.clone(),
            startswith: args.content_startswith.clone(),
            endswith: args.content_endswith.clone(),
        })
    } else {
        None
    };

    let comment_filter = if args.comment.is_some()
        || args.comment_contains.is_some()
        || args.comment_startswith.is_some()
        || args.comment_endswith.is_some()
        || args.comment_absent
        || args.comment_present
    {
        Some(CommentFilter {
            exact: args.comment.clone(),
            contains: args.comment_contains.clone(),
            startswith: args.comment_startswith.clone(),
            endswith: args.comment_endswith.clone(),
            absent: args.comment_absent,
            present: args.comment_present,
        })
    } else {
        None
    };

    let tag_filter = if args.tag.is_some()
        || args.tag_contains.is_some()
        || args.tag_startswith.is_some()
        || args.tag_endswith.is_some()
        || args.tag_absent.is_some()
        || args.tag_present.is_some()
    {
        Some(TagFilter {
            exact: args.tag.clone(),
            contains: args.tag_contains.clone(),
            startswith: args.tag_startswith.clone(),
            endswith: args.tag_endswith.clone(),
            absent: args.tag_absent.clone(),
            present: args.tag_present.clone(),
        })
    } else {
        None
    };

    ListOptions {
        record_type: args.r#type.clone(),
        name: name_filter,
        content: content_filter,
        comment: comment_filter,
        tag: tag_filter,
        proxied: args.proxied,
        search: args.search.clone(),
        order: args.order.clone(),
        direction: args.direction.clone(),
        page: args.page,
        per_page: args.per_page,
        match_mode: args.r#match.clone(),
        tag_match: args.tag_match.clone(),
    }
}

/// Get a single DNS record by ID or name.
pub fn cmd_get(client: &CloudflareClient, args: cli::GetArgs, fmt: OutputFormat) -> Result<()> {
    let record = resolve_record(
        client,
        args.id.as_deref(),
        args.name.as_deref(),
        args.r#type.as_deref(),
    )?;
    print_record(&record, fmt);
    Ok(())
}

/// Search DNS records by name or content.
pub fn cmd_search(
    client: &CloudflareClient,
    args: cli::SearchArgs,
    fmt: OutputFormat,
) -> Result<()> {
    let mut opts = ListOptions {
        search: Some(args.query),
        ..Default::default()
    };
    if let Some(ref t) = args.r#type {
        opts.record_type = Some(t.clone());
    }
    let (records, _) = client.list_records(&opts)?;
    print_records(&records, fmt);
    Ok(())
}

/// Count DNS records in the zone.
pub fn cmd_count(client: &CloudflareClient, args: cli::CountArgs) -> Result<()> {
    let opts = ListOptions {
        per_page: Some(1), // minimize data transfer
        record_type: args.r#type.clone(),
        ..Default::default()
    };
    let (_, info) = client.list_records(&opts)?;
    let count = info.and_then(|ri| ri.total_count).unwrap_or(0);
    println!("{count}");
    Ok(())
}
