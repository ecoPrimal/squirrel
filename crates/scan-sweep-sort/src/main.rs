//! scan-sweep-sort binary — run the full pipeline, tempo-relative.
//!
//! Replaces: scan_sweep_sort.py (2,744L)
//! Cron: */10 (single entry — everything else is tempo-relative)

use scan_sweep_sort::braid::braid_learn;
use scan_sweep_sort::config::{live_dir, CLASSIFICATION_URL, FEEDS, GRAPH_URL};
use scan_sweep_sort::fleet::{backscatter_clean, fleet_sort};
use scan_sweep_sort::pipeline::{evolve, fossilize, scan, sort_phase, sweep};
use scan_sweep_sort::twist::twist;
use scan_sweep_sort::util::{fetch, journal_append, write_json};
use scan_sweep_sort::worldclock::{genome_is_stale, genome_totality, worldclock};

use serde_json::{json, Value};
use std::time::Instant;

/// Perspective arrows for fossil output.
fn arrow(perspective: &str) -> &'static str {
    match perspective {
        "on" => "⊙",
        "inside" => "⊕",
        "above" => "↑",
        "below" => "↓",
        "beside" => "↔",
        "before" => "←",
        "after" => "→",
        "through" => "⊘",
        _ => "·",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();
    let t0 = Instant::now();

    // Worldclock — read the membrane's own tempo
    let tempo = worldclock();
    let tempo_label = tempo.get("tempo").and_then(|t| t.as_str()).unwrap_or("?");

    // Genome — refresh if stale (tempo-relative, not daily cron)
    let genome_refreshed = if genome_is_stale() {
        let _ = genome_totality();
        true
    } else {
        false
    };

    // TODO: housekeep() — conditional maintenance (port next)
    // TODO: tower_probe() — organ health + ops (port next)
    // TODO: mesh_observe() — mesh health, speed, commensals (port next)
    // TODO: chipmunk_ear() — dialogue void closer (port next)
    // TODO: natural_classify() — biosphere + landmark sort signals (port next)
    // TODO: billboard_braid() — hash billboard, append to braid (port next)
    // TODO: symbiosis_overlay() — mutualism overrides (port next)
    // TODO: sourdough_feed() — feed legacy feeds (port next)

    // Fetch all data sources (shared — no duplicate HTTP calls)
    let chipmunk = fetch(FEEDS.get("chipmunk").copied().unwrap_or(""));
    let human = fetch(FEEDS.get("human").copied().unwrap_or(""));
    let classification = fetch(CLASSIFICATION_URL);
    let graph = fetch(GRAPH_URL);

    // Fleet sort (local — reads antibodies.json)
    let fleet_data = match fleet_sort() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("  fleet: error — {e}");
            json!({"total_entities": 0, "sort_classes": {}, "profiles": {}, "entities": {}})
        }
    };

    // Backscatter — lick the commensals clean
    let backscatter = backscatter_clean(&fleet_data).unwrap_or(json!({}));

    // Braid learning (uses chipmunk + human we already fetched)
    let braid_data = match braid_learn(&chipmunk, &human) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("  braid: error — {e}");
            json!({"graph": {"active_routes": 0, "density": 0}})
        }
    };

    // 5-stage pipeline
    let scan_data = scan(&classification, &graph, &human, &chipmunk, &braid_data, &fleet_data);

    // Fleet cross-reference confidence boost (wave 190)
    // (mutability requires cloning scan_data — deferred to avoid complexity for now)

    let sweep_data = sweep(&scan_data);
    let sort_data = sort_phase(&scan_data, &sweep_data);
    let fossil_data = fossilize(&scan_data, &sweep_data, &sort_data);
    let evolve_data = evolve(&scan_data, &sweep_data, &sort_data, &fossil_data);
    let twist_data = twist(&scan_data, &sort_data);

    let result = json!({
        "timestamp": now.to_rfc3339(),
        "pipeline": "scan → sweep → sort → fossilize → evolve → twist",
        "worldclock": tempo,
        "scan": scan_data,
        "sweep": sweep_data,
        "sort": sort_data,
        "fossilize": fossil_data,
        "evolve": evolve_data,
        "twist": twist_data,
    });

    write_json(&live_dir().join("scan_sweep_sort.json"), &result)?;
    write_json(&live_dir().join("twist.json"), &twist_data)?;

    // Proton pulse decay
    proton_pulse_decay();

    // Journal entry
    let _ = journal_append(
        "scan-sweep-sort",
        &json!({
            "ts": now.to_rfc3339(), "type": "scan_sweep_sort",
            "entities": scan_data.get("entities_discovered"),
            "fleet": scan_data.get("fleet_entities"),
            "confidence": scan_data.get("avg_confidence"),
            "selectivity": sort_data.get("unified_sort").and_then(|u| u.get("selectivity")),
            "fossil_count": fossil_data.get("fossil_count"),
        }),
    );

    // TODO: write_now(), write_human_expects(), write_null_mirror() (port next)
    // TODO: write_dynamics(), write_landmarks(), write_topology_abstract() (port next)
    // TODO: write_geodesic_mesh(), write_talos(), write_engagement() (port next)

    // ── Print summary ──
    let elapsed = t0.elapsed().as_secs_f64();
    println!(
        "scan → sweep → sort → fossilize → evolve → twist ({elapsed:.1}s) tempo={tempo_label}"
    );
    if genome_refreshed {
        println!("  GENOME:    refreshed (was stale)");
    }

    let entities_disc = scan_data
        .get("entities_discovered")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let fleet_ent = scan_data
        .get("fleet_entities")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let edges_disc = scan_data
        .get("edges_discovered")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    println!("  SCAN:      {entities_disc} detroit + {fleet_ent} fleet entities, {edges_disc} edges");

    let braid_routes = braid_data
        .get("graph")
        .and_then(|g| g.get("active_routes"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let braid_density = braid_data
        .get("graph")
        .and_then(|g| g.get("density"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    println!("  BRAID:     {braid_routes} routes, density {:.1}%", braid_density * 100.0);

    if backscatter.get("cleaned").is_some() {
        let cleaned = backscatter
            .get("cleaned")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cleaning_pct = backscatter
            .get("cleaning_pct")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let licensed = backscatter
            .get("licensed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let hostile_pct = backscatter
            .get("hostile_pct")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        println!("  BACKSCATTER: {cleaned} cleaned ({cleaning_pct:.1}%), {licensed} licensed ({hostile_pct:.1}%)");
    }

    if let Some(us) = sort_data.get("unified_sort") {
        let s = us.get("self").and_then(|v| v.as_u64()).unwrap_or(0);
        let ns = us.get("non_self").and_then(|v| v.as_u64()).unwrap_or(0);
        let p = us.get("partial").and_then(|v| v.as_u64()).unwrap_or(0);
        let n = us.get("null").and_then(|v| v.as_u64()).unwrap_or(0);
        println!("  SORT:      self={s} non-self={ns} partial={p} null={n}");
    }

    let fossil_count = fossil_data
        .get("fossil_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let evolve_count = evolve_data
        .get("evolution_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    println!("  FOSSILIZE: {fossil_count} observations");
    println!("  EVOLVE:    {evolve_count} recommendations");

    // Twist summary
    if let Some(tri) = twist_data.get("triangle") {
        let p = tri.get("pink").and_then(Value::as_f64).unwrap_or(0.0);
        let g = tri.get("green").and_then(Value::as_f64).unwrap_or(0.0);
        let b = tri.get("blue").and_then(Value::as_f64).unwrap_or(0.0);
        let coh = tri.get("coherence").and_then(Value::as_f64).unwrap_or(0.0);
        println!(
            "  TWIST:     pink={p:.3} green={g:.3} blue={b:.3} coherence={coh:.3}"
        );
    }
    let gl = twist_data
        .get("green_line")
        .and_then(|g| g.get("value"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let bl = twist_data
        .get("brown_line")
        .and_then(|b| b.get("value"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let tr = twist_data
        .get("twist")
        .and_then(|t| t.get("rate"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let auto_count = twist_data
        .get("auto_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    println!(
        "           green_line={gl:.3} brown_line={bl:.3} twist_rate={tr:.3} auto={auto_count}"
    );

    if let Some(actions) = twist_data.get("auto_actions").and_then(Value::as_array) {
        for a in actions {
            let act = a.get("action").and_then(Value::as_str).unwrap_or("?");
            let reason = a.get("reason").and_then(Value::as_str).unwrap_or("");
            let prio = a.get("priority").and_then(Value::as_str).unwrap_or("?");
            println!("  ⊗ [{prio:>6}] {act}: {reason}");
        }
    }

    if let Some(observations) = fossil_data.get("observations").and_then(|v| v.as_array()) {
        for o in observations {
            let p = o.get("perspective").and_then(|v| v.as_str()).unwrap_or("?");
            let t = o.get("text").and_then(|v| v.as_str()).unwrap_or("");
            println!("  {} [{:>6}] {}", arrow(p), p, t);
        }
    }

    Ok(())
}

/// Decay the proton pulse perturbation by 0.95× each cycle.
fn proton_pulse_decay() {
    let pp_path = live_dir().join("proton-pulse.json");
    if !pp_path.exists() {
        return;
    }
    let Ok(text) = std::fs::read_to_string(&pp_path) else {
        return;
    };
    let Ok(mut pp): Result<serde_json::Value, _> = serde_json::from_str(&text) else {
        return;
    };

    if let Some(obj) = pp.as_object_mut() {
        let pert = obj
            .get("perturbation")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let decayed = if pert * 0.95 < 0.001 {
            0.0
        } else {
            (pert * 0.95 * 1_000_000.0).round() / 1_000_000.0
        };
        obj.insert("perturbation".into(), json!(decayed));
        obj.insert(
            "decayed_at".into(),
            json!(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        );
        let _ = write_json(&pp_path, &pp);
    }
}
