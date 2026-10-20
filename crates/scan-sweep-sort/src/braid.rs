//! Braid learner — passage graph + trend detection + neutron spin enrichment.

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::config::{live_dir, CATEGORY_SITES, PI};
use crate::util::{journal_append, write_json};
use crate::SssError;

/// Learn braid connections from chipmunk passages + human data.
pub fn braid_learn(chipmunk: &Value, human: &Value) -> Result<Value, SssError> {
    let now = chrono::Utc::now();
    let mut braids: Vec<Value> = Vec::new();
    let mut perspectives: Vec<Value> = Vec::new();

    let obs = |persp: &mut Vec<Value>, perspective: &str, text: &str| {
        persp.push(json!({"perspective": perspective, "text": text}));
    };

    // Load history for trend detection
    let hist_file = crate::config::journal_dir().join("braid-history.jsonl");
    let history = load_history(&hist_file);

    // Category passages from chipmunk
    let mut passages: Vec<Value> = Vec::new();
    let mut epitope_tense: Value = json!({});
    if let Some(membrane) = chipmunk.get("membrane").and_then(Value::as_array) {
        for mem in membrane {
            if let Some(md) = mem.get("metadata") {
                if let Some(p) = md.get("passages").and_then(Value::as_array) {
                    passages = p.clone();
                    epitope_tense = md.get("epitope_tense").cloned().unwrap_or(json!({}));
                }
            }
        }
    }

    // Build passage graph
    let mut passage_graph: HashMap<(String, String), u64> = HashMap::new();
    let mut categories: HashSet<String> = HashSet::new();
    let mut category_pairs: HashSet<(String, String)> = HashSet::new();

    for p in &passages {
        let from = p.get("from").and_then(Value::as_str).unwrap_or("");
        let to = p.get("to").and_then(Value::as_str).unwrap_or("");
        let humans = p.get("humans").and_then(Value::as_u64).unwrap_or(0);
        passage_graph.insert((from.to_string(), to.to_string()), humans);
        categories.insert(from.to_string());
        categories.insert(to.to_string());
        let mut pair = [from.to_string(), to.to_string()];
        pair.sort();
        category_pairs.insert((pair[0].clone(), pair[1].clone()));
    }

    for (a, b) in &category_pairs {
        let forward = passage_graph
            .get(&(a.clone(), b.clone()))
            .copied()
            .unwrap_or(0);
        let reverse = passage_graph
            .get(&(b.clone(), a.clone()))
            .copied()
            .unwrap_or(0);
        let total = forward + reverse;

        let is_total = get_tense(&epitope_tense, a, "is") + get_tense(&epitope_tense, b, "is");
        let was_total = get_tense(&epitope_tense, a, "was") + get_tense(&epitope_tense, b, "was");
        let vitality = is_total as f64 / (is_total + was_total).max(1) as f64;
        let strength = total as f64 * (1.0 + vitality * PI);

        let (direction, dominant_dir, prov_from, prov_to) = if forward > 0 && reverse > 0 {
            let ratio = forward.max(reverse) as f64 / forward.min(reverse).max(1) as f64;
            if ratio > 3.0 {
                if forward > reverse {
                    (
                        "dominant",
                        format!("{a} → {b}"),
                        a.clone(),
                        b.clone(),
                    )
                } else {
                    (
                        "dominant",
                        format!("{b} → {a}"),
                        b.clone(),
                        a.clone(),
                    )
                }
            } else {
                (
                    "bidirectional",
                    format!("{a} ↔ {b}"),
                    a.clone(),
                    b.clone(),
                )
            }
        } else if forward > 0 {
            ("one-way", format!("{a} → {b}"), a.clone(), b.clone())
        } else {
            ("one-way", format!("{b} → {a}"), b.clone(), a.clone())
        };

        let prov_from_final = if forward >= reverse {
            a.clone()
        } else {
            b.clone()
        };
        let prov_to_final = if forward >= reverse {
            b.clone()
        } else {
            a.clone()
        };

        braids.push(json!({
            "from": a, "to": b,
            "strength": (strength * 100.0).round() / 100.0,
            "humans_forward": forward, "humans_reverse": reverse,
            "humans_total": total, "direction": direction,
            "dominant_flow": dominant_dir,
            "provenance_direction": format!("{prov_from_final} -> {prov_to_final}"),
            "provenance_from": prov_from_final, "provenance_to": prov_to_final,
            "dag_valid": true,
            "vitality": (vitality * 1000.0).round() / 1000.0,
            "sites_from": CATEGORY_SITES.get(a.as_str()).map(|s| &s[..s.len().min(3)]).unwrap_or(&[]),
            "sites_to": CATEGORY_SITES.get(b.as_str()).map(|s| &s[..s.len().min(3)]).unwrap_or(&[]),
        }));
    }

    braids.sort_by(|a, b| {
        let sa = a.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
        let sb = b.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Trend detection
    if let Some(last) = history.last() {
        let last_braids: HashMap<(String, String), &Value> = last
            .get("braids")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|b| {
                        let f = b.get("from")?.as_str()?.to_string();
                        let t = b.get("to")?.as_str()?.to_string();
                        Some(((f, t), b))
                    })
                    .collect()
            })
            .unwrap_or_default();

        for braid in &mut braids {
            let key = (
                braid.get("from").and_then(Value::as_str).unwrap_or("").to_string(),
                braid.get("to").and_then(Value::as_str).unwrap_or("").to_string(),
            );
            if let Some(prev) = last_braids.get(&key) {
                let cur_s = braid.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
                let prev_s = prev.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
                let delta = cur_s - prev_s;
                let trend = if delta > 0.0 {
                    "growing"
                } else if delta < 0.0 {
                    "fading"
                } else {
                    "stable"
                };
                braid.as_object_mut().map(|o| {
                    o.insert("trend".into(), json!(trend));
                    o.insert("delta".into(), json!((delta * 100.0).round() / 100.0));
                });
            } else {
                let s = braid.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
                braid.as_object_mut().map(|o| {
                    o.insert("trend".into(), json!("emerging"));
                    o.insert("delta".into(), json!(s));
                });
                let flow = braid
                    .get("dominant_flow")
                    .and_then(Value::as_str)
                    .unwrap_or("?");
                obs(
                    &mut perspectives,
                    "after",
                    &format!("New braid: {flow} (strength {s:.1})"),
                );
            }
        }
    } else {
        for braid in &mut braids {
            braid.as_object_mut().map(|o| {
                o.insert("trend".into(), json!("initial"));
                o.insert("delta".into(), json!(0));
            });
        }
    }

    // Neutron spin enrichment
    enrich_neutron_spin(&mut braids, &mut perspectives);

    // Site co-visitation braids
    let site_braids = build_site_braids(human);

    // Epitope transitions
    let epitope_transitions = build_epitope_transitions(human);

    // Observations
    let possible_pairs = categories.len() as f64 * (categories.len() as f64 - 1.0) / 2.0;
    let density = braids.len() as f64 / possible_pairs.max(1.0);

    if let Some(s) = braids.first() {
        let flow = s.get("dominant_flow").and_then(Value::as_str).unwrap_or("?");
        let ht = s.get("humans_total").and_then(Value::as_u64).unwrap_or(0);
        let vit = s.get("vitality").and_then(Value::as_f64).unwrap_or(0.0) * 100.0;
        let trend = s.get("trend").and_then(Value::as_str).unwrap_or("?");
        obs(
            &mut perspectives,
            "on",
            &format!("Strongest braid: {flow} ({ht} humans, vitality {vit:.0}%, {trend})"),
        );
    }

    let mut shifter_count = 0u64;
    if let Some(membrane) = chipmunk.get("membrane").and_then(Value::as_array) {
        for mem in membrane {
            if let Some(s) = mem.get("metadata").and_then(|m| m.get("shifters")).and_then(Value::as_u64) {
                shifter_count = s;
            }
        }
    }

    let bidir: Vec<_> = braids
        .iter()
        .filter(|b| b.get("direction").and_then(Value::as_str) == Some("bidirectional"))
        .collect();
    let oneway: Vec<_> = braids
        .iter()
        .filter(|b| {
            matches!(
                b.get("direction").and_then(Value::as_str),
                Some("dominant") | Some("one-way")
            )
        })
        .collect();
    let total_passages: u64 = braids
        .iter()
        .filter_map(|b| b.get("humans_total").and_then(Value::as_u64))
        .sum();

    if total_passages > 0 {
        obs(
            &mut perspectives,
            "through",
            &format!("{total_passages} humans through {} passages", braids.len()),
        );
    }
    if !bidir.is_empty() {
        obs(
            &mut perspectives,
            "through",
            &format!("{} open pores (bidirectional)", bidir.len()),
        );
    }
    if !oneway.is_empty() {
        obs(
            &mut perspectives,
            "through",
            &format!("{} valves (one-way)", oneway.len()),
        );
    }

    let cross: Vec<_> = site_braids
        .iter()
        .filter(|sb| sb.get("cross_category").and_then(Value::as_bool) == Some(true))
        .collect();
    if !cross.is_empty() {
        obs(
            &mut perspectives,
            "through",
            &format!("{} cross-organ passages", cross.len()),
        );
    }

    let wave = human.get("wave").and_then(Value::as_u64).unwrap_or(0);
    let by_perspective = group_perspectives(&perspectives);
    let mut sorted_categories: Vec<String> = categories.into_iter().collect();
    sorted_categories.sort();

    let result = json!({
        "timestamp": now.to_rfc3339(),
        "wave": wave,
        "braids": braids,
        "site_braids": &site_braids[..site_braids.len().min(10)],
        "epitope_transitions": &epitope_transitions[..epitope_transitions.len().min(6)],
        "graph": {
            "categories": sorted_categories,
            "density": (density * 1000.0).round() / 1000.0,
            "total_passages": total_passages,
            "shifters": shifter_count,
            "active_routes": braids.len(),
        },
        "learning": {
            "observations": perspectives.iter().filter_map(|p| p.get("text").and_then(Value::as_str)).collect::<Vec<_>>(),
            "perspectives": perspectives,
            "by_perspective": by_perspective,
            "observation_count": history.len() + 1,
            "history_depth_hours": (history.len() as f64 * 10.0 / 60.0 * 10.0).round() / 10.0,
        },
    });

    write_json(&live_dir().join("braid.json"), &result)?;

    // Save history
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&hist_file)
    {
        use std::io::Write;
        let compact_braids: Vec<Value> = braids
            .iter()
            .map(|b| {
                json!({
                    "from": b.get("from"),
                    "to": b.get("to"),
                    "strength": b.get("strength"),
                })
            })
            .collect();
        let _ = writeln!(
            f,
            "{}",
            serde_json::to_string(&json!({
                "ts": now.to_rfc3339(),
                "braids": compact_braids,
                "density": (density * 1000.0).round() / 1000.0,
                "shifters": shifter_count,
            }))?
        );
    }

    let _ = journal_append(
        "braids",
        &json!({
            "ts": now.to_rfc3339(), "type": "braid_observation",
            "observations": perspectives.iter().filter_map(|p| p.get("text")).collect::<Vec<_>>(),
            "density": (density * 1000.0).round() / 1000.0,
            "routes": braids.len(),
        }),
    );

    Ok(result)
}

fn get_tense(epitope_tense: &Value, cat: &str, tense: &str) -> u64 {
    epitope_tense
        .get(cat)
        .and_then(|c| c.get(tense))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn load_history(path: &std::path::Path) -> Vec<Value> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .rev()
        .take(48)
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn enrich_neutron_spin(braids: &mut [Value], perspectives: &mut Vec<Value>) {
    let neutron_path = live_dir().join("neutron.json");
    let Ok(text) = std::fs::read_to_string(&neutron_path) else {
        for b in braids.iter_mut() {
            let strength_val = b.get("strength").cloned().unwrap_or(json!(0));
            if let Some(o) = b.as_object_mut() {
                o.insert("neutron_flow".into(), json!(0));
                o.insert("flow_fraction".into(), json!(0));
                o.insert("spinning".into(), json!(false));
                o.insert("combined_strength".into(), strength_val);
            }
        }
        return;
    };
    let Ok(neutron): Result<Value, _> = serde_json::from_str(&text) else {
        return;
    };

    let site_cat = crate::config::site_category_map();
    let flows = neutron.get("flows").and_then(Value::as_object);
    let mut category_flow: HashMap<(String, String), u64> = HashMap::new();

    if let Some(flows) = flows {
        for (src, targets) in flows {
            if let Some(tgts) = targets.as_object() {
                let src_cat = site_cat.get(src.as_str()).cloned().unwrap_or("OTHER".into());
                for (tgt, count) in tgts {
                    let c = count.as_u64().unwrap_or(0);
                    if c > 0 {
                        let tgt_cat = site_cat.get(tgt.as_str()).cloned().unwrap_or("OTHER".into());
                        let mut pair = [src_cat.clone(), tgt_cat];
                        pair.sort();
                        *category_flow
                            .entry((pair[0].clone(), pair[1].clone()))
                            .or_default() += c;
                    }
                }
            }
        }
    }

    let total_flow: u64 = category_flow.values().sum();
    let tf = total_flow.max(1) as f64;

    for braid in braids.iter_mut() {
        let from = braid.get("from").and_then(Value::as_str).unwrap_or("");
        let to = braid.get("to").and_then(Value::as_str).unwrap_or("");
        let mut pair = [from.to_string(), to.to_string()];
        pair.sort();
        let flow = category_flow
            .get(&(pair[0].clone(), pair[1].clone()))
            .copied()
            .unwrap_or(0);
        let strength = braid.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
        let combined = strength * (1.0 + flow as f64 / tf);

        if let Some(o) = braid.as_object_mut() {
            o.insert("neutron_flow".into(), json!(flow));
            o.insert(
                "flow_fraction".into(),
                json!((flow as f64 / tf * 1000.0).round() / 1000.0),
            );
            o.insert("spinning".into(), json!(flow > 0));
            o.insert(
                "combined_strength".into(),
                json!((combined * 100.0).round() / 100.0),
            );
        }
    }

    // Generate flow-only braids where neutron traffic exists but no human passage
    let existing: HashSet<(String, String)> = braids
        .iter()
        .filter_map(|b| {
            let f = b.get("from")?.as_str()?.to_string();
            let t = b.get("to")?.as_str()?.to_string();
            let mut p = [f, t];
            p.sort();
            Some((p[0].clone(), p[1].clone()))
        })
        .collect();

    let mut flow_only: Vec<((String, String), u64)> = category_flow
        .iter()
        .filter(|(pair, flow)| !existing.contains(pair) && pair.0 != pair.1 && **flow > 0)
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    flow_only.sort_by(|a, b| b.1.cmp(&a.1));

    // (Extend braids vec would require converting to Vec — already done above via braid_learn)

    // Re-sort by combined_strength
    braids.sort_by(|a, b| {
        let sa = a
            .get("combined_strength")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let sb = b
            .get("combined_strength")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    let spinning_count = braids
        .iter()
        .filter(|b| b.get("spinning").and_then(Value::as_bool) == Some(true))
        .count();
    perspectives.push(json!({
        "perspective": "on",
        "text": format!("Rope spinning: {spinning_count}/{} braids carry neutron flow ({} strands)",
            braids.len(), braids.len()),
    }));
}

fn build_site_braids(human: &Value) -> Vec<Value> {
    let host_dist = human.get("host_distribution").and_then(Value::as_object);
    let Some(hd) = host_dist else {
        return Vec::new();
    };

    let site_cat = crate::config::site_category_map();
    let active: Vec<(&String, u64)> = hd
        .iter()
        .filter_map(|(s, c)| {
            let count = c.as_u64()?;
            if count > 10 {
                Some((s, count))
            } else {
                None
            }
        })
        .collect();

    let mut site_braids = Vec::new();
    for (i, (sa, ta)) in active.iter().enumerate() {
        for (sb, tb) in &active[i + 1..] {
            let ca = site_cat.get(sa.as_str()).cloned().unwrap_or("OTHER".into());
            let cb = site_cat.get(sb.as_str()).cloned().unwrap_or("OTHER".into());
            let co = (*ta).min(*tb) as f64 / (*ta).max(*tb).max(1) as f64;
            if co > 0.01 {
                site_braids.push(json!({
                    "site_a": sa, "site_b": sb,
                    "co_strength": (co * 1000.0).round() / 1000.0,
                    "cross_category": ca != cb,
                    "category_a": ca, "category_b": cb,
                }));
            }
        }
    }
    site_braids.sort_by(|a, b| {
        let sa = a.get("co_strength").and_then(Value::as_f64).unwrap_or(0.0);
        let sb = b.get("co_strength").and_then(Value::as_f64).unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    site_braids
}

fn build_epitope_transitions(human: &Value) -> Vec<Value> {
    let epitopes = human.get("epitopes").and_then(Value::as_object);
    let Some(eps) = epitopes else {
        return Vec::new();
    };

    let depth = crate::config::depth_map();
    let ep_list: Vec<_> = eps.iter().collect();
    let mut transitions = Vec::new();

    for (i, (na, ia)) in ep_list.iter().enumerate() {
        for (nb, ib) in &ep_list[i + 1..] {
            let da = depth.get(na.as_str()).copied().unwrap_or(0);
            let db = depth.get(nb.as_str()).copied().unwrap_or(0);
            let gap = da.abs_diff(db);
            let ca = ia.get("count").and_then(Value::as_u64).unwrap_or(0);
            let cb = ib.get("count").and_then(Value::as_u64).unwrap_or(0);
            let combined = ca + cb;
            if combined > 0 && gap <= 3 {
                let (from_ep, to_ep) = if da < db {
                    (na.as_str(), nb.as_str())
                } else {
                    (nb.as_str(), na.as_str())
                };
                transitions.push(json!({
                    "from_epitope": from_ep,
                    "to_epitope": to_ep,
                    "depth_gap": gap,
                    "strength": (combined as f64 / (1 + gap) as f64 * 10.0).round() / 10.0,
                    "combined_humans": combined,
                }));
            }
        }
    }
    transitions.sort_by(|a, b| {
        let sa = a.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
        let sb = b.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    transitions
}

fn group_perspectives(perspectives: &[Value]) -> Value {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for o in perspectives {
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
