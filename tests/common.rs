//! Shared test utilities and mock server setup.

use cfdns::config::Config;
use tiny_http::{Response, Server};

/// Create a mock server and return (server, Config pointing at it).
pub fn mock_setup() -> (Server, Config) {
    let server = Server::http("127.0.0.1:0").expect("failed to start mock server");
    let addr = server.server_addr().to_ip().expect("no ip addr");
    let base_url = format!("http://127.0.0.1:{}", addr.port());
    let config = Config {
        api_token: "test-token".into(),
        zone_id: "test-zone-id".into(),
        domain_name: "example.com".into(),
        base_url,
    };
    (server, config)
}

/// Helper: respond to a single request with a JSON body.
pub fn respond_json(server: &Server, body: &str) {
    let req = server.recv().expect("no request received");
    let response = Response::from_string(body).with_header(
        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
    );
    req.respond(response).expect("failed to respond");
}

/// Helper: receive a request and return (method, url, body).
pub fn recv_request(server: &Server) -> (String, String, String) {
    let mut req = server.recv().expect("no request received");
    let method = req.method().to_string();
    let url = req.url().to_string();
    let mut body = String::new();
    req.as_reader().read_to_string(&mut body).unwrap_or(0);
    (method, url, body)
}
