//! Tests for model serialization, deserialization, config, and helpers.

use cfdns::config::Config;
use cfdns::models::*;

// ---------------------------------------------------------------------------
// Config validation
// ---------------------------------------------------------------------------

#[test]
fn test_config_validation_missing_token() {
    let config = Config {
        api_token: String::new(),
        zone_id: "zone".into(),
        domain_name: "example.com".into(),
        base_url: "http://localhost".into(),
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validation_missing_zone() {
    let config = Config {
        api_token: "token".into(),
        zone_id: String::new(),
        domain_name: "example.com".into(),
        base_url: "http://localhost".into(),
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validation_ok() {
    let config = Config {
        api_token: "token".into(),
        zone_id: "zone".into(),
        domain_name: "example.com".into(),
        base_url: "http://localhost".into(),
    };
    assert!(config.validate().is_ok());
}

// ---------------------------------------------------------------------------
// FQDN resolution
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_fqdn_subdomain() {
    let config = Config {
        api_token: "t".into(),
        zone_id: "z".into(),
        domain_name: "example.com".into(),
        base_url: "http://localhost".into(),
    };
    assert_eq!(config.resolve_fqdn("www"), "www.example.com");
}

#[test]
fn test_resolve_fqdn_already_full() {
    let config = Config {
        api_token: "t".into(),
        zone_id: "z".into(),
        domain_name: "example.com".into(),
        base_url: "http://localhost".into(),
    };
    assert_eq!(config.resolve_fqdn("sub.example.com"), "sub.example.com");
}

#[test]
fn test_resolve_fqdn_no_domain() {
    let config = Config {
        api_token: "t".into(),
        zone_id: "z".into(),
        domain_name: String::new(),
        base_url: "http://localhost".into(),
    };
    assert_eq!(config.resolve_fqdn("whatever"), "whatever");
}

// ---------------------------------------------------------------------------
// ListOptions query string generation
// ---------------------------------------------------------------------------

#[test]
fn test_list_options_empty() {
    let opts = ListOptions::default();
    assert!(opts.to_query_pairs().is_empty());
}

#[test]
fn test_list_options_full() {
    let opts = ListOptions {
        record_type: Some("A".into()),
        name: Some(ListFilter {
            exact: Some("example.com".into()),
            contains: Some("exam".into()),
            ..Default::default()
        }),
        content: Some(ListFilter {
            startswith: Some("192".into()),
            ..Default::default()
        }),
        comment: Some(CommentFilter {
            present: true,
            ..Default::default()
        }),
        tag: Some(TagFilter {
            exact: Some("env:prod".into()),
            ..Default::default()
        }),
        proxied: Some(true),
        search: Some("test".into()),
        order: Some("name".into()),
        direction: Some("asc".into()),
        page: Some(2),
        per_page: Some(50),
        match_mode: Some("all".into()),
        tag_match: Some("any".into()),
    };

    let pairs = opts.to_query_pairs();
    let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();

    assert!(keys.contains(&"type"));
    assert!(keys.contains(&"name.exact"));
    assert!(keys.contains(&"name.contains"));
    assert!(keys.contains(&"content.startswith"));
    assert!(keys.contains(&"comment.present"));
    assert!(keys.contains(&"tag.exact"));
    assert!(keys.contains(&"proxied"));
    assert!(keys.contains(&"search"));
    assert!(keys.contains(&"order"));
    assert!(keys.contains(&"direction"));
    assert!(keys.contains(&"page"));
    assert!(keys.contains(&"per_page"));
    assert!(keys.contains(&"match"));
    assert!(keys.contains(&"tag_match"));
}

// ---------------------------------------------------------------------------
// TtlValue serialization
// ---------------------------------------------------------------------------

#[test]
fn test_ttl_value_auto() {
    let ttl = TtlValue::Auto;
    assert_eq!(format!("{ttl}"), "auto");
    let json = serde_json::to_string(&ttl).unwrap();
    assert_eq!(json, "1");
}

#[test]
fn test_ttl_value_seconds() {
    let ttl = TtlValue::Seconds(300);
    assert_eq!(format!("{ttl}"), "300");
    let json = serde_json::to_string(&ttl).unwrap();
    assert_eq!(json, "300");
}

#[test]
fn test_ttl_value_deserialize() {
    let ttl: TtlValue = serde_json::from_str("1").unwrap();
    assert_eq!(ttl, TtlValue::Auto);
    let ttl: TtlValue = serde_json::from_str("3600").unwrap();
    assert_eq!(ttl, TtlValue::Seconds(3600));
}

// ---------------------------------------------------------------------------
// RecordType helpers
// ---------------------------------------------------------------------------

#[test]
fn test_supports_proxy() {
    assert!(RecordType::supports_proxy("A"));
    assert!(RecordType::supports_proxy("AAAA"));
    assert!(RecordType::supports_proxy("CNAME"));
    assert!(!RecordType::supports_proxy("MX"));
    assert!(!RecordType::supports_proxy("TXT"));
    assert!(!RecordType::supports_proxy("SRV"));
}

#[test]
fn test_uses_data_field() {
    assert!(RecordType::uses_data_field("SRV"));
    assert!(RecordType::uses_data_field("CAA"));
    assert!(RecordType::uses_data_field("MX"));
    assert!(RecordType::uses_data_field("HTTPS"));
    assert!(!RecordType::uses_data_field("A"));
    assert!(!RecordType::uses_data_field("CNAME"));
    assert!(!RecordType::uses_data_field("TXT"));
}

// ---------------------------------------------------------------------------
// DnsRecord deserialization
// ---------------------------------------------------------------------------

#[test]
fn test_dns_record_deserialization_minimal() {
    let json =
        r#"{"id":"x","name":"a.com","type":"A","content":"1.1.1.1","ttl":1,"proxied":false}"#;
    let record: DnsRecord = serde_json::from_str(json).unwrap();
    assert_eq!(record.id, "x");
    assert_eq!(record.name, "a.com");
    assert_eq!(record.record_type, "A");
    assert_eq!(record.content, "1.1.1.1");
    assert_eq!(record.ttl, TtlValue::Auto);
    assert!(!record.proxied);
}

#[test]
fn test_dns_record_deserialization_with_data() {
    let json = r#"{
        "id":"srv1",
        "name":"_sip._tcp.example.com",
        "type":"SRV",
        "content":"",
        "ttl":3600,
        "proxied":false,
        "priority":10,
        "data":{"weight":60,"port":5060,"target":"sip.example.com"},
        "tags":["env:prod","tier:1"]
    }"#;
    let record: DnsRecord = serde_json::from_str(json).unwrap();
    assert_eq!(record.record_type, "SRV");
    assert_eq!(record.priority, Some(10));
    assert_eq!(record.tags.len(), 2);
    let data = record.data.unwrap();
    assert_eq!(data["port"], 5060);
    assert_eq!(data["target"], "sip.example.com");
}
