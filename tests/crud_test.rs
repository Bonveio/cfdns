//! Tests for create, update, overwrite, and delete operations.

mod common;

use cfdns::api::CloudflareClient;
use cfdns::models::*;
use common::{mock_setup, recv_request, respond_json};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Create record
// ---------------------------------------------------------------------------

#[test]
fn test_create_record() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let req = CreateRecordRequest {
        name: "test.example.com".into(),
        record_type: "A".into(),
        content: Some("192.168.1.1".into()),
        ttl: 1,
        proxied: Some(false),
        priority: None,
        comment: Some("test record".into()),
        tags: vec!["env:test".into()],
        data: None,
        settings: None,
    };

    let handle = std::thread::spawn(move || client.create_record(&req));

    // Verify the request
    let (method, url, body) = recv_request(&server);
    assert_eq!(method, "POST");
    assert!(url.ends_with("/dns_records"));

    // Parse and verify request body
    let body_json: serde_json::Value = serde_json::from_str(&body).expect("invalid json body");
    assert_eq!(body_json["name"], "test.example.com");
    assert_eq!(body_json["type"], "A");
    assert_eq!(body_json["content"], "192.168.1.1");
    assert_eq!(body_json["comment"], "test record");

    // We consumed the request already via recv_request, thread will error
    let _ = handle.join();
}

#[test]
fn test_create_record_success() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let req = CreateRecordRequest {
        name: "new.example.com".into(),
        record_type: "A".into(),
        content: Some("5.6.7.8".into()),
        ttl: 300,
        proxied: Some(false),
        priority: None,
        comment: None,
        tags: vec![],
        data: None,
        settings: None,
    };

    let handle = std::thread::spawn(move || client.create_record(&req));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":{"id":"new-id","name":"new.example.com","type":"A","content":"5.6.7.8","ttl":300,"proxied":false,"proxiable":true,"tags":[]}}"#,
    );

    let record = handle.join().unwrap().expect("create failed");
    assert_eq!(record.id, "new-id");
    assert_eq!(record.name, "new.example.com");
    assert_eq!(record.content, "5.6.7.8");
    assert_eq!(record.ttl, TtlValue::Seconds(300));
}

// ---------------------------------------------------------------------------
// Update record (PATCH)
// ---------------------------------------------------------------------------

#[test]
fn test_update_record() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let req = UpdateRecordRequest {
        content: Some("9.8.7.6".into()),
        comment: Some("updated".into()),
        ..Default::default()
    };

    let handle = std::thread::spawn(move || client.update_record("rec-update", &req));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":{"id":"rec-update","name":"up.example.com","type":"A","content":"9.8.7.6","ttl":1,"proxied":false,"proxiable":true,"tags":[],"comment":"updated"}}"#,
    );

    let record = handle.join().unwrap().expect("update failed");
    assert_eq!(record.id, "rec-update");
    assert_eq!(record.content, "9.8.7.6");
    assert_eq!(record.comment, Some("updated".into()));
}

// ---------------------------------------------------------------------------
// Overwrite record (PUT)
// ---------------------------------------------------------------------------

#[test]
fn test_overwrite_record() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let req = CreateRecordRequest {
        name: "ow.example.com".into(),
        record_type: "A".into(),
        content: Some("10.0.0.1".into()),
        ttl: 600,
        proxied: Some(false),
        priority: None,
        comment: None,
        tags: vec![],
        data: None,
        settings: None,
    };

    let handle = std::thread::spawn(move || client.overwrite_record("ow-id", &req));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":{"id":"ow-id","name":"ow.example.com","type":"A","content":"10.0.0.1","ttl":600,"proxied":false,"proxiable":true,"tags":[]}}"#,
    );

    let record = handle.join().unwrap().expect("overwrite failed");
    assert_eq!(record.id, "ow-id");
    assert_eq!(record.content, "10.0.0.1");
    assert_eq!(record.ttl, TtlValue::Seconds(600));
}

// ---------------------------------------------------------------------------
// Delete record
// ---------------------------------------------------------------------------

#[test]
fn test_delete_record() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.delete_record("rec-del"));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":{"id":"rec-del"}}"#,
    );

    let result = handle.join().unwrap().expect("delete failed");
    assert_eq!(result["id"], "rec-del");
}

// ---------------------------------------------------------------------------
// API error handling
// ---------------------------------------------------------------------------

#[test]
fn test_api_error() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.get_record("bad-id"));

    respond_json(
        &server,
        r#"{"success":false,"errors":[{"code":7003,"message":"Could not route to /zones/test-zone-id/dns_records/bad-id, perhaps your object identifier is invalid?"}],"messages":[],"result":null}"#,
    );

    let result = handle.join().unwrap();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("7003"));
}

// ---------------------------------------------------------------------------
// CreateRecordRequest serialization
// ---------------------------------------------------------------------------

#[test]
fn test_create_request_serialization() {
    let req = CreateRecordRequest {
        name: "test.com".into(),
        record_type: "A".into(),
        content: Some("1.2.3.4".into()),
        ttl: 1,
        proxied: Some(true),
        priority: None,
        comment: None,
        tags: vec![],
        data: None,
        settings: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["name"], "test.com");
    assert_eq!(parsed["type"], "A");
    assert_eq!(parsed["proxied"], true);
    // skip_serializing_if should omit None fields
    assert!(parsed.get("priority").is_none());
    assert!(parsed.get("data").is_none());
    assert!(parsed.get("settings").is_none());
}

#[test]
fn test_create_request_with_data() {
    let mut data = HashMap::new();
    data.insert("flags".into(), serde_json::json!(0));
    data.insert("tag".into(), serde_json::json!("issue"));
    data.insert("value".into(), serde_json::json!("letsencrypt.org"));

    let req = CreateRecordRequest {
        name: "example.com".into(),
        record_type: "CAA".into(),
        content: None,
        ttl: 1,
        proxied: None,
        priority: None,
        comment: None,
        tags: vec![],
        data: Some(data),
        settings: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["data"]["tag"], "issue");
    assert_eq!(parsed["data"]["value"], "letsencrypt.org");
}

// ---------------------------------------------------------------------------
// UpdateRecordRequest serialization (skip_serializing_if)
// ---------------------------------------------------------------------------

#[test]
fn test_update_request_minimal() {
    let req = UpdateRecordRequest {
        content: Some("new-ip".into()),
        ..Default::default()
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["content"], "new-ip");
    // All other fields should be absent
    assert!(parsed.get("name").is_none());
    assert!(parsed.get("type").is_none());
    assert!(parsed.get("ttl").is_none());
}
