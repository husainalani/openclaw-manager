use crate::utils::shell;
use tauri::command;
use log::{info, debug};

/// Check if OpenClaw is installed
#[command]
pub async fn check_openclaw_installed() -> Result<bool, String> {
    info!("[Process Check] Checking if OpenClaw is installed...");
    // Use get_openclaw_path to check, because command_exists may be unreliable on Windows
    let installed = shell::get_openclaw_path().is_some();
    info!("[Process Check] OpenClaw installation status: {}", if installed { "installed" } else { "not installed" });
    Ok(installed)
}

/// Get OpenClaw version
#[command]
pub async fn get_openclaw_version() -> Result<Option<String>, String> {
    info!("[Process Check] Getting OpenClaw version...");
    // Use run_openclaw to get the version
    match shell::run_openclaw(&["--version"]) {
        Ok(version) => {
            let v = version.trim().to_string();
            info!("[Process Check] OpenClaw version: {}", v);
            Ok(Some(v))
        },
        Err(e) => {
            debug!("[Process Check] Failed to get version: {}", e);
            Ok(None)
        },
    }
}

/// Check if port is in use (by attempting to connect to openclaw gateway)
#[command]
pub async fn check_port_in_use(port: u16) -> Result<bool, String> {
    info!("[Process Check] Checking if port {} is in use...", port);

    // Use openclaw health to check if gateway is running
    // If port is the default 18789, use openclaw health directly
    if port == 18789 {
        debug!("[Process Check] Using openclaw gateway health to check port 18789...");
        let result = shell::run_openclaw(&["gateway", "health", "--timeout", "2000"]);
        // If health command succeeds, the port is occupied by gateway
        let in_use = result.is_ok();
        info!("[Process Check] Port 18789 status: {}", if in_use { "in use" } else { "available" });
        return Ok(in_use);
    }

    // For non-default ports, try using TCP connection check
    debug!("[Process Check] Using TCP connection to check port {}...", port);
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("127.0.0.1:{}", port);
    let socket_addr: std::net::SocketAddr = addr
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(500)) {
        Ok(_) => {
            info!("[Process Check] Port {} is in use", port);
            Ok(true)
        },
        Err(_) => {
            info!("[Process Check] Port {} is available", port);
            Ok(false)
        },
    }
}

#[derive(serde::Serialize)]
pub struct SecureVersionInfo {
    pub current_version: String,
    pub is_secure: bool,
}

/// Check if current OpenClaw version is secure (>= 2026.1.29)
#[command]
pub async fn check_secure_version() -> Result<SecureVersionInfo, String> {
    info!("[Process Check] Checking OpenClaw version security...");
    match shell::run_openclaw(&["--version"]) {
        Ok(version) => {
            let v = version.trim().to_string();
            // Basic string comparison assuming YYYY.M.D format
            let is_secure = v >= "2026.1.29".to_string();
            
            info!("[Process Check] Version: {}, Secure: {}", v, is_secure);
            Ok(SecureVersionInfo {
                current_version: v,
                is_secure,
            })
        },
        Err(e) => {
            debug!("[Process Check] Failed to get version for security check: {}", e);
            // If we can't get version, assume insecure or handle error
            Err(e)
        },
    }
}

/// Get Node.js version
#[command]
pub async fn get_node_version() -> Result<Option<String>, String> {
    info!("[Process Check] Getting Node.js version...");
    if !shell::command_exists("node") {
        info!("[Process Check] Node.js is not installed");
        return Ok(None);
    }

    match shell::run_command_output("node", &["--version"]) {
        Ok(version) => {
            info!("[Process Check] Node.js version: {}", version);
            Ok(Some(version))
        },
        Err(e) => {
            debug!("[Process Check] Failed to get Node.js version: {}", e);
            Ok(None)
        },
    }
}

/// Check if Ollama is installed
#[command]
pub async fn check_ollama_installed() -> Result<bool, String> {
    info!("[Ollama Check] Checking if Ollama is installed...");
    let installed = shell::command_exists("ollama");
    info!("[Ollama Check] Ollama installation status: {}", if installed { "installed" } else { "not installed" });
    Ok(installed)
}

/// Get installed Ollama models
#[command]
pub async fn get_ollama_models() -> Result<Vec<String>, String> {
    info!("[Ollama Check] Getting installed Ollama models...");
    match shell::run_command_output("ollama", &["list"]) {
        Ok(output) => {
            let mut models = Vec::new();
            // Output format: NAME               ID           SIZE   MODIFIED   
            //                qwen3.5:9b         abcd1234ef   5.5GB  3 days ago
            for (i, line) in output.lines().enumerate() {
                if i == 0 || line.trim().is_empty() {
                    continue; // Skip header and empty lines
                }
                if let Some(name) = line.split_whitespace().next() {
                    models.push(name.to_string());
                }
            }
            info!("[Ollama Check] Found {} installed models", models.len());
            Ok(models)
        },
        Err(e) => {
            debug!("[Ollama Check] Failed to get Ollama models: {}", e);
            Err(e)
        },
    }
}

/// Install / pull an Ollama model
#[command]
pub async fn install_ollama_model(model_name: String) -> Result<String, String> {
    info!("[Ollama Check] Installing Ollama model: {}", model_name);
    // Use `ollama pull` instead of `ollama run` so it doesn't stay interactive.
    match shell::run_command_output("ollama", &["pull", &model_name]) {
        Ok(_) => {
            info!("[Ollama Check] Successfully installed model: {}", model_name);
            Ok(format!("Successfully installed {}", model_name))
        },
        Err(e) => {
            debug!("[Ollama Check] Failed to install Ollama model {}: {}", model_name, e);
            Err(e)
        },
    }
}
