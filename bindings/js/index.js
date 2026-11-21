const path = require('path');
const fs = require('fs');

let wasm = null;
const wasmPkgPath = path.join(__dirname, 'pkg', 'kv_prefix_guard.js');
if (fs.existsSync(wasmPkgPath)) {
  wasm = require(wasmPkgPath);
}

function canonicalizeVal(val, isRoot) {
  if (Array.isArray(val)) {
    const mapped = val.map((v) => canonicalizeVal(v, false));
    if (isRoot) {
      mapped.sort((a, b) => {
        const na = (a && (a.name || (a.function && a.function.name))) || '';
        const nb = (b && (b.name || (b.function && b.function.name))) || '';
        return String(na).localeCompare(String(nb));
      });
    }
    return mapped;
  }
  if (val && typeof val === 'object') {
    const sorted = {};
    for (const k of Object.keys(val).sort()) {
      sorted[k] = canonicalizeVal(val[k], false);
    }
    return sorted;
  }
  return val;
}

function canonicalizeToolsJson(rawToolsJson) {
  if (wasm && typeof wasm.wasm_canonicalize_tools_json === 'function') {
    return wasm.wasm_canonicalize_tools_json(String(rawToolsJson || ''));
  }
  try {
    const parsed = JSON.parse(String(rawToolsJson || '').trim());
    return JSON.stringify(canonicalizeVal(parsed, true));
  } catch {
    return String(rawToolsJson || '').trim();
  }
}

function distillTurnForCommit(content) {
  if (wasm && typeof wasm.wasm_distill_turn_for_commit === 'function') {
    return wasm.wasm_distill_turn_for_commit(String(content || ''));
  }
  throw new Error('WASM binary pkg/kv_prefix_guard.js not built yet. Run build_and_verify.sh first.');
}

function pruneCodeForActiveRefinement(rawCode) {
  if (wasm && typeof wasm.wasm_prune_code_for_active_refinement === 'function') {
    return wasm.wasm_prune_code_for_active_refinement(String(rawCode || ''));
  }
  throw new Error('WASM binary pkg/kv_prefix_guard.js not built yet. Run build_and_verify.sh first.');
}

function compactCodeSkeleton(rawCode, maxLines = 30) {
  if (wasm && typeof wasm.wasm_compact_code_skeleton === 'function') {
    return wasm.wasm_compact_code_skeleton(String(rawCode || ''), maxLines);
  }
  throw new Error('WASM binary pkg/kv_prefix_guard.js not built yet. Run build_and_verify.sh first.');
}

module.exports = {
  canonicalizeToolsJson,
  distillTurnForCommit,
  pruneCodeForActiveRefinement,
  compactCodeSkeleton,
};
