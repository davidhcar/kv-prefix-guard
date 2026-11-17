use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Recursively canonicalizes JSON tool definitions (`[Tools]` tier) by sorting all object keys
/// lexicographically via BTreeMap and ordering tool declarations by tool `name`.
/// Prevents silent `[Tools] -> [System] -> [Messages]` prefix cache invalidation caused by
/// non-deterministic dictionary key orderings across turns.
pub fn canonicalize_tools_json(raw_tools_json: &str) -> String {
    let trimmed = raw_tools_json.trim();
    let Ok(val) = serde_json::from_str::<Value>(trimmed) else {
        return trimmed.to_string();
    };
    let canonical = canonicalize_json_value(val, true);
    serde_json::to_string(&canonical).unwrap_or_else(|_| trimmed.to_string())
}

fn canonicalize_json_value(val: Value, is_root: bool) -> Value {
    match val {
        Value::Object(map) => {
            let mut sorted = BTreeMap::new();
            for (k, v) in map {
                sorted.insert(k, canonicalize_json_value(v, false));
            }
            let mut out = serde_json::Map::new();
            for (k, v) in sorted {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        Value::Array(arr) => {
            let mut items: Vec<Value> = arr
                .into_iter()
                .map(|v| canonicalize_json_value(v, false))
                .collect();
            if is_root {
                items.sort_by(|a, b| {
                    let name_a = a
                        .get("name")
                        .or_else(|| a.get("function").and_then(|f| f.get("name")))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let name_b = b
                        .get("name")
                        .or_else(|| b.get("function").and_then(|f| f.get("name")))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    name_a.cmp(name_b)
                });
            }
            Value::Array(items)
        }
        other => other,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub control_type: String,
}

impl CacheControl {
    pub fn ephemeral() -> Self {
        Self {
            control_type: "ephemeral".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoZoneConfig {
    pub base_system_prompt: String,
    pub session_date_utc: String,
    pub session_output_requirement: String,
    pub trailing_task_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnInput {
    pub user_message: String,
    pub dynamic_output_override: Option<String>,
    pub live_tool_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedPrompt {
    pub zone1_system_prompt: String,
    pub zone2_user_prompt: String,
    pub messages: Vec<ChatMessage>,
    pub zone1_hash: u64,
}

/// Deterministic FNV-1a 64-bit hash for zero-dependency prefix and block verification.
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Normalizes an ISO-8601 timestamp (e.g. "2026-11-16T21:41:58.123Z") to a stable day-level
/// UTC date anchor ("2026-11-16") so second-level clock ticks never invalidate Zone 1.
pub fn normalize_day_anchor_utc(timestamp_or_date: &str) -> String {
    let trimmed = timestamp_or_date.trim();
    if let Some((date_part, _)) = trimmed.split_once('T') {
        if date_part.len() == 10 {
            return date_part.to_string();
        }
    }
    if trimmed.len() >= 10 && trimmed.as_bytes()[4] == b'-' && trimmed.as_bytes()[7] == b'-' {
        return trimmed[..10].to_string();
    }
    trimmed.to_string()
}

/// Composes a Two-Zone prompt package where Zone 1 (system prompt + day-anchored date +
/// base output contract) remains 100% byte-identical across all turns, while dynamic
/// per-turn overrides and live MCP tool results are routed strictly to Zone 2 (user tail).
pub fn compose_two_zone_turn(
    config: &TwoZoneConfig,
    committed_history: &[ChatMessage],
    turn: &TurnInput,
) -> ComposedPrompt {
    let day_anchor = normalize_day_anchor_utc(&config.session_date_utc);
    let mut zone1 = config.base_system_prompt.trim_end().to_string();

    let temporal_tag = format!("[LIVE TEMPORAL CONTEXT: Date: {}]", day_anchor);
    if !zone1.contains("[LIVE TEMPORAL CONTEXT") {
        zone1.push_str("\n\n");
        zone1.push_str(&temporal_tag);
    }

    let req_trimmed = config.session_output_requirement.trim();
    if !req_trimmed.is_empty() && !zone1.contains(req_trimmed) {
        zone1.push_str("\n\n### OUTPUT REQUIREMENT\n");
        zone1.push_str(req_trimmed);
    }

    if let Some(scope) = &config.trailing_task_scope {
        let s = scope.trim();
        if !s.is_empty() {
            zone1.push_str("\n\n");
            zone1.push_str(s);
        }
    }

    let mut zone2 = turn.user_message.trim().to_string();

    // Route dynamic per-turn format overrides to Zone 2 (never mutating Zone 1)
    if !committed_history.is_empty() {
        if let Some(dyn_override) = &turn.dynamic_output_override {
            let ov = dyn_override.trim();
            if !ov.is_empty() && ov != req_trimmed && !zone2.contains(ov) {
                if !zone2.is_empty() {
                    zone2.push_str("\n\n### DYNAMIC OUTPUT REQUIREMENT\n");
                    zone2.push_str(ov);
                } else {
                    zone2 = format!("### DYNAMIC OUTPUT REQUIREMENT\n{}", ov);
                }
            }
        }
    }

    // Route volatile MCP / search tool context to Zone 2
    if let Some(live_ctx) = &turn.live_tool_context {
        let ctx = live_ctx.trim();
        if !ctx.is_empty() {
            zone2 = format!(
                "### REAL-TIME VERIFIED CONTEXT\n{}\n\n{}",
                ctx, zone2
            );
        }
    }

    let mut messages = Vec::with_capacity(committed_history.len() + 2);
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: zone1.clone(),
        cache_control: None,
    });
    for msg in committed_history {
        messages.push(msg.clone());
    }
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: zone2.clone(),
        cache_control: None,
    });

    let zone1_hash = fnv1a_64(zone1.as_bytes());

    ComposedPrompt {
        zone1_system_prompt: zone1,
        zone2_user_prompt: zone2,
        messages,
        zone1_hash,
    }
}

/// Estimates token count (~4 UTF-8 bytes per token) and partitions a message sequence
/// into fixed-size token blocks (e.g. 16 or 64 tokens for vLLM RadixAttention, 128 for OpenAI),
/// returning the cumulative chaining block hashes.
/// In a radix KV-cache, block `k` is a cache hit iff hashes[0..=k] match the prior turn.
pub fn compute_radix_block_hashes(messages: &[ChatMessage], chars_per_block: usize) -> Vec<u64> {
    let step = chars_per_block.max(16);
    let mut serialized = String::new();
    for m in messages {
        serialized.push_str(&m.role);
        serialized.push(':');
        serialized.push_str(&m.content);
        serialized.push('\n');
    }

    let bytes = serialized.as_bytes();
    let mut hashes = Vec::with_capacity((bytes.len() / step) + 1);
    let mut running_hash: u64 = 0xcbf29ce484222325;

    for chunk in bytes.chunks(step) {
        for &b in chunk {
            running_hash ^= b as u64;
            running_hash = running_hash.wrapping_mul(0x100000001b3);
        }
        hashes.push(running_hash);
    }
    hashes
}

/// Computes the exact prefix KV-cache hit ratio in [0.0, 1.0] when transitioning from
/// `previous_turn_messages` (excluding its final uncached generation) to `current_turn_messages` prefix.
pub fn measure_prefix_cache_hit_ratio(
    previous_prefix: &[ChatMessage],
    current_prefix: &[ChatMessage],
    chars_per_block: usize,
) -> f64 {
    let prev_hashes = compute_radix_block_hashes(previous_prefix, chars_per_block);
    let curr_hashes = compute_radix_block_hashes(current_prefix, chars_per_block);
    if prev_hashes.is_empty() || curr_hashes.is_empty() {
        return 0.0;
    }

    let comparable = prev_hashes.len().min(curr_hashes.len());
    let mut matched = 0usize;
    for i in 0..comparable {
        if prev_hashes[i] == curr_hashes[i] {
            matched += 1;
        } else {
            break;
        }
    }
    (matched as f64) / (prev_hashes.len() as f64)
}
