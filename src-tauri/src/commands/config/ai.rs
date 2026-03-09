use crate::models::{
    AIConfigOverview, ConfiguredModel, ConfiguredProvider,
    ModelConfig, OfficialProvider, SuggestedModel,
};
use crate::utils::platform;
use log::info;
use serde_json::{json, Value};
use tauri::command;

/// Get official Provider list (preset templates)
#[command]
pub async fn get_official_providers() -> Result<Vec<OfficialProvider>, String> {
    info!("[Official Provider] Getting official Provider preset list...");

    let providers = vec![
        OfficialProvider {
            id: "anthropic".to_string(),
            name: "Anthropic Claude".to_string(),
            icon: "🟣".to_string(),
            default_base_url: Some("https://api.anthropic.com".to_string()),
            api_type: "anthropic-messages".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/anthropic".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "claude-opus-4-5-20251101".to_string(),
                    name: "Claude Opus 4.5".to_string(),
                    description: Some("Most powerful version, suitable for complex tasks".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "claude-sonnet-4-5-20250929".to_string(),
                    name: "Claude Sonnet 4.5".to_string(),
                    description: Some("Balanced version, high cost-performance ratio".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            icon: "🟢".to_string(),
            default_base_url: Some("https://api.openai.com/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/openai".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "gpt-4o".to_string(),
                    name: "GPT-4o".to_string(),
                    description: Some("Latest multimodal model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(4096),
                    recommended: true,
                },
                SuggestedModel {
                    id: "gpt-4o-mini".to_string(),
                    name: "GPT-4o Mini".to_string(),
                    description: Some("Fast and economical version".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(4096),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "moonshot".to_string(),
            name: "Moonshot".to_string(),
            icon: "🌙".to_string(),
            default_base_url: Some("https://api.moonshot.cn/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/moonshot".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "kimi-k2.5".to_string(),
                    name: "Kimi K2.5".to_string(),
                    description: Some("Latest flagship model".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "moonshot-v1-128k".to_string(),
                    name: "Moonshot 128K".to_string(),
                    description: Some("Ultra-long context".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "qwen".to_string(),
            name: "Qwen (Tongyi Qianwen)".to_string(),
            icon: "🔮".to_string(),
            default_base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/qwen".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "qwen-max".to_string(),
                    name: "Qwen Max".to_string(),
                    description: Some("Most powerful version".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "qwen-plus".to_string(),
                    name: "Qwen Plus".to_string(),
                    description: Some("Balanced version".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            icon: "🔵".to_string(),
            default_base_url: Some("https://api.deepseek.com".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: None,
            suggested_models: vec![
                SuggestedModel {
                    id: "deepseek-chat".to_string(),
                    name: "DeepSeek V3".to_string(),
                    description: Some("Latest chat model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "deepseek-reasoner".to_string(),
                    name: "DeepSeek R1".to_string(),
                    description: Some("Reasoning-enhanced model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "glm".to_string(),
            name: "GLM (Zhipu)".to_string(),
            icon: "🔷".to_string(),
            default_base_url: Some("https://api.z.ai/api/anthropic".to_string()),
            api_type: "anthropic-messages".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/glm".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "glm-5".to_string(),
                    name: "GLM-5".to_string(),
                    description: Some("Latest flagship model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "minimax".to_string(),
            name: "MiniMax".to_string(),
            icon: "🟡".to_string(),
            default_base_url: Some("https://api.minimax.io/anthropic".to_string()),
            api_type: "anthropic-messages".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/minimax".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "minimax-m2.1".to_string(),
                    name: "MiniMax M2.1".to_string(),
                    description: Some("Latest model".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "venice".to_string(),
            name: "Venice AI".to_string(),
            icon: "🏛️".to_string(),
            default_base_url: Some("https://api.venice.ai/api/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/venice".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "llama-3.3-70b".to_string(),
                    name: "Llama 3.3 70B".to_string(),
                    description: Some("Privacy-first inference".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "openrouter".to_string(),
            name: "OpenRouter".to_string(),
            icon: "🔄".to_string(),
            default_base_url: Some("https://openrouter.ai/api/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://docs.openclaw.ai/providers/openrouter".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "anthropic/claude-opus-4-5".to_string(),
                    name: "Claude Opus 4.5".to_string(),
                    description: Some("Access via OpenRouter".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "ollama".to_string(),
            name: "Ollama (Local)".to_string(),
            icon: "🟠".to_string(),
            default_base_url: Some("http://127.0.0.1:11434/v1".to_string()),
            api_type: "ollama".to_string(),
            requires_api_key: false,
            default_api_key: Some("ollama-local".to_string()),
            docs_url: Some("https://docs.openclaw.ai/providers/ollama".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "qwen3.5:9b".to_string(),
                    name: "qwen3.5:9b".to_string(),
                    description: Some("Run locally".to_string()),
                    context_window: Some(262144),
                    max_tokens: Some(4096),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "google".to_string(),
            name: "Google Gemini".to_string(),
            icon: "✨".to_string(),
            default_base_url: Some("https://generativelanguage.googleapis.com/v1beta/openai/".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            default_api_key: None,
            docs_url: Some("https://ai.google.dev/gemini-api/docs/openai".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "gemini-3-flash-preview".to_string(),
                    name: "Gemini 3 Flash".to_string(),
                    description: Some("Fast and efficient multimodal model (Preview)".to_string()),
                    context_window: Some(1048576),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "gemini-3-pro-preview".to_string(),
                    name: "Gemini 3 Pro".to_string(),
                    description: Some("Complex reasoning tasks (Preview)".to_string()),
                    context_window: Some(1048576),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
    ];

    info!(
        "[Official Provider] Returned {} official Provider presets",
        providers.len()
    );
    Ok(providers)
}

/// Get AI configuration overview
#[command]
pub async fn get_ai_config() -> Result<AIConfigOverview, String> {
    info!("[AI Config] Getting AI configuration overview...");

    let config_path = platform::get_config_file_path();
    info!("[AI Config] Configuration file path: {}", config_path);

    let config = super::load_openclaw_config()?;
    log::debug!("[AI Config] Configuration content: {}", serde_json::to_string_pretty(&config).unwrap_or_default());

    // Parse primary model
    let primary_model = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    info!("[AI Config] Primary model: {:?}", primary_model);

    // Parse available model list
    let available_models: Vec<String> = config
        .pointer("/agents/defaults/models")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();
    info!("[AI Config] Number of available models: {}", available_models.len());

    // Parse configured Providers
    let mut configured_providers: Vec<ConfiguredProvider> = Vec::new();

    let providers_value = config.pointer("/models/providers");
    info!("[AI Config] providers node exists: {}", providers_value.is_some());

    if let Some(providers) = providers_value.and_then(|v| v.as_object()) {
        info!("[AI Config] Found {} Providers", providers.len());

        for (provider_name, provider_config) in providers {
            info!("[AI Config] Parsing Provider: {}", provider_name);

            let base_url = provider_config
                .get("baseUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let api_key = provider_config
                .get("apiKey")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let api_key_masked = api_key.as_ref().map(|key| {
                if key.len() > 8 {
                    format!("{}...{}", &key[..4], &key[key.len() - 4..])
                } else {
                    "****".to_string()
                }
            });

            // Parse model list
            let models_array = provider_config.get("models").and_then(|v| v.as_array());
            info!("[AI Config] Provider {} models array: {:?}", provider_name, models_array.map(|a| a.len()));

            let models: Vec<ConfiguredModel> = models_array
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m.get("id")?.as_str()?.to_string();
                            let name = m
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&id)
                                .to_string();
                            let full_id = format!("{}/{}", provider_name, id);
                            let is_primary = primary_model.as_ref() == Some(&full_id);

                            info!("[AI Config] Parsed model: {} (is_primary: {})", full_id, is_primary);

                            Some(ConfiguredModel {
                                full_id,
                                id,
                                name,
                                api_type: m.get("api").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                context_window: m
                                    .get("contextWindow")
                                    .and_then(|v| v.as_u64())
                                    .map(|n| n as u32),
                                max_tokens: m
                                    .get("maxTokens")
                                    .and_then(|v| v.as_u64())
                                    .map(|n| n as u32),
                                is_primary,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            info!("[AI Config] Provider {} parsing complete: {} models", provider_name, models.len());

            configured_providers.push(ConfiguredProvider {
                name: provider_name.clone(),
                base_url,
                api_key_masked,
                has_api_key: api_key.is_some(),
                models,
            });
        }
    } else {
        info!("[AI Config] providers configuration not found or incorrect format");
    }

    info!(
        "[AI Config] Final result - Primary model: {:?}, {} Providers, {} available models",
        primary_model,
        configured_providers.len(),
        available_models.len()
    );

    Ok(AIConfigOverview {
        primary_model,
        configured_providers,
        available_models,
    })
}

/// Add or update Provider
#[command]
pub async fn save_provider(
    provider_name: String,
    base_url: String,
    api_key: Option<String>,
    api_type: String,
    models: Vec<ModelConfig>,
) -> Result<String, String> {
    info!(
        "[Save Provider] Saving Provider: {} ({} models)",
        provider_name,
        models.len()
    );

    let mut config = super::load_openclaw_config()?;

    // Ensure paths exist
    if config.get("models").is_none() {
        config["models"] = json!({});
    }
    if config["models"].get("providers").is_none() {
        config["models"]["providers"] = json!({});
    }
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("models").is_none() {
        config["agents"]["defaults"]["models"] = json!({});
    }

    // Build model configuration
    let models_json: Vec<Value> = models
        .iter()
        .map(|m| {
            let mut model_obj = json!({
                "id": m.id,
                "name": m.name,
                "api": m.api.clone().unwrap_or(api_type.clone()),
                "input": if m.input.is_empty() { vec!["text".to_string()] } else { m.input.clone() },
            });

            if let Some(cw) = m.context_window {
                model_obj["contextWindow"] = json!(cw);
            }
            if let Some(mt) = m.max_tokens {
                model_obj["maxTokens"] = json!(mt);
            }
            if let Some(r) = m.reasoning {
                model_obj["reasoning"] = json!(r);
            }
            if let Some(cost) = &m.cost {
                model_obj["cost"] = json!({
                    "input": cost.input,
                    "output": cost.output,
                    "cacheRead": cost.cache_read,
                    "cacheWrite": cost.cache_write,
                });
            } else {
                model_obj["cost"] = json!({
                    "input": 0,
                    "output": 0,
                    "cacheRead": 0,
                    "cacheWrite": 0,
                });
            }

            model_obj
        })
        .collect();

    // Build Provider configuration
    let mut provider_config = json!({
        "baseUrl": base_url,
        "models": models_json,
    });

    // Handle API Key: if a new non-empty key is provided, use it; otherwise preserve the existing one
    if let Some(key) = api_key {
        if !key.is_empty() {
            // Use the newly provided API Key
            provider_config["apiKey"] = json!(key);
            info!("[Save Provider] Using new API Key");
        } else {
            // Empty string means no change, try to preserve the existing API Key
            if let Some(existing_key) = config
                .pointer(&format!("/models/providers/{}/apiKey", provider_name))
                .and_then(|v| v.as_str())
            {
                provider_config["apiKey"] = json!(existing_key);
                info!("[Save Provider] Preserving existing API Key");
            }
        }
    } else {
        // None means no change, try to preserve the existing API Key
        if let Some(existing_key) = config
            .pointer(&format!("/models/providers/{}/apiKey", provider_name))
            .and_then(|v| v.as_str())
        {
            provider_config["apiKey"] = json!(existing_key);
            info!("[Save Provider] Preserving existing API Key");
        }
    }

    // Save Provider configuration
    config["models"]["providers"][&provider_name] = provider_config;

    // Add models to agents.defaults.models
    for model in &models {
        let full_id = format!("{}/{}", provider_name, model.id);
        config["agents"]["defaults"]["models"][&full_id] = json!({});
    }

    // Update metadata
    let now = chrono::Utc::now().to_rfc3339();
    if config.get("meta").is_none() {
        config["meta"] = json!({});
    }
    config["meta"]["lastTouchedAt"] = json!(now);

    super::save_openclaw_config(&config)?;
    info!("[Save Provider] Provider {} saved successfully", provider_name);

    Ok(format!("Provider {} saved", provider_name))
}

/// Delete Provider
#[command]
pub async fn delete_provider(provider_name: String) -> Result<String, String> {
    info!("[Delete Provider] Deleting Provider: {}", provider_name);

    let mut config = super::load_openclaw_config()?;

    // Delete Provider configuration
    if let Some(providers) = config
        .pointer_mut("/models/providers")
        .and_then(|v| v.as_object_mut())
    {
        providers.remove(&provider_name);
    }

    // Delete related models
    if let Some(models) = config
        .pointer_mut("/agents/defaults/models")
        .and_then(|v| v.as_object_mut())
    {
        let keys_to_remove: Vec<String> = models
            .keys()
            .filter(|k| k.starts_with(&format!("{}/", provider_name)))
            .cloned()
            .collect();

        for key in keys_to_remove {
            models.remove(&key);
        }
    }

    // If primary model belongs to this Provider, clear primary model
    if let Some(primary) = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
    {
        if primary.starts_with(&format!("{}/", provider_name)) {
            config["agents"]["defaults"]["model"]["primary"] = json!(null);
        }
    }

    super::save_openclaw_config(&config)?;
    info!("[Delete Provider] Provider {} deleted", provider_name);

    Ok(format!("Provider {} deleted", provider_name))
}

/// Set primary model
#[command]
pub async fn set_primary_model(model_id: String) -> Result<String, String> {
    info!("[Set Primary Model] Setting primary model: {}", model_id);

    let mut config = super::load_openclaw_config()?;

    // Ensure paths exist
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("model").is_none() {
        config["agents"]["defaults"]["model"] = json!({});
    }

    // Set primary model
    config["agents"]["defaults"]["model"]["primary"] = json!(model_id);

    super::save_openclaw_config(&config)?;
    info!("[Set Primary Model] Primary model set to: {}", model_id);

    Ok(format!("Primary model set to {}", model_id))
}

/// Add model to available list
#[command]
pub async fn add_available_model(model_id: String) -> Result<String, String> {
    info!("[Add Model] Adding model to available list: {}", model_id);

    let mut config = super::load_openclaw_config()?;

    // Ensure paths exist
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("models").is_none() {
        config["agents"]["defaults"]["models"] = json!({});
    }

    // Add model
    config["agents"]["defaults"]["models"][&model_id] = json!({});

    super::save_openclaw_config(&config)?;
    info!("[Add Model] Model {} added", model_id);

    Ok(format!("Model {} added", model_id))
}

/// Remove model from available list
#[command]
pub async fn remove_available_model(model_id: String) -> Result<String, String> {
    info!("[Remove Model] Removing model from available list: {}", model_id);

    let mut config = super::load_openclaw_config()?;

    if let Some(models) = config
        .pointer_mut("/agents/defaults/models")
        .and_then(|v| v.as_object_mut())
    {
        models.remove(&model_id);
    }

    super::save_openclaw_config(&config)?;
    info!("[Remove Model] Model {} removed", model_id);

    Ok(format!("Model {} removed", model_id))
}

// ============ Legacy Compatibility ============

/// Get all supported AI Providers (legacy compatibility)
#[command]
pub async fn get_ai_providers() -> Result<Vec<crate::models::AIProviderOption>, String> {
    info!("[AI Provider] Getting supported AI Provider list (legacy)...");

    let official = get_official_providers().await?;
    let providers: Vec<crate::models::AIProviderOption> = official
        .into_iter()
        .map(|p| crate::models::AIProviderOption {
            id: p.id,
            name: p.name,
            icon: p.icon,
            default_base_url: p.default_base_url,
            requires_api_key: p.requires_api_key,
            models: p
                .suggested_models
                .into_iter()
                .map(|m| crate::models::AIModelOption {
                    id: m.id,
                    name: m.name,
                    description: m.description,
                    recommended: m.recommended,
                })
                .collect(),
        })
        .collect();

    Ok(providers)
}
