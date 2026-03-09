use crate::utils::{file, platform, log_sanitizer};
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use tauri::command;

pub mod ai;
pub mod mcp;
pub mod channels;
pub mod agents;
pub mod settings;

pub use ai::*;
pub use mcp::*;
pub use channels::*;
pub use agents::*;
pub use settings::*;

/// Maximum allowed size for the config file (10 MB).  Rejects oversized input before
/// parsing to prevent unbounded memory allocation (DoS / malformed-file attack).
pub(super) const MAX_CONFIG_SIZE: usize = 10 * 1024 * 1024;

/// Load openclaw.json configuration
pub(super) fn load_openclaw_config() -> Result<Value, String> {
    let config_path = platform::get_config_file_path();

    if !file::file_exists(&config_path) {
        return Ok(json!({}));
    }

    let content =
        file::read_file(&config_path).map_err(|e| format!("Failed to read configuration file: {}", e))?;

    if content.len() > MAX_CONFIG_SIZE {
        return Err(format!(
            "Configuration file exceeds maximum allowed size ({} MB)",
            MAX_CONFIG_SIZE / 1024 / 1024
        ));
    }

    // Strip UTF-8 BOM if present (Windows editors sometimes add this)
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    serde_json::from_str(content).map_err(|e| format!("Failed to parse configuration file: {}", e))
}

/// Save openclaw.json configuration
pub(super) fn save_openclaw_config(config: &Value) -> Result<(), String> {
    let config_path = platform::get_config_file_path();

    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize configuration: {}", e))?;

    file::write_file(&config_path, &content).map_err(|e| format!("Failed to write configuration file: {}", e))
}

/// Load manager.json configuration (manager-specific settings)
pub(super) fn load_manager_config() -> Result<Value, String> {
    let config_path = platform::get_manager_config_file_path();

    if !file::file_exists(&config_path) {
        return Ok(json!({}));
    }

    let content =
        file::read_file(&config_path).map_err(|e| format!("Failed to read manager configuration file: {}", e))?;

    // Strip UTF-8 BOM if present
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    serde_json::from_str(content).map_err(|e| format!("Failed to parse manager configuration file: {}", e))
}

/// Save manager.json configuration
pub(super) fn save_manager_config(config: &Value) -> Result<(), String> {
    let config_path = platform::get_manager_config_file_path();

    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize manager configuration: {}", e))?;

    file::write_file(&config_path, &content).map_err(|e| format!("Failed to write manager configuration file: {}", e))
}

/// Get complete configuration
#[command]
pub async fn get_config() -> Result<Value, String> {
    info!("[Get Config] Reading openclaw.json configuration...");
    let result = load_openclaw_config();
    match &result {
        Ok(_) => info!("[Get Config] Configuration read successfully"),
        Err(e) => error!("[Get Config] Failed to read configuration: {}", e),
    }
    result
}

/// Save configuration
#[command]
pub async fn save_config(config: Value) -> Result<String, String> {
    info!("[Save Config] Saving openclaw.json configuration...");
    debug!(
        "[Save Config] Configuration content: {}",
        log_sanitizer::sanitize(&serde_json::to_string_pretty(&config).unwrap_or_default())
    );
    match save_openclaw_config(&config) {
        Ok(_) => {
            info!("[Save Config] Configuration saved successfully");
            Ok("Configuration saved".to_string())
        }
        Err(e) => {
            error!("[Save Config] Failed to save configuration: {}", e);
            Err(e)
        }
    }
}

/// Get environment variable value
#[command]
pub async fn get_env_value(key: String) -> Result<Option<String>, String> {
    info!("[Get Env] Reading environment variable: {}", key);
    let env_path = platform::get_env_file_path();
    let value = file::read_env_value(&env_path, &key);
    match &value {
        Some(v) => debug!(
            "[Get Env] {}={} (masked)",
            key,
            if v.len() > 8 { "***" } else { v }
        ),
        None => debug!("[Get Env] {} does not exist", key),
    }
    Ok(value)
}

/// Save environment variable value
#[command]
pub async fn save_env_value(key: String, value: String) -> Result<String, String> {
    info!("[Save Env] Saving environment variable: {}", key);
    let env_path = platform::get_env_file_path();
    debug!("[Save Env] Environment file path: {}", env_path);

    match file::set_env_value(&env_path, &key, &value) {
        Ok(_) => {
            info!("[Save Env] Environment variable {} saved successfully", key);
            Ok("Environment variable saved".to_string())
        }
        Err(e) => {
            error!("[Save Env] Failed to save: {}", e);
            Err(format!("Failed to save environment variable: {}", e))
        }
    }
}

// ============ Gateway Token Commands ============

/// Generate a cryptographically secure 256-bit hex token using the OS CSPRNG.
fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Get or create Gateway Token
#[command]
pub async fn get_or_create_gateway_token() -> Result<String, String> {
    info!("[Gateway Token] Getting or creating Gateway Token...");

    let mut config = load_openclaw_config()?;

    // Check if token already exists
    if let Some(token) = config
        .pointer("/gateway/auth/token")
        .and_then(|v| v.as_str())
    {
        if !token.is_empty() {
            info!("[Gateway Token] Using existing Token");
            return Ok(token.to_string());
        }
    }

    // Generate new token
    let new_token = generate_token();
    info!("[Gateway Token] Generated new Token");

    // Ensure path exists
    if config.get("gateway").is_none() {
        config["gateway"] = json!({});
    }
    if config["gateway"].get("auth").is_none() {
        config["gateway"]["auth"] = json!({});
    }

    // Set token and mode
    config["gateway"]["auth"]["token"] = json!(new_token);
    config["gateway"]["auth"]["mode"] = json!("token");
    config["gateway"]["mode"] = json!("local");

    // Save configuration
    save_openclaw_config(&config)?;

    info!("[Gateway Token] Token saved to configuration");
    Ok(new_token)
}

/// Get Dashboard URL (with token)
#[command]
pub async fn get_dashboard_url() -> Result<String, String> {
    info!("[Dashboard URL] Getting Dashboard URL...");

    let token = get_or_create_gateway_token().await?;
    let url = format!("http://localhost:18789?token={}", token);

    info!("[Dashboard URL] URL generated");
    Ok(url)
}

/// Repair device token mismatch by deleting stale identity and paired device files.
/// After calling this, the gateway should be restarted to regenerate fresh device identity.
#[command]
pub async fn repair_device_token() -> Result<String, String> {
    info!("[Device Token Repair] Starting device token repair...");

    let config_dir = platform::get_config_dir();
    let identity_file = format!(
        "{}{}identity{}device.json",
        config_dir,
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    );
    let paired_file = format!(
        "{}{}devices{}paired.json",
        config_dir,
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    );

    let mut deleted = Vec::new();

    // Delete identity/device.json (stale device keypair)
    match std::fs::remove_file(&identity_file) {
        Ok(_) => {
            info!("[Device Token Repair] Deleted: {}", identity_file);
            deleted.push("identity/device.json".to_string());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("[Device Token Repair] Not found (already clean): {}", identity_file);
        }
        Err(e) => {
            warn!("[Device Token Repair] Failed to delete {}: {}", identity_file, e);
        }
    }

    // Delete devices/paired.json (stale paired device entries)
    match std::fs::remove_file(&paired_file) {
        Ok(_) => {
            info!("[Device Token Repair] Deleted: {}", paired_file);
            deleted.push("devices/paired.json".to_string());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("[Device Token Repair] Not found (already clean): {}", paired_file);
        }
        Err(e) => {
            warn!("[Device Token Repair] Failed to delete {}: {}", paired_file, e);
        }
    }

    // Delete identity/device-auth.json (stale device auth token)
    let device_auth_file = format!(
        "{}{}identity{}device-auth.json",
        config_dir,
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    );
    match std::fs::remove_file(&device_auth_file) {
        Ok(_) => {
            info!("[Device Token Repair] Deleted: {}", device_auth_file);
            deleted.push("identity/device-auth.json".to_string());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("[Device Token Repair] Not found (already clean): {}", device_auth_file);
        }
        Err(e) => {
            warn!("[Device Token Repair] Failed to delete {}: {}", device_auth_file, e);
        }
    }

    if deleted.is_empty() {
        info!("[Device Token Repair] No stale files found, identity was already clean");
        Ok("Device identity already clean. Please restart the service.".to_string())
    } else {
        info!("[Device Token Repair] Cleaned {} stale file(s): {:?}", deleted.len(), deleted);
        Ok(format!("Cleaned stale device files: {}. Please restart the service.", deleted.join(", ")))
    }
}
