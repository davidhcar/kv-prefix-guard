"""
Runnable Python Example & Integration Test:
Multi-step MCP Tool-Calling & Reasoning Agent using `kv_prefix_guard`.
Verifies:
1. Canonical `[Tools]` JSON schema normalization across turns.
2. Preservation of cryptographically signed `<thinking signature="...">` blocks
   while stripping unsigned `<scratchpad>` / `<thought>` planning traces.
3. Commit-time Relational Event-Graph distillation for HTML5/WebGL and Mermaid turns.
"""

import kv_prefix_guard as kpg


def run_mcp_reasoning_agent_example() -> None:
    # 1. Verify canonical `[Tools]` schema hash invariance across reordered dicts
    tools_t1 = '[{"description":"Run query","name":"mcp_sql","input_schema":{"required":["q"],"type":"object"}},{"name":"mcp_metrics","description":"Read metrics"}]'
    tools_t2 = '[{"name":"mcp_metrics","description":"Read metrics"},{"input_schema":{"type":"object","required":["q"]},"name":"mcp_sql","description":"Run query"}]'

    c1 = kpg.py_canonicalize_tools_json(tools_t1)
    c2 = kpg.py_canonicalize_tools_json(tools_t2)
    assert c1 == c2, "Canonicalized tool schemas must be byte-identical"

    # 2. Verify signed thinking preservation vs. unsigned scratchpad removal
    raw_assistant_t1 = (
        '<thinking signature="sig_claude_5_5_verified">Signed reasoning trace</thinking>\n'
        "<scratchpad>Drafting internal DOM and Mermaid nodes...</scratchpad>\n"
        "Status: Complete\n"
        "```mermaid\n"
        "flowchart TD\n"
        "  subgraph Ingestion\n"
        "    A[Gateway Router] --> B[Radix Cache Index]\n"
        "    B --> C[Prefill Worker Pool]\n"
        "  end\n"
        "  C --> D[Token Stream Transducer]\n"
        "  D --> E[Client Viewport]\n"
        "  classDef primary fill:#1e293b,stroke:#38bdf8,color:#fff;\n"
        "  classDef secondary fill:#0f172a,stroke:#94a3b8,color:#e2e8f0;\n"
        "```"
    )

    committed_t1 = kpg.py_distill_turn_for_commit(raw_assistant_t1)
    assert 'signature="sig_claude_5_5_verified"' in committed_t1
    assert "<scratchpad>" not in committed_t1
    assert "Status: Complete" not in committed_t1
    assert "[RELATIONAL DIAGRAM BLUEPRINT: flowchart TD]" in committed_t1
    assert "Ingestion" in committed_t1
    assert "A --> B" in committed_t1

    # 3. Verify idempotence: D(D(x)) == D(x)
    committed_twice = kpg.py_distill_turn_for_commit(committed_t1)
    assert committed_t1 == committed_twice, "Commit-time distillation must be strictly idempotent"

    # 4. Verify AST code skeletonization on multi-method TypeScript service
    raw_ts_code = """
import { EventEmitter } from 'events';
export interface CacheTelemetry {
    hitRatio: number;
    prefillMs: number;
}
export class PrefixCacheController {
    public evaluatePrefix(turnId: string, tokens: number[]): CacheTelemetry {
        const hits = tokens.filter((t) => t > 0).length;
        const ratio = hits / Math.max(1, tokens.length);
        return { hitRatio: ratio, prefillMs: 12.4 };
    }
}
"""
    skeleton = kpg.py_compact_code_skeleton(raw_ts_code, 25)
    assert "export interface CacheTelemetry" in skeleton
    assert "export class PrefixCacheController" in skeleton
    assert "public evaluatePrefix(turnId: string, tokens: number[]): CacheTelemetry;" in skeleton
    assert "Math.max" not in skeleton

    print("Python MCP reasoning agent example passed all assertions.")


if __name__ == "__main__":
    run_mcp_reasoning_agent_example()
