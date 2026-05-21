// Provider metadata — keys, base URLs, API-key-less providers
use std::collections::HashSet;

pub const PROVIDER_ENV_KEYS: &[(&str, &str)] = &[
    ("openrouter", "OPENROUTER_API_KEY"), ("anthropic", "ANTHROPIC_API_KEY"),
    ("openai", "OPENAI_API_KEY"), ("google", "GOOGLE_API_KEY"),
    ("xai", "XAI_API_KEY"), ("groq", "GROQ_API_KEY"),
    ("deepseek", "DEEPSEEK_API_KEY"), ("together", "TOGETHER_API_KEY"),
    ("fireworks", "FIREWORKS_API_KEY"), ("cerebras", "CEREBRAS_API_KEY"),
    ("mistral", "MISTRAL_API_KEY"), ("perplexity", "PERPLEXITY_API_KEY"),
    ("huggingface", "HF_TOKEN"), ("hf", "HF_TOKEN"),
    ("qwen", "QWEN_API_KEY"), ("minimax", "MINIMAX_API_KEY"),
    ("glm", "GLM_API_KEY"), ("kimi", "KIMI_API_KEY"), ("nvidia", "NVIDIA_API_KEY"),
];

pub const PROVIDER_BASE_URLS: &[(&str, &str)] = &[
    ("openai", "https://api.openai.com/v1"), ("openrouter", "https://openrouter.ai/api/v1"),
    ("deepseek", "https://api.deepseek.com/v1"), ("groq", "https://api.groq.com/openai/v1"),
    ("mistral", "https://api.mistral.ai/v1"), ("together", "https://api.together.xyz/v1"),
    ("fireworks", "https://api.fireworks.ai/inference/v1"), ("cerebras", "https://api.cerebras.ai/v1"),
    ("perplexity", "https://api.perplexity.ai"), ("huggingface", "https://router.huggingface.co/v1"),
    ("anthropic", "https://api.anthropic.com/v1"),
];

pub const NON_DISCOVERABLE_PROVIDERS: &[&str] = &[
    "auto","custom","nous","google","xai","openai-codex","xai-oauth",
    "qwen-oauth","qwen","minimax","kimi-coding",
];

pub fn providers_without_api_keys() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert("custom"); set.insert("lmstudio"); set.insert("ollama");
    set.insert("vllm"); set.insert("llamacpp"); set.insert("openai-codex");
    set
}

#[tauri::command]
pub fn provider_does_not_need_api_key(provider: String) -> bool {
    providers_without_api_keys().contains(provider.to_lowercase().as_str())
}

pub fn expected_env_key(provider: &str, base_url: &str) -> Option<String> {
    let lower = provider.trim().to_lowercase();
    for (k, v) in PROVIDER_ENV_KEYS { if *k == lower { return Some(v.to_string()); } }
    let url_lower = base_url.to_lowercase();
    for (k, v) in PROVIDER_ENV_KEYS {
        if *k != lower && url_lower.contains(k) { return Some(v.to_string()); }
    }
    None
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_providers_without_keys() { let s = providers_without_api_keys(); assert!(s.contains("ollama")); assert!(s.contains("custom")); }
    #[test] fn test_provider_does_not_need_key() { assert!(provider_does_not_need_api_key("ollama".into())); assert!(!provider_does_not_need_api_key("openai".into())); }
    #[test] fn test_expected_env_key() { assert_eq!(expected_env_key("openai","").unwrap(), "OPENAI_API_KEY"); }
}
