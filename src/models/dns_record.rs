//! DNS record models covering the 12 most commonly used Cloudflare DNS record types.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::error::{ApiErrorDetail, ApiMessage};

// ---------------------------------------------------------------------------
// API envelope
// ---------------------------------------------------------------------------

/// Generic Cloudflare API response envelope.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(default)]
    pub errors: Vec<ApiErrorDetail>,
    #[serde(default)]
    pub messages: Vec<ApiMessage>,
    pub result: Option<T>,
    pub result_info: Option<ResultInfo>,
}

/// Pagination metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct ResultInfo {
    pub count: Option<u64>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub total_count: Option<u64>,
    pub total_pages: Option<u64>,
}

// ---------------------------------------------------------------------------
// DNS Record – unified representation
// ---------------------------------------------------------------------------

/// Represents a single DNS record as returned by the Cloudflare API.
///
/// This is a unified struct that covers all 21 record types. Type-specific
/// fields live in the `data` map (e.g. SRV weight/port, CAA flags/tag, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_ttl")]
    pub ttl: TtlValue,
    #[serde(default)]
    pub proxied: bool,
    #[serde(default)]
    pub proxiable: bool,
    #[serde(default)]
    pub priority: Option<u16>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub settings: Option<RecordSettings>,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub modified_on: Option<String>,
    #[serde(default)]
    pub comment_modified_on: Option<String>,
    #[serde(default)]
    pub tags_modified_on: Option<String>,
}

fn default_ttl() -> TtlValue {
    TtlValue::Auto
}

/// TTL can be `1` (auto) or an integer number of seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtlValue {
    Auto,
    Seconds(u32),
}

impl Serialize for TtlValue {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        match self {
            TtlValue::Auto => serializer.serialize_u32(1),
            TtlValue::Seconds(s) => serializer.serialize_u32(*s),
        }
    }
}

impl<'de> Deserialize<'de> for TtlValue {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let v = u32::deserialize(deserializer)?;
        Ok(if v <= 1 {
            TtlValue::Auto
        } else {
            TtlValue::Seconds(v)
        })
    }
}

impl fmt::Display for TtlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TtlValue::Auto => write!(f, "auto"),
            TtlValue::Seconds(s) => write!(f, "{s}"),
        }
    }
}

impl TtlValue {
    pub fn as_u32(self) -> u32 {
        match self {
            TtlValue::Auto => 1,
            TtlValue::Seconds(s) => s,
        }
    }

    pub fn from_u32(v: u32) -> Self {
        if v <= 1 {
            TtlValue::Auto
        } else {
            TtlValue::Seconds(v)
        }
    }
}

/// Record-level settings (currently IPv4/IPv6 only flags for CNAME flattening).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv4_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6_only: Option<bool>,
}

// ---------------------------------------------------------------------------
// Input structures for create / update
// ---------------------------------------------------------------------------

/// Input for creating a new DNS record.
#[derive(Debug, Clone, Serialize)]
pub struct CreateRecordRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<RecordSettings>,
}

/// Input for updating (PATCH) an existing DNS record.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateRecordRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub record_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<RecordSettings>,
}

/// Input for overwriting (PUT) an existing DNS record (all fields required).
pub type OverwriteRecordRequest = CreateRecordRequest;

// ---------------------------------------------------------------------------
// Record types enum (for validation / display)
// ---------------------------------------------------------------------------

/// Commonly used DNS record types supported by the Cloudflare API.
///
/// This covers the 12 most frequently used types. The API accepts any valid
/// DNS record type string, so rare types can still be used via raw strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum RecordType {
    A,
    AAAA,
    CAA,
    CNAME,
    HTTPS,
    MX,
    NS,
    PTR,
    SRV,
    SVCB,
    TXT,
}

impl RecordType {
    /// Commonly used record types for CLI help/validation hints.
    pub const COMMON: &'static [&'static str] = &[
        "A", "AAAA", "CAA", "CNAME", "HTTPS", "MX", "NS", "PTR", "SRV", "SVCB", "TXT",
    ];

    /// Returns true if this record type supports the `proxied` field.
    pub fn supports_proxy(type_str: &str) -> bool {
        matches!(type_str, "A" | "AAAA" | "CNAME")
    }

    /// Returns true if this record type uses the `data` field instead of `content`.
    pub fn uses_data_field(type_str: &str) -> bool {
        matches!(type_str, "CAA" | "HTTPS" | "MX" | "SRV" | "SVCB")
    }
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::A => "A",
            Self::AAAA => "AAAA",
            Self::CAA => "CAA",
            Self::CNAME => "CNAME",
            Self::HTTPS => "HTTPS",
            Self::MX => "MX",
            Self::NS => "NS",
            Self::PTR => "PTR",
            Self::SRV => "SRV",
            Self::SVCB => "SVCB",
            Self::TXT => "TXT",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// List query options
// ---------------------------------------------------------------------------

/// Query parameters for the List DNS Records endpoint.
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub record_type: Option<String>,
    pub name: Option<ListFilter>,
    pub content: Option<ListFilter>,
    pub comment: Option<CommentFilter>,
    pub tag: Option<TagFilter>,
    pub proxied: Option<bool>,
    pub search: Option<String>,
    pub order: Option<String>,
    pub direction: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub match_mode: Option<String>,
    pub tag_match: Option<String>,
}

/// Structured filter with contains/exact/startswith/endswith variants.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    pub exact: Option<String>,
    pub contains: Option<String>,
    pub startswith: Option<String>,
    pub endswith: Option<String>,
}

/// Comment-specific filter with additional absent/present variants.
#[derive(Debug, Clone, Default)]
pub struct CommentFilter {
    pub exact: Option<String>,
    pub contains: Option<String>,
    pub startswith: Option<String>,
    pub endswith: Option<String>,
    pub absent: bool,
    pub present: bool,
}

/// Tag-specific filter.
#[derive(Debug, Clone, Default)]
pub struct TagFilter {
    pub exact: Option<String>,
    pub contains: Option<String>,
    pub startswith: Option<String>,
    pub endswith: Option<String>,
    pub absent: Option<String>,
    pub present: Option<String>,
}

impl ListOptions {
    /// Convert options into URL query parameters.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref t) = self.record_type {
            params.push(("type".into(), t.clone()));
        }

        // Name filters
        if let Some(ref f) = self.name {
            Self::push_filter(&mut params, "name", f);
        }

        // Content filters
        if let Some(ref f) = self.content {
            Self::push_filter(&mut params, "content", f);
        }

        // Comment filters
        if let Some(ref cf) = self.comment {
            if let Some(ref v) = cf.exact {
                params.push(("comment.exact".into(), v.clone()));
            }
            if let Some(ref v) = cf.contains {
                params.push(("comment.contains".into(), v.clone()));
            }
            if let Some(ref v) = cf.startswith {
                params.push(("comment.startswith".into(), v.clone()));
            }
            if let Some(ref v) = cf.endswith {
                params.push(("comment.endswith".into(), v.clone()));
            }
            if cf.absent {
                params.push(("comment.absent".into(), String::new()));
            }
            if cf.present {
                params.push(("comment.present".into(), String::new()));
            }
        }

        // Tag filters
        if let Some(ref tf) = self.tag {
            if let Some(ref v) = tf.exact {
                params.push(("tag.exact".into(), v.clone()));
            }
            if let Some(ref v) = tf.contains {
                params.push(("tag.contains".into(), v.clone()));
            }
            if let Some(ref v) = tf.startswith {
                params.push(("tag.startswith".into(), v.clone()));
            }
            if let Some(ref v) = tf.endswith {
                params.push(("tag.endswith".into(), v.clone()));
            }
            if let Some(ref v) = tf.absent {
                params.push(("tag.absent".into(), v.clone()));
            }
            if let Some(ref v) = tf.present {
                params.push(("tag.present".into(), v.clone()));
            }
        }

        if let Some(ref v) = self.search {
            params.push(("search".into(), v.clone()));
        }
        if let Some(p) = self.proxied {
            params.push(("proxied".into(), p.to_string()));
        }
        if let Some(ref v) = self.order {
            params.push(("order".into(), v.clone()));
        }
        if let Some(ref v) = self.direction {
            params.push(("direction".into(), v.clone()));
        }
        if let Some(p) = self.page {
            params.push(("page".into(), p.to_string()));
        }
        if let Some(pp) = self.per_page {
            params.push(("per_page".into(), pp.to_string()));
        }
        if let Some(ref m) = self.match_mode {
            params.push(("match".into(), m.clone()));
        }
        if let Some(ref tm) = self.tag_match {
            params.push(("tag_match".into(), tm.clone()));
        }

        params
    }

    fn push_filter(params: &mut Vec<(String, String)>, prefix: &str, f: &ListFilter) {
        if let Some(ref v) = f.exact {
            params.push((format!("{prefix}.exact"), v.clone()));
        }
        if let Some(ref v) = f.contains {
            params.push((format!("{prefix}.contains"), v.clone()));
        }
        if let Some(ref v) = f.startswith {
            params.push((format!("{prefix}.startswith"), v.clone()));
        }
        if let Some(ref v) = f.endswith {
            params.push((format!("{prefix}.endswith"), v.clone()));
        }
    }
}
