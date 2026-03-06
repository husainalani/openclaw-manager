use crate::utils::{file, platform};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::command;

/// Security profile for tools access
#[command]
pub async fn get_tools_profile() -> Result<String, String> {
    info!("[Config] Getting tools profile...");
    let config = super::load_openclaw_config()?;
    let profile = config
        .pointer("/tools/profile")
        .and_then(|v| v.as_str())
        .unwrap_or("messaging")
        .to_string();
    Ok(profile)
}

#[command]
pub async fn save_tools_profile(profile: String) -> Result<String, String> {
    info!("[Config] Saving tools profile: {}", profile);
    let mut config = super::load_openclaw_config()?;
    if config.get("tools").is_none() {
        config["tools"] = json!({});
    }
    config["tools"]["profile"] = json!(profile);
    super::save_openclaw_config(&config)?;
    Ok("Tools profile saved".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PdfConfig {
    #[serde(alias = "pdfMaxPages", alias = "max_pages")]
    pub max_pages: Option<u64>,
    #[serde(alias = "pdfMaxBytesMb", alias = "max_bytes_mb")]
    pub max_bytes_mb: Option<f64>,
}

#[command]
pub async fn get_pdf_config() -> Result<PdfConfig, String> {
    info!("[Config] Getting PDF config...");
    let config = super::load_openclaw_config()?;
    let max_pages = config.get("pdfMaxPages").and_then(|v| v.as_u64());
    let max_bytes_mb = config.get("pdfMaxBytesMb").and_then(|v| v.as_f64());
    Ok(PdfConfig { max_pages, max_bytes_mb })
}

#[command]
pub async fn save_pdf_config(pdf_config: PdfConfig) -> Result<String, String> {
    info!("[Config] Saving PDF config...");
    let mut config = super::load_openclaw_config()?;
    if let Some(pages) = pdf_config.max_pages {
        config["pdfMaxPages"] = json!(pages);
    } else if let Some(obj) = config.as_object_mut() {
        obj.remove("pdfMaxPages");
    }
    if let Some(mb) = pdf_config.max_bytes_mb {
        config["pdfMaxBytesMb"] = json!(mb);
    } else if let Some(obj) = config.as_object_mut() {
        obj.remove("pdfMaxBytesMb");
    }
    super::save_openclaw_config(&config)?;
    Ok("PDF config saved".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    pub provider: Option<String>,
}

#[command]
pub async fn get_memory_config() -> Result<MemoryConfig, String> {
    info!("[Config] Getting memory config...");
    let config = super::load_openclaw_config()?;
    let provider = config
        .pointer("/memorySearch/provider")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok(MemoryConfig { provider })
}

#[command]
pub async fn save_memory_config(memory_config: MemoryConfig) -> Result<String, String> {
    info!("[Config] Saving memory config...");
    let mut config = super::load_openclaw_config()?;
    if let Some(provider) = memory_config.provider {
        if config.get("memorySearch").is_none() {
            config["memorySearch"] = json!({});
        }
        config["memorySearch"]["provider"] = json!(provider);
    } else if let Some(obj) = config.as_object_mut() {
        obj.remove("memorySearch");
    }
    super::save_openclaw_config(&config)?;
    Ok("Memory config saved".to_string())
}

// ============ OpenClaw Home Directory ============

/// Get the OpenClaw home directory path (~/.openclaw)
#[command]
pub async fn get_openclaw_home_dir() -> Result<String, String> {
    Ok(platform::get_config_dir())
}

// ============ Heartbeat & Compaction ============

/// Heartbeat configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub every: Option<String>,
    pub target: Option<String>,
}

/// Compaction configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub enabled: bool,
    pub threshold: Option<u32>,
    pub context_pruning: bool,
    pub max_context_messages: Option<u32>,
}

/// Get heartbeat configuration
#[command]
pub async fn get_heartbeat_config() -> Result<HeartbeatConfig, String> {
    info!("[Heartbeat] Getting heartbeat config...");
    let config = super::load_openclaw_config()?;

    let every = config.pointer("/agents/defaults/heartbeat/every")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let target = config.pointer("/agents/defaults/heartbeat/target")
        .and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(HeartbeatConfig { every, target })
}

/// Save heartbeat configuration
#[command]
pub async fn save_heartbeat_config(every: Option<String>, target: Option<String>) -> Result<String, String> {
    info!("[Heartbeat] Saving heartbeat config: every={:?}, target={:?}", every, target);
    let mut config = super::load_openclaw_config()?;

    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }

    if every.is_some() || target.is_some() {
        let mut hb = json!({});
        if let Some(e) = &every { hb["every"] = json!(e); }
        if let Some(t) = &target { hb["target"] = json!(t); }
        config["agents"]["defaults"]["heartbeat"] = hb;
    } else {
        // Remove heartbeat if both are None
        if let Some(defaults) = config["agents"]["defaults"].as_object_mut() {
            defaults.remove("heartbeat");
        }
    }

    super::save_openclaw_config(&config)?;
    Ok("Heartbeat configuration saved".to_string())
}

/// Get compaction configuration
#[command]
pub async fn get_compaction_config() -> Result<CompactionConfig, String> {
    info!("[Compaction] Getting compaction config...");
    let config = super::load_openclaw_config()?;

    let compaction_val = config.pointer("/agents/defaults/compaction");
    let pruning_val = config.pointer("/agents/defaults/contextPruning");

    let enabled = compaction_val.map(|v| {
        // compaction can be true/false or an object with settings
        v.as_bool().unwrap_or(true)
    }).unwrap_or(false);

    let threshold = compaction_val
        .and_then(|v| v.get("threshold"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let context_pruning = pruning_val.map(|v| v.as_bool().unwrap_or(false)).unwrap_or(false);

    let max_context_messages = pruning_val
        .and_then(|v| v.get("maxMessages"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    Ok(CompactionConfig { enabled, threshold, context_pruning, max_context_messages })
}

/// Save compaction configuration
#[command]
pub async fn save_compaction_config(
    enabled: bool,
    threshold: Option<u32>,
    context_pruning: bool,
    max_context_messages: Option<u32>,
) -> Result<String, String> {
    info!("[Compaction] Saving compaction config: enabled={}, pruning={}", enabled, context_pruning);
    let mut config = super::load_openclaw_config()?;

    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }

    if enabled {
        let mut comp = json!({});
        if let Some(t) = threshold { comp["threshold"] = json!(t); }
        config["agents"]["defaults"]["compaction"] = comp;
    } else {
        if let Some(defaults) = config["agents"]["defaults"].as_object_mut() {
            defaults.remove("compaction");
        }
    }

    if context_pruning {
        let mut pruning = json!(true);
        if let Some(max) = max_context_messages {
            pruning = json!({ "maxMessages": max });
        }
        config["agents"]["defaults"]["contextPruning"] = pruning;
    } else {
        if let Some(defaults) = config["agents"]["defaults"].as_object_mut() {
            defaults.remove("contextPruning");
        }
    }

    super::save_openclaw_config(&config)?;
    Ok("Compaction configuration saved".to_string())
}

// ============ Workspace & Agent Personality ============

/// Workspace configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub workspace: Option<String>,
    pub timezone: Option<String>,
    pub time_format: Option<String>,
    pub skip_bootstrap: bool,
    pub bootstrap_max_chars: Option<u32>,
}

/// Get workspace configuration
#[command]
pub async fn get_workspace_config() -> Result<WorkspaceConfig, String> {
    info!("[Workspace] Getting workspace config...");
    let config = super::load_openclaw_config()?;

    let workspace = config.pointer("/agents/defaults/workspace")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let timezone = config.pointer("/manager/timezone")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let time_format = config.pointer("/manager/time_format")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let skip_bootstrap = config.pointer("/agents/defaults/skipBootstrap")
        .and_then(|v| v.as_bool()).unwrap_or(false);
    let bootstrap_max_chars = config.pointer("/agents/defaults/bootstrapMaxChars")
        .and_then(|v| v.as_u64()).map(|v| v as u32);

    Ok(WorkspaceConfig { workspace, timezone, time_format, skip_bootstrap, bootstrap_max_chars })
}

/// Save workspace configuration
#[command]
pub async fn save_workspace_config(
    workspace: Option<String>,
    timezone: Option<String>,
    time_format: Option<String>,
    skip_bootstrap: bool,
    bootstrap_max_chars: Option<u32>,
) -> Result<String, String> {
    info!("[Workspace] Saving workspace config...");
    let mut config = super::load_openclaw_config()?;

    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }

    // Set or remove each field in agents.defaults
    if let Some(defaults) = config.pointer_mut("/agents/defaults").and_then(|v| v.as_object_mut()) {
        match &workspace {
            Some(w) if !w.is_empty() => { defaults.insert("workspace".into(), json!(w)); }
            _ => { defaults.remove("workspace"); }
        }
        if skip_bootstrap {
            defaults.insert("skipBootstrap".into(), json!(true));
        } else {
            defaults.remove("skipBootstrap");
        }
        match bootstrap_max_chars {
            Some(max) => { defaults.insert("bootstrapMaxChars".into(), json!(max)); }
            None => { defaults.remove("bootstrapMaxChars"); }
        }
        // Remove timezone/timeFormat from defaults if present (migrate to manager)
        defaults.remove("timezone");
        defaults.remove("timeFormat");
    }

    // Set manager fields
    if config.get("manager").is_none() { config["manager"] = json!({}); }
    if let Some(manager) = config.get_mut("manager").and_then(|v| v.as_object_mut()) {
        match &timezone {
            Some(tz) if !tz.is_empty() => { manager.insert("timezone".into(), json!(tz)); }
            _ => { manager.remove("timezone"); }
        }
        match &time_format {
            Some(tf) if !tf.is_empty() => { manager.insert("time_format".into(), json!(tf)); }
            _ => { manager.remove("time_format"); }
        }
    }

    super::save_openclaw_config(&config)?;
    Ok("Workspace configuration saved".to_string())
}

/// Get a personality file from the workspace directory
#[command]
pub async fn get_personality_file(filename: String) -> Result<String, String> {
    info!("[Personality] Reading file: {}", filename);

    // Validate filename
    let allowed = ["AGENTS.md", "SOUL.md", "TOOLS.md"];
    if !allowed.contains(&filename.as_str()) {
        return Err(format!("Invalid file: {}. Allowed: {:?}", filename, allowed));
    }

    // Get workspace path from config, fallback to ~/.openclaw
    let config = super::load_openclaw_config()?;
    let workspace = config.pointer("/agents/defaults/workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let dir = if workspace.is_empty() {
        platform::get_config_dir()
    } else {
        workspace.to_string()
    };

    let filepath = if platform::is_windows() {
        format!("{}\\{}", dir, filename)
    } else {
        format!("{}/{}", dir, filename)
    };

    match file::read_file(&filepath) {
        Ok(content) => Ok(content),
        Err(_) => Ok(String::new()), // File doesn't exist yet, return empty
    }
}

/// Save a personality file to the workspace directory
#[command]
pub async fn save_personality_file(filename: String, content: String) -> Result<String, String> {
    info!("[Personality] Saving file: {}", filename);

    let allowed = ["AGENTS.md", "SOUL.md", "TOOLS.md"];
    if !allowed.contains(&filename.as_str()) {
        return Err(format!("Invalid file: {}. Allowed: {:?}", filename, allowed));
    }

    let config = super::load_openclaw_config()?;
    let workspace = config.pointer("/agents/defaults/workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let dir = if workspace.is_empty() {
        platform::get_config_dir()
    } else {
        workspace.to_string()
    };

    let filepath = if platform::is_windows() {
        format!("{}\\{}", dir, filename)
    } else {
        format!("{}/{}", dir, filename)
    };

    file::write_file(&filepath, &content)
        .map_err(|e| format!("Failed to save {}: {}", filename, e))?;

    Ok(format!("{} saved successfully", filename))
}

// ============ Browser Control ============

/// Browser configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub enabled: bool,
    pub color: Option<String>,
}

/// Get browser configuration
#[command]
pub async fn get_browser_config() -> Result<BrowserConfig, String> {
    info!("[Browser] Getting browser config...");
    let config = super::load_openclaw_config()?;

    // Read from meta (Manager specific)
    let enabled = config.pointer("/meta/gui/browser/enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default to true if not set

    let color = config.pointer("/meta/gui/browser/color")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(BrowserConfig { enabled, color })
}

/// Save browser configuration
#[command]
pub async fn save_browser_config(enabled: bool, color: Option<String>) -> Result<String, String> {
    info!("[Browser] Saving browser config: enabled={}, color={:?}", enabled, color);
    let mut config = super::load_openclaw_config()?;

    // Store in meta.gui.browser to avoid polluting core config
    if config.get("meta").is_none() { config["meta"] = json!({}); }
    if config["meta"].get("gui").is_none() { config["meta"]["gui"] = json!({}); }

    let mut browser_config = json!({
        "enabled": enabled
    });

    if let Some(c) = color {
        if !c.is_empty() {
            browser_config["color"] = json!(c);
        }
    }

    config["meta"]["gui"]["browser"] = browser_config;

    super::save_openclaw_config(&config)?;
    Ok("Browser configuration saved".to_string())
}

// ============ Web Search ============

/// Web Search configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub brave_api_key: Option<String>,
}

/// Get web search configuration
#[command]
pub async fn get_web_config() -> Result<WebConfig, String> {
    info!("[Web] Getting web search config...");
    let config = super::load_openclaw_config()?;

    let brave_api_key = config.pointer("/web/braveApiKey")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(WebConfig { brave_api_key })
}

/// Save web search configuration
#[command]
pub async fn save_web_config(brave_api_key: Option<String>) -> Result<String, String> {
    info!("[Web] Saving web search config...");
    let mut config = super::load_openclaw_config()?;

    if config.get("web").is_none() {
        config["web"] = json!({});
    }

    match brave_api_key {
        Some(key) if !key.is_empty() => {
            config["web"]["braveApiKey"] = json!(key);
        }
        _ => {
            if let Some(web) = config.get_mut("web").and_then(|v| v.as_object_mut()) {
                web.remove("braveApiKey");
            }
        }
    }


    super::save_openclaw_config(&config)?;
    Ok("Web search configuration saved".to_string())
}

// ============ Gateway Configuration ============

/// Gateway configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub port: u16,
    pub log_level: String,
}

/// Get gateway configuration
#[command]
pub async fn get_gateway_config() -> Result<GatewayConfig, String> {
    info!("[Gateway] Getting gateway config...");
    let config = super::load_openclaw_config()?;

    let port = config.pointer("/gateway/port")
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .unwrap_or(3000);

    let log_level = config.pointer("/manager/log_level")
        .and_then(|v| v.as_str())
        .or_else(|| config.pointer("/gateway/logLevel").and_then(|v| v.as_str())) // Legacy fallback
        .map(|s| s.to_string())
        .unwrap_or_else(|| "info".to_string());

    Ok(GatewayConfig { port, log_level })
}

/// Save gateway configuration
#[command]
pub async fn save_gateway_config(port: u16, log_level: String) -> Result<String, String> {
    info!("[Gateway] Saving gateway config: port={}, level={}", port, log_level);
    let mut config = super::load_openclaw_config()?;

    if config.get("gateway").is_none() {
        config["gateway"] = json!({});
    }

    if let Some(gateway) = config.get_mut("gateway").and_then(|v| v.as_object_mut()) {
        gateway.insert("port".to_string(), json!(port));
        // Remove legacy logLevel if exists
        gateway.remove("logLevel");
        gateway.remove("log_level");
    }

    if config.get("manager").is_none() {
        config["manager"] = json!({});
    }

    if let Some(manager) = config.get_mut("manager").and_then(|v| v.as_object_mut()) {
        manager.insert("log_level".to_string(), json!(log_level));
    }

    super::save_openclaw_config(&config)?;
    Ok("Gateway configuration saved".to_string())
}

// ============ Configuration Management ============

/// Export configuration
#[command]
pub async fn export_config(path: String) -> Result<String, String> {
    info!("[Config] Exporting config to: {}", path);
    let config = super::load_openclaw_config()?;

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    file::write_file(&path, &content)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    Ok(format!("Configuration exported to {}", path))
}

/// Import configuration
#[command]
pub async fn import_config(path: String) -> Result<String, String> {
    info!("[Config] Importing config from: {}", path);

    let content = file::read_file(&path)
        .map_err(|e| format!("Failed to read import file: {}", e))?;

    let new_config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON file: {}", e))?;

    if !new_config.is_object() {
        return Err("Imported file is not a valid configuration object".to_string());
    }

    super::save_openclaw_config(&new_config)?;

    Ok("Configuration imported successfully".to_string())
}
