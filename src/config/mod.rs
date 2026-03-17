//! Configuration management: environment variables, config files, and profiles.

use crate::models::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Resolved runtime configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub api_token: String,
    pub zone_id: String,
    pub domain_name: String,
    pub base_url: String,
}

/// On-disk configuration file structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

/// A named profile within the config file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub api_token: Option<String>,
    pub zone_id: Option<String>,
    pub domain_name: Option<String>,
    pub base_url: Option<String>,
}

impl Config {
    /// Standard Cloudflare API v4 base URL.
    const DEFAULT_BASE_URL: &'static str = "https://api.cloudflare.com/client/v4";

    /// Build configuration by layering:
    ///   1. Config file defaults
    ///   2. Config file profile (if selected)
    ///   3. Environment variables (highest priority)
    pub fn load(profile_name: Option<&str>) -> Result<Self> {
        let file_config = Self::load_config_file();

        // Determine which profile to use
        let profile = file_config.as_ref().and_then(|fc| {
            let name = profile_name
                .map(String::from)
                .or_else(|| std::env::var("CFDNS_PROFILE").ok())
                .or_else(|| fc.default_profile.clone())
                .unwrap_or_else(|| "default".into());
            fc.profiles.get(&name).cloned()
        });

        let p = profile.unwrap_or_default();

        let api_token = std::env::var("CLOUDFLARE_API_TOKEN")
            .ok()
            .or(p.api_token)
            .unwrap_or_default();

        let zone_id = std::env::var("CLOUDFLARE_ZONE_ID")
            .ok()
            .or(p.zone_id)
            .unwrap_or_default();

        let domain_name = std::env::var("CLOUDFLARE_DOMAIN_NAME")
            .ok()
            .or(p.domain_name)
            .unwrap_or_default();

        let base_url = std::env::var("CLOUDFLARE_BASE_URL")
            .ok()
            .or(p.base_url)
            .unwrap_or_else(|| Self::DEFAULT_BASE_URL.into());

        Ok(Config {
            api_token,
            zone_id,
            domain_name,
            base_url,
        })
    }

    /// Validate that required fields are present.
    pub fn validate(&self) -> Result<()> {
        if self.api_token.is_empty() {
            return Err(AppError::Config(
                "CLOUDFLARE_API_TOKEN is not set. Set it via environment variable or config file."
                    .into(),
            ));
        }
        if self.zone_id.is_empty() {
            return Err(AppError::Config(
                "CLOUDFLARE_ZONE_ID is not set. Set it via environment variable or config file."
                    .into(),
            ));
        }
        Ok(())
    }

    /// Resolve a subdomain argument to a fully qualified domain name.
    pub fn resolve_fqdn(&self, name: &str) -> String {
        if self.domain_name.is_empty() || name.ends_with(&self.domain_name) {
            return name.to_string();
        }
        format!("{name}.{}", self.domain_name)
    }

    /// Return the config file path: `~/.config/cfdns/config.toml`.
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("cfdns").join("config.toml"))
    }

    /// Load the config file from disk, returning `None` if it doesn't exist.
    fn load_config_file() -> Option<ConfigFile> {
        // Check local project config first
        let local = PathBuf::from(".cfdns.toml");
        if local.is_file() {
            if let Ok(data) = std::fs::read_to_string(&local) {
                if let Ok(cf) = toml::from_str(&data) {
                    return Some(cf);
                }
            }
        }

        // Then check global config
        let path = Self::config_path()?;
        let data = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&data).ok()
    }

    /// Write config file to a custom path, optionally with provided values.
    pub fn write_config_to_path(
        path: &PathBuf,
        values: Option<(String, String, String)>,
    ) -> Result<PathBuf> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let (api_token, zone_id, domain_name) = values.unwrap_or_else(|| {
            (
                "your-api-token-here".into(),
                "your-zone-id-here".into(),
                "example.com".into(),
            )
        });

        let example = ConfigFile {
            default_profile: Some("default".into()),
            profiles: {
                let mut m = HashMap::new();
                m.insert(
                    "default".into(),
                    Profile {
                        api_token: Some(api_token),
                        zone_id: Some(zone_id),
                        domain_name: Some(domain_name),
                        base_url: None,
                    },
                );
                m
            },
        };

        let content = toml::to_string_pretty(&example)
            .map_err(|e| AppError::Other(format!("Failed to serialize config: {e}")))?;
        std::fs::write(path, content)?;
        Ok(path.clone())
    }

    /// Resolve config path: use provided path or default.
    pub fn resolve_config_path(custom_path: Option<&str>) -> Result<PathBuf> {
        match custom_path {
            Some(p) => Ok(PathBuf::from(p)),
            None => Self::config_path()
                .ok_or_else(|| AppError::Config("Cannot determine config directory".into())),
        }
    }
}
