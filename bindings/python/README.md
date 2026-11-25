# `kv-prefix-guard` (Python Native `PyO3` C-Extension, `PyPI`)

**Context engineering, prefix-invariant KV-cache partitioning, $K$-bounded prompt caching breakpoint scheduler, and commit-time relational artifact distillation for multi-turn AI agents and MCP gateways ($86.2\%$ token reduction, $95.4\%$ KV-cache hit rate, $6.74\times$ lower prefill cost).**

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.24488870.svg)](https://doi.org/10.5281/zenodo.24488870)
[![PyPI](https://img.shields.io/pypi/v/kv-prefix-guard.svg)](https://pypi.org/project/kv-prefix-guard/)
[![Crates.io](https://img.shields.io/crates/v/kv-prefix-guard.svg)](https://crates.io/crates/kv-prefix-guard)
[![npm](https://img.shields.io/npm/v/kv-prefix-guard.svg)](https://www.npmjs.com/package/kv-prefix-guard)

- **Author:** **David Charles Michael** (`davidchar@gmail.com` | [ORCID: `0009-0007-8679-6941`](https://orcid.org/0009-0007-8679-6941) | [DOI: `10.5281/zenodo.24488870`](https://doi.org/10.5281/zenodo.24488870))
- **License:** MIT
- **Native Rust `PyO3` (`abi3-py39`) Speed:** Compiled to native machine code with zero secondary LLM calls. Designed for LangGraph, LangChain, LlamaIndex, PydanticAI, LiteLLM, CrewAI, AutoGen, and Python MCP servers.

---

## 1. Installation

```bash
pip install -U kv-prefix-guard
```

---

## 2. How to Implement & Verified Output (Python)

### Runnable Code Snippet (`python3 examples/mcp_reasoning_agent.py`)

```python
import kv_prefix_guard as kpg

# 1. Canonical `[Tools]` JSON schema normalization across reordered Python dicts
tools_t1 = '[{"description":"Run query","name":"mcp_sql","input_schema":{"required":["q"],"type":"object"}}]'
tools_t2 = '[{"input_schema":{"type":"object","required":["q"]},"name":"mcp_sql","description":"Run query"}]'
assert kpg.py_canonicalize_tools_json(tools_t1) == kpg.py_canonicalize_tools_json(tools_t2)

# 2. Preserve signed `<thinking signature="...">` while scrubbing `<scratchpad>` & distilling Mermaid
raw_assistant_t1 = (
    '<thinking signature="sig_claude_5_5_verified">Signed reasoning trace</thinking>\n'
    '<scratchpad>Drafting internal DOM and Mermaid nodes...</scratchpad>\n'
    'Status: Complete\n'
    '```mermaid\nflowchart TD\n  subgraph Ingestion\n    A[Gateway] --> B[Radix Cache]\n  end\n```'
)
committed_t1 = kpg.py_distill_turn_for_commit(raw_assistant_t1)
assert kpg.py_distill_turn_for_commit(committed_t1) == committed_t1  # Strictly idempotent
print(committed_t1)

# 3. Brace-depth AST code skeletonization
skeleton = kpg.py_compact_code_skeleton(
    "export class PrefixCacheController {\n  public evaluate(tokens: number[]): number {\n    return tokens.length;\n  }\n}",
    15,
)
print(skeleton)
```
