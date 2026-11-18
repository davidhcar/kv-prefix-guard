use crate::partitioner::{CacheControl, ChatMessage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderFormat {
    AnthropicClaude,
    GoogleGemini,
    OpenAiGpt,
    MetaLlamaVllm,
}

impl ProviderFormat {
    pub fn from_model_or_provider(name: &str) -> Self {
        let lower = name.to_ascii_lowercase();
        if lower.contains("anthropic") || lower.contains("claude") {
            Self::AnthropicClaude
        } else if lower.contains("google") || lower.contains("gemini") {
            Self::GoogleGemini
        } else if lower.contains("llama") || lower.contains("vllm") || lower.contains("sglang") {
            Self::MetaLlamaVllm
        } else {
            Self::OpenAiGpt
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledPayload {
    pub provider: ProviderFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolated_system_instruction: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub active_breakpoints: Vec<usize>,
}

/// Schedules up to K <= 4 ephemeral `cache_control` breakpoints across a multi-turn conversation.
/// - For Anthropic Claude: places breakpoint 1 on the system prompt and up to 3 monotonic
///   breakpoints on committed assistant turns (preserving older assistant breakpoints across turns).
/// - For Google Gemini: isolates `systemInstruction` for implicit/explicit context caching and
///   marks up to 3 assistant checkpoints.
/// - For OpenAI GPT/o-series and Meta Llama (vLLM/SGLang): strips explicit `cache_control` fields
///   so strict OpenAI-compatible schemas never reject the payload while preserving 100% prefix alignment.
pub fn schedule_cache_breakpoints(
    messages: &[ChatMessage],
    provider: ProviderFormat,
    max_total_breakpoints: usize,
) -> ScheduledPayload {
    let cap = max_total_breakpoints.clamp(1, 4);

    match provider {
        ProviderFormat::OpenAiGpt | ProviderFormat::MetaLlamaVllm => {
            let clean_messages: Vec<ChatMessage> = messages
                .iter()
                .map(|m| ChatMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                    cache_control: None,
                })
                .collect();
            ScheduledPayload {
                provider,
                isolated_system_instruction: None,
                messages: clean_messages,
                active_breakpoints: Vec::new(),
            }
        }
        ProviderFormat::GoogleGemini => {
            let mut system_text: Option<String> = None;
            let mut non_system: Vec<ChatMessage> = Vec::with_capacity(messages.len());
            for m in messages {
                if m.role == "system" && system_text.is_none() {
                    system_text = Some(m.content.clone());
                } else {
                    non_system.push(ChatMessage {
                        role: if m.role == "assistant" {
                            "model".to_string()
                        } else {
                            m.role.clone()
                        },
                        content: m.content.clone(),
                        cache_control: m.cache_control.clone(),
                    });
                }
            }

            let assistant_budget = if system_text.is_some() {
                cap.saturating_sub(1)
            } else {
                cap
            };
            let last_idx = non_system.len().saturating_sub(1);
            let assistant_indices: Vec<usize> = non_system
                .iter()
                .enumerate()
                .filter_map(|(idx, m)| {
                    if (m.role == "model" || m.role == "assistant") && idx < last_idx {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();

            let keep_start = assistant_indices.len().saturating_sub(assistant_budget);
            let selected: Vec<usize> = assistant_indices[keep_start..].to_vec();
            let mut active_bp = Vec::new();

            for (idx, m) in non_system.iter_mut().enumerate() {
                if selected.contains(&idx) && !m.content.is_empty() {
                    m.cache_control = Some(CacheControl::ephemeral());
                    active_bp.push(idx);
                } else {
                    m.cache_control = None;
                }
            }

            ScheduledPayload {
                provider,
                isolated_system_instruction: system_text,
                messages: non_system,
                active_breakpoints: active_bp,
            }
        }
        ProviderFormat::AnthropicClaude => {
            let has_system = messages.iter().any(|m| m.role == "system");
            let max_msg_bp = if has_system {
                cap.saturating_sub(1)
            } else {
                cap
            };
            let last_idx = messages.len().saturating_sub(1);
            let penultimate_idx = if messages.len() > 2 {
                Some(messages.len() - 2)
            } else {
                None
            };

            let assistant_indices: Vec<usize> = messages
                .iter()
                .enumerate()
                .filter_map(|(idx, m)| {
                    if m.role == "assistant" && idx < last_idx {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();

            let keep_start = assistant_indices.len().saturating_sub(max_msg_bp);
            let selected_assistants: Vec<usize> = assistant_indices[keep_start..].to_vec();

            let mut out = Vec::with_capacity(messages.len());
            let mut active_bp = Vec::new();

            for (idx, m) in messages.iter().enumerate() {
                let should_cache = m.cache_control.is_some()
                    || (idx == 0 && m.role == "system")
                    || selected_assistants.contains(&idx)
                    || (selected_assistants.is_empty() && penultimate_idx == Some(idx))
                    || (idx == last_idx && idx > 0 && m.content.len() >= 2500 && active_bp.len() < cap);

                let cache_control = if should_cache && !m.content.is_empty() && active_bp.len() < cap {
                    active_bp.push(idx);
                    Some(CacheControl::ephemeral())
                } else {
                    None
                };

                out.push(ChatMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                    cache_control,
                });
            }

            ScheduledPayload {
                provider,
                isolated_system_instruction: None,
                messages: out,
                active_breakpoints: active_bp,
            }
        }
    }
}
