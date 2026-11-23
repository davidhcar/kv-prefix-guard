# `kv-prefix-guard`

**Wire-protocol context partitioning, $K$-bounded cache breakpoint scheduling ($K \le 4$), and commit-time relational artifact distillation for multi-turn LLM KV-caches ($86.2\%$ context token reduction, $95.4\%$ block-aligned KV-cache hit rate, and $6.74\times$ lower net prefill cost).**

[![Crates.io](https://img.shields.io/crates/v/kv-prefix-guard.svg)](https://crates.io/crates/kv-prefix-guard)
[![Crates.io Downloads](https://img.shields.io/crates/d/kv-prefix-guard.svg?label=crates.io%20downloads)](https://crates.io/crates/kv-prefix-guard)
[![PyPI](https://img.shields.io/pypi/v/kv-prefix-guard.svg)](https://pypi.org/project/kv-prefix-guard/)
[![PyPI Downloads](https://static.pepy.tech/badge/kv-prefix-guard)](https://pepy.tech/projects/kv-prefix-guard)
[![npm](https://img.shields.io/npm/v/kv-prefix-guard.svg)](https://www.npmjs.com/package/kv-prefix-guard)
[![npm Downloads](https://img.shields.io/npm/dt/kv-prefix-guard.svg?label=npm%20downloads)](https://www.npmjs.com/package/kv-prefix-guard)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

- **Author:** **David Charles Michael** ([ORCID: `0009-0007-8679-6941`](https://orcid.org/0009-0007-8679-6941) | `davidchar@gmail.com`)
- **Built For Modern Applied AI & Context Engineering Stacks:** Multi-turn coding agents (`Claude Code`, `Cursor`, `Windsurf`, `OpenHands`, `SWE-agent`), Model Context Protocol (`MCP`) servers and clients, AI gateways (`LiteLLM`, `OpenRouter`, `Portkey`), and agent orchestration frameworks (`LangGraph`, `LangChain`, `LlamaIndex`, `Vercel AI SDK`, `PydanticAI`, `AutoGen`, `CrewAI`, `DSPy`, `SmolAgents`, `Mastra`).
- **High-Throughput Cache-Preserving Alternative To:** Retroactive mid-prefix token pruners (`LLMLingua`, `LongLLMLingua`, `Selective Context`), sliding-window chat history truncation, and unnormalized dynamic system prompts that invalidate downstream KV-cache blocks.
- **Compatible With All 4 Frontier & Open-Weight KV-Cache Protocols:**
  1. **Explicit Breakpoint Caching (`[Tools] -> [System] -> [Messages]`, $K \le 4$ `cache_control: {"type": "ephemeral"}`):** Anthropic Claude API (including Claude 5.5 Opus / Sonnet and extended thinking blocks).
  2. **Implicit Block-Aligned Prefix Caching ($\ge 1,024$ tokens in $128$-token blocks):** OpenAI GPT API (including GPT-6, GPT-5, and reasoning series).
  3. **Isolated System-Context Caching (`systemInstruction` + `contents`):** Google Gemini API (including Gemini 4 and 3.8 series).
  4. **Radix-Tree Block Hashing (`PagedAttention` / `RadixAttention`, $16$- and $64$-token blocks):** `vLLM` and `SGLang` serving open-weight Meta Llama 4, DeepSeek, and Qwen deployments.

---

## Why Use `kv-prefix-guard` Instead of `LLMLingua` or Sliding Windows?

In causal self-attention, the key and value vectors at token position $j$ depend on all tokens in $[0, j-1]$. If an application edits tokens inside an earlier turn (via retroactive token dropping, sliding-window eviction, unordered tool JSON serialization, or per-turn system timestamps), every cached KV block after the edit offset is invalidated. Under discounted cache-read pricing ($\alpha = 0.10$), triggering a cache miss costs **more** than leaving the prompt uncompressed whenever the retained token ratio exceeds $r > \alpha$.

| Pipeline / Baseline (`MultiTurnCacheBench-1200`, $14,400$ turns) | Track A Hit (%) | Track B Hit (%) | Track C Hit (%) | Overall KV Hit (%) | Context Token Reduction (%) | Structural Contract Survival (%) | Relative Billed Prefill Cost (%) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Uncompressed Append-Only** | 99.1% | 99.1% | 99.5% | 99.3% | 0.0% | 100.0% | 100.0% ($1.00\times$) |
| **Naive Dynamic System / Tools** | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 100.0% | 410.2% ($4.10\times$ worse) |
| **Sliding-Window Eviction ($W = 4$)** | 27.4% | 27.2% | 15.7% | 21.3% | 44.5% | 61.2% | 188.5% ($1.89\times$ worse) |
| **Mid-Prefix Salience Pruner (`LLMLingua`-Style)** | 8.0% | 8.6% | 4.5% | 6.0% | 46.6% | 83.6% | 209.3% ($2.09\times$ worse) |
| **Static Skeleton (No Two-Zone Decoupling)** | 21.2% | 25.9% | 14.7% | 18.4% | 86.2% | 98.5% | 48.6% ($2.06\times$ cheaper) |
| **`kv-prefix-guard` (Ours)** | **94.6%** | **93.5%** | **96.3%** | **95.4%** | **86.2%** | **98.5%** | **14.8% ($6.74\times$ cheaper vs. Uncompressed, $14.1\times$ vs. `LLMLingua`)** |

---

## 5 Core Capabilities & Architecture

1. **Canonical `[Tools] -> [System]` Normalization & Two-Zone Splitting (`partitioner.rs`):** Sorts tool JSON schema keys deterministically via recursive `BTreeMap` structures, anchors session dates to a stable UTC calendar day in **Zone 1** (`[LIVE TEMPORAL CONTEXT: Date: YYYY-MM-DD]`), and isolates volatile MCP tool telemetry and dynamic format overrides in the **Zone-2** user tail.
2. **Provider-Aware Monotonic Breakpoint Scheduler ($K \le 4$, `breakpoints.rs`):** Pins breakpoint $1$ on the immutable Zone-1 system anchor and advances up to $3$ ephemeral checkpoints across committed assistant turns for Anthropic Claude, maps `isolated_system_instruction` and `model` roles for Google Gemini, and strips `cache_control` fields for strict OpenAI GPT and `vLLM`/`SGLang` validators.
3. **Signature-Safe Scratchpad Scrubber (`pruners.rs`):** Removes unsigned inline XML planning blocks (`<scratchpad>`, `<thought>`, `<plan>`, `<think>`) and operational status lines upon turn commit while leaving cryptographically signed `<thinking signature="...">` blocks untouched.
4. **Idempotent Commit-Time Relational & AST Distiller (`distiller.rs`):** Condenses completed HTML5/Canvas/Three.js WebGL applications, Mermaid architecture diagrams, 3D voxel scenes, multi-section legal/technical documents, and general source code once at commit time ($\mathcal{D}(\mathcal{D}(a_t)) = \mathcal{D}(a_t)$) into $120\text{--}180$ token relational blueprints and brace-depth AST skeletons that preserve $100\%$ of top-level DOM `#id` selectors, state stores, function signatures, and directed event edges.
5. **Active-Turn Bulk Literal Pruners & Cold-Start Session Compaction:** Strips Base64 data URIs ($\ge 60$ bytes), long SVG `d="..."` vector paths ($\ge 100$ bytes), numeric WebGL float arrays ($\ge 150$ bytes), and repeated Mermaid `classDef`/`style` lines during active code refinement turns, and collapses superseded blueprints when resuming saved sessions.

---

## Tri-Registry Installation & Quickstart

### 1. Python (`PyPI`, Native `PyO3` C-Extension)
```bash
pip install -U kv-prefix-guard
```

```python
import kv_prefix_guard as kpg

# A. Canonicalize unordered MCP / OpenAI / Anthropic tool JSON schemas for prefix-hash invariance
canon_tools = kpg.py_canonicalize_tools_json(
    '[{"name":"sql_query","input_schema":{"type":"object","required":["q"]}}]'
)

# B. Idempotent commit-time distillation (preserves signed thinking + DOM/Mermaid/AST contracts)
raw_assistant_turn = """<thinking signature="sig_verified_123">Signed reasoning</thinking>
<scratchpad>Drafting Three.js dashboard...</scratchpad>
```html
<!DOCTYPE html><html><head><title>Telemetry Console</title></head>
<body>
  <button id="refresh-btn">Refresh</button>
  <canvas id="latency-canvas"></canvas>
  <script>
    let pollIntervalMs = 2000;
    function handleRefresh(evt) { pollIntervalMs = 1000; }
    document.getElementById('refresh-btn').addEventListener('click', handleRefresh);
  </script>
</body></html>
```"""

committed_turn = kpg.py_distill_turn_for_commit(raw_assistant_turn)
print(committed_turn)

# C. Brace-depth AST code skeletonization (TypeScript / Python / Rust / Go)
skeleton = kpg.py_compact_code_skeleton(
    "export class CacheRouter {\n  public route(key: string): boolean {\n    const h = key.length * 31;\n    return h > 0;\n  }\n}",
    10,
)
print(skeleton)
```

### 2. JavaScript / TypeScript (`npm`, Zero-Dependency WebAssembly)
```bash
npm install kv-prefix-guard
```

```ts
import {
  canonicalizeToolsJson,
  distillTurnForCommit,
  pruneCodeForActiveRefinement,
} from 'kv-prefix-guard';

// 1. Canonicalize tool declarations so dictionary key order never invalidates block 0
const toolsJson = canonicalizeToolsJson(JSON.stringify([{ name: 'mcp_search', type: 'object' }]));

// 2. Distill completed assistant turns once before appending to conversation history
const blueprint = distillTurnForCommit(assistantHtmlOrMermaidResponse);

// 3. Prune Base64 assets, SVG path strings, and Float32Array buffers from active refinement code
const cleanActiveCode = pruneCodeForActiveRefinement(currentArtifactCode);
```

### 3. Rust (`crates.io`)
```bash
cargo add kv-prefix-guard
```

```rust
use kv_prefix_guard::{
    canonicalize_tools_json, KvPrefixGuardSession, TurnInput, TwoZoneConfig,
};

let config = TwoZoneConfig {
    base_system_prompt: "You are a distributed systems architect.".to_string(),
    session_date_utc: "2026-11-23T18:30:00Z".to_string(),
    session_output_requirement: "Emit runnable HTML5 or Mermaid diagrams.".to_string(),
    trailing_task_scope: None,
};

let mut session = KvPrefixGuardSession::new(config, "claude-opus-5-5", 4);
let (turn_prompt, schedule) = session.prepare_turn(&TurnInput {
    user_message: "Build a cluster KV-cache hit monitor.".to_string(),
    dynamic_output_override: None,
    live_tool_context: Some("us-east-1 p99: 11.4ms".to_string()),
});
```

---

## Citation

See [`CITATION.cff`](CITATION.cff), or cite:

```bibtex
@article{michael2026kvprefixguard,
  title   = {Prefix-Invariant Context Partitioning and Commit-Time Relational Artifact Distillation for Multi-Turn Agentic and Multimodal LLM KV-Caches},
  author  = {Michael, David Charles},
  year    = {2026},
  url     = {https://github.com/davidhcar/kv-prefix-guard},
  note    = {ORCID: 0009-0007-8679-6941}
}
```

## License

MIT License. Copyright (c) 2026 David Charles Michael. See [`LICENSE`](LICENSE).
