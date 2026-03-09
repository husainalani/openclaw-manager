use crate::utils::{platform, shell};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::command;

/// Agent configuration for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: Option<String>,
    pub workspace: Option<String>,
    #[serde(alias = "agentDir", alias = "agent_dir")]
    pub agent_dir: Option<String>,
    pub model: Option<String>,
    pub sandbox: Option<bool>,
    pub heartbeat: Option<String>,
    pub default: Option<bool>,
    pub subagents: Option<SubagentConfig>,
}

/// Per-agent subagent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubagentConfig {
    #[serde(alias = "allowAgents", alias = "allow_agents")]
    pub allow_agents: Option<Vec<String>>,
}

/// Global subagent defaults
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubagentDefaults {
    #[serde(alias = "maxSpawnDepth", alias = "max_spawn_depth")]
    pub max_spawn_depth: Option<u32>,
    #[serde(alias = "maxChildrenPerAgent", alias = "max_children_per_agent")]
    pub max_children_per_agent: Option<u32>,
    #[serde(alias = "maxConcurrent", alias = "max_concurrent")]
    pub max_concurrent: Option<u32>,
    #[serde(alias = "attachmentsEnabled", alias = "attachments_enabled")]
    pub attachments_enabled: Option<bool>,
    #[serde(alias = "attachmentsMaxTotalBytes", alias = "attachments_max_total_bytes")]
    pub attachments_max_total_bytes: Option<u64>,
}

/// Agent binding rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBinding {
    #[serde(alias = "agentId", alias = "agent_id")]
    pub agent_id: String,
    #[serde(alias = "matchRule", alias = "match_rule")]
    pub match_rule: MatchRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    pub channel: Option<String>,
    #[serde(alias = "accountId", alias = "account_id")]
    pub account_id: Option<String>,
    pub peer: Option<serde_json::Value>,
}

/// Combined agents config for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfigResponse {
    pub agents: Vec<AgentInfo>,
    pub bindings: Vec<AgentBinding>,
    pub subagent_defaults: SubagentDefaults,
}

/// Get multi-agent routing configuration
#[command]
pub async fn get_agents_config() -> Result<AgentsConfigResponse, String> {
    info!("[Agents] Getting agents configuration...");
    let config = super::load_openclaw_config()?;

    let mut agents = Vec::new();
    let mut bindings = Vec::new();

    // Read agents.list — supports both array format (correct) and object format (legacy)
    if let Some(list_arr) = config.pointer("/agents/list").and_then(|v| v.as_array()) {
        // Correct format: array of { id, workspace, agentDir, model, ... }
        for agent_val in list_arr {
            agents.push(AgentInfo {
                id: agent_val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                name: agent_val.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                workspace: agent_val.get("workspace").and_then(|v| v.as_str()).map(|s| s.to_string()),
                agent_dir: agent_val.get("agentDir").and_then(|v| v.as_str()).map(|s| s.to_string()),
                model: agent_val.pointer("/model/primary").and_then(|v| v.as_str()).map(|s| s.to_string()),
                sandbox: agent_val.get("sandbox").and_then(|v| v.as_bool()),
                heartbeat: agent_val.pointer("/heartbeat/every").and_then(|v| v.as_str()).map(|s| s.to_string()),
                default: agent_val.get("default").and_then(|v| v.as_bool()),
                subagents: agent_val.get("subagents").and_then(|v| {
                    let allow = v.get("allowAgents").and_then(|a| a.as_array()).map(|arr| {
                        arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
                    });
                    Some(SubagentConfig { allow_agents: allow })
                }),
            });
        }
    } else if let Some(list_obj) = config.pointer("/agents/list").and_then(|v| v.as_object()) {
        // Legacy format: object with id as keys
        for (id, agent_val) in list_obj {
            agents.push(AgentInfo {
                id: id.clone(),
                name: agent_val.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                workspace: agent_val.get("workspace").and_then(|v| v.as_str()).map(|s| s.to_string()),
                agent_dir: agent_val.get("agentDir").and_then(|v| v.as_str()).map(|s| s.to_string()),
                model: agent_val.pointer("/model/primary").and_then(|v| v.as_str()).map(|s| s.to_string()),
                sandbox: agent_val.get("sandbox").and_then(|v| v.as_bool()),
                heartbeat: agent_val.pointer("/heartbeat/every").and_then(|v| v.as_str()).map(|s| s.to_string()),
                default: agent_val.get("default").and_then(|v| v.as_bool()),
                subagents: agent_val.get("subagents").and_then(|v| {
                    let allow = v.get("allowAgents").and_then(|a| a.as_array()).map(|arr| {
                        arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
                    });
                    Some(SubagentConfig { allow_agents: allow })
                }),
            });
        }
    }

    // Read bindings — check top-level first (correct), then agents.bindings (legacy)
    let bindings_arr = config.get("bindings").and_then(|v| v.as_array())
        .or_else(|| config.pointer("/agents/bindings").and_then(|v| v.as_array()));

    if let Some(bindings_arr) = bindings_arr {
        for binding_val in bindings_arr {
            let empty_match = json!({});
            let match_obj = binding_val.get("match").unwrap_or(&empty_match);

            bindings.push(AgentBinding {
                agent_id: binding_val.get("agentId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                match_rule: MatchRule {
                    channel: match_obj.get("channel").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    account_id: match_obj.get("accountId").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    peer: match_obj.get("peer").cloned(),
                }
            });
        }
    }

    // Read global subagent defaults from agents.defaults.subagents and tools.sessions_spawn.attachments
    let subagent_defaults = if let Some(sub_val) = config.pointer("/agents/defaults/subagents") {
        SubagentDefaults {
            max_spawn_depth: sub_val.get("maxSpawnDepth").and_then(|v| v.as_u64()).map(|v| v as u32),
            max_children_per_agent: sub_val.get("maxChildrenPerAgent").and_then(|v| v.as_u64()).map(|v| v as u32),
            max_concurrent: sub_val.get("maxConcurrent").and_then(|v| v.as_u64()).map(|v| v as u32),
            attachments_enabled: config.pointer("/tools/sessions_spawn/attachments/enabled").and_then(|v| v.as_bool()),
            attachments_max_total_bytes: config.pointer("/tools/sessions_spawn/attachments/maxTotalBytes").and_then(|v| v.as_u64()),
        }
    } else {
        SubagentDefaults {
            max_spawn_depth: None,
            max_children_per_agent: None,
            max_concurrent: None,
            attachments_enabled: config.pointer("/tools/sessions_spawn/attachments/enabled").and_then(|v| v.as_bool()),
            attachments_max_total_bytes: config.pointer("/tools/sessions_spawn/attachments/maxTotalBytes").and_then(|v| v.as_u64()),
        }
    };

    info!("[Agents] Found {} agents, {} bindings", agents.len(), bindings.len());
    Ok(AgentsConfigResponse { agents, bindings, subagent_defaults })
}

/// Save (add/update) an agent
#[command]
pub async fn save_agent(agent: AgentInfo) -> Result<String, String> {
    info!("[Agents] Saving agent: {}", agent.id);
    let mut config = super::load_openclaw_config()?;

    // Ensure agents object exists
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }

    // Build agent object (array element format with "id" field)
    let mut agent_obj = json!({ "id": agent.id });
    if let Some(name) = &agent.name {
        if !name.is_empty() {
            agent_obj["name"] = json!(name);
        }
    }
    if let Some(workspace) = &agent.workspace {
        if !workspace.is_empty() {
            agent_obj["workspace"] = json!(workspace);
        }
    }
    if let Some(agent_dir) = &agent.agent_dir {
        if !agent_dir.is_empty() {
            agent_obj["agentDir"] = json!(agent_dir);
        }
    }
    if let Some(model) = &agent.model {
        if !model.is_empty() {
            agent_obj["model"] = json!({ "primary": model });
        }
    }
    if let Some(sandbox) = agent.sandbox {
        agent_obj["sandbox"] = json!(sandbox);
    }
    if let Some(heartbeat) = &agent.heartbeat {
        if !heartbeat.is_empty() {
            agent_obj["heartbeat"] = json!({ "every": heartbeat });
        }
    }
    if let Some(is_default) = agent.default {
        if is_default {
            agent_obj["default"] = json!(true);
        }
    }
    if let Some(sub) = &agent.subagents {
        if let Some(allow) = &sub.allow_agents {
            if !allow.is_empty() {
                agent_obj["subagents"] = json!({ "allowAgents": allow });
            }
        }
    }

    // Migrate legacy object format to array if needed
    let mut list = if let Some(arr) = config["agents"].get("list").and_then(|v| v.as_array()) {
        arr.clone()
    } else if let Some(obj) = config["agents"].get("list").and_then(|v| v.as_object()) {
        // Convert legacy object to array
        obj.iter().map(|(id, val)| {
            let mut entry = val.clone();
            entry["id"] = json!(id);
            entry
        }).collect()
    } else {
        Vec::new()
    };

    // For NEW agents: use `openclaw agents add <id> --workspace <dir>` to create proper directory structure
    // The --workspace flag is required to make the CLI non-interactive
    let is_new_agent = !list.iter().any(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent.id));
    let mut cli_error: Option<String> = None;
    let is_reserved_name = agent.id.eq_ignore_ascii_case("main"); // Check if name is "main" to bypass CLI

    if is_new_agent {
        if !is_reserved_name {
            let openclaw_home = platform::get_config_dir();
            let workspace_dir = if let Some(ws) = &agent.workspace {
                ws.clone()
            } else if agent.default == Some(true) {
                std::path::Path::new(&openclaw_home).join("workspace").to_string_lossy().to_string()
            } else {
                std::path::Path::new(&openclaw_home).join(format!("workspace-{}", agent.id)).to_string_lossy().to_string()
            };

            info!("[Agents] New agent '{}' — running `openclaw agents add --workspace {}`", agent.id, workspace_dir);
            match shell::run_openclaw(&["agents", "add", &agent.id, "--workspace", &workspace_dir]) {
                Ok(output) => {
                    info!("[Agents] openclaw agents add succeeded: {}", output);
                }
                Err(e) => {
                    // NOTE: The CLI may exit with code 1 due to TUI stdin issues in non-interactive mode,
                    // but it still writes the agent entry to openclaw.json successfully.
                    warn!("[Agents] openclaw agents add exited with error (may still have written config): {}", e);
                    cli_error = Some(e);
                }
            }

            // CRITICAL: Always reload config after CLI runs — it may have written the entry
            config = super::load_openclaw_config()?;
            list = if let Some(arr) = config["agents"].get("list").and_then(|v| v.as_array()) {
                arr.clone()
            } else if let Some(obj) = config["agents"].get("list").and_then(|v| v.as_object()) {
                obj.iter().map(|(id, val)| {
                    let mut entry = val.clone();
                    entry["id"] = json!(id);
                    entry
                }).collect()
            } else {
                Vec::new()
            };
        } else {
             info!("[Agents] Skipping CLI for reserved name '{}', will create manually.", agent.id);
        }
    }

    // Find agent in list (handle case-insensitive match if CLI normalized the ID, e.g. AgentTest -> agenttest)
    let match_index = list.iter().position(|a| {
        a.get("id").and_then(|v| v.as_str()) == Some(&agent.id)
    }).or_else(|| {
        list.iter().position(|a| {
             a.get("id").and_then(|v| v.as_str()).map(|s| s.to_lowercase()) == Some(agent.id.to_lowercase())
        })
    });

    // Helper closure to create agent directories
    let ensure_directories = |agent_entry: &serde_json::Value| {
        let openclaw_home = platform::get_config_dir();

        // 1. Agent Config Directory
        // Use configured 'agentDir' or default to ~/.openclaw/agents/<id>/agent
        // The CLI standard is to have the agent files inside an `agent` subdirectory
        let agent_dir_path = if let Some(dir) = agent_entry.get("agentDir").and_then(|v| v.as_str()) {
             std::path::PathBuf::from(dir)
        } else {
             let id = agent_entry.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
             std::path::Path::new(&openclaw_home).join("agents").join(id).join("agent")
        };

        if !agent_dir_path.exists() {
             info!("[Agents] Creating agent directory: {:?}", agent_dir_path);
             let _ = std::fs::create_dir_all(&agent_dir_path);
        }

        // SOUL.md
        let soul_path = agent_dir_path.join("SOUL.md");
        if !soul_path.exists() {
             info!("[Agents] SOUL.md missing, creating default");
             let name = agent_entry.get("name").and_then(|v| v.as_str()).unwrap_or("agent");
             let default_soul = format!("You are {}, a helpful AI assistant.", name);
             let _ = std::fs::write(soul_path, default_soul);
        }

        // models.json
        let models_path = agent_dir_path.join("models.json");
        if !models_path.exists() {
             info!("[Agents] models.json missing, creating default");
             let default_models = json!({
                "providers": {
                    "glm": {
                        "baseUrl": "https://api.z.ai/api/anthropic",
                        "apiKey": "",
                        "models": [
                            {
                                "id": "glm-4",
                                "name": "GLM-4",
                                "api": "openai-completions",
                                "reasoning": false,
                                "input": ["text", "image"],
                                "contextWindow": 128000,
                                "maxTokens": 8192
                            }
                        ]
                    }
                }
             });
             // Pretty print the JSON
             if let Ok(content) = serde_json::to_string_pretty(&default_models) {
                 let _ = std::fs::write(models_path, content);
             }
        }

        // 2. Workspace Directory
        // Use configured 'workspace' or default to ~/.openclaw/workspace-<id>
        let workspace_path = if let Some(ws) = agent_entry.get("workspace").and_then(|v| v.as_str()) {
             std::path::PathBuf::from(ws)
        } else {
             let id = agent_entry.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
             std::path::Path::new(&openclaw_home).join(format!("workspace-{}", id))
        };

        if !workspace_path.exists() {
             info!("[Agents] Creating workspace directory: {:?}", workspace_path);
             let _ = std::fs::create_dir_all(&workspace_path);
        }

        // Return paths to update config if they were defaults
        (agent_dir_path.to_string_lossy().to_string(), workspace_path.to_string_lossy().to_string())
    };

    // Update or add the agent
    if let Some(idx) = match_index {
        let existing = &mut list[idx];

        // Merge: only overwrite fields the user explicitly set (non-empty)
        if let Some(name) = &agent.name {
            if !name.is_empty() {
                existing["name"] = json!(name);
            }
        }
        if let Some(model) = &agent.model {
            if !model.is_empty() {
                existing["model"] = json!({ "primary": model });
            }
        }
        if let Some(is_default) = agent.default {
            if is_default {
                existing["default"] = json!(true);
            }
        }

        // Enforce "Main" agent properties
        if agent.id.eq_ignore_ascii_case("main") {
            // "Main" should always be default unless user explicitly sets another default (which handles itself)
            // But to ensure fallback behavior, we mark it.
            existing["default"] = json!(true);
        }

        if let Some(sub) = &agent.subagents {
            if let Some(allow) = &sub.allow_agents {
                if !allow.is_empty() {
                    existing["subagents"] = json!({ "allowAgents": allow });
                }
            }
        }
        if let Some(sandbox) = agent.sandbox {
            existing["sandbox"] = json!(sandbox);
        }
        if let Some(heartbeat) = &agent.heartbeat {
            if !heartbeat.is_empty() {
                existing["heartbeat"] = json!({ "every": heartbeat });
            }
        }

        // Repair directories for existing agent
        let _ = ensure_directories(existing);

    } else {
        // Not found in config (New agent, manual addition)

        // If we tried to create it via CLI and it's missing (and NOT reserved), that means CLI strictly failed.
        if let Some(err) = cli_error {
             if !is_reserved_name {
                 return Err(format!("Failed to create agent via CLI: {}. Check logs or name uniqueness.", err));
             }
        }

        // Add to list
        let mut new_entry = agent_obj.clone();

        // Ensure directories and get default paths if we need to explicitly save them
        let (actual_agent_dir, actual_workspace) = ensure_directories(&new_entry);

        // If user didn't specify paths, save the defaults we just used/created
        if new_entry.get("agentDir").is_none() {
             new_entry["agentDir"] = json!(actual_agent_dir);
        }
        if new_entry.get("workspace").is_none() {
             new_entry["workspace"] = json!(actual_workspace);
        }

        list.push(new_entry);
    }

    config["agents"]["list"] = json!(list);

    // Auto-create binding if a Telegram bot account is available and this agent has no binding yet
    let agent_id = agent.id.clone();
    let available_accounts: Vec<String> = config.pointer("/channels/telegram/accounts")
        .and_then(|v| v.as_object())
        .map(|accts| accts.keys().cloned().collect())
        .unwrap_or_default();

    if !available_accounts.is_empty() {
        // Check if this agent already has ANY binding
        let has_existing_binding = config.get("bindings")
            .and_then(|v| v.as_array())
            .map(|bindings| bindings.iter().any(|b| {
                b.get("agentId").and_then(|v| v.as_str()) == Some(&agent_id)
            }))
            .unwrap_or(false);

        if !has_existing_binding {
            // Find accounts already bound to other agents
            let bound_accounts: Vec<String> = config.get("bindings")
                .and_then(|v| v.as_array())
                .map(|bindings| bindings.iter().filter_map(|b| {
                    b.get("match").and_then(|m| m.get("accountId")).and_then(|v| v.as_str()).map(|s| s.to_string())
                }).collect())
                .unwrap_or_default();

            // Prefer: exact match > substring match > first unbound account > first account
            let best_account = available_accounts.iter()
                .find(|a| **a == agent_id) // exact match
                .or_else(|| available_accounts.iter().find(|a| a.contains(&agent_id) || agent_id.contains(a.as_str()))) // substring
                .or_else(|| available_accounts.iter().find(|a| !bound_accounts.contains(a))) // unbound
                .or_else(|| available_accounts.first()) // fallback
                .cloned();

            if let Some(account_id) = best_account {
                info!("[Agents] Auto-creating binding for agent '{}' → account '{}'", agent_id, account_id);
                if config.get("bindings").is_none() {
                    config["bindings"] = json!([]);
                }
                if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
                    bindings.push(json!({
                        "agentId": agent_id,
                        "match": { "channel": "telegram", "accountId": account_id }
                    }));
                }
            }
        }
    }

    super::save_openclaw_config(&config)?;
    Ok(format!("Agent '{}' saved", agent.id))
}

/// Save global subagent defaults
#[command]
pub async fn save_subagent_defaults(defaults: SubagentDefaults) -> Result<String, String> {
    info!("[Agents] Saving subagent defaults");
    let mut config = super::load_openclaw_config()?;

    // Ensure agents.defaults exists
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }

    let mut sub_obj = json!({});
    if let Some(depth) = defaults.max_spawn_depth {
        sub_obj["maxSpawnDepth"] = json!(depth);
    }
    if let Some(children) = defaults.max_children_per_agent {
        sub_obj["maxChildrenPerAgent"] = json!(children);
    }
    if let Some(concurrent) = defaults.max_concurrent {
        sub_obj["maxConcurrent"] = json!(concurrent);
    }

    config["agents"]["defaults"]["subagents"] = sub_obj;

    // Subagent sessions_spawn inline file attachments
    if defaults.attachments_enabled.is_some() || defaults.attachments_max_total_bytes.is_some() {
        if config.get("tools").is_none() {
            config["tools"] = json!({});
        }
        if config["tools"].get("sessions_spawn").is_none() {
            config["tools"]["sessions_spawn"] = json!({});
        }
        if config["tools"]["sessions_spawn"].get("attachments").is_none() {
            config["tools"]["sessions_spawn"]["attachments"] = json!({});
        }

        if let Some(enabled) = defaults.attachments_enabled {
            config["tools"]["sessions_spawn"]["attachments"]["enabled"] = json!(enabled);
        }
        if let Some(max_bytes) = defaults.attachments_max_total_bytes {
            config["tools"]["sessions_spawn"]["attachments"]["maxTotalBytes"] = json!(max_bytes);
        }
    }

    super::save_openclaw_config(&config)?;
    Ok("Subagent defaults saved".to_string())
}

/// Delete an agent
#[command]
pub async fn delete_agent(agent_id: String) -> Result<String, String> {
    info!("[Agents] Deleting agent: {}", agent_id);
    let mut config = super::load_openclaw_config()?;

    // 1. Find the agent to get its paths (before deleting from config)
    let mut agent_dir_to_delete: Option<String> = None;
    let mut workspace_to_delete: Option<String> = None;

    if let Some(list) = config.pointer("/agents/list").and_then(|v| v.as_array()) {
        if let Some(agent) = list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent_id)) {
            // Get agent directory
            if let Some(dir) = agent.get("agentDir").and_then(|v| v.as_str()) {
                agent_dir_to_delete = Some(dir.to_string());
            }
            // Get workspace directory
            if let Some(ws) = agent.get("workspace").and_then(|v| v.as_str()) {
                workspace_to_delete = Some(ws.to_string());
            } else {
                // Fallback: deduce workspace path if default pattern was used
                let openclaw_home = platform::get_config_dir();
                let default_ws = std::path::Path::new(&openclaw_home).join(format!("workspace-{}", agent_id));
                if default_ws.exists() {
                    workspace_to_delete = Some(default_ws.to_string_lossy().to_string());
                }
            }
        }
    }

    // 2. Delete the files (if they exist)
    // We do this BEFORE updating config, but we don't abort if it fails (just warn)
    // because we still want to remove the broken/stale entry from config.

    if let Some(agent_dir) = agent_dir_to_delete {
        let path = std::path::Path::new(&agent_dir);
        let mut path_to_remove = path;

        // Safety check: if the standard structure is ~/.openclaw/agents/<id>/agent
        // we want to delete the <id> folder to also clear sessions and other subdirectories.
        // We only do this if we are certain the grandparent is "agents".
        if path.ends_with("agent") {
            if let Some(parent) = path.parent() {
                if let Some(grandparent) = parent.parent() {
                    let grandparent_name = grandparent.file_name().unwrap_or_default().to_string_lossy();
                    if grandparent_name == "agents" {
                        path_to_remove = parent;
                    }
                }
            }
        }

        // Final safety guard: NEVER delete the openclaw_home itself or anything suspiciously short
        let openclaw_home = platform::get_config_dir();
        if path_to_remove.to_string_lossy() == openclaw_home || path_to_remove.components().count() <= 2 {
            warn!("[Agents] SAFETY ABORT: Refusing to delete root or dangerously short path: {:?}", path_to_remove);
        } else if path_to_remove.exists() {
            info!("[Agents] Removing agent directory tree: {:?}", path_to_remove);
            if let Err(e) = std::fs::remove_dir_all(path_to_remove) {
                warn!("[Agents] Failed to remove agent directory {:?}: {}", path_to_remove, e);
            }
        }
    } else {
        // Fallback: try default location if not specified in config
        let openclaw_home = platform::get_config_dir();
        // Default structure is now ~/.openclaw/agents/<id> (which contains agent/, sessions/, etc.)
        let default_agent_root = std::path::Path::new(&openclaw_home).join("agents").join(&agent_id);

        if default_agent_root.exists() {
             info!("[Agents] Removing default agent directory tree: {:?}", default_agent_root);
             if let Err(e) = std::fs::remove_dir_all(&default_agent_root) {
                warn!("[Agents] Failed to remove default agent directory: {}", e);
            }
        }
    }

    if let Some(workspace) = workspace_to_delete {
        let path = std::path::Path::new(&workspace);
        let openclaw_home = platform::get_config_dir();

        if path.to_string_lossy() == openclaw_home || path.components().count() <= 2 {
            warn!("[Agents] SAFETY ABORT: Refusing to delete root or dangerously short workspace path: {:?}", path);
        } else if path.exists() {
            info!("[Agents] Removing workspace directory: {}", workspace);
            if let Err(e) = std::fs::remove_dir_all(path) {
                warn!("[Agents] Failed to remove workspace directory {}: {}", workspace, e);
            }
        }
    }

    // 3. Remove from agents.list (array format)
    if let Some(list) = config.pointer_mut("/agents/list").and_then(|v| v.as_array_mut()) {
        list.retain(|a| a.get("id").and_then(|v| v.as_str()) != Some(&agent_id));
    }

    // Remove related bindings (top-level)
    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        bindings.retain(|b| b.get("agentId").and_then(|v| v.as_str()) != Some(&agent_id));
    }
    // Also clean legacy agents.bindings
    if let Some(bindings) = config.pointer_mut("/agents/bindings").and_then(|v| v.as_array_mut()) {
        bindings.retain(|b| b.get("agentId").and_then(|v| v.as_str()) != Some(&agent_id));
    }

    super::save_openclaw_config(&config)?;
    Ok(format!("Agent '{}' and its files were deleted", agent_id))
}

/// Save an agent binding rule
#[command]

pub async fn save_agent_binding(binding: AgentBinding) -> Result<String, String> {
    info!("[Agents] Saving binding for agent: {}", binding.agent_id);
    let mut config = super::load_openclaw_config()?;

    // Ensure top-level bindings array exists
    if config.get("bindings").is_none() {
        config["bindings"] = json!([]);
    }

    // Migrate legacy agents.bindings to top-level if present
    if let Some(legacy) = config.pointer("/agents/bindings").and_then(|v| v.as_array()).map(|a| a.clone()) {
        if let Some(top) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
            for b in legacy {
                top.push(b);
            }
        }
        // Remove legacy location
        if let Some(agents) = config.get_mut("agents").and_then(|v| v.as_object_mut()) {
            agents.remove("bindings");
        }
    }

    let mut match_obj = json!({});
    if let Some(ch) = &binding.match_rule.channel {
        if !ch.is_empty() { match_obj["channel"] = json!(ch); }
    }
    if let Some(acc) = &binding.match_rule.account_id {
        if !acc.is_empty() { match_obj["accountId"] = json!(acc); }
    }
    if let Some(peer) = &binding.match_rule.peer {
        match_obj["peer"] = peer.clone();
    }

    let binding_obj = json!({
        "agentId": binding.agent_id,
        "match": match_obj
    });

    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        bindings.push(binding_obj);
    }

    super::save_openclaw_config(&config)?;
    Ok(format!("Binding for agent '{}' saved", binding.agent_id))
}

/// Delete an agent binding by index
#[command]
pub async fn delete_agent_binding(index: usize) -> Result<String, String> {
    info!("[Agents] Deleting binding at index: {}", index);
    let mut config = super::load_openclaw_config()?;

    // Try top-level bindings first (correct location)
    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        if index < bindings.len() {
            bindings.remove(index);
            super::save_openclaw_config(&config)?;
            return Ok(format!("Binding at index {} deleted", index));
        } else {
            return Err(format!("Binding index {} out of range", index));
        }
    }

    // Fallback to legacy agents.bindings
    if let Some(bindings) = config.pointer_mut("/agents/bindings").and_then(|v| v.as_array_mut()) {
        if index < bindings.len() {
            bindings.remove(index);
            super::save_openclaw_config(&config)?;
            return Ok(format!("Binding at index {} deleted", index));
        } else {
            return Err(format!("Binding index {} out of range", index));
        }
    }

    Err("No bindings found".to_string())
}

// ============ Agent Soul / Personality ============

/// Read the personality (SOUL.md) for an agent
#[command]
pub async fn get_agent_system_prompt(agent_id: String, workspace: Option<String>) -> Result<String, String> {
    let base = workspace.unwrap_or_else(|| platform::get_config_dir());
    let sep = if cfg!(windows) { "\\" } else { "/" };

    // Resolve agent directory from config to handle case where ID != dir name
    let config = super::load_openclaw_config().map_err(|e| e.to_string())?;
    let agent_dir_rel = config.pointer("/agents/list")
        .and_then(|v| v.as_array())
        .and_then(|list| list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent_id)))
        .and_then(|agent| agent.get("agentDir").and_then(|v| v.as_str()))
        .map(|s| s.replace("/", sep)) //normalize separators
        .unwrap_or_else(|| format!("agents{}{}", sep, agent_id)); // fallback

    // If agentDir is already an absolute path, use it directly; otherwise join with base
    let dir_config = if std::path::Path::new(&agent_dir_rel).is_absolute() {
        agent_dir_rel
    } else {
        format!("{}{}{}", base, sep, agent_dir_rel)
    };

    // Try locations in order of likelihood - prioritizing the CORRECT one first
    let paths = vec![
        format!("{}{}SOUL.md", dir_config, sep),                                // 1. agents/{id}/SOUL.md (CORRECT)
        format!("{}{}{}{}{}{}SOUL.md", base, sep, "agent", sep, agent_id, sep), // 2. agent/{id}/SOUL.md (Legacy/Buggy)
        format!("{}{}agent{}SOUL.md", dir_config, sep, sep),                    // 3. agents/{id}/agent/SOUL.md (Legacy/Buggy)
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            info!("[Agents] Found SOUL.md at: {}", path);
            return std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read SOUL.md: {}", e));
        }
    }

    Ok(String::new())
}

/// Save the personality (SOUL.md) for an agent
#[command]
pub async fn save_agent_system_prompt(agent_id: String, workspace: Option<String>, content: String) -> Result<String, String> {
    let base = workspace.unwrap_or_else(|| platform::get_config_dir());
    let sep = if cfg!(windows) { "\\" } else { "/" };

    // Resolve agent directory from config
    let config = super::load_openclaw_config().map_err(|e| e.to_string())?;
    let agent_dir_rel = config.pointer("/agents/list")
        .and_then(|v| v.as_array())
        .and_then(|list| list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent_id)))
        .and_then(|agent| agent.get("agentDir").and_then(|v| v.as_str()))
        .map(|s| s.replace("/", sep))
        .unwrap_or_else(|| format!("agents{}{}", sep, agent_id));

    // If agentDir is already an absolute path, use it directly; otherwise join with base
    let dir_config = if std::path::Path::new(&agent_dir_rel).is_absolute() {
        agent_dir_rel
    } else {
        format!("{}{}{}", base, sep, agent_dir_rel)
    };

    // ONLY save to the correct canonical path
    let path = format!("{}{}SOUL.md", dir_config, sep);

    if let Some(parent) = std::path::Path::new(&path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory for {}: {}", path, e));
        }
    }

    match std::fs::write(&path, &content) {
        Ok(_) => {
            info!("[Agents] Wrote SOUL.md to: {}", path);
            Ok(format!("Personality (SOUL.md) saved for agent '{}'", agent_id))
        },
        Err(e) => Err(format!("Failed to save SOUL.md to {}: {}", path, e))
    }
}

/// Test agent routing: given an account ID, find which agent handles it
#[command]
pub async fn test_agent_routing(account_id: String) -> Result<serde_json::Value, String> {
    let config = super::load_openclaw_config()?;

    // Walk through bindings to find a match
    let bindings = config.get("bindings").and_then(|v| v.as_array());

    if let Some(bindings) = bindings {
        let empty_match = json!({});
        for binding in bindings {
            let match_obj = binding.get("match").unwrap_or(&empty_match);
            let binding_account = match_obj.get("accountId").and_then(|v| v.as_str());
            let binding_channel = match_obj.get("channel").and_then(|v| v.as_str());

            // Check if this binding matches
            let account_matches = binding_account.map(|a| a == account_id).unwrap_or(true); // None = catch-all
            let channel_matches = binding_channel.map(|c| c == "telegram").unwrap_or(true);

            if account_matches && channel_matches {
                let agent_id = binding.get("agentId").and_then(|v| v.as_str()).unwrap_or("unknown");

                // Find agent details
                let agent_info = config.pointer("/agents/list")
                    .and_then(|v| v.as_array())
                    .and_then(|list| list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(agent_id)));

                // Read SOUL.md preview (try all 3 locations)
                let base = platform::get_config_dir();
                let sep = if cfg!(windows) { "\\" } else { "/" };
                let agent_dir_rel = agent_info.and_then(|a| a.get("agentDir").and_then(|v| v.as_str()))
                    .map(|s| s.replace("/", sep))
                    .unwrap_or_else(|| format!("agents{}{}", sep, agent_id));

                let dir_config = format!("{}{}{}", base, sep, agent_dir_rel);
                let check_paths = vec![
                    format!("{}{}{}{}{}{}SOUL.md", base, sep, "agent", sep, agent_id, sep),
                    format!("{}{}agent{}SOUL.md", dir_config, sep, sep),
                    format!("{}{}SOUL.md", dir_config, sep),
                ];

                let mut prompt_preview = String::new();
                for path in check_paths {
                    if std::path::Path::new(&path).exists() {
                        prompt_preview = std::fs::read_to_string(&path).unwrap_or_default();
                        break;
                    }
                }
                let prompt_preview = if prompt_preview.len() > 200 {
                    format!("{}...", &prompt_preview[..200])
                } else {
                    prompt_preview
                };

                return Ok(json!({
                    "matched": true,
                    "agent_id": agent_id,
                    "agent_dir": agent_info.and_then(|a| a.get("agentDir").and_then(|v| v.as_str())),
                    "model": agent_info.and_then(|a| a.pointer("/model/primary").and_then(|v| v.as_str())),
                    "system_prompt_preview": prompt_preview,
                    "binding": binding
                }));
            }
        }
    }

    Ok(json!({
        "matched": false,
        "agent_id": "default",
        "message": "No specific binding found. Messages will be handled by the default agent."
    }))
}
