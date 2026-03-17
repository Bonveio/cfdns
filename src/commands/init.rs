//! Init command handler.

use crate::cli;
use crate::config::Config;
use crate::models::*;

/// Initialize a config file with example values.
pub fn cmd_init(args: cli::InitArgs) -> Result<()> {
    let path = Config::resolve_config_path(args.path.as_deref())?;

    if args.interactive {
        let api_token = rpassword::prompt_password("Enter Cloudflare API Token: ")
            .map_err(|e| AppError::Other(format!("Failed to read password: {e}")))?;
        let zone_id = rpassword::prompt_password("Enter Cloudflare Zone ID: ")
            .map_err(|e| AppError::Other(format!("Failed to read password: {e}")))?;
        let domain_name = rpassword::prompt_password("Enter Domain Name (e.g., example.com): ")
            .map_err(|e| AppError::Other(format!("Failed to read password: {e}")))?;

        let final_path =
            Config::write_config_to_path(&path, Some((api_token, zone_id, domain_name)))?;
        println!("Config file written to: {}", final_path.display());
    } else {
        let final_path = Config::write_config_to_path(&path, None)?;
        println!("Config file written to: {}", final_path.display());
        println!("Edit it with your Cloudflare API token and zone ID.");
    }
    Ok(())
}
