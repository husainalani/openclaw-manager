use crate::models::MCPConfig;
use crate::utils::{platform, shell, log_sanitizer};
use log::{info, warn, error};
use std::collections::HashMap;
use tauri::command;

/// Load MCP config from separate mcps.json file
fn load_mcp_config_file() -> Result<HashMap<String, MCPConfig>, String> {
    let config_path = platform::get_mcp_config_file_path();
    let path = std::path::Path::new(&config_path);

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read mcps.json: {}", e))?;

    let configs: HashMap<String, MCPConfig> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse mcps.json: {}", e))?;

    Ok(configs)
}

/// Save MCP config to separate mcps.json file AND sync to ~/.mcporter/mcporter.json
fn save_mcp_config_file(configs: &HashMap<String, MCPConfig>) -> Result<(), String> {
    // 1. Save to Manager's private config (mcps.json)
    let config_path = platform::get_mcp_config_file_path();
    let content = serde_json::to_string_pretty(configs)
        .map_err(|e| format!("Failed to serialize MCP config: {}", e))?;

    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write mcps.json: {}", e))?;

    // 2. Sync enabled servers to system mcporter config (~/.mcporter/mcporter.json)
    if let Err(e) = sync_to_mcporter(configs) {
        warn!("Failed to sync to mcporter: {}", e);
        // Don't fail the whole save operation if sync fails
    }

    Ok(())
}

fn sync_to_mcporter(configs: &HashMap<String, MCPConfig>) -> Result<(), String> {
    let mcporter_path = platform::get_mcporter_config_file_path();
    let path = std::path::Path::new(&mcporter_path);

    // Create ~/.mcporter directory if missing
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create mcporter config dir: {}", e))?;
        }
    }

    // Load existing mcporter config or create new
    let mut root_val: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read mcporter.json: {}", e))?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| serde_json::json!({ "mcpServers": {} }))
    } else {
        serde_json::json!({ "mcpServers": {} })
    };

    // Ensure mcpServers object exists
    if root_val.get("mcpServers").is_none() {
        root_val["mcpServers"] = serde_json::json!({});
    }

    let mcp_servers_obj = root_val["mcpServers"]
        .as_object_mut()
        .ok_or_else(|| "mcpServers is not a JSON object".to_string())?;

    // Sync: Add/Update enabled servers from Manager
    for (name, config) in configs {
        if config.enabled {
            // Convert MCPConfig to serde_json::Value
            // Note: We skip 'enabled' field as mcporter doesn't use it (presence = enabled)
            let mut server_val = serde_json::to_value(config)
                .map_err(|e| format!("Failed to serialize config for {}: {}", name, e))?;

            if let Some(obj) = server_val.as_object_mut() {
                obj.remove("enabled");
            }

            mcp_servers_obj.insert(name.clone(), server_val);
        } else {
            // Remove disabled servers if they were previously synced
            mcp_servers_obj.remove(name);
        }
    }

    // Important: We do NOT remove servers that are in mcporter but NOT in Manager,
    // to respect user's manual edits or other tools. We only manage the ones we know about.

    // Write back
    let new_content = serde_json::to_string_pretty(&root_val)
        .map_err(|e| format!("Failed to serialize mcporter config: {}", e))?;

    std::fs::write(path, new_content)
        .map_err(|e| format!("Failed to write mcporter.json: {}", e))?;

    Ok(())
}

/// Get MCP configuration
#[command]
pub async fn get_mcp_config() -> Result<HashMap<String, MCPConfig>, String> {
    info!("[MCP Config] Getting MCP configuration...");

    let configs = load_mcp_config_file()?;

    info!("[MCP Config] Found {} MCP servers", configs.len());
    Ok(configs)
}

/// Save MCP configuration
#[command]
pub async fn save_mcp_config(
    name: String,
    config: Option<MCPConfig>,
) -> Result<String, String> {
    info!("[Save MCP] Saving MCP configuration for: {}", name);

    let mut configs = load_mcp_config_file()?;

    if let Some(mcp) = config {
        configs.insert(name.clone(), mcp);
        info!("[Save MCP] Updated configuration for {}", name);
    } else {
        configs.remove(&name);
        info!("[Save MCP] Deleted configuration for {}", name);
    }

    save_mcp_config_file(&configs)?;
    Ok(format!("MCP configuration saved for {}", name))
}

/// Validate that a git repository URL is a safe HTTPS URL and extract the repo name.
/// Returns the sanitised repo name on success.
fn validate_git_url_and_extract_repo(url: &str) -> Result<String, String> {
    // Only allow HTTPS git URLs to prevent local file / ssh injection
    if !url.starts_with("https://") {
        return Err("Only HTTPS repository URLs are allowed".to_string());
    }

    let repo_name = url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Could not extract repository name from URL".to_string())?
        .to_string();

    // Strict allowlist: only alphanumeric, hyphens, underscores, dots.
    // No `..`, no path separators, no shell metacharacters.
    let valid = repo_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
    if !valid || repo_name.starts_with('.') || repo_name.contains("..") {
        return Err(format!(
            "Repository name '{}' contains invalid characters",
            repo_name
        ));
    }

    if repo_name.len() > 128 {
        return Err("Repository name is too long (max 128 characters)".to_string());
    }

    Ok(repo_name)
}

/// Install MCP server from a Git repository URL
#[command]
pub async fn install_mcp_from_git(url: String) -> Result<String, String> {
    info!("[MCP Install] Installing MCP from validated URL");

    let repo_name = validate_git_url_and_extract_repo(&url)?;

    info!("[MCP Install] Repository name: {}", repo_name);

    // Create mcps directory if it doesn't exist
    let mcps_dir = platform::get_mcp_install_dir();
    std::fs::create_dir_all(&mcps_dir)
        .map_err(|e| format!("Failed to create mcps directory: {}", e))?;

    // Build install path and canonicalize the parent to defend against symlink attacks
    let mcps_dir_path = std::path::Path::new(&mcps_dir);
    let install_path = mcps_dir_path.join(&repo_name);

    // Ensure the resolved install path is actually under mcps_dir (path-traversal guard)
    let canonical_mcps = std::fs::canonicalize(&mcps_dir)
        .unwrap_or_else(|_| mcps_dir_path.to_path_buf());
    // We can't canonicalize a path that doesn't exist yet, so check the parent instead
    let install_str = install_path.to_string_lossy().to_string();

    // Remove existing directory if present (re-install)
    if install_path.exists() {
        // Double-check canonical path is confined before any destructive operation
        if let Ok(canonical_install) = std::fs::canonicalize(&install_path) {
            if !canonical_install.starts_with(&canonical_mcps) {
                return Err("Install path escapes the MCP directory — aborting".to_string());
            }
        }
        info!("[MCP Install] Removing existing installation at {}", install_str);
        std::fs::remove_dir_all(&install_path)
            .map_err(|e| format!("Failed to remove existing directory: {}", e))?;
    }

    // Step 1: Clone the repository
    info!("[MCP Install] Cloning repository...");
    let clone_output = shell::run_command("git", &["clone", &url, &install_str])
        .map_err(|e| format!("Failed to run git clone: {}", e))?;

    if !clone_output.status.success() {
        let stderr = String::from_utf8_lossy(&clone_output.stderr);
        return Err(format!("Git clone failed: {}", stderr));
    }
    info!("[MCP Install] Clone successful");

    // Step 2: npm install
    info!("[MCP Install] Running npm install...");
    let npm_cmd = if platform::is_windows() { "npm.cmd" } else { "npm" };

    let mut npm_install = std::process::Command::new(npm_cmd);
    npm_install.args(&["install"]).current_dir(&install_str);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        npm_install.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let install_output = npm_install.output()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        return Err(format!("npm install failed: {}", stderr));
    }
    info!("[MCP Install] npm install successful");

    // Step 3: npm run build
    info!("[MCP Install] Running npm run build...");
    let mut npm_build = std::process::Command::new(npm_cmd);
    npm_build.args(&["run", "build"]).current_dir(&install_str);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        npm_build.creation_flags(0x08000000);
    }

    let build_output = npm_build.output()
        .map_err(|e| format!("Failed to run npm run build: {}", e))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        warn!("[MCP Install] npm run build failed (may not have a build step): {}", stderr);
        // Don't fail — some MCPs don't need a build step
    } else {
        info!("[MCP Install] npm run build successful");
    }

    // Step 4: Auto-configure in mcps.json
    info!("[MCP Install] Configuring MCP in mcps.json...");
    let mut configs = load_mcp_config_file()?;

    // Determine the entry point (dist/index.js or index.js)
    let dist_index = if platform::is_windows() {
        format!("{}\\dist\\index.js", install_str)
    } else {
        format!("{}/dist/index.js", install_str)
    };

    let entry_point = if std::path::Path::new(&dist_index).exists() {
        dist_index
    } else {
        let root_index = if platform::is_windows() {
            format!("{}\\index.js", install_str)
        } else {
            format!("{}/index.js", install_str)
        };
        if std::path::Path::new(&root_index).exists() {
            root_index
        } else {
            dist_index
        }
    };

    configs.insert(repo_name.clone(), MCPConfig {
        command: "node".to_string(),
        args: vec![entry_point, "--stdio".to_string()],
        env: HashMap::new(),
        url: String::new(),
        enabled: true,
    });

    save_mcp_config_file(&configs)?;
    info!("[MCP Install] Installation complete for {}", repo_name);
    Ok(format!("Successfully installed MCP: {}", repo_name))
}

/// Uninstall an MCP server
#[command]
pub async fn uninstall_mcp(name: String) -> Result<String, String> {
    info!("[MCP Uninstall] Uninstalling MCP: {}", name);

    // Remove directory
    let mcps_dir = platform::get_mcp_install_dir();
    let install_path = if platform::is_windows() {
        format!("{}\\{}", mcps_dir, name)
    } else {
        format!("{}/{}", mcps_dir, name)
    };

    if std::path::Path::new(&install_path).exists() {
        std::fs::remove_dir_all(&install_path)
            .map_err(|e| format!("Failed to remove MCP directory: {}", e))?;
        info!("[MCP Uninstall] Removed directory: {}", install_path);
    }

    // Remove from mcps.json
    let mut configs = load_mcp_config_file()?;
    configs.remove(&name);
    save_mcp_config_file(&configs)?;

    info!("[MCP Uninstall] Uninstalled MCP: {}", name);
    Ok(format!("Successfully uninstalled MCP: {}", name))
}

/// Check if mcporter is installed
#[command]
pub async fn check_mcporter_installed() -> Result<bool, String> {
    info!("[mcporter] Checking if mcporter is installed...");
    let installed = shell::command_exists("mcporter");
    info!("[mcporter] Installed: {}", installed);
    Ok(installed)
}

/// Install mcporter via npm
#[command]
pub async fn install_mcporter() -> Result<String, String> {
    info!("[mcporter] Installing mcporter globally via npm...");

    let npm_cmd = if platform::is_windows() { "npm.cmd" } else { "npm" };

    let mut cmd = std::process::Command::new(npm_cmd);
    cmd.args(&["install", "-g", "mcporter"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("npm install -g mcporter failed: {}", stderr));
    }

    info!("[mcporter] Installation successful");
    Ok("mcporter installed successfully".to_string())
}

/// Uninstall Mcporter
#[command]
pub async fn uninstall_mcporter() -> Result<String, String> {
    info!("Uninstalling mcporter globally via npm");

    #[cfg(target_os = "windows")]
    let program = "cmd";
    #[cfg(target_os = "windows")]
    let args = ["/C", "npm uninstall -g @openclaw/mcporter"];

    #[cfg(not(target_os = "windows"))]
    let program = "npm";
    #[cfg(not(target_os = "windows"))]
    let args = ["uninstall", "-g", "@openclaw/mcporter"];

    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute npm uninstall: {}", e))?;

    if output.status.success() {
        info!("mcporter uninstalled successfully");
        Ok("MCPorter uninstalled successfully".to_string())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("Failed to uninstall mcporter: {}", error_msg);
        Err(format!("Failed to uninstall mcporter: {}", error_msg))
    }
}

/// Install MCP server as an OpenClaw plugin (using openclaw plugins install)
#[command]
pub async fn install_mcp_plugin(url: String) -> Result<String, String> {
    info!("[MCP Plugin] Installing MCP plugin from: {}", url);

    let result = shell::run_openclaw(&["plugins", "install", &url])
        .map_err(|e| format!("Failed to install plugin: {}", e))?;

    info!("[MCP Plugin] Installation result: {}", result);
    Ok(format!("Successfully installed MCP plugin from: {}", url))
}

/// Allowed config key pattern: dot-separated alphanumeric segments, e.g. "gateway.auth.mode"
fn validate_config_key(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 256 {
        return Err("Config key must be between 1 and 256 characters".to_string());
    }
    // Only allow: letters, digits, dots, hyphens, underscores — no shell specials
    let valid = key.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
    if !valid {
        return Err(format!(
            "Config key '{}' contains invalid characters (only alphanumeric, '.', '-', '_' allowed)",
            key
        ));
    }
    Ok(())
}

/// Set openclaw config via CLI (openclaw config set <key> <value>)
#[command]
pub async fn openclaw_config_set(key: String, value: String) -> Result<String, String> {
    // Validate key to prevent injection into the openclaw CLI
    validate_config_key(&key)?;

    // Value size guard
    if value.len() > 65536 {
        return Err("Config value exceeds maximum allowed length (64 KB)".to_string());
    }

    // Log key but never log the raw value — it may contain API keys or secrets
    info!("[Config CLI] Setting config key: {}", key);

    let result = shell::run_openclaw(&["config", "set", &key, &value])
        .map_err(|e| format!("Failed to set config: {}", e))?;

    info!("[Config CLI] Set result: {}", log_sanitizer::sanitize(&result));
    Ok(format!("Set {}", key))
}

/// Validate a given config JSON string by writing to a temporary file and running openclaw config validate --json
#[command]
pub async fn validate_openclaw_config(config_json: String) -> Result<String, String> {
    info!("[Config CLI] Validating config json");

    // Guard against oversized payloads before writing to disk
    if config_json.len() > super::MAX_CONFIG_SIZE {
        return Err(format!(
            "Config JSON exceeds maximum allowed size ({} MB)",
            super::MAX_CONFIG_SIZE / 1024 / 1024
        ));
    }

    // Create a temp file with a cryptographically random name to prevent TOCTOU attacks
    use rand::RngCore;
    let mut rng_bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut rng_bytes);
    let rand_suffix: String = rng_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("openclaw_config_{}.json", rand_suffix));

    std::fs::write(&temp_file, &config_json)
        .map_err(|e| format!("Failed to write temp config file: {}", e))?;

    // Restrict temp file to owner-only on Unix (prevents other local users reading it)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&temp_file, std::fs::Permissions::from_mode(0o600));
    }

    let temp_file_str = temp_file.to_string_lossy().to_string();

    let openclaw_path = crate::utils::shell::get_openclaw_path().ok_or_else(|| {
        let _ = std::fs::remove_file(&temp_file);
        "Cannot find openclaw command".to_string()
    })?;

    let mut cmd = std::process::Command::new(&openclaw_path);
    cmd.args(&["config", "validate", "--json"]);
    cmd.env("OPENCLAW_CONFIG", &temp_file_str);
    cmd.env("PATH", crate::utils::shell::get_extended_path());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd.output().map_err(|e| {
        let _ = std::fs::remove_file(&temp_file);
        format!("Failed to execute config validate: {}", e)
    })?;

    let _ = std::fs::remove_file(&temp_file);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(if !stderr.is_empty() { stderr } else { stdout })
    }
}

/// Allowlist of commands that may be spawned during MCP server testing.
/// Anything not in this list is rejected before process creation.
const ALLOWED_MCP_COMMANDS: &[&str] = &[
    "node", "npx", "python", "python3", "uv", "uvx", "deno", "bun", "java",
];

/// Validate that the command base-name is in the MCP allowlist.
fn validate_mcp_command(cmd: &str) -> Result<(), String> {
    // Reject absolute/relative paths — only bare command names are allowed
    if cmd.contains('/') || cmd.contains('\\') {
        return Err(format!(
            "Command '{}' must be a bare name without path separators",
            cmd
        ));
    }
    // Strip Windows extensions for comparison
    let normalized = cmd
        .trim_end_matches(".cmd")
        .trim_end_matches(".exe")
        .to_lowercase();

    if ALLOWED_MCP_COMMANDS.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "Command '{}' is not in the allowed list ({:?})",
            cmd,
            ALLOWED_MCP_COMMANDS
        ))
    }
}

/// Validate that a URL target begins with http:// or https:// for MCP testing.
fn validate_mcp_url(url: &str) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err("MCP URL must start with http:// or https://".to_string())
    }
}

/// On Windows, resolve a whitelisted command to its full .cmd or .exe path so we
/// can invoke it directly with Command::new, avoiding cmd /c shell interpretation.
#[cfg(windows)]
fn resolve_windows_command(cmd: &str) -> Option<String> {
    use std::process::Command as SysCmd;
    for ext in &[".cmd", ".exe", ""] {
        let candidate = format!("{}{}", cmd, ext);
        if let Ok(out) = SysCmd::new("where").arg(&candidate).output() {
            if out.status.success() {
                let path = String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Test an MCP server connectivity
#[command]
pub async fn test_mcp_server(server_type: String, target: String, command: Option<String>, args: Option<Vec<String>>) -> Result<String, String> {
    info!("[MCP Test] Testing MCP server: type={}", server_type);

    if server_type == "url" {
        // Validate the target is a safe HTTP/HTTPS URL before handing to curl
        validate_mcp_url(&target)?;

        // Remote HTTP MCP: POST an MCP initialize request to the URL
        let mut cmd = std::process::Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
        cmd.args(&[
            "-s", "-w", "\n%{http_code}",
            "-X", "POST",
            "-H", "Content-Type: application/json",
            "-H", "Accept: text/event-stream, application/json",
            "-d", r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#,
            "--max-time", "10",
            &target,
        ]);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        match cmd.output() {
            Ok(out) => {
                let output_str = String::from_utf8_lossy(&out.stdout).to_string();
                let lines: Vec<&str> = output_str.trim().lines().collect();
                let status_code = lines.last().unwrap_or(&"0");
                let body = if lines.len() > 1 { lines[..lines.len()-1].join("\n") } else { String::new() };

                if status_code.starts_with("2") {
                    // Try to extract server name from JSON response
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                        if let Some(name) = json.pointer("/result/serverInfo/name") {
                            return Ok(format!("✅ Server reachable: {} (HTTP {})", name.as_str().unwrap_or("unknown"), status_code));
                        }
                    }
                    // Try to parse SSE response for server info
                    for line in body.lines() {
                        if line.starts_with("data:") {
                            let data = line.trim_start_matches("data:").trim();
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(name) = json.pointer("/result/serverInfo/name") {
                                    return Ok(format!("✅ Server reachable: {} (HTTP {})", name.as_str().unwrap_or("unknown"), status_code));
                                }
                            }
                        }
                    }
                    Ok(format!("✅ Server reachable (HTTP {})", status_code))
                } else {
                    Err(format!("❌ Server returned HTTP {}", status_code))
                }
            }
            Err(e) => Err(format!("Failed to test URL: {}", e))
        }
    } else {
        // Local stdio MCP: spawn the whitelisted command directly with proper args
        let cmd_name = command.unwrap_or(target.clone());
        let cmd_args = args.unwrap_or_default();

        // Security gate: reject any command not in the allowlist
        validate_mcp_command(&cmd_name)?;

        info!("[MCP Test] Spawning whitelisted command: {}", cmd_name);

        let extended_path = shell::get_extended_path();

        // On Windows, resolve the full executable path (avoids cmd /c shell interpretation)
        #[cfg(windows)]
        let mut cmd = {
            let exe = resolve_windows_command(&cmd_name)
                .ok_or_else(|| format!("Could not locate '{}' in PATH", cmd_name))?;
            let mut c = std::process::Command::new(&exe);
            c.args(&cmd_args);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = std::process::Command::new(&cmd_name);
            c.args(&cmd_args);
            c
        };

        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("PATH", &extended_path);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                // Send MCP initialize request via stdin
                if let Some(ref mut stdin) = child.stdin {
                    use std::io::Write;
                    let init_msg = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
                    let _ = writeln!(stdin, "Content-Length: {}\r\n\r\n{}", init_msg.len(), init_msg);
                }

                // Wait briefly then check
                std::thread::sleep(std::time::Duration::from_millis(3000));

                match child.try_wait() {
                    Ok(Some(status)) => {
                        // Process exited — read stderr for error info
                        let stderr = child.stderr.take().map(|mut s| {
                            let mut buf = String::new();
                            use std::io::Read;
                            let _ = s.read_to_string(&mut buf);
                            buf
                        }).unwrap_or_default();

                        if status.success() {
                            Ok("✅ Server process started and exited cleanly".to_string())
                        } else {
                            Err(format!("❌ Server exited with {}\n{}", status, stderr.trim()))
                        }
                    }
                    Ok(None) => {
                        // Still running — good! Kill it and report success
                        let _ = child.kill();
                        Ok(format!("✅ Server is running (process started successfully)\nCommand: {} {}", cmd_name, cmd_args.join(" ")))
                    }
                    Err(e) => {
                        let _ = child.kill();
                        Err(format!("Failed to check process: {}", e))
                    }
                }
            }
            Err(e) => {
                Err(format!("❌ Failed to start server: {}\nCommand: {} {}", e, cmd_name, cmd_args.join(" ")))
            }
        }
    }
}
