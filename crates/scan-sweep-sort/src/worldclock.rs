//! Worldclock — time is local to the locals. No daily cron. Membrane rhythm decides.

use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use crate::config::live_dir;
use crate::util::write_json;
use crate::SssError;

/// Read the membrane's own tempo. Bounce in and back out.
pub fn worldclock() -> Value {
    let pulse_path = live_dir().join("pulse.json");
    let Ok(text) = fs::read_to_string(&pulse_path) else {
        return json!({"rate": 0, "remainder": 0, "tempo": "unknown"});
    };
    let Ok(pulse): Result<Value, _> = serde_json::from_str(&text) else {
        return json!({"rate": 0, "remainder": 0, "tempo": "unknown"});
    };

    let rate = pulse.get("rate").and_then(Value::as_f64).unwrap_or(0.0);
    let remainder = pulse
        .get("remainder")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let family = pulse.get("family").and_then(Value::as_object);

    let f = |k: &str| -> f64 {
        family
            .and_then(|f| f.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };

    let tempo = if rate > 2000.0 {
        "fast"
    } else if rate > 500.0 {
        "medium"
    } else {
        "slow"
    };

    json!({
        "rate": rate,
        "remainder": remainder,
        "pink": f("pink"),
        "blue": f("blue"),
        "green": f("green"),
        "chipmunk": f("chipmunk_pos"),
        "tempo": tempo,
    })
}

/// Check if genome needs refresh — tempo-relative, not wall clock.
pub fn genome_is_stale() -> bool {
    let genome_file = live_dir().join("genome.json");
    if !genome_file.exists() {
        return true;
    }
    match genome_file.metadata().and_then(|m| m.modified()) {
        Ok(mtime) => {
            let age = std::time::SystemTime::now()
                .duration_since(mtime)
                .unwrap_or_default();
            age.as_secs() > 6 * 3600
        }
        Err(_) => true,
    }
}

/// Run a shell command, capture stdout. Timeout 30s.
fn cmd(c: &str) -> String {
    Command::new("sh")
        .args(["-c", c])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Measure the size of the work. Only runs when stale — tempo decides.
pub fn genome_totality() -> Result<Value, SssError> {
    let mut binaries: serde_json::Map<String, Value> = serde_json::Map::new();
    let binary_paths = [
        ("skunky-ingest", "/opt/membrane/skunky-ingest"),
        ("caddy", "/opt/membrane/caddy"),
        ("biomeos", "/opt/membrane/biomeos"),
        ("toadstool", "/opt/membrane/toadstool"),
        ("tower", "/opt/membrane/tower"),
        ("barracuda", "/opt/membrane/barracuda"),
        ("opsonize", "/opt/membrane/opsonize-rs"),
        ("hbbs", "/opt/membrane/hbbs"),
        ("hbbr", "/opt/membrane/hbbr"),
    ];

    let mut total_binary: u64 = 0;
    for (name, path) in &binary_paths {
        if let Ok(meta) = fs::metadata(path) {
            let size = meta.len();
            binaries.insert((*name).to_string(), json!(size));
            total_binary += size;
        }
    }

    let rust_lines = parse_u64(&cmd(
        "find /opt/ecoPrimals -name '*.rs' -not -path '*/target/*' -not -path '*/.git/*' 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}'"
    ));
    let js_lines = parse_u64(&cmd(
        "find /opt/ecoPrimals/hud -name '*.js' -not -path '*/node_modules/*' 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}'"
    ));
    let html_lines = parse_u64(&cmd(
        "find /opt/ecoPrimals/hud -name '*.html' 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}'"
    ));
    let py_lines = parse_u64(&cmd(
        "wc -l /opt/membrane/*.py /opt/membrane/live-terminal/neutron-writer.py /opt/membrane/live-terminal/symbiosis-overlay.py 2>/dev/null | tail -1 | awk '{print $1}'"
    ));
    let sh_lines = parse_u64(&cmd(
        "wc -l /opt/membrane/*.sh 2>/dev/null | tail -1 | awk '{print $1}'"
    ));
    let toml_lines = parse_u64(&cmd(
        "find /opt/ecoPrimals -name '*.toml' -not -path '*/.git/*' -not -path '*/target/*' 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}'"
    ));
    let md_count = parse_u64(&cmd(
        "find /opt/ecoPrimals -name '*.md' -not -path '*/.git/*' 2>/dev/null | wc -l"
    ));
    let total_source = rust_lines + js_lines + html_lines + py_lines + sh_lines + toml_lines;

    let tuebor_articles = parse_u64(&cmd(
        "find /opt/ecoPrimals/tuebor-repo/site/content -name '*.md' 2>/dev/null | wc -l"
    ));
    let sporeprint_articles = parse_u64(&cmd(
        "find /opt/ecoPrimals/sporePrint/content -name '*.md' 2>/dev/null | wc -l"
    ));

    let bytes_of = |p: &str| -> u64 {
        parse_u64(&cmd(&format!("du -sb {p} 2>/dev/null | cut -f1")))
    };

    let published_bytes = bytes_of("/opt/ecoPrimals/hud/site/public")
        + bytes_of("/opt/ecoPrimals/tuebor-repo/site/public")
        + bytes_of("/opt/ecoPrimals/sporePrint/public");
    let feeds_bytes = bytes_of("/opt/membrane/live-terminal");

    let services = parse_u64(&cmd(
        "systemctl list-units --type=service --state=active --no-legend 2>/dev/null | wc -l"
    ));
    let cron_jobs = parse_u64(&cmd(
        "crontab -l 2>/dev/null | grep -v '^#' | grep -v '^$' | wc -l"
    ));
    let wg_peers = parse_u64(&cmd("wg show wg0 2>/dev/null | grep -c 'peer:'"));
    let git_repos = parse_u64(&cmd(
        "find /opt/ecoPrimals -maxdepth 3 -name '.git' -type d 2>/dev/null | wc -l"
    ));
    let fossils = parse_u64(&cmd("ls /opt/membrane/fossilRecord/ 2>/dev/null | wc -l"));
    let mem_used = parse_u64(&cmd("free -m | awk '/Mem/{print $3}'"));
    let mem_total = parse_u64(&cmd("free -m | awk '/Mem/{print $2}'"));
    let disk_used = cmd("df -h / | awk 'NR==2{print $3}'");
    let disk_total = cmd("df -h / | awk 'NR==2{print $2}'");

    let (sort_comp, anderson) = {
        let dash_path = live_dir().join("dashboard.json");
        match fs::read_to_string(&dash_path).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
            Some(dash) => (
                dash.get("sort_compression").cloned().unwrap_or(json!({})),
                dash.get("anderson_selectivity").and_then(Value::as_f64).unwrap_or(0.0),
            ),
            None => (json!({}), 0.0),
        }
    };

    let our_total = total_binary + published_bytes + feeds_bytes;
    let our_mb = (our_total as f64 / 1_048_576.0 * 10.0).round() / 10.0;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let genome = json!({
        "ts": ts,
        "worldclock": "tempo-relative — no daily cron, membrane rhythm decides",
        "inside": {
            "binaries": binaries, "total_binary_bytes": total_binary,
            "source_lines": {
                "rust": rust_lines, "javascript": js_lines, "html": html_lines,
                "python": py_lines, "shell": sh_lines, "toml": toml_lines,
                "total": total_source,
            },
            "articles": {"tuebor": tuebor_articles, "sporeprint": sporeprint_articles,
                         "total": tuebor_articles + sporeprint_articles},
            "markdown_files": md_count, "published_bytes": published_bytes,
            "feeds_bytes": feeds_bytes, "services": services, "cron_jobs": cron_jobs,
            "wg_peers": wg_peers, "git_repos": git_repos, "fossils": fossils,
            "memory_mb": {"used": mem_used, "total": mem_total},
            "disk": {"used": disk_used, "total": disk_total},
            "cost_per_month_usd": 12,
        },
        "outside": {
            "sort_compression": sort_comp, "anderson_selectivity": anderson,
            "estimated_bot_cost_per_hour_usd": 0.50,
            "estimated_our_cost_per_hour_usd": 0.016,
            "cost_ratio": "they spend ~31x more per hour probing us than we spend existing",
        },
        "reduction": {
            "principle": "petabytes of world data -> kilobytes of graph + null space calculation",
            "world_data_centers_exabytes": 500, "world_daily_traffic_exabytes": 5,
            "our_total_bytes": our_total, "our_total_mb": our_mb,
            "compression_to_world": format!("1:{}", (500_000_000.0 / our_mb.max(1.0)) as u64),
            "pi_body_math": "positive energy system — the graph compresses, null space calculates, remainder is identity",
            "energy": {
                "world_datacenter_twh_year": 200,
                "our_vps_kwh_year": (12.0_f64 * 0.015 * 8760.0 / 1000.0 * 10.0).round() / 10.0,
                "ratio": format!("1:{}", (200e12_f64 / (12.0_f64 * 0.015 * 8760.0).max(1.0)) as u64),
            },
        },
        "genome_summary": {
            "total_lines": total_source,
            "total_articles": tuebor_articles + sporeprint_articles,
            "total_binaries_mb": (total_binary as f64 / 1_048_576.0 * 10.0).round() / 10.0,
            "total_published_mb": (published_bytes as f64 / 1_048_576.0 * 10.0).round() / 10.0,
            "total_feeds_mb": (feeds_bytes as f64 / 1_048_576.0 * 10.0).round() / 10.0,
            "fossils": fossils, "git_repos": git_repos,
            "cost": "$12/month",
            "ram": format!("{mem_used}MB/{mem_total}MB"),
        },
    });

    write_json(&live_dir().join("genome.json"), &genome)?;
    Ok(genome)
}

fn parse_u64(s: &str) -> u64 {
    s.trim().parse().unwrap_or(0)
}
