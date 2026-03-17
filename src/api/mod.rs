//! Cloudflare DNS API client using ureq (synchronous, pure-Rust TLS).

use crate::config::Config;
use crate::models::*;

use std::time::Duration;

/// Synchronous HTTP client for the Cloudflare DNS API.
pub struct CloudflareClient {
    config: Config,
    agent: ureq::Agent,
}

/// HTTP method for API requests.
enum Method {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

impl CloudflareClient {
    pub fn new(config: Config) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .build();
        Self { config, agent }
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.config.base_url)
    }

    /// Generic JSON API request that handles response parsing and error checking.
    fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<T> {
        let url = self.url(path);
        let auth = format!("Bearer {}", self.config.api_token);

        let req = match method {
            Method::Get => self.agent.get(&url),
            Method::Post => self.agent.post(&url),
            Method::Patch => self.agent.request("PATCH", &url),
            Method::Put => self.agent.put(&url),
            Method::Delete => self.agent.delete(&url),
        }
        .set("Authorization", &auth);

        let resp = if let Some(b) = body {
            let json = serde_json::to_string(b).map_err(AppError::Json)?;
            req.set("Content-Type", "application/json")
                .send_string(&json)
        } else {
            req.call()
        }
        .map_err(|e| AppError::Http(e.to_string()))?;

        let body_str = resp
            .into_string()
            .map_err(|e| AppError::Http(e.to_string()))?;

        let api_resp: ApiResponse<T> = serde_json::from_str(&body_str)
            .map_err(|e| AppError::Http(format!("Parse error: {e}\nBody: {body_str}")))?;

        self.check_errors(&api_resp)?;
        api_resp
            .result
            .ok_or_else(|| AppError::Http("API returned success but no result".into()))
    }

    /// GET request returning full ApiResponse (for list operations with pagination).
    fn get_response<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<ApiResponse<T>> {
        let url = self.url(path);
        let auth = format!("Bearer {}", self.config.api_token);

        let resp = self
            .agent
            .get(&url)
            .set("Authorization", &auth)
            .call()
            .map_err(|e| AppError::Http(e.to_string()))?;

        let body_str = resp
            .into_string()
            .map_err(|e| AppError::Http(e.to_string()))?;

        let api_resp: ApiResponse<T> = serde_json::from_str(&body_str)
            .map_err(|e| AppError::Http(format!("Parse error: {e}\nBody: {body_str}")))?;

        self.check_errors(&api_resp)?;
        Ok(api_resp)
    }

    fn check_errors<T>(&self, resp: &ApiResponse<T>) -> Result<()> {
        if !resp.success {
            let first = resp.errors.first();
            return Err(AppError::Api(ApiError {
                code: first.map_or(0, |e| e.code),
                message: first.map_or_else(|| "Unknown API error".into(), |e| e.message.clone()),
                errors: resp.errors.clone(),
            }));
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Public DNS record operations
    // ------------------------------------------------------------------

    /// List DNS records with optional filters.
    pub fn list_records(&self, opts: &ListOptions) -> Result<(Vec<DnsRecord>, Option<ResultInfo>)> {
        self.config.validate()?;
        let qs = build_query_string(&opts.to_query_pairs());
        let path = format!("/zones/{}/dns_records{qs}", self.config.zone_id);
        let resp: ApiResponse<Vec<DnsRecord>> = self.get_response(&path)?;
        Ok((resp.result.unwrap_or_default(), resp.result_info))
    }

    /// Get a single DNS record by ID.
    pub fn get_record(&self, record_id: &str) -> Result<DnsRecord> {
        self.config.validate()?;
        let path = format!("/zones/{}/dns_records/{record_id}", self.config.zone_id);
        self.request::<DnsRecord>(Method::Get, &path, None::<&()>)
    }

    /// Create a new DNS record.
    pub fn create_record(&self, req: &CreateRecordRequest) -> Result<DnsRecord> {
        self.config.validate()?;
        let path = format!("/zones/{}/dns_records", self.config.zone_id);
        self.request(Method::Post, &path, Some(req))
    }

    /// Update (PATCH) an existing DNS record.
    pub fn update_record(&self, record_id: &str, req: &UpdateRecordRequest) -> Result<DnsRecord> {
        self.config.validate()?;
        let path = format!("/zones/{}/dns_records/{record_id}", self.config.zone_id);
        self.request(Method::Patch, &path, Some(req))
    }

    /// Overwrite (PUT) an existing DNS record.
    pub fn overwrite_record(
        &self,
        record_id: &str,
        req: &OverwriteRecordRequest,
    ) -> Result<DnsRecord> {
        self.config.validate()?;
        let path = format!("/zones/{}/dns_records/{record_id}", self.config.zone_id);
        self.request(Method::Put, &path, Some(req))
    }

    /// Delete a DNS record by ID.
    pub fn delete_record(&self, record_id: &str) -> Result<serde_json::Value> {
        self.config.validate()?;
        let path = format!("/zones/{}/dns_records/{record_id}", self.config.zone_id);
        self.request::<serde_json::Value>(Method::Delete, &path, None::<&()>)
    }

    /// Find a record by name (convenience helper).
    pub fn find_record_by_name(&self, name: &str) -> Result<DnsRecord> {
        let fqdn = self.config.resolve_fqdn(name);
        let opts = ListOptions {
            name: Some(ListFilter {
                exact: Some(fqdn.clone()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (records, _) = self.list_records(&opts)?;
        records
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Other(format!("No DNS record found for '{fqdn}'")))
    }

    /// Find a record by name and type (convenience helper).
    pub fn find_record_by_name_and_type(&self, name: &str, record_type: &str) -> Result<DnsRecord> {
        let fqdn = self.config.resolve_fqdn(name);
        let opts = ListOptions {
            name: Some(ListFilter {
                exact: Some(fqdn.clone()),
                ..Default::default()
            }),
            record_type: Some(record_type.to_string()),
            ..Default::default()
        };
        let (records, _) = self.list_records(&opts)?;
        records
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Other(format!("No {record_type} record found for '{fqdn}'")))
    }
}

/// Build URL query string from key-value pairs.
fn build_query_string(pairs: &[(String, String)]) -> String {
    if pairs.is_empty() {
        return String::new();
    }
    let encoded: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect();
    format!("?{}", encoded.join("&"))
}

/// Minimal percent-encoding for URL query parameters.
fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            _ => {
                result.push_str(&format!("%{b:02X}"));
            }
        }
    }
    result
}
