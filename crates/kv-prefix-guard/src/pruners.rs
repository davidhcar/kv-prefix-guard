/// Deterministic single-pass reasoning-trace scrubber and non-destructive payload pruner.
/// Zero regular-expression crate dependencies for predictable O(n) execution.

const REASONING_TAGS: [&str; 6] = [
    "thinking",
    "thought",
    "reasoning",
    "scratchpad",
    "plan",
    "think",
];

const OPERATIONAL_HEADERS: [&str; 9] = [
    "temporal anchor:",
    "live temporal context:",
    "confirmed format:",
    "format compliance:",
    "execution status:",
    "operating parameters:",
    "system directive:",
    "protocol status:",
    "status:",
];

fn find_ascii_case_insensitive(haystack: &str, needle: &str, start_from: usize) -> Option<usize> {
    let h_bytes = haystack.as_bytes();
    let n_bytes = needle.as_bytes();
    if n_bytes.is_empty() || start_from + n_bytes.len() > h_bytes.len() {
        return None;
    }
    let limit = h_bytes.len() - n_bytes.len();
    for i in start_from..=limit {
        if h_bytes[i..i + n_bytes.len()].eq_ignore_ascii_case(n_bytes) {
            return Some(i);
        }
    }
    None
}

/// Strips inline text-level XML scratchpads (`<scratchpad>...</scratchpad>`, `<thought>...</thought>`,
/// `<plan>...</plan>`, `<think>...</think>`, and unsigned `<thinking>...</thinking>`) and operational
/// echo lines before committing an assistant turn into the immutable KV-cache prefix.
/// Preserves any `<thinking signature="...">` blocks that carry a cryptographic provider signature.
pub fn purge_reasoning_artifacts(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut current = text.to_string();

    for tag in REASONING_TAGS {
        let open_pat = format!("<{}", tag);
        let close_pat = format!("</{}>", tag);
        let mut search_from = 0usize;
        loop {
            let Some(open_idx) = find_ascii_case_insensitive(&current, &open_pat, search_from) else {
                break;
            };
            let after_tag = open_idx + open_pat.len();
            let valid_boundary = current
                .as_bytes()
                .get(after_tag)
                .map(|&b| b == b'>' || b.is_ascii_whitespace())
                .unwrap_or(false);
            if !valid_boundary {
                search_from = after_tag;
                continue;
            }
            let Some(gt_rel) = current[after_tag..].find('>') else {
                break;
            };
            let attr_slice = &current[after_tag..after_tag + gt_rel];
            let content_start = after_tag + gt_rel + 1;

            // Never strip cryptographically signed provider thinking blocks
            if attr_slice.to_ascii_lowercase().contains("signature=") {
                search_from = content_start;
                continue;
            }

            if let Some(close_idx) = find_ascii_case_insensitive(&current, &close_pat, content_start)
            {
                let remove_end = close_idx + close_pat.len();
                current.replace_range(open_idx..remove_end, "");
            } else if current[..open_idx].trim().is_empty() {
                current.replace_range(open_idx..current.len(), "");
                break;
            } else {
                break;
            }
        }
    }

    let mut filtered_lines = Vec::new();
    for line in current.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("[live temporal context:") && lower.ends_with(']') {
            continue;
        }
        let is_op_header = OPERATIONAL_HEADERS
            .iter()
            .any(|h| lower.starts_with(h));
        if is_op_header {
            continue;
        }
        filtered_lines.push(line);
    }

    filtered_lines.join("\n").trim_start().to_string()
}

/// Collapses inline Base64 data URIs (`data:image/...;base64,<60+ chars>`) while preserving MIME metadata.
pub fn prune_embedded_media(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut cursor = 0usize;
    while let Some(idx) = find_ascii_case_insensitive(code, "data:", cursor) {
        out.push_str(&code[cursor..idx]);
        let rest = &code[idx..];
        if let Some(b64_rel) = find_ascii_case_insensitive(rest, ";base64,", 0) {
            let mime_part = &rest[..b64_rel];
            let valid_mime = mime_part.len() <= 48
                && (mime_part[5..].starts_with("image/")
                    || mime_part[5..].starts_with("audio/")
                    || mime_part[5..].starts_with("font/")
                    || mime_part[5..].starts_with("application/"));
            if valid_mime {
                let payload_start = idx + b64_rel + 8;
                let bytes = code.as_bytes();
                let mut payload_end = payload_start;
                while payload_end < bytes.len() {
                    let b = bytes[payload_end];
                    if b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=' {
                        payload_end += 1;
                    } else {
                        break;
                    }
                }
                if payload_end - payload_start >= 60 {
                    out.push_str(mime_part);
                    out.push_str(";base64,...[embedded-media-asset]");
                    cursor = payload_end;
                    continue;
                }
            }
        }
        out.push_str("data:");
        cursor = idx + 5;
    }
    out.push_str(&code[cursor..]);
    out
}

fn prune_long_attribute(code: &str, attr_prefix: &str, marker: &str, min_len: usize) -> String {
    let mut out = String::with_capacity(code.len());
    let mut cursor = 0usize;
    while let Some(idx) = find_ascii_case_insensitive(code, attr_prefix, cursor) {
        let at_boundary = idx == 0
            || !code.as_bytes()[idx - 1].is_ascii_alphanumeric()
                && code.as_bytes()[idx - 1] != b'_'
                && code.as_bytes()[idx - 1] != b'-';
        let after_eq = idx + attr_prefix.len();
        let quote_byte = code.as_bytes().get(after_eq).copied();
        if at_boundary && (quote_byte == Some(b'"') || quote_byte == Some(b'\'')) {
            let q = quote_byte.unwrap() as char;
            let val_start = after_eq + 1;
            if let Some(close_rel) = code[val_start..].find(q) {
                let val_end = val_start + close_rel;
                let val_slice = &code[val_start..val_end];
                if val_slice.len() >= min_len {
                    out.push_str(&code[cursor..val_start]);
                    let preview_end = val_slice
                        .char_indices()
                        .nth(30)
                        .map(|(i, _)| i)
                        .unwrap_or(val_slice.len());
                    out.push_str(val_slice[..preview_end].trim());
                    out.push_str(marker);
                    out.push(q);
                    cursor = val_end + 1;
                    continue;
                }
            }
        }
        out.push_str(&code[cursor..after_eq]);
        cursor = after_eq;
    }
    out.push_str(&code[cursor..]);
    out
}

/// Prunes long SVG path strings (`d="..."` >= 100 chars) and polyline `points="..."` attributes.
pub fn prune_svg_vectors(code: &str) -> String {
    let pass1 = prune_long_attribute(code, "d=", "... [vector-path-data]", 100);
    prune_long_attribute(&pass1, "points=", "... [vector-points]", 100)
}

/// Prunes large numeric WebGL / Canvas vertex arrays (`Float32Array([...])` or `[0.12, -0.45, ...]` >= 150 chars).
pub fn prune_coordinate_buffers(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let bytes = code.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            let start = i + 1;
            let mut j = start;
            let mut comma_count = 0usize;
            while j < bytes.len() {
                let b = bytes[j];
                if b.is_ascii_digit()
                    || b == b'-'
                    || b == b'+'
                    || b == b'.'
                    || b == b','
                    || b.is_ascii_whitespace()
                {
                    if b == b',' {
                        comma_count += 1;
                    }
                    j += 1;
                } else {
                    break;
                }
            }
            if j < bytes.len() && bytes[j] == b']' && (j - start) >= 150 && comma_count >= 12 {
                out.push('[');
                let inner = &code[start..j];
                let preview_len = inner.len().min(40);
                out.push_str(inner[..preview_len].trim());
                out.push_str("... /* [numeric coordinate/vertex buffer collapsed] */]");
                i = j + 1;
                continue;
            }
        }
        let ch = code[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Prunes long static HTML paragraph text (> 300 chars without child tags) while keeping DOM tags intact.
pub fn prune_static_paragraphs(code: &str) -> String {
    let mut current = code.to_string();
    for tag in ["p", "article"] {
        let open_pat = format!("<{}", tag);
        let close_pat = format!("</{}>", tag);
        let mut cursor = 0usize;
        while let Some(open_idx) = find_ascii_case_insensitive(&current, &open_pat, cursor) {
            let after_tag = open_idx + open_pat.len();
            let valid_tag = current
                .as_bytes()
                .get(after_tag)
                .map(|&b| b == b'>' || b.is_ascii_whitespace())
                .unwrap_or(false);
            if !valid_tag {
                cursor = after_tag;
                continue;
            }
            let Some(gt_rel) = current[after_tag..].find('>') else {
                break;
            };
            let body_start = after_tag + gt_rel + 1;
            let Some(close_idx) = find_ascii_case_insensitive(&current, &close_pat, body_start) else {
                break;
            };
            let inner = &current[body_start..close_idx];
            if inner.len() > 300 && !inner.contains('<') {
                let preview_end = inner
                    .char_indices()
                    .nth(100)
                    .map(|(i, _)| i)
                    .unwrap_or(inner.len());
                let replacement = format!(
                    "{}... [static content text preserved]",
                    inner[..preview_end].trim()
                );
                current.replace_range(body_start..close_idx, &replacement);
                cursor = body_start + replacement.len() + close_pat.len();
            } else {
                cursor = close_idx + close_pat.len();
            }
        }
    }
    current
}

/// Prunes massive voxel coordinate arrays (`"voxels": [[...], [...], ...]`) after the first 2 tuples.
pub fn prune_voxel_matrices(code: &str) -> String {
    let Some(v_idx) = code.find("\"voxels\"") else {
        return code.to_string();
    };
    let Some(arr_open_rel) = code[v_idx..].find('[') else {
        return code.to_string();
    };
    let arr_start = v_idx + arr_open_rel + 1;
    let bytes = code.as_bytes();
    let mut depth = 1isize;
    let mut i = arr_start;
    let mut tuple_closes = Vec::new();
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'[' {
            depth += 1;
        } else if bytes[i] == b']' {
            depth -= 1;
            if depth == 1 {
                tuple_closes.push(i);
            }
        }
        i += 1;
    }
    let arr_end = i.saturating_sub(1);
    if tuple_closes.len() > 2 && (arr_end - tuple_closes[1]) > 100 {
        let keep_until = tuple_closes[1] + 1;
        let collapsed_count = tuple_closes.len() - 2;
        let mut out = String::with_capacity(code.len());
        out.push_str(&code[..keep_until]);
        out.push_str(&format!(
            ", /* ... [{} additional voxels collapsed] */ ",
            collapsed_count
        ));
        out.push_str(&code[arr_end..]);
        return out;
    }
    code.to_string()
}

/// Prunes repetitive Mermaid styling directives (`classDef`, `style`, `linkStyle`) while
/// preserving 100% of node definitions, subgraphs, and directed edges.
pub fn prune_diagram_styling(code: &str) -> String {
    let mut class_defs = Vec::new();
    let mut style_count = 0usize;
    let mut link_style_count = 0usize;
    let mut kept_lines = Vec::new();

    for line in code.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("classDef ") {
            if let Some(name) = rest.split_whitespace().next() {
                class_defs.push(name.to_string());
            }
            continue;
        }
        if trimmed.starts_with("style ") && trimmed.contains("fill:") {
            style_count += 1;
            continue;
        }
        if trimmed.starts_with("linkStyle ") && trimmed.contains("stroke:") {
            link_style_count += 1;
            continue;
        }
        kept_lines.push(line);
    }

    if class_defs.len() < 2 && style_count < 2 && link_style_count < 2 {
        return code.to_string();
    }

    let mut out = kept_lines.join("\n");
    if !class_defs.is_empty() {
        out.push_str(&format!(
            "\n  %% [classDefs collapsed: {}] %%",
            class_defs.join(", ")
        ));
    }
    if style_count > 0 {
        out.push_str("\n  %% [node styles collapsed] %%");
    }
    if link_style_count > 0 {
        out.push_str("\n  %% [linkStyles collapsed] %%");
    }
    out
}

/// Runs the full non-destructive active-turn code pruner pipeline in O(n) time.
pub fn prune_code_for_active_refinement(raw_code: &str) -> String {
    if raw_code.len() < 100 {
        return raw_code.to_string();
    }
    let s1 = prune_embedded_media(raw_code);
    let s2 = prune_svg_vectors(&s1);
    let s3 = prune_coordinate_buffers(&s2);
    let s4 = prune_static_paragraphs(&s3);
    let s5 = prune_voxel_matrices(&s4);
    prune_diagram_styling(&s5)
}
