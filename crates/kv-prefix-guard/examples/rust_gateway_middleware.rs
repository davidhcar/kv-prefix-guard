use kv_prefix_guard::{
    canonicalize_tools_json, KvPrefixGuardSession, TurnInput, TwoZoneConfig,
};

fn main() {
    let raw_tools_t1 = r#"[{"description":"Execute SQL","name":"query_db","input_schema":{"required":["sql"],"type":"object"}},{"name":"fetch_metrics","description":"Fetch telemetry"}]"#;
    let raw_tools_t2 = r#"[{"name":"fetch_metrics","description":"Fetch telemetry"},{"input_schema":{"type":"object","required":["sql"]},"name":"query_db","description":"Execute SQL"}]"#;

    let canon_t1 = canonicalize_tools_json(raw_tools_t1);
    let canon_t2 = canonicalize_tools_json(raw_tools_t2);
    assert_eq!(canon_t1, canon_t2, "Canonical tool schemas must match byte-for-byte");

    let config = TwoZoneConfig {
        base_system_prompt: "You are an enterprise distributed systems architect.".to_string(),
        session_date_utc: "2026-11-16T22:05:19.441Z".to_string(),
        session_output_requirement: "Emit complete runnable HTML5 or Mermaid diagrams.".to_string(),
        trailing_task_scope: Some("Architecture: Multi-Region KV-Cache Telemetry".to_string()),
    };

    let mut session = KvPrefixGuardSession::new(config, "claude-opus-5-5", 4);

    let (t1_prompt, _t1_sched) = session.prepare_turn(&TurnInput {
        user_message: "Create an interactive cluster cache monitor dashboard.".to_string(),
        dynamic_output_override: None,
        live_tool_context: None,
    });

    session.commit_exchange(
        "Create an interactive cluster cache monitor dashboard.",
        "<scratchpad>Drafting layout for cluster monitor...</scratchpad>\n```html\n<!DOCTYPE html><html><head><title>Cluster Cache Monitor</title></head><body><button id=\"sync-btn\">Sync Nodes</button><canvas id=\"hit-rate-chart\"></canvas><script>let nodeCount = 16; function handleSync(e) { nodeCount += 1; } function renderLoop() { requestAnimationFrame(renderLoop); } document.getElementById('sync-btn').addEventListener('click', handleSync); requestAnimationFrame(renderLoop);</script></body></html>\n```",
    );

    let (t2_prompt, t2_sched) = session.prepare_turn(&TurnInput {
        user_message: "Add a regional p99 latency breakdown panel.".to_string(),
        dynamic_output_override: Some("Include CSV export button.".to_string()),
        live_tool_context: Some("us-east-1 p99: 11.4ms | us-west-2 p99: 14.2ms".to_string()),
    });

    assert_eq!(t1_prompt.zone1_hash, t2_prompt.zone1_hash);
    assert_eq!(t2_sched.active_breakpoints, vec![0, 2]);
    println!("Rust gateway example verified: Zone-1 hash = {:x}, breakpoints = {:?}", t2_prompt.zone1_hash, t2_sched.active_breakpoints);
}
