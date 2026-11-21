pub mod breakpoints;
pub mod distiller;
pub mod partitioner;
pub mod pruners;

pub use breakpoints::{schedule_cache_breakpoints, ProviderFormat, ScheduledPayload};
pub use distiller::{
    compact_code_skeleton, compact_cold_start_session, count_line_braces,
    distill_app_graph_blueprint, distill_diagram_blueprint, distill_document_graph_blueprint,
    distill_turn_for_commit, distill_voxel_blueprint,
};
pub use partitioner::{
    canonicalize_tools_json, compose_two_zone_turn, compute_radix_block_hashes, fnv1a_64,
    measure_prefix_cache_hit_ratio, normalize_day_anchor_utc, CacheControl, ChatMessage,
    ComposedPrompt, TurnInput, TwoZoneConfig,
};
pub use pruners::{
    prune_code_for_active_refinement, prune_coordinate_buffers, prune_diagram_styling,
    prune_embedded_media, prune_static_paragraphs, prune_svg_vectors, prune_voxel_matrices,
    purge_reasoning_artifacts,
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Stateful session manager enforcing Zone-1 prefix hash invariance, commit-time
/// relational artifact distillation, and K-bounded cache breakpoint scheduling.
#[derive(Debug, Clone)]
pub struct KvPrefixGuardSession {
    pub config: TwoZoneConfig,
    pub committed_history: Vec<ChatMessage>,
    pub provider: ProviderFormat,
    pub max_breakpoints: usize,
}

impl KvPrefixGuardSession {
    pub fn new(config: TwoZoneConfig, model_or_provider: &str, max_breakpoints: usize) -> Self {
        Self {
            config,
            committed_history: Vec::new(),
            provider: ProviderFormat::from_model_or_provider(model_or_provider),
            max_breakpoints: max_breakpoints.clamp(1, 4),
        }
    }

    /// Prepares the ScheduledPayload for the active user turn without mutating Zone 1.
    pub fn prepare_turn(&self, turn: &TurnInput) -> (ComposedPrompt, ScheduledPayload) {
        let composed = compose_two_zone_turn(&self.config, &self.committed_history, turn);
        let scheduled =
            schedule_cache_breakpoints(&composed.messages, self.provider, self.max_breakpoints);
        (composed, scheduled)
    }

    /// Commits a completed user-assistant exchange into the immutable prefix history,
    /// running commit-time inline scratchpad scrubbing and relational blueprint distillation
    /// exactly once so subsequent turns achieve 100% prefix cache continuity.
    pub fn commit_exchange(&mut self, raw_user_message: &str, raw_assistant_response: &str) {
        let clean_user = raw_user_message.trim().to_string();
        let distilled_assistant = distill_turn_for_commit(raw_assistant_response);
        self.committed_history.push(ChatMessage {
            role: "user".to_string(),
            content: clean_user,
            cache_control: None,
        });
        self.committed_history.push(ChatMessage {
            role: "assistant".to_string(),
            content: distilled_assistant,
            cache_control: None,
        });
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_distill_turn_for_commit(content: &str) -> String {
    distill_turn_for_commit(content)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_prune_code_for_active_refinement(raw_code: &str) -> String {
    prune_code_for_active_refinement(raw_code)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_canonicalize_tools_json(raw_tools_json: &str) -> String {
    canonicalize_tools_json(raw_tools_json)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_compact_code_skeleton(raw_code: &str, max_lines: usize) -> String {
    compact_code_skeleton(raw_code, max_lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_zone_system_prompt_immutability_across_turns() {
        let config = TwoZoneConfig {
            base_system_prompt: "You are a deterministic systems architecture assistant.".to_string(),
            session_date_utc: "2026-11-16T21:44:51.902Z".to_string(),
            session_output_requirement: "Emit runnable HTML5 or Mermaid diagrams.".to_string(),
            trailing_task_scope: Some("Domain: Distributed Inference Caching".to_string()),
        };

        let mut session = KvPrefixGuardSession::new(config, "claude-sonnet-5-5", 4);

        let (t1_composed, t1_sched) = session.prepare_turn(&TurnInput {
            user_message: "Design an interactive cache latency dashboard.".to_string(),
            dynamic_output_override: None,
            live_tool_context: None,
        });

        session.commit_exchange(
            "Design an interactive cache latency dashboard.",
            "<scratchpad>Let us plan the DOM layout and canvas chart...</scratchpad>\nStatus: Complete\n```html\n<!DOCTYPE html><html><head><title>Cache Monitor</title></head><body><button id=\"refresh-btn\">Refresh</button><canvas id=\"latency-canvas\"></canvas><script>let hitRate = 0.99; function handleRefresh(evt) { hitRate = 1.0; } function renderLoop() { requestAnimationFrame(renderLoop); } document.getElementById('refresh-btn').addEventListener('click', handleRefresh); requestAnimationFrame(renderLoop);</script></body></html>\n```",
        );

        let (t2_composed, t2_sched) = session.prepare_turn(&TurnInput {
            user_message: "Add a 99th percentile TTFT line series to the canvas.".to_string(),
            dynamic_output_override: Some("Include dark-mode theme toggle.".to_string()),
            live_tool_context: Some("MCP Metric Snapshot: p99_ttft_ms = 42.1".to_string()),
        });

        // 1. Zone 1 system prompt and hash must be 100% byte-for-byte identical
        assert_eq!(t1_composed.zone1_system_prompt, t2_composed.zone1_system_prompt);
        assert_eq!(t1_composed.zone1_hash, t2_composed.zone1_hash);
        assert!(t1_composed
            .zone1_system_prompt
            .contains("[LIVE TEMPORAL CONTEXT: Date: 2026-11-16]"));

        // 2. Dynamic override and MCP tool context are isolated in Zone 2 user tail
        assert!(t2_composed.zone2_user_prompt.contains("REAL-TIME VERIFIED CONTEXT"));
        assert!(t2_composed.zone2_user_prompt.contains("DYNAMIC OUTPUT REQUIREMENT"));
        assert!(!t2_composed.zone1_system_prompt.contains("REAL-TIME VERIFIED CONTEXT"));

        // 3. Scratchpad was purged and HTML app was distilled into Relational Event-Graph Blueprint
        let committed_asst = &session.committed_history[1].content;
        assert!(!committed_asst.contains("<scratchpad>"));
        assert!(!committed_asst.contains("Status: Complete"));
        assert!(committed_asst.contains("[RELATIONAL ARTIFACT BLUEPRINT: \"Cache Monitor\"]"));
        assert!(committed_asst.contains("#refresh-btn"));
        assert!(committed_asst.contains("#latency-canvas"));
        assert!(committed_asst.contains("handleRefresh(evt)"));
        assert!(committed_asst.contains("--[ON_CLICK]-->"));
        assert!(committed_asst.contains("--[TICK_CADENCE]-->"));

        // 4. Breakpoints are scheduled on system (idx 0) and committed assistant (idx 2)
        assert_eq!(t1_sched.active_breakpoints, vec![0]);
        assert_eq!(t2_sched.active_breakpoints, vec![0, 2]);
    }

    #[test]
    fn test_signed_thinking_blocks_preserved_while_unsigned_scratchpads_stripped() {
        let mixed = "<thinking signature=\"sig_anthropic_99281ab\">Verified reasoning block</thinking>\n<scratchpad>Internal draft notes</scratchpad>\nDeliverable output line.";
        let cleaned = purge_reasoning_artifacts(mixed);
        assert!(cleaned.contains("signature=\"sig_anthropic_99281ab\""));
        assert!(!cleaned.contains("<scratchpad>"));
        assert!(cleaned.contains("Deliverable output line."));
    }

    #[test]
    fn test_distillation_idempotence_and_contract_survival() {
        let raw_turn = "<thought>Analyzing 3D WebGL scene...</thought>\n```html\n<!DOCTYPE html><html><head><title>Orbital Simulator</title></head><body><div id=\"hud-panel\"></div><canvas id=\"gl-viewport\"></canvas><script src=\"https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js\"></script><script>let orbitalVelocity = 7.8; const cameraState = { zoom: 1.0 }; function updatePhysics(dt) { orbitalVelocity *= 1.0; } function animateFrame() { requestAnimationFrame(animateFrame); } document.getElementById('hud-panel').addEventListener('click', updatePhysics); requestAnimationFrame(animateFrame);</script></body></html>\n```";

        let once = distill_turn_for_commit(raw_turn);
        let twice = distill_turn_for_commit(&once);

        // Theorem 2 idempotence: D(D(x)) == D(x)
        assert_eq!(once, twice);
        assert!(once.contains("#hud-panel"));
        assert!(once.contains("#gl-viewport"));
        assert!(once.contains("orbitalVelocity"));
        assert!(once.contains("updatePhysics(dt)"));
        assert!(once.contains("Three.js 3D WebGL"));
    }

    #[test]
    fn test_document_graph_and_cold_start_compaction() {
        let contract_doc = format!(
            "# Master Services Agreement\n## 1. Scope of Services\n{}\n## 2. Indemnification\n{}\n## 3. Limitation of Liability\n{}\n## 4. Term and Termination\n{}",
            "Service provider obligations and SLAs. ".repeat(10),
            "Mutual indemnification provisions. ".repeat(10),
            "Aggregate liability cap rules. ".repeat(10),
            "Termination notice periods. ".repeat(10)
        );
        let distilled_doc = distill_document_graph_blueprint(&contract_doc);
        assert!(distilled_doc.contains("[RELATIONAL ARTIFACT BLUEPRINT: \"Master Services Agreement\"]"));
        assert!(distilled_doc.contains("(Clause:Indemnity) --[BOUND_BY_CAP]--> (Clause:Liability)"));
        assert!(distilled_doc.contains("(Clause:Breach) --[TRIGGERS_NOTICE]--> (Clause:Termination)"));

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "System anchor".to_string(),
                cache_control: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Refine v1:\n[CURRENT ARTIFACT CODE]\n<html>...large v1...</html>".to_string(),
                cache_control: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: contract_doc.clone(),
                cache_control: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Refine v2".to_string(),
                cache_control: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: contract_doc,
                cache_control: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Latest turn request".to_string(),
                cache_control: None,
            },
        ];

        let hydrated = compact_cold_start_session(&messages);
        assert!(hydrated[1]
            .content
            .contains("[PREVIOUS ARTIFACT CODE: Compacted in conversation history]"));
        assert!(hydrated[2].content.contains("HISTORICAL ARTIFACT VERSION"));
        assert!(hydrated[4]
            .content
            .contains("[RELATIONAL ARTIFACT BLUEPRINT: \"Master Services Agreement\"]"));
    }

    #[test]
    fn test_provider_formats_across_us_frontier_models() {
        assert_eq!(
            ProviderFormat::from_model_or_provider("claude-opus-5-5"),
            ProviderFormat::AnthropicClaude
        );
        assert_eq!(
            ProviderFormat::from_model_or_provider("gemini-4-carbon"),
            ProviderFormat::GoogleGemini
        );
        assert_eq!(
            ProviderFormat::from_model_or_provider("gpt-6-astra"),
            ProviderFormat::OpenAiGpt
        );
        assert_eq!(
            ProviderFormat::from_model_or_provider("Llama-4-Maverick-17B-128E-vllm"),
            ProviderFormat::MetaLlamaVllm
        );
    }

    #[test]
    fn test_active_code_pruners_preserve_structure() {
        let b64_payload = "A".repeat(120);
        let svg_path = "M 10 20 L 30 40 C 50 60, 70 80, 90 100 ".repeat(5);
        let float_buf = "0.125, -0.450, 1.023, 0.882, -0.331, 0.991, ".repeat(6);
        let para_text = "This is a static documentation paragraph inside the widget. ".repeat(8);
        let raw_code = format!(
            "<img id=\"avatar\" src=\"data:image/png;base64,{}\" /><svg><path id=\"wave\" d=\"{}\" /></svg><p>{}</p><script>const verts = [{}];</script>",
            b64_payload, svg_path, para_text, float_buf
        );

        let pruned = prune_code_for_active_refinement(&raw_code);
        assert!(pruned.len() < raw_code.len() / 2);
        assert!(pruned.contains("id=\"avatar\""));
        assert!(pruned.contains("data:image/png;base64,...[embedded-media-asset]"));
        assert!(pruned.contains("id=\"wave\""));
        assert!(pruned.contains("... [vector-path-data]"));
        assert!(pruned.contains("... [static content text preserved]</p>"));
        assert!(pruned.contains("const verts = ["));
        assert!(pruned.contains("/* [numeric coordinate/vertex buffer collapsed] */"));
    }

    #[test]
    fn test_canonical_tools_schema_normalization() {
        let tools_turn1 = r#"[{"description":"Query DB","name":"sql_query","input_schema":{"required":["q"],"type":"object"}},{"name":"read_file","description":"Read file"}]"#;
        let tools_turn2 = r#"[{"description":"Read file","name":"read_file"},{"input_schema":{"type":"object","required":["q"]},"name":"sql_query","description":"Query DB"}]"#;

        let c1 = canonicalize_tools_json(tools_turn1);
        let c2 = canonicalize_tools_json(tools_turn2);
        assert_eq!(c1, c2);
        assert_eq!(fnv1a_64(c1.as_bytes()), fnv1a_64(c2.as_bytes()));
    }
}
