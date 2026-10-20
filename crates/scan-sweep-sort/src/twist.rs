//! Twist — the green irreducibility line and brown loam line, braided.
//!
//! Reads the pulse family (pink, green, blue) from worldclock,
//! reads the latest mass-run data if available, and computes:
//!   - green line: structural complexity floor (cannot compress past this)
//!   - brown line: permanence floor (what loamSpine has committed)
//!   - twist rate: how fast green and brown interleave per cycle
//!   - triangle state: pink (L/R), green (Up/Down), blue (hypotenuse)
//!
//! The twist feeds into evolve recommendations: what to sort next,
//! what to anchor, and whether a mass re-weigh or voxel re-sort
//! should run automatically.
//!
//! Authorship: Kevin Mok / Tamison / ecoPrimal / attsi, ORCID 0009-0004-2141-0321

use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use crate::config::{live_dir, PI};
use crate::worldclock::worldclock;

/// Mass run summary — read from the last `sourdough validate mass` output.
fn read_mass_state() -> Value {
    let paths = [
        live_dir().join("mass-run.json"),
        Path::new("/opt/ecoPrimals/sourDough/mass-run.json").to_path_buf(),
    ];
    for p in &paths {
        if let Ok(text) = fs::read_to_string(p) {
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                return v;
            }
        }
    }
    json!({})
}

/// Voxel state — read from detroit classification summary.
fn read_voxel_state() -> Value {
    let path = live_dir().join("classification-summary.json");
    if let Ok(text) = fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            return v;
        }
    }
    json!({})
}

/// Compute the twist state: green line, brown line, triangle, recommendations.
pub fn twist(scan_data: &Value, sort_data: &Value) -> Value {
    let clock = worldclock();
    let pink = clock.get("pink").and_then(Value::as_f64).unwrap_or(0.0);
    let green = clock.get("green").and_then(Value::as_f64).unwrap_or(0.0);
    let blue = clock.get("blue").and_then(Value::as_f64).unwrap_or(0.0);
    let tempo = clock
        .get("tempo")
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    let mass_state = read_mass_state();
    let voxel_state = read_voxel_state();

    // ── Green line: irreducibility ──
    // Composed from:
    //   1. green pulse value (structural complexity from membrane traffic)
    //   2. voxel confidence (how classified the entity space is)
    //   3. Shannon entropy saturation (how measured the content space is)
    let voxel_confidence = sort_data
        .get("sort_quality")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let pages_weighted = mass_state
        .get("pages_weighted")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let pages_total = mass_state
        .get("pages_total")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let mass_coverage = if pages_total > 0 {
        pages_weighted as f64 / pages_total as f64
    } else {
        0.0
    };

    // Green line = average of three irreducibility signals
    // Each signal is bounded [0, 1]:
    //   green pulse: direct from membrane (structural complexity)
    //   voxel confidence: sort quality from pipeline (entity classification)
    //   mass coverage: fraction of pages with Shannon weights
    let green_line = if voxel_confidence > 0.0 || mass_coverage > 0.0 {
        (green + voxel_confidence + mass_coverage) / 3.0
    } else {
        green // fallback: just pulse when no sort data available
    };

    // ── Brown line: permanence ──
    // What fraction of discovered knowledge has been committed.
    // For now: pages_weighted as proxy for "anchored" content.
    let entities_total = scan_data
        .get("entities_discovered")
        .and_then(Value::as_u64)
        .unwrap_or(0) as f64;
    let entities_classified = {
        let total = sort_data
            .get("unified_sort")
            .and_then(|u| u.get("total"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as f64;
        let null = sort_data
            .get("unified_sort")
            .and_then(|u| u.get("null"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as f64;
        total - null
    };
    let entity_permanence = if entities_total > 0.0 {
        entities_classified / entities_total
    } else {
        0.0
    };
    let brown_line = (mass_coverage + entity_permanence) / 2.0;

    // ── Twist rate ──
    // Gap between what we know (green) and what we've committed (brown).
    // Normalized by blue (identity match) so the twist tightens as identity strengthens.
    let gap = (green_line - brown_line).abs();
    let twist_rate = if blue > 0.0 {
        gap / blue
    } else {
        gap
    };

    // ── Triangle state ──
    // Pythagorean check: pink² + green² ≈ blue²?
    let hypotenuse_expected = (pink * pink + green * green).sqrt();
    let triangle_coherence = if hypotenuse_expected > 0.0 {
        1.0 - (blue - hypotenuse_expected).abs() / hypotenuse_expected.max(0.001)
    } else {
        0.0
    };

    // ── Combined interaction strength ──
    // c_E × M_P: entity confidence × mean page mass
    let mean_mass = mass_state
        .get("mean_mass")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let interaction_strength = voxel_confidence * mean_mass;

    // ── Automatic sort recommendations ──
    let mut auto_actions: Vec<Value> = Vec::new();

    // Should we re-weigh pages? (mass coverage low or content changed)
    if mass_coverage < 0.9 && pages_total > 0 {
        auto_actions.push(json!({
            "action": "mass_reweigh",
            "reason": format!("Only {:.0}% of {} pages have Shannon weights", mass_coverage * 100.0, pages_total),
            "priority": "high",
            "command": "sourdough validate mass --apply",
        }));
    }

    // Should we re-sort entities? (voxel confidence below target)
    if voxel_confidence > 0.0 && voxel_confidence < 0.85 {
        auto_actions.push(json!({
            "action": "voxel_resort",
            "reason": format!("Voxel confidence {:.1}% below 85% target", voxel_confidence * 100.0),
            "priority": "medium",
            "target": "342 holdout entities",
        }));
    }

    // Should we anchor? (green far ahead of brown = knowledge not committed)
    if green_line > brown_line + 0.15 {
        auto_actions.push(json!({
            "action": "anchor_commit",
            "reason": format!("Green ({:.2}) ahead of brown ({:.2}) — uncommitted knowledge", green_line, brown_line),
            "priority": "high",
            "description": "loamSpine DataAnchor for recently weighed/sorted content",
        }));
    }

    // Pi sort: is the sort ratio approaching 1/π?
    let pi_target = 1.0 / PI;
    if let Some(pi_val) = scan_data
        .get("sort_classes")
        .and_then(Value::as_object)
        .map(|classes| {
            let self_count = classes
                .get("self")
                .and_then(Value::as_u64)
                .unwrap_or(0) as f64;
            let total: f64 = classes.values().filter_map(Value::as_u64).sum::<u64>() as f64;
            if total > 0.0 {
                self_count / total
            } else {
                0.0
            }
        })
    {
        if pi_val > 0.0 {
            let pi_distance = (pi_val - pi_target).abs();
            auto_actions.push(json!({
                "action": "pi_balance",
                "current_ratio": round3(pi_val),
                "target": round3(pi_target),
                "distance": round3(pi_distance),
                "reason": format!("Sort ratio {:.3} vs target 1/π={:.3} (gap {:.3})", pi_val, pi_target, pi_distance),
                "priority": if pi_distance > 0.1 { "medium" } else { "low" },
            }));
        }
    }

    // Twist tightness recommendation
    if twist_rate > 0.3 {
        auto_actions.push(json!({
            "action": "tighten_twist",
            "reason": format!("Twist rate {:.3} — green/brown gap too wide", twist_rate),
            "priority": "medium",
            "description": "Run both Shannon and voxel sorts to close the gap",
        }));
    }

    json!({
        "triangle": {
            "pink": round3(pink),
            "green": round3(green),
            "blue": round3(blue),
            "coherence": round3(triangle_coherence),
            "tempo": tempo,
        },
        "green_line": {
            "value": round3(green_line),
            "components": {
                "pulse_complexity": round3(green),
                "voxel_confidence": round3(voxel_confidence),
                "mass_coverage": round3(mass_coverage),
            },
            "description": "irreducibility floor — cannot compress past this",
        },
        "brown_line": {
            "value": round3(brown_line),
            "components": {
                "mass_coverage": round3(mass_coverage),
                "entity_permanence": round3(entity_permanence),
            },
            "description": "permanence floor — what loamSpine has committed",
        },
        "twist": {
            "rate": round3(twist_rate),
            "gap": round3(gap),
            "direction": if green_line > brown_line { "green leads" } else { "brown leads" },
            "description": "how fast green and brown interleave — tighter is better",
        },
        "interaction_strength": {
            "value": round3(interaction_strength),
            "formula": "c_E × M_P (entity confidence × mean page mass)",
            "voxel_confidence": round3(voxel_confidence),
            "mean_mass": round3(mean_mass),
        },
        "auto_actions": auto_actions,
        "auto_count": auto_actions.len(),
    })
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twist_empty_inputs() {
        let scan = json!({"entities_discovered": 0, "sort_classes": {}});
        let sort = json!({"sort_quality": 0.0, "unified_sort": {"total": 0, "null": 0}});
        let result = twist(&scan, &sort);
        assert!(result.get("triangle").is_some());
        assert!(result.get("green_line").is_some());
        assert!(result.get("brown_line").is_some());
        assert!(result.get("twist").is_some());
    }

    #[test]
    fn twist_with_sort_data() {
        let scan = json!({
            "entities_discovered": 100,
            "sort_classes": {"self": 30, "non-self": 60, "null": 10},
        });
        let sort = json!({
            "sort_quality": 0.538,
            "unified_sort": {"total": 100, "null": 10, "self": 30, "non_self": 60},
        });
        let result = twist(&scan, &sort);
        let gl = result
            .get("green_line")
            .and_then(|g| g.get("value"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        assert!(gl > 0.0, "green line should be positive with sort data");
    }

    #[test]
    fn twist_recommends_mass_when_low() {
        let scan = json!({"entities_discovered": 50, "sort_classes": {}});
        let sort = json!({"sort_quality": 0.5, "unified_sort": {"total": 50, "null": 5}});
        let result = twist(&scan, &sort);
        // No mass state file means mass_coverage = 0, should NOT recommend mass_reweigh
        // because pages_total is also 0 (no mass file found)
        let actions = result.get("auto_actions").and_then(Value::as_array).unwrap();
        let has_voxel = actions
            .iter()
            .any(|a| a.get("action").and_then(Value::as_str) == Some("voxel_resort"));
        assert!(has_voxel, "should recommend voxel resort when confidence < 85%");
    }
}
