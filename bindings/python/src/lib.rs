use ::kv_prefix_guard::{
    canonicalize_tools_json, compact_code_skeleton, distill_app_graph_blueprint,
    distill_diagram_blueprint, distill_document_graph_blueprint, distill_turn_for_commit,
    distill_voxel_blueprint, prune_code_for_active_refinement, purge_reasoning_artifacts,
};
use pyo3::prelude::*;

#[pyfunction]
fn py_canonicalize_tools_json(raw_tools_json: &str) -> String {
    canonicalize_tools_json(raw_tools_json)
}

#[pyfunction]
fn py_distill_turn_for_commit(content: &str) -> String {
    distill_turn_for_commit(content)
}

#[pyfunction]
fn py_purge_reasoning_artifacts(text: &str) -> String {
    purge_reasoning_artifacts(text)
}

#[pyfunction]
fn py_prune_code_for_active_refinement(raw_code: &str) -> String {
    prune_code_for_active_refinement(raw_code)
}

#[pyfunction]
fn py_distill_app_graph_blueprint(raw_html: &str, fallback_title: &str) -> String {
    distill_app_graph_blueprint(raw_html, fallback_title)
}

#[pyfunction]
fn py_distill_diagram_blueprint(raw_mermaid: &str) -> String {
    distill_diagram_blueprint(raw_mermaid)
}

#[pyfunction]
fn py_distill_document_graph_blueprint(raw_doc: &str) -> String {
    distill_document_graph_blueprint(raw_doc)
}

#[pyfunction]
fn py_distill_voxel_blueprint(raw_json: &str) -> String {
    distill_voxel_blueprint(raw_json)
}

#[pyfunction]
fn py_compact_code_skeleton(raw_code: &str, max_lines: usize) -> String {
    compact_code_skeleton(raw_code, max_lines)
}

#[pymodule]
#[pyo3(name = "kv_prefix_guard")]
fn kv_prefix_guard_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_canonicalize_tools_json, m)?)?;
    m.add_function(wrap_pyfunction!(py_distill_turn_for_commit, m)?)?;
    m.add_function(wrap_pyfunction!(py_purge_reasoning_artifacts, m)?)?;
    m.add_function(wrap_pyfunction!(py_prune_code_for_active_refinement, m)?)?;
    m.add_function(wrap_pyfunction!(py_distill_app_graph_blueprint, m)?)?;
    m.add_function(wrap_pyfunction!(py_distill_diagram_blueprint, m)?)?;
    m.add_function(wrap_pyfunction!(py_distill_document_graph_blueprint, m)?)?;
    m.add_function(wrap_pyfunction!(py_distill_voxel_blueprint, m)?)?;
    m.add_function(wrap_pyfunction!(py_compact_code_skeleton, m)?)?;
    Ok(())
}
