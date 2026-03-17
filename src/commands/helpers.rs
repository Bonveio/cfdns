//! Helper functions used across command handlers.

use crate::api::CloudflareClient;
use crate::models::*;

use std::collections::HashMap;

/// Resolve a DNS record by either ID or name (+ optional type).
pub fn resolve_record(
    client: &CloudflareClient,
    id: Option<&str>,
    name: Option<&str>,
    record_type: Option<&str>,
) -> Result<DnsRecord> {
    match (id, name) {
        (Some(record_id), _) => client.get_record(record_id),
        (None, Some(record_name)) => match record_type {
            Some(rt) => client.find_record_by_name_and_type(record_name, rt),
            None => client.find_record_by_name(record_name),
        },
        (None, None) => Err(AppError::Other(
            "Provide either a name or --id to identify the record.".into(),
        )),
    }
}

/// Parse a JSON string into a HashMap for the `data` field.
pub fn parse_json_data(
    json_str: Option<&str>,
) -> Result<Option<HashMap<String, serde_json::Value>>> {
    match json_str {
        Some(s) => {
            let map: HashMap<String, serde_json::Value> = serde_json::from_str(s)
                .map_err(|e| AppError::Other(format!("Invalid --data JSON: {e}")))?;
            Ok(Some(map))
        }
        None => Ok(None),
    }
}

/// Parse a JSON string into RecordSettings.
pub fn parse_settings(json_str: Option<&str>) -> Result<Option<RecordSettings>> {
    match json_str {
        Some(s) => {
            let settings: RecordSettings = serde_json::from_str(s)
                .map_err(|e| AppError::Other(format!("Invalid --settings JSON: {e}")))?;
            Ok(Some(settings))
        }
        None => Ok(None),
    }
}

/// Parse comma-separated tags into a Vec.
pub fn parse_tags(tags_str: Option<&str>) -> Vec<String> {
    match tags_str {
        Some(s) if !s.is_empty() => s
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}
