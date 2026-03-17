//! Tests for list, get, find, and search operations.

mod common;

use cfdns::api::CloudflareClient;
use cfdns::models::*;
use common::{mock_setup, recv_request, respond_json};
use tiny_http::Response;

// ---------------------------------------------------------------------------
// List records
// ---------------------------------------------------------------------------

#[test]
fn test_list_records_empty() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.list_records(&ListOptions::default()));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":[],"result_info":{"count":0,"page":1,"per_page":100,"total_count":0,"total_pages":1}}"#,
    );

    let (records, info) = handle.join().unwrap().expect("list failed");
    assert!(records.is_empty());
    assert!(info.is_some());
    assert_eq!(info.unwrap().total_count, Some(0));
}

#[test]
fn test_list_records_with_results() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.list_records(&ListOptions::default()));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":[
            {"id":"rec1","name":"example.com","type":"A","content":"1.2.3.4","ttl":1,"proxied":false,"proxiable":true,"tags":[]},
            {"id":"rec2","name":"www.example.com","type":"CNAME","content":"example.com","ttl":300,"proxied":true,"proxiable":true,"tags":[]}
        ],"result_info":{"count":2,"page":1,"per_page":100,"total_count":2,"total_pages":1}}"#,
    );

    let (records, _) = handle.join().unwrap().expect("list failed");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id, "rec1");
    assert_eq!(records[0].record_type, "A");
    assert_eq!(records[0].content, "1.2.3.4");
    assert_eq!(records[1].id, "rec2");
    assert_eq!(records[1].record_type, "CNAME");
    assert!(records[1].proxied);
}

#[test]
fn test_list_records_with_filters() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let opts = ListOptions {
        record_type: Some("A".into()),
        name: Some(ListFilter {
            exact: Some("example.com".into()),
            ..Default::default()
        }),
        per_page: Some(10),
        ..Default::default()
    };

    let handle = std::thread::spawn(move || client.list_records(&opts));

    let (method, url, _body) = recv_request(&server);
    assert_eq!(method, "GET");
    assert!(url.contains("type=A"));
    assert!(url.contains("name.exact=example.com"));
    assert!(url.contains("per_page=10"));

    // We consumed the request already via recv_request so can't respond;
    // the thread will get an error, but we've verified the URL was correct.
    let _ = handle.join();
}

// ---------------------------------------------------------------------------
// Get record
// ---------------------------------------------------------------------------

#[test]
fn test_get_record() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.get_record("rec123"));

    let (method, url, _) = recv_request(&server);
    assert_eq!(method, "GET");
    assert!(url.contains("/dns_records/rec123"));

    let _ = handle.join();
}

#[test]
fn test_get_record_success() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.get_record("abc123"));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":{"id":"abc123","name":"test.example.com","type":"A","content":"10.0.0.1","ttl":1,"proxied":false,"proxiable":true,"tags":[]}}"#,
    );

    let record = handle.join().unwrap().expect("get failed");
    assert_eq!(record.id, "abc123");
    assert_eq!(record.name, "test.example.com");
    assert_eq!(record.record_type, "A");
    assert_eq!(record.content, "10.0.0.1");
}

// ---------------------------------------------------------------------------
// Find record by name
// ---------------------------------------------------------------------------

#[test]
fn test_find_record_by_name() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.find_record_by_name("www"));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":[{"id":"found-id","name":"www.example.com","type":"A","content":"3.4.5.6","ttl":1,"proxied":false,"proxiable":true,"tags":[]}],"result_info":{"count":1,"page":1,"per_page":100,"total_count":1,"total_pages":1}}"#,
    );

    let record = handle.join().unwrap().expect("find failed");
    assert_eq!(record.id, "found-id");
    assert_eq!(record.name, "www.example.com");
}

#[test]
fn test_find_record_by_name_not_found() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.find_record_by_name("nonexistent"));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":[],"result_info":{"count":0,"page":1,"per_page":100,"total_count":0,"total_pages":1}}"#,
    );

    let result = handle.join().unwrap();
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Find record by name and type
// ---------------------------------------------------------------------------

#[test]
fn test_find_record_by_name_and_type() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || client.find_record_by_name_and_type("mail", "MX"));

    respond_json(
        &server,
        r#"{"success":true,"errors":[],"messages":[],"result":[{"id":"mx-id","name":"mail.example.com","type":"MX","content":"mail.example.com","ttl":1,"proxied":false,"proxiable":false,"priority":10,"tags":[]}],"result_info":{"count":1,"page":1,"per_page":100,"total_count":1,"total_pages":1}}"#,
    );

    let record = handle.join().unwrap().expect("find failed");
    assert_eq!(record.id, "mx-id");
    assert_eq!(record.record_type, "MX");
    assert_eq!(record.priority, Some(10));
}

// ---------------------------------------------------------------------------
// Auth header verification
// ---------------------------------------------------------------------------

#[test]
fn test_auth_header_present() {
    let (server, config) = mock_setup();
    let client = CloudflareClient::new(config);

    let handle = std::thread::spawn(move || {
        client.list_records(&ListOptions {
            per_page: Some(1),
            ..Default::default()
        })
    });

    let req = server.recv().expect("no request");
    // Verify authorization header
    let auth = req
        .headers()
        .iter()
        .find(|h| h.field.equiv("Authorization"));
    assert!(auth.is_some());
    assert_eq!(auth.unwrap().value.as_str(), "Bearer test-token");

    let response = Response::from_string(
        r#"{"success":true,"errors":[],"messages":[],"result":[],"result_info":{"count":0,"page":1,"per_page":1,"total_count":0,"total_pages":1}}"#,
    );
    req.respond(response).unwrap();

    let _ = handle.join().unwrap();
}
