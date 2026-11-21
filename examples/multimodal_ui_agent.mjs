import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const kpg = require('../bindings/js/index.js');

// 1. Canonical tool schema ordering
const toolsTurn1 = JSON.stringify([
  { name: 'search_docs', description: 'Search docs', parameters: { required: ['q'], type: 'object' } },
  { name: 'read_trace', description: 'Read trace' },
]);
const toolsTurn2 = JSON.stringify([
  { description: 'Read trace', name: 'read_trace' },
  { parameters: { type: 'object', required: ['q'] }, name: 'search_docs', description: 'Search docs' },
]);

assert.equal(
  kpg.canonicalizeToolsJson(toolsTurn1),
  kpg.canonicalizeToolsJson(toolsTurn2),
  'Tool schemas must canonicalize to identical JSON strings'
);

// 2. Commit-time Relational Event-Graph Blueprint for HTML5 / Three.js WebGL app
const rawHtmlTurn = `<scratchpad>Planning Three.js scene and controls...</scratchpad>
\`\`\`html
<!DOCTYPE html>
<html>
<head><title>Telemetry 3D Globe</title></head>
<body>
  <button id="toggle-rotation-btn">Toggle Rotation</button>
  <canvas id="webgl-globe-canvas"></canvas>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js"></script>
  <script>
    let rotationSpeed = 0.015;
    const meshGroup = { nodes: 64 };
    function handleToggleRotation(evt) { rotationSpeed = rotationSpeed > 0 ? 0 : 0.015; }
    function animateGlobe() { requestAnimationFrame(animateGlobe); }
    document.getElementById('toggle-rotation-btn').addEventListener('click', handleToggleRotation);
    requestAnimationFrame(animateGlobe);
  </script>
</body>
</html>
\`\`\``;

const distilled1 = kpg.distillTurnForCommit(rawHtmlTurn);
const distilled2 = kpg.distillTurnForCommit(distilled1);

assert.equal(distilled1, distilled2, 'Distillation must be idempotent across turns');
assert.ok(!distilled1.includes('<scratchpad>'), 'Unsigned scratchpad must be stripped');
assert.ok(distilled1.includes('[RELATIONAL ARTIFACT BLUEPRINT: "Telemetry 3D Globe"]'));
assert.ok(distilled1.includes('#toggle-rotation-btn'));
assert.ok(distilled1.includes('#webgl-globe-canvas'));
assert.ok(distilled1.includes('handleToggleRotation(evt)'));
assert.ok(distilled1.includes('--[ON_CLICK]-->'));

// 3. Active refinement code pruner (Base64 + SVG path + Float32Array)
const rawRefinementCode = `<img id="logo" src="data:image/png;base64,${'B'.repeat(120)}" /><svg><path id="curve" d="${'M 0 0 L 10 20 '.repeat(12)}" /></svg><script>const buf = [${'0.11, -0.22, 0.33, '.repeat(12)}];</script>`;
const pruned = kpg.pruneCodeForActiveRefinement(rawRefinementCode);

assert.ok(pruned.length < rawRefinementCode.length / 2);
assert.ok(pruned.includes('id="logo"'));
assert.ok(pruned.includes('id="curve"'));
assert.ok(pruned.includes('...[embedded-media-asset]'));
assert.ok(pruned.includes('... [vector-path-data]'));
assert.ok(pruned.includes('/* [numeric coordinate/vertex buffer collapsed] */'));

console.log('Node.js / WASM multimodal UI agent example passed all assertions.');
