//! 5-stage pipeline: SCAN → SWEEP → SORT → FOSSILIZE → EVOLVE.

use serde_json::{json, Value};
use std::collections::HashMap;

use crate::config::PI;

/// Stage 1: Discover entities — detroit classification, fleet antibodies, braids, traffic.
pub fn scan(
    classification: &Value,
    graph: &Value,
    human: &Value,
    chipmunk: &Value,
    braid_data: &Value,
    fleet_data: &Value,
) -> Value {
    // Unwrap entities dict if classification has a wrapper
    let entities = if let Some(ents) = classification.get("entities").and_then(Value::as_object) {
        ents.clone()
    } else if let Some(obj) = classification.as_object() {
        obj.clone()
    } else {
        serde_json::Map::new()
    };
    let entity_count = entities.len();

    let edges = graph
        .get("edges")
        .or_else(|| graph.get("links"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    let mut sort_classes: HashMap<String, u64> = HashMap::new();
    let mut methods: HashMap<String, u64> = HashMap::new();
    let mut confidences: Vec<f64> = Vec::new();

    for v in entities.values() {
        let Some(obj) = v.as_object() else { continue };
        let cls = obj
            .get("sort_class")
            .or_else(|| obj.get("classification"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let meth = obj
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        *sort_classes.entry(cls.to_string()).or_default() += 1;
        *methods.entry(meth.to_string()).or_default() += 1;
        if let Some(c) = obj.get("confidence").and_then(Value::as_f64) {
            confidences.push(c);
        }
    }

    let avg_confidence = if confidences.is_empty() {
        0.0
    } else {
        confidences.iter().sum::<f64>() / confidences.len() as f64
    };

    let mut edge_types: HashMap<String, u64> = HashMap::new();
    let edge_arr = graph
        .get("edges")
        .or_else(|| graph.get("links"))
        .and_then(Value::as_array);
    if let Some(arr) = edge_arr {
        for e in arr {
            let t = e
                .get("type")
                .or_else(|| e.get("label"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            *edge_types.entry(t.to_string()).or_default() += 1;
        }
    }

    let mut human_sessions: u64 = 0;
    let mut epitope_types: HashMap<String, u64> = HashMap::new();
    if let Some(eps) = human.get("epitopes").and_then(Value::as_object) {
        human_sessions = human
            .get("energy")
            .and_then(|e| e.get("total_sessions"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        for (name, info) in eps {
            let count = info.get("count").and_then(Value::as_u64).unwrap_or(0);
            epitope_types.insert(name.clone(), count);
        }
    }

    let mut anderson_modes: HashMap<String, u64> = HashMap::new();
    if let Some(membrane) = chipmunk.get("membrane").and_then(Value::as_array) {
        for mem in membrane {
            if let Some(anderson) = mem
                .get("metadata")
                .and_then(|m| m.get("anderson"))
                .and_then(Value::as_object)
            {
                if let Some(modes) = anderson.get("modes").and_then(Value::as_array) {
                    for mode in modes {
                        let cls = mode
                            .get("class")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let count = mode.get("count").and_then(Value::as_u64).unwrap_or(0);
                        *anderson_modes.entry(cls.to_string()).or_default() += count;
                    }
                }
            }
        }
    }

    let braid_routes = braid_data
        .get("graph")
        .and_then(|g| g.get("active_routes"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let braid_density = braid_data
        .get("graph")
        .and_then(|g| g.get("density"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let shifters = braid_data
        .get("graph")
        .and_then(|g| g.get("shifters"))
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let fleet_classes: HashMap<String, u64> = fleet_data
        .get("sort_classes")
        .and_then(Value::as_object)
        .map(|o| {
            o.iter()
                .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0)))
                .collect()
        })
        .unwrap_or_default();
    let fleet_profiles: HashMap<String, u64> = fleet_data
        .get("profiles")
        .and_then(Value::as_object)
        .map(|o| {
            o.iter()
                .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0)))
                .collect()
        })
        .unwrap_or_default();
    let fleet_total = fleet_data
        .get("total_entities")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    // Sort and limit methods + edge_types to top 10
    let method_top = top_n(&methods, 10);
    let edge_top = top_n(&edge_types, 10);

    json!({
        "entities_discovered": entity_count,
        "fleet_entities": fleet_total,
        "fleet_classes": fleet_classes,
        "fleet_profiles": fleet_profiles,
        "edges_discovered": edges,
        "nodes_discovered": nodes,
        "sort_classes": sort_classes,
        "classification_methods": methods.len(),
        "method_breakdown": method_top,
        "avg_confidence": round3(avg_confidence),
        "edge_types": edge_top,
        "human_sessions": human_sessions,
        "epitope_types": epitope_types,
        "anderson_modes": anderson_modes,
        "braid_routes": braid_routes,
        "braid_density": round3(braid_density),
        "shifters": shifters,
    })
}

/// Stage 2: Compress, find structure, deduplicate.
pub fn sweep(scan_data: &Value) -> Value {
    let mut patterns: Vec<Value> = Vec::new();
    let total = scan_data
        .get("entities_discovered")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let unknown = get_sort_class(scan_data, "unknown");
    let classified = total.saturating_sub(unknown);
    let coverage = classified as f64 / total.max(1) as f64;

    patterns.push(json!({
        "name": "classification_coverage",
        "value": round3(coverage),
        "description": format!("{classified}/{total} entities classified ({:.0}%)", coverage * 100.0),
    }));

    let avg_conf = scan_data
        .get("avg_confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    if avg_conf > 0.0 {
        patterns.push(json!({
            "name": "confidence_mean",
            "value": avg_conf,
            "description": format!("average sort confidence {:.1}%", avg_conf * 100.0),
        }));
    }

    let node_count = scan_data
        .get("nodes_discovered")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let edge_count = scan_data
        .get("edges_discovered")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if node_count > 0 {
        let max_e = node_count * (node_count - 1) / 2;
        let ed = edge_count as f64 / max_e.max(1) as f64;
        patterns.push(json!({
            "name": "graph_density",
            "value": (ed * 10000.0).round() / 10000.0,
            "description": format!("{edge_count} edges across {node_count} nodes"),
        }));
    }

    let human_sessions = scan_data
        .get("human_sessions")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if human_sessions > 0 && total > 0 {
        let ar = human_sessions as f64 / total as f64;
        patterns.push(json!({
            "name": "human_attention_ratio",
            "value": (ar * 100.0).round() / 100.0,
            "description": format!("{human_sessions} human sessions across {total} entities"),
        }));
    }

    let methods = scan_data
        .get("classification_methods")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    patterns.push(json!({
        "name": "cascade_methods",
        "value": methods,
        "description": format!("{methods} distinct classification methods used"),
    }));

    if total > 0 {
        let pi_ratio = classified as f64 / total.max(1) as f64;
        patterns.push(json!({
            "name": "pi_sort_ratio",
            "value": round3(pi_ratio),
            "target": round3(1.0 / PI),
            "description": format!("sort ratio {pi_ratio:.3} (target 1/π = {:.3})", 1.0 / PI),
        }));
    }

    let braid_routes = scan_data
        .get("braid_routes")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if braid_routes > 0 {
        patterns.push(json!({
            "name": "braid_flow",
            "value": braid_routes,
            "description": format!("{braid_routes} active braid routes"),
        }));
    }

    let fleet_entities = scan_data
        .get("fleet_entities")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if fleet_entities > 0 {
        let fp_len = scan_data
            .get("fleet_profiles")
            .and_then(Value::as_object)
            .map(|o| o.len())
            .unwrap_or(0);
        patterns.push(json!({
            "name": "fleet_refinement",
            "value": fleet_entities,
            "description": format!("{fleet_entities} fleet entities classified into {fp_len} profiles"),
        }));
    }

    let et_len = scan_data
        .get("edge_types")
        .and_then(Value::as_object)
        .map(|o| o.len())
        .unwrap_or(0);
    let ep_len = scan_data
        .get("epitope_types")
        .and_then(Value::as_object)
        .map(|o| o.len())
        .unwrap_or(0);
    let am_len = scan_data
        .get("anderson_modes")
        .and_then(Value::as_object)
        .map(|o| o.len())
        .unwrap_or(0);

    json!({
        "patterns_found": patterns.len(),
        "patterns": patterns,
        "compression_summary": {
            "sort_classes": 4,
            "edge_types_found": et_len,
            "epitope_types": ep_len,
            "anderson_modes": am_len,
        }
    })
}

/// Stage 3: Classify self/non-self/partial/null + fleet + braids.
pub fn sort_phase(scan_data: &Value, _sweep_data: &Value) -> Value {
    let total_self_det = get_sort_class(scan_data, "self")
        + get_anderson(scan_data, "human")
        + get_anderson(scan_data, "sovereign");
    let total_nonself_det = get_sort_class(scan_data, "non-self")
        + get_anderson(scan_data, "chorus")
        + get_anderson(scan_data, "spectrum");
    let total_partial_det = get_sort_class(scan_data, "partial") + get_anderson(scan_data, "ghost");
    let total_null_det = get_sort_class(scan_data, "null");

    let fc = scan_data
        .get("fleet_classes")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let fc_u = |k: &str| -> u64 { fc.get(k).and_then(Value::as_u64).unwrap_or(0) };

    let total_self = total_self_det + fc_u("commensal");
    let total_nonself = total_nonself_det + fc_u("parasite") + fc_u("pathogen");
    let total_partial = total_partial_det + fc_u("ghost") + fc_u("stealth") + fc_u("partial");
    let total_null = total_null_det + fc_u("null");

    let grand_total = total_self + total_nonself + total_partial + total_null;
    let selectivity = if grand_total > 0 {
        total_self as f64 / (total_self + total_nonself).max(1) as f64
    } else {
        0.0
    };

    let sort_classes_val = scan_data
        .get("sort_classes")
        .cloned()
        .unwrap_or(json!({}));

    json!({
        "summary": {
            "detroit_entities": sort_classes_val,
            "membrane_traffic": scan_data.get("anderson_modes").cloned().unwrap_or(json!({})),
            "human_epitopes": scan_data.get("epitope_types").cloned().unwrap_or(json!({})),
            "fleet_classes": fc,
        },
        "unified_sort": {
            "self": total_self,
            "non_self": total_nonself,
            "partial": total_partial,
            "null": total_null,
            "total": grand_total,
            "selectivity": round3(selectivity),
        },
        "unsorted": compute_unsorted(scan_data),
        "sort_quality": scan_data.get("avg_confidence").and_then(Value::as_f64).map(round3).unwrap_or(0.0),
    })
}

/// Stage 4: Record what we learned from 8 perspectives.
pub fn fossilize(scan_data: &Value, sweep_data: &Value, sort_data: &Value) -> Value {
    let mut observations: Vec<Value> = Vec::new();

    let obs = |out: &mut Vec<Value>, perspective: &str, text: &str| {
        out.push(json!({"perspective": perspective, "text": text}));
    };

    let u = sort_data.get("unified_sort");
    let sel = u
        .and_then(|u| u.get("selectivity"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    obs(
        &mut observations,
        "on",
        &format!(
            "Selectivity {:.1}% — self vs non-self at the surface",
            sel * 100.0
        ),
    );

    // Sweep pattern observations
    if let Some(patterns) = sweep_data.get("patterns").and_then(Value::as_array) {
        for p in patterns {
            let name = p.get("name").and_then(Value::as_str).unwrap_or("");
            let desc = p.get("description").and_then(Value::as_str).unwrap_or("");
            match name {
                "human_attention_ratio" => obs(&mut observations, "on", &format!("Human attention: {desc}")),
                "classification_coverage" => obs(&mut observations, "inside", &format!("Entity coverage: {desc}")),
                "confidence_mean" => obs(&mut observations, "inside", &format!("Sort confidence: {desc}")),
                "graph_density" => obs(&mut observations, "above", &format!("Graph: {desc}")),
                "pi_sort_ratio" => obs(&mut observations, "above", &format!("π ratio: {desc}")),
                "cascade_methods" => obs(&mut observations, "beside", &format!("Classification: {desc}")),
                _ => {}
            }
        }
    }

    let partial = u
        .and_then(|u| u.get("partial"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if partial > 0 {
        obs(
            &mut observations,
            "inside",
            &format!("{partial} entities partially sorted — still in transit"),
        );
    }

    let ft = scan_data
        .get("fleet_entities")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if ft > 0 {
        let fc_len = scan_data
            .get("fleet_classes")
            .and_then(Value::as_object)
            .map(|o| o.len())
            .unwrap_or(0);
        let fp_len = scan_data
            .get("fleet_profiles")
            .and_then(Value::as_object)
            .map(|o| o.len())
            .unwrap_or(0);
        obs(
            &mut observations,
            "inside",
            &format!("Fleet: {ft} entities → {fc_len} classes, {fp_len} profiles"),
        );
    }

    let total_val = u
        .and_then(|u| u.get("total"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    obs(
        &mut observations,
        "above",
        &format!("Total: {total_val} entities across detroit + membrane + fleet"),
    );

    let null_val = u
        .and_then(|u| u.get("null"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if null_val > 0 {
        obs(
            &mut observations,
            "below",
            &format!("{null_val} entities at null"),
        );
    }

    let shifters = scan_data
        .get("shifters")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if shifters > 0 {
        obs(
            &mut observations,
            "beside",
            &format!("{shifters} shifters crossing boundaries"),
        );
    }

    let avg_conf = scan_data
        .get("avg_confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    if avg_conf > 0.0 && avg_conf < 0.85 {
        obs(
            &mut observations,
            "before",
            &format!("Confidence {:.1}% still below 85% target", avg_conf * 100.0),
        );
    }

    let braid_routes = scan_data
        .get("braid_routes")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if braid_routes < 8 {
        obs(
            &mut observations,
            "before",
            &format!("Only {braid_routes} active braid routes"),
        );
    }

    // Through perspective — membrane traffic
    let anderson = scan_data
        .get("anderson_modes")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let am_u = |k: &str| -> u64 { anderson.get(k).and_then(Value::as_u64).unwrap_or(0) };

    let human_through = am_u("human") + am_u("sovereign");
    let total_bounce = am_u("chorus") + am_u("spectrum");
    if human_through > 0 {
        obs(
            &mut observations,
            "through",
            &format!("{human_through} passed through (human + sovereign)"),
        );
    }
    if total_bounce > 0 {
        let ratio = total_bounce / human_through.max(1);
        obs(
            &mut observations,
            "through",
            &format!("{total_bounce} bounced off — {ratio}× more non-self than self"),
        );
    }
    if braid_routes > 0 {
        obs(
            &mut observations,
            "through",
            &format!("{braid_routes} passage routes — the holes in the membrane"),
        );
    }

    let ep_count = scan_data
        .get("epitope_types")
        .and_then(Value::as_object)
        .map(|o| o.len())
        .unwrap_or(0);
    if ep_count > 0 {
        obs(
            &mut observations,
            "through",
            &format!("{ep_count} epitope types — different pore shapes"),
        );
    }

    let by_perspective = group_obs(&observations);

    json!({
        "observations": observations,
        "flat": observations.iter().filter_map(|o| o.get("text").and_then(Value::as_str)).collect::<Vec<_>>(),
        "by_perspective": by_perspective,
        "fossil_count": observations.len(),
    })
}

/// Stage 5: Feed learning back — recommendations.
pub fn evolve(scan_data: &Value, _sweep_data: &Value, sort_data: &Value, _fossil_data: &Value) -> Value {
    let mut recs: Vec<Value> = Vec::new();

    let avg_conf = scan_data
        .get("avg_confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    if avg_conf > 0.0 && avg_conf < 0.85 {
        recs.push(json!({
            "action": "rescan_low_confidence",
            "reason": format!("Confidence {:.1}% below 85%", avg_conf * 100.0),
            "priority": "medium",
        }));
    }

    let unsorted = sort_data
        .get("unsorted")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if unsorted > 0 {
        recs.push(json!({
            "action": "classify_remaining",
            "reason": format!("{unsorted} entities still unsorted"),
            "priority": "high",
        }));
    }

    let braid_routes = scan_data
        .get("braid_routes")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if braid_routes < 8 {
        recs.push(json!({
            "action": "discover_braids",
            "reason": format!("Only {braid_routes} braid routes"),
            "priority": "medium",
        }));
    }

    let human_sessions = scan_data
        .get("human_sessions")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if human_sessions > 0 {
        recs.push(json!({
            "action": "human_sort_assist",
            "reason": format!("{human_sessions} humans present"),
            "priority": "high",
        }));
    }

    json!({
        "recommendations": recs,
        "evolution_count": recs.len(),
    })
}

// ── Helpers ──

fn compute_unsorted(scan_data: &Value) -> u64 {
    let total_sorted: u64 = scan_data
        .get("sort_classes")
        .and_then(Value::as_object)
        .map(|o| o.values().filter_map(Value::as_u64).sum())
        .unwrap_or(0);
    let disc = scan_data
        .get("entities_discovered")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    disc.saturating_sub(total_sorted)
}

fn get_sort_class(scan_data: &Value, class: &str) -> u64 {
    scan_data
        .get("sort_classes")
        .and_then(|sc| sc.get(class))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn get_anderson(scan_data: &Value, mode: &str) -> u64 {
    scan_data
        .get("anderson_modes")
        .and_then(|am| am.get(mode))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

fn top_n(m: &HashMap<String, u64>, n: usize) -> Value {
    let mut pairs: Vec<_> = m.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1));
    let obj: serde_json::Map<String, Value> = pairs
        .into_iter()
        .take(n)
        .map(|(k, &v)| (k.clone(), json!(v)))
        .collect();
    Value::Object(obj)
}

fn group_obs(observations: &[Value]) -> Value {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for o in observations {
        let p = o
            .get("perspective")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let t = o
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        groups.entry(p).or_default().push(t);
    }
    json!(groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_empty_inputs() {
        let empty = json!({});
        let result = scan(&empty, &empty, &empty, &empty, &empty, &empty);
        assert_eq!(result.get("entities_discovered").and_then(Value::as_u64), Some(0));
    }

    #[test]
    fn sweep_produces_patterns() {
        let scan_data = json!({
            "entities_discovered": 100, "sort_classes": {"self": 50, "unknown": 20},
            "avg_confidence": 0.8, "nodes_discovered": 10, "edges_discovered": 20,
            "human_sessions": 50, "classification_methods": 3, "edge_types": {},
            "epitope_types": {}, "anderson_modes": {}, "braid_routes": 5,
            "fleet_entities": 30, "fleet_profiles": {"a": 1, "b": 2},
        });
        let result = sweep(&scan_data);
        let count = result.get("patterns_found").and_then(Value::as_u64).unwrap_or(0);
        assert!(count >= 5, "expected ≥5 patterns, got {count}");
    }

    #[test]
    fn sort_phase_computes_selectivity() {
        let scan_data = json!({
            "sort_classes": {"self": 100, "non-self": 20, "partial": 10, "null": 5},
            "anderson_modes": {"human": 50, "sovereign": 10, "chorus": 5, "spectrum": 3, "ghost": 2},
            "fleet_classes": {"commensal": 30, "parasite": 5, "pathogen": 2, "ghost": 3, "null": 8},
            "fleet_profiles": {},
            "avg_confidence": 0.85,
            "entities_discovered": 135,
        });
        let sweep_data = json!({});
        let result = sort_phase(&scan_data, &sweep_data);
        let sel = result.get("unified_sort")
            .and_then(|u| u.get("selectivity"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        assert!(sel > 0.5, "selectivity should favor self, got {sel}");
    }

    #[test]
    fn evolve_produces_recommendations() {
        let scan_data = json!({
            "avg_confidence": 0.7, "braid_routes": 3, "human_sessions": 42,
        });
        let sort_data = json!({"unsorted": 15});
        let result = evolve(&scan_data, &json!({}), &sort_data, &json!({}));
        let count = result.get("evolution_count").and_then(Value::as_u64).unwrap_or(0);
        assert!(count >= 3, "expected ≥3 recommendations, got {count}");
    }
}
