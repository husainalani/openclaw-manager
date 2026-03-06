use crate::models::ChannelConfig;
use crate::utils::{file, platform, shell};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::command;

/// Telegram account info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramAccount {
    pub id: String,
    #[serde(alias = "botToken", alias = "bot_token")]
    pub bot_token: String,
    #[serde(alias = "groupPolicy", alias = "group_policy")]
    pub group_policy: Option<String>,
    #[serde(alias = "dmPolicy", alias = "dm_policy")]
    pub dm_policy: Option<String>,
    #[serde(alias = "streamMode", alias = "stream_mode")]
    pub stream_mode: Option<String>,
    #[serde(alias = "exclusiveTopics", alias = "exclusive_topics")]
    pub exclusive_topics: Option<Vec<String>>,
    pub groups: Option<serde_json::Value>,
    pub primary: Option<bool>,
    #[serde(alias = "allowFrom", alias = "allow_from")]
    pub allow_from: Option<Vec<String>>,
}

/// Feishu plugin status
#[derive(Debug, Serialize, Deserialize)]
pub struct FeishuPluginStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub plugin_name: Option<String>,
}

/// Get channel configuration - read from openclaw.json and env file
#[command]
pub async fn get_channels_config() -> Result<Vec<ChannelConfig>, String> {
    info!("[Channel Config] Getting channel configuration list...");

    let config = super::load_openclaw_config()?;
    let channels_obj = config.get("channels").cloned().unwrap_or(json!({}));
    let env_path = platform::get_env_file_path();
    debug!("[Channel Config] Environment file path: {}", env_path);

    let mut channels = Vec::new();

    // List of supported channel types and their test fields
    let channel_types = vec![
        ("telegram", "telegram", vec!["userId"]),
        ("discord", "discord", vec!["testChannelId"]),
        ("slack", "slack", vec!["testChannelId"]),
        ("feishu", "feishu", vec!["testChatId"]),
        ("whatsapp", "whatsapp", vec![]),
        ("imessage", "imessage", vec![]),
        ("wechat", "wechat", vec![]),
        ("dingtalk", "dingtalk", vec![]),
    ];

    for (channel_id, channel_type, test_fields) in channel_types {
        let channel_config = channels_obj.get(channel_id);

        let enabled = channel_config
            .and_then(|c| c.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Convert channel configuration to HashMap
        let mut config_map: HashMap<String, Value> = if let Some(cfg) = channel_config {
            if let Some(obj) = cfg.as_object() {
                obj.iter()
                    .filter(|(k, _)| *k != "enabled") // Exclude enabled field
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // Read test fields from env file
        for field in test_fields {
            let env_key = format!(
                "OPENCLAW_{}_{}",
                channel_id.to_uppercase(),
                field.to_uppercase()
            );
            if let Some(value) = file::read_env_value(&env_path, &env_key) {
                config_map.insert(field.to_string(), json!(value));
            }
        }

        // Clean up any legacy 'pairing' or 'allowlist' keys that shouldn't be here
        config_map.remove("pairing");
        config_map.remove("allowlist");

        // Determine if configured (has any non-empty configuration items)
        let has_config = !config_map.is_empty() || enabled;

        channels.push(ChannelConfig {
            id: channel_id.to_string(),
            channel_type: channel_type.to_string(),
            enabled: has_config,
            config: config_map,
        });
    }

    info!("[Channel Config] Returned {} channel configurations", channels.len());
    for ch in &channels {
        debug!("[Channel Config] - {}: enabled={}", ch.id, ch.enabled);
    }
    Ok(channels)
}

/// Save channel configuration - save to openclaw.json
#[command]
pub async fn save_channel_config(channel: ChannelConfig) -> Result<String, String> {
    info!(
        "[Save Channel Config] Saving channel configuration: {} ({})",
        channel.id, channel.channel_type
    );

    let mut config = super::load_openclaw_config()?;
    let env_path = platform::get_env_file_path();
    debug!("[Save Channel Config] Environment file path: {}", env_path);

    // DEBUG: Log received keys
    info!("[Save Channel Config] Config keys: {:?}", channel.config.keys());

    // Ensure channels object exists
    if config.get("channels").is_none() {
        config["channels"] = json!({});
    }

    if config.get("plugins").is_none() {
        config["plugins"] = json!({
            "allow": [],
            "entries": {}
        });
    }
    if config["plugins"].get("allow").is_none() {
        config["plugins"]["allow"] = json!([]);
    }
    if config["plugins"].get("entries").is_none() {
        config["plugins"]["entries"] = json!({});
    }

    // These fields are only for testing, not saved to openclaw.json, but saved to env file
    let test_only_fields = vec!["userId", "testChatId", "testChannelId"];

    // Update channels configuration - MERGE with existing
    if let Some(existing_channel) = config["channels"].get_mut(&channel.id).and_then(|v| v.as_object_mut()) {
        existing_channel.insert("enabled".to_string(), json!(true));

        // Clean up legacy invalid keys
        existing_channel.remove("pairing");
        existing_channel.remove("allowlist");

        for (key, value) in &channel.config {
            if test_only_fields.contains(&key.as_str()) {
                let env_key = format!("OPENCLAW_{}_{}", channel.id.to_uppercase(), key.to_uppercase());
                if let Some(val_str) = value.as_str() {
                    let _ = file::set_env_value(&env_path, &env_key, val_str);
                }
            } else {
                 existing_channel.insert(key.clone(), value.clone());
            }
        }
    } else {
        let mut channel_obj = json!({ "enabled": true });

        for (key, value) in &channel.config {
            if test_only_fields.contains(&key.as_str()) {
                let env_key = format!("OPENCLAW_{}_{}", channel.id.to_uppercase(), key.to_uppercase());
                if let Some(val_str) = value.as_str() {
                    let _ = file::set_env_value(&env_path, &env_key, val_str);
                }
            } else {
                channel_obj[key] = value.clone();
            }
        }
        config["channels"][&channel.id] = channel_obj;
    }

    // Cleanup legacy attempts
    if let Some(plugin_entry) = config["plugins"]["entries"].get_mut(&channel.id).and_then(|v| v.as_object_mut()) {
        plugin_entry.remove("allowlist");
        plugin_entry.remove("pairing");
    }
    // Remove global allowlist (invalid at root level)
    if let Some(obj) = config.as_object_mut() {
        obj.remove("allowlist");
    }

    // Save configuration
    info!("[Save Channel Config] Writing configuration file...");
    match super::save_openclaw_config(&config) {
        Ok(_) => {
            info!(
                "[Save Channel Config] {} configuration saved successfully",
                channel.channel_type
            );
            Ok(format!("{} configuration saved", channel.channel_type))
        }
        Err(e) => {
            error!("[Save Channel Config] Failed to save: {}", e);
            Err(e)
        }
    }
}

/// Clear channel configuration - delete specified channel configuration from openclaw.json
#[command]
pub async fn clear_channel_config(channel_id: String) -> Result<String, String> {
    info!("[Clear Channel Config] Clearing channel configuration: {}", channel_id);

    let mut config = super::load_openclaw_config()?;
    let env_path = platform::get_env_file_path();

    // Delete channel from channels object
    if let Some(channels) = config.get_mut("channels").and_then(|v| v.as_object_mut()) {
        channels.remove(&channel_id);
        info!("[Clear Channel Config] Deleted from channels: {}", channel_id);
    }

    // Delete from plugins.allow array
    if let Some(allow_arr) = config.pointer_mut("/plugins/allow").and_then(|v| v.as_array_mut()) {
        allow_arr.retain(|v| v.as_str() != Some(&channel_id));
        info!("[Clear Channel Config] Deleted from plugins.allow: {}", channel_id);
    }

    // Delete from plugins.entries
    if let Some(entries) = config.pointer_mut("/plugins/entries").and_then(|v| v.as_object_mut()) {
        entries.remove(&channel_id);
        info!("[Clear Channel Config] Deleted from plugins.entries: {}", channel_id);
    }

    // Clear related environment variables
    let env_prefixes = vec![
        format!("OPENCLAW_{}_USERID", channel_id.to_uppercase()),
        format!("OPENCLAW_{}_TESTCHATID", channel_id.to_uppercase()),
        format!("OPENCLAW_{}_TESTCHANNELID", channel_id.to_uppercase()),
    ];
    for env_key in env_prefixes {
        let _ = file::remove_env_value(&env_path, &env_key);
    }

    // Save configuration
    match super::save_openclaw_config(&config) {
        Ok(_) => {
            info!("[Clear Channel Config] {} configuration cleared", channel_id);
            Ok(format!("{} configuration cleared", channel_id))
        }
        Err(e) => {
            error!("[Clear Channel Config] Failed to clear: {}", e);
            Err(e)
        }
    }
}

// ============ Telegram Multi-Account Management ============

/// Get all Telegram bot accounts
#[command]
pub async fn get_telegram_accounts() -> Result<Vec<TelegramAccount>, String> {
    info!("[Telegram Accounts] Getting accounts...");
    let config = super::load_openclaw_config()?;

    let mut accounts = Vec::new();

    // Check for multi-account structure: channels.telegram.accounts
    if let Some(accts) = config.pointer("/channels/telegram/accounts").and_then(|v| v.as_object()) {
        for (id, acct_val) in accts {
            accounts.push(TelegramAccount {
                id: id.to_lowercase().replace(' ', "-"),
                bot_token: acct_val.get("botToken").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                group_policy: acct_val.get("groupPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                dm_policy: acct_val.get("dmPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                stream_mode: acct_val.get("streamMode").and_then(|v| v.as_str()).map(|s| s.to_string()),
                exclusive_topics: {
                    // Re-infer exclusive topics from group config
                    // Logic: If a group has requireMention=true and specific topics have requireMention=false, those are exclusive topics.
                    let mut inferred_topics = Vec::new();
                    if let Some(groups_map) = acct_val.get("groups").and_then(|g| g.as_object()) {
                        for (_, group_val) in groups_map {
                             // Check if group is muted (requireMention=true)
                             if group_val.get("requireMention").and_then(|v| v.as_bool()).unwrap_or(false) {
                                 if let Some(topics_map) = group_val.get("topics").and_then(|t| t.as_object()) {
                                     for (tid, tval) in topics_map {
                                         // Check if topic is unmuted (requireMention=false)
                                         if !tval.get("requireMention").and_then(|v| v.as_bool()).unwrap_or(true) {
                                             inferred_topics.push(tid.clone());
                                         }
                                     }
                                 }
                             }
                        }
                    }
                    if inferred_topics.is_empty() { None } else { Some(inferred_topics) }
                },
                groups: acct_val.get("groups").cloned(),
                primary: None, // Will be set below
                allow_from: acct_val.get("allowFrom")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| {
                        if let Some(s) = v.as_str() { Some(s.to_string()) }
                        else if let Some(n) = v.as_i64() { Some(n.to_string()) }
                        else { None }
                    }).collect()),
            });
        }
    }

    // Fallback: single-bot config (botToken at top level)
    if accounts.is_empty() {
        if let Some(token) = config.pointer("/channels/telegram/botToken").and_then(|v| v.as_str()) {
            if !token.is_empty() {
                accounts.push(TelegramAccount {
                    id: "default".to_string(),
                    bot_token: token.to_string(),
                    group_policy: config.pointer("/channels/telegram/groupPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    dm_policy: config.pointer("/channels/telegram/dmPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    stream_mode: config.pointer("/channels/telegram/streamMode").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    exclusive_topics: None,
                    groups: config.pointer("/channels/telegram/groups").cloned(),
                    primary: None,
                    allow_from: config.pointer("/channels/telegram/allowFrom")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| {
                            if let Some(s) = v.as_str() { Some(s.to_string()) }
                            else if let Some(n) = v.as_i64() { Some(n.to_string()) }
                            else { None }
                        }).collect()),
                });
            }
        }
    }



    // Load primary bot account from manager.json (safe from Core schema)
    let manager_config = super::load_manager_config().unwrap_or(json!({}));
    let primary_account_id = manager_config.pointer("/primaryBotAccount").and_then(|v: &Value| v.as_str());

    if let Some(pid) = primary_account_id {
        for acct in &mut accounts {
            if acct.id == pid {
                acct.primary = Some(true);
            } else {
                acct.primary = Some(false);
            }
        }
    }

    info!("[Telegram Accounts] Found {} accounts", accounts.len());
    Ok(accounts)
}

/// Save a Telegram bot account
#[command]
pub async fn save_telegram_account(account: TelegramAccount) -> Result<String, String> {
    // Normalize account ID to lowercase and replace spaces with dashes
    let account_id = account.id.to_lowercase().replace(' ', "-");
    info!("[Telegram Accounts] Saving account: {}", account_id);
    let mut config = super::load_openclaw_config()?;

    // Ensure channels.telegram exists
    if config.get("channels").is_none() {
        config["channels"] = json!({});
    }
    if config["channels"].get("telegram").is_none() {
        config["channels"]["telegram"] = json!({ "enabled": true });
    }

    // Ensure accounts object exists
    if config["channels"]["telegram"].get("accounts").is_none() {
        config["channels"]["telegram"]["accounts"] = json!({});
    }

    // Migrate single-bot to accounts if this is the first additional account
    if let Some(top_token) = config["channels"]["telegram"].get("botToken").and_then(|v| v.as_str()).map(|s| s.to_string()) {
        if !top_token.is_empty() {
            // Move existing single-bot config to accounts["default"]
            let mut existing = json!({
                "botToken": top_token,
                "groupPolicy": config["channels"]["telegram"].get("groupPolicy").cloned().unwrap_or(json!(null)),
                "dmPolicy": config["channels"]["telegram"].get("dmPolicy").cloned().unwrap_or(json!(null)),
                "streamMode": config["channels"]["telegram"].get("streamMode").cloned().unwrap_or(json!(null)),
                "groups": config["channels"]["telegram"].get("groups").cloned().unwrap_or(json!(null)),
            });

            // Migrate allowList
            if let Some(allow_from) = config["channels"]["telegram"].get("allowFrom").cloned() {
                existing["allowFrom"] = allow_from;
            }
             if let Some(group_allow_from) = config["channels"]["telegram"].get("groupAllowFrom").cloned() {
                existing["groupAllowFrom"] = group_allow_from;
            }

            config["channels"]["telegram"]["accounts"]["default"] = existing;

            // Remove top-level single-bot fields
            if let Some(tg) = config["channels"]["telegram"].as_object_mut() {
                tg.remove("botToken");
                tg.remove("groupPolicy");
                tg.remove("dmPolicy");
                tg.remove("streamMode");
                tg.remove("groups");
                tg.remove("allowFrom");
                tg.remove("groupAllowFrom");
            }
        }
    }

    // If this account is set as primary, unset primary for all others
    // (This is now handled by only storing one ID in `meta`, so no need to iterate and clear others manually)

    // Build account object
    let mut acct_obj = json!({
        "botToken": account.bot_token,
    });
    if let Some(gp) = &account.group_policy {
        acct_obj["groupPolicy"] = json!(gp);
    }
    if let Some(dp) = &account.dm_policy {
        acct_obj["dmPolicy"] = json!(dp);
    }

    // Save allowFrom (DM user IDs) — handled independently of dm_policy
    info!("[Telegram Accounts] allow_from received: {:?}", account.allow_from);
    let dm_policy_str = account.dm_policy.as_deref().unwrap_or("");
    if dm_policy_str == "open" {
        // dmPolicy="open" requires allowFrom to include "*"
        acct_obj["allowFrom"] = json!(["*"]);
    } else if let Some(ref af) = account.allow_from {
        if !af.is_empty() {
            // Convert string IDs to numbers where possible for Core compatibility
            let allow_vals: Vec<serde_json::Value> = af.iter().map(|id| {
                if let Ok(n) = id.parse::<i64>() { json!(n) } else { json!(id) }
            }).collect();
            info!("[Telegram Accounts] Saving allowFrom: {:?}", allow_vals);
            acct_obj["allowFrom"] = json!(allow_vals);
        }
    } else {
        // Auto-inherit from primary bot if no explicit allow_from provided
        let primary_id = super::load_manager_config()
            .unwrap_or(json!({}))
            .pointer("/primaryBotAccount")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(pid) = primary_id {
            if pid != account_id {
                // Read primary account's allowFrom
                if let Some(primary_allow) = config.pointer(&format!("/channels/telegram/accounts/{}/allowFrom", pid))
                    .and_then(|v| v.as_array()) {
                    if !primary_allow.is_empty() && primary_allow.iter().any(|v| v.as_str() != Some("*")) {
                        acct_obj["allowFrom"] = json!(primary_allow);
                    }
                }
            }
        }
    }
    if let Some(sm) = &account.stream_mode {
        acct_obj["streamMode"] = json!(sm);
    }
    // Do NOT save primary to the account object (schema limit)
    // if let Some(pr) = account.primary {
    //    if pr { acct_obj["primary"] = json!(true); }
    // }

    // Update meta.primaryBotAccount
    // Update primaryBotAccount in manager.json (to avoid schema validation errors in Core)
    let mut manager_config = super::load_manager_config().unwrap_or(json!({}));

    if account.primary == Some(true) {
        manager_config["primaryBotAccount"] = json!(account_id);

        // --- NEW LOGIC DISABLED: Do NOT auto-create main agent or binding ---
        /*
        // 1. Ensure "main" agent exists pointing to ~/.openclaw/workspace
        let openclaw_home = platform::get_config_dir();
        // Resolve ~/.openclaw/workspace
        let main_workspace = std::path::Path::new(&openclaw_home).join("workspace");
        let main_workspace_str = main_workspace.to_string_lossy().to_string();

        let mut agents_list = if let Some(arr) = config["agents"].get("list").and_then(|v| v.as_array()) {
            arr.clone()
        } else {
            Vec::new()
        };

        let mut main_agent_exists = false;
        for agent in &mut agents_list {
            if agent.get("id").and_then(|v| v.as_str()) == Some("main") {
                main_agent_exists = true;
                // Ensure workspace is set correctly if it was missing or different?
                // For now, let's just assume if it exists, the user might have customized it.
                // But we should ensure the directory exists.
                if let Err(e) = std::fs::create_dir_all(&main_workspace) {
                     error!("[Telegram Accounts] Failed to create main workspace: {}", e);
                }
                break;
            }
        }

        if !main_agent_exists {
            info!("[Telegram Accounts] Creating 'main' agent for primary bot");
            // Create agentDir path: ~/.openclaw/agents/main/agent
            let main_agent_dir = std::path::Path::new(&openclaw_home).join("agents").join("main").join("agent");
            let main_agent_dir_str = main_agent_dir.to_string_lossy().to_string().replace('\\', "/");

            let main_agent = json!({
                "id": "main",
                "name": "General",
                "workspace": main_workspace_str,
                "agentDir": main_agent_dir_str,
                "default": true,
                "model": { "primary": "glm/glm-5" }
            });
            agents_list.push(main_agent);

            // Auto-create workspace directory
             if let Err(e) = std::fs::create_dir_all(&main_workspace) {
                 error!("[Telegram Accounts] Failed to create main workspace: {}", e);
            }
            // Auto-create agentDir and sessions directories
            let _ = std::fs::create_dir_all(&main_agent_dir);
            let sessions_dir = std::path::Path::new(&openclaw_home).join("agents").join("main").join("sessions");
            let _ = std::fs::create_dir_all(&sessions_dir);

            let soul_path = main_workspace.join("SOUL.md");
            if !soul_path.exists() {
                let root_soul = std::path::Path::new(&openclaw_home).join("SOUL.md");
                 if root_soul.exists() {
                     let _ = std::fs::copy(&root_soul, &soul_path);
                 } else {
                     let _ = std::fs::write(&soul_path, "# Primary Agent\n\nYou are the primary assistant.");
                 }
                 let _ = std::fs::write(main_workspace.join("AGENTS.md"), "# Agent Instructions\n\nBe helpful.");
                 let _ = std::fs::write(main_workspace.join("IDENTITY.md"), "name: Primary\nemoji: 🦞");
            }

            // Save updated agents list
             if config.get("agents").is_none() { config["agents"] = json!({}); }
            config["agents"]["list"] = json!(agents_list);
        }

        // 2. Ensure binding exists: main -> account.id
        let mut bindings = if let Some(arr) = config.get("bindings").and_then(|v| v.as_array()) {
            arr.clone()
        } else {
            Vec::new()
        };

        // Remove any existing binding for "main" agent to avoid duplicates/conflicts?
        // Or check if it already points to this account.
        let mut binding_exists = false;
        for b in &mut bindings {
            if b.get("agentId").and_then(|v| v.as_str()) == Some("main") {
                // Update existing binding to point to this account
                 if let Some(m) = b.get_mut("match").and_then(|v| v.as_object_mut()) {
                     m.insert("accountId".to_string(), json!(account.id));
                     m.insert("channel".to_string(), json!("telegram"));
                 }
                 binding_exists = true;
                 break;
            }
        }

        if !binding_exists {
            info!("[Telegram Accounts] Binding 'main' agent to primary bot");
            bindings.push(json!({
                "agentId": "main",
                "match": {
                    "channel": "telegram",
                    "accountId": account.id
                }
            }));
        }
        config["bindings"] = json!(bindings);
        */
        // --- END NEW LOGIC ---

    } else {
        // If we are saving this account and it is NOT primary, check if it WAS the primary account
        let current_primary = manager_config.pointer("/primaryBotAccount").and_then(|v| v.as_str());
        if current_primary == Some(account_id.as_str()) {
            if let Some(obj) = manager_config.as_object_mut() {
                obj.remove("primaryBotAccount");
            }
        }
    }

    if let Err(e) = super::save_manager_config(&manager_config) {
        error!("[Telegram Accounts] Failed to save manager config: {}", e);
        // Continue anyway, as we still want to save the account config
    }

    // Clean up legacy location in openclaw.json
    if let Some(meta) = config.get_mut("meta").and_then(|v| v.as_object_mut()) {
        meta.remove("primaryBotAccount");
    }

    // Handle groups configuration
    // If exclusive_topics is set, we need to modify the group config to enforce it
    // 1. Set group-level requireMention = true (default behavior: ignore everything)
    // 2. Set topic-level requireMention = false for whitelisted topics (exception: auto-reply)
    let mut groups_json = account.groups.clone();

    if let Some(exclusive_topics) = &account.exclusive_topics {
        if !exclusive_topics.is_empty() {
             // We also save the raw list so the UI can reload it (using a hidden field or relying on inference)
             // However, OpenClaw core rejects unknown fields. So we must ONLY output valid config.
             // Strategy: The UI will need to infer exclusive topics from the config structure if we can't save the field.
             // OR: We save it as a comment? No, JSON doesn't support comments.
             // COMPROMISE: We will NOT save "exclusiveTopics" to the file to avoid validation errors.
             // The UI will have to populate the field by checking if a group has topics configured.
             // For now, let's just apply the logic to the groups logic.

            if let Some(groups_map) = groups_json.as_mut().and_then(|g| g.as_object_mut()) {
                for (_, group_val) in groups_map.iter_mut() {
                    if let Some(group_obj) = group_val.as_object_mut() {
                        // Enforce whitelist logic:
                        // 1. Group requires mention (mute general)
                        group_obj.insert("requireMention".to_string(), json!(true));
                        group_obj.insert("enabled".to_string(), json!(true));

                        // 2. Allow specific topics
                        let mut topics_map = serde_json::Map::new();
                        for topic_id in exclusive_topics {
                            let mut topic_config = serde_json::Map::new();
                            topic_config.insert("requireMention".to_string(), json!(false));
                            topics_map.insert(topic_id.clone(), json!(topic_config));
                        }

                        // 3. Explicitly block topics owned by OTHER bot accounts
                        //    This prevents cross-talk when OpenClaw core doesn't
                        //    fall back to group-level requireMention for unlisted topics.
                        if let Some(all_accts) = config.pointer("/channels/telegram/accounts").and_then(|v| v.as_object()) {
                            for (other_id, other_val) in all_accts {
                                if other_id == &account.id { continue; }
                                if let Some(other_groups) = other_val.get("groups").and_then(|g| g.as_object()) {
                                    for (_, other_group) in other_groups {
                                        if let Some(other_topics) = other_group.get("topics").and_then(|t| t.as_object()) {
                                            for (other_tid, _) in other_topics {
                                                if !exclusive_topics.contains(other_tid) && !topics_map.contains_key(other_tid) {
                                                    let mut block_config = serde_json::Map::new();
                                                    block_config.insert("requireMention".to_string(), json!(true));
                                                    topics_map.insert(other_tid.clone(), json!(block_config));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        group_obj.insert("topics".to_string(), json!(topics_map));
                    }
                }
            }
        }
    }

    if let Some(g) = groups_json {
        acct_obj["groups"] = g;
    }

    // NOTE: We do NOT save "exclusiveTopics" field to avoid schema validation errors in OpenClaw core.
    // The UI state for this field might be lost on restart unless we infer it back from the topics structure,
    // but the *behavior* will be correct.
    // Remove any old keys with different casing to prevent duplicates
    // e.g. if "Chronos" exists and we're saving as "chronos", remove "Chronos"
    if let Some(accts) = config.pointer_mut("/channels/telegram/accounts").and_then(|v| v.as_object_mut()) {
        let old_keys: Vec<String> = accts.keys()
            .filter(|k| k.to_lowercase().replace(' ', "-") == account_id && *k != &account_id)
            .cloned()
            .collect();
        for old_key in old_keys {
            info!("[Telegram Accounts] Removing old key '{}' (normalized to '{}')", old_key, account_id);
            accts.remove(&old_key);
        }
    }

    config["channels"]["telegram"]["accounts"][&account_id] = acct_obj;

    // Ensure telegram is enabled and in plugins
    config["channels"]["telegram"]["enabled"] = json!(true);
    if config.get("plugins").is_none() {
        config["plugins"] = json!({ "allow": ["telegram"], "entries": { "telegram": { "enabled": true } } });
    }

    super::save_openclaw_config(&config)?;
    Ok(format!("Account '{}' saved", account_id))
}

/// Delete a Telegram bot account
#[command]
pub async fn delete_telegram_account(account_id: String) -> Result<String, String> {
    let account_id = account_id.to_lowercase().replace(' ', "-");
    info!("[Telegram Accounts] Deleting account: {}", account_id);
    let mut config = super::load_openclaw_config()?;

    if let Some(accts) = config.pointer_mut("/channels/telegram/accounts").and_then(|v| v.as_object_mut()) {
        accts.remove(&account_id);
    }

    // Also clean up any bindings referencing this account
    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        bindings.retain(|b| b.pointer("/match/accountId").and_then(|v| v.as_str()) != Some(&account_id));
    }

    super::save_openclaw_config(&config)?;
    Ok(format!("Account '{}' deleted", account_id))
}

// ============ Feishu Plugin Management ============

/// Check if Feishu plugin is installed
#[command]
pub async fn check_feishu_plugin() -> Result<FeishuPluginStatus, String> {
    info!("[Feishu Plugin] Checking Feishu plugin installation status...");

    // Execute openclaw plugins list command
    match shell::run_openclaw(&["plugins", "list"]) {
        Ok(output) => {
            debug!("[Feishu Plugin] plugins list output: {}", output);

            // Find line containing feishu (case-insensitive)
            let lines: Vec<&str> = output.lines().collect();
            let feishu_line = lines.iter().find(|line| {
                line.to_lowercase().contains("feishu")
            });

            if let Some(line) = feishu_line {
                info!("[Feishu Plugin] Feishu plugin installed: {}", line);

                // Try to parse version number (usually format is "name@version" or "name version")
                let version = if line.contains('@') {
                    line.split('@').last().map(|s| s.trim().to_string())
                } else {
                    // Try to match version number pattern (e.g. 0.1.2)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.iter()
                        .find(|p| p.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                        .map(|s| s.to_string())
                };

                Ok(FeishuPluginStatus {
                    installed: true,
                    version,
                    plugin_name: Some(line.trim().to_string()),
                })
            } else {
                info!("[Feishu Plugin] Feishu plugin not installed");
                Ok(FeishuPluginStatus {
                    installed: false,
                    version: None,
                    plugin_name: None,
                })
            }
        }
        Err(e) => {
            warn!("[Feishu Plugin] Failed to check plugin list: {}", e);
            // If command fails, assume plugin is not installed
            Ok(FeishuPluginStatus {
                installed: false,
                version: None,
                plugin_name: None,
            })
        }
    }
}

/// Install Feishu plugin
#[command]
pub async fn install_feishu_plugin() -> Result<String, String> {
    info!("[Feishu Plugin] Starting Feishu plugin installation...");

    // First check if already installed
    let status = check_feishu_plugin().await?;
    if status.installed {
        info!("[Feishu Plugin] Feishu plugin already installed, skipping");
        return Ok(format!("Feishu plugin already installed: {}", status.plugin_name.unwrap_or_default()));
    }

    // Install Feishu plugin
    // Note: Using @m1heng-clawd/feishu package name
    info!("[Feishu Plugin] Executing openclaw plugins install @m1heng-clawd/feishu ...");
    match shell::run_openclaw(&["plugins", "install", "@m1heng-clawd/feishu"]) {
        Ok(output) => {
            info!("[Feishu Plugin] Installation output: {}", output);

            // Verify installation result
            let verify_status = check_feishu_plugin().await?;
            if verify_status.installed {
                info!("[Feishu Plugin] Feishu plugin installed successfully");
                Ok(format!("Feishu plugin installed successfully: {}", verify_status.plugin_name.unwrap_or_default()))
            } else {
                warn!("[Feishu Plugin] Installation command succeeded but plugin not found");
                Err("Installation command succeeded but plugin not found, please check openclaw version".to_string())
            }
        }
        Err(e) => {
            error!("[Feishu Plugin] Installation failed: {}", e);
            Err(format!("Failed to install Feishu plugin: {}\n\nPlease run manually: openclaw plugins install @m1heng-clawd/feishu", e))
        }
    }
}
