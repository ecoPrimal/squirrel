// SPDX-License-Identifier: AGPL-3.0-or-later
// Membrane Lens — portable MCP server (Rust)
//
// Single binary, compiles to ~2MB. Fetches everything over HTTPS.
// Works on any OS. No SSH, no local files, no secrets.
//
// Install in any Cursor IDE:
//   1. cargo build --release (or download pre-built binary)
//   2. Add to ~/.cursor/mcp.json:
//      {
//        "mcpServers": {
//          "squirrel": {
//            "command": "/path/to/membrane-lens",
//            "disabled": false,
//            "alwaysAllow": ["membrane_counts","observe_membrane","read_journal",
//              "chipmunk_status","human_epitopes","topology","scan_sweep_sort",
//              "human_notes","braid_connections","raw_feed","read_stories",
//              "neutron_read","neutron_write"]
//          }
//        }
//      }
//   3. Restart Cursor.
//
// The feeds ARE the genetics. The lens is just the pore shape.
// Python for LLM classes. JavaScript for browsers. Rust for the native membrane.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const MEMBRANE: &str = "https://hud.primals.eco";
const SIGNAL: &str = "https://signal.primals.eco";
const NEUTRON: &str = "https://neutron.primals.eco";

fn fetch(url: &str) -> Value {
    match ureq::get(url)
        .set("User-Agent", "membrane-lens/1.0")
        .timeout(std::time::Duration::from_secs(12))
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(body) => serde_json::from_str(&body).unwrap_or(json!({"error": "invalid JSON"})),
            Err(e) => json!({"error": e.to_string()}),
        },
        Err(e) => json!({"error": e.to_string()}),
    }
}

fn tools_list() -> Value {
    json!([
        {"name":"membrane_counts","description":"Quick counts from inside the membrane — entities, humans, fleet, epitopes, sort quality, mesh nodes, heartbeat.","inputSchema":{"type":"object","properties":{}}},
        {"name":"observe_membrane","description":"Observe the current membrane state — human visitors, fleet behavior, tempo, selectivity.","inputSchema":{"type":"object","properties":{"focus":{"type":"string","enum":["humans","fleet","membrane","all"],"description":"Default: all"}}}},
        {"name":"read_journal","description":"Read squirrel's observation journal from the membrane's public feed.","inputSchema":{"type":"object","properties":{"last_n":{"type":"integer","description":"Entries to return. Default: 5."}}}},
        {"name":"chipmunk_status","description":"Get chipmunk's current state — wake level, depth chain, membrane stats.","inputSchema":{"type":"object","properties":{}}},
        {"name":"human_epitopes","description":"Get live human epitope classification.","inputSchema":{"type":"object","properties":{}}},
        {"name":"topology","description":"Get the live site topology.","inputSchema":{"type":"object","properties":{}}},
        {"name":"scan_sweep_sort","description":"Run the full scan→sweep→sort→fossilize→evolve pipeline.","inputSchema":{"type":"object","properties":{}}},
        {"name":"human_notes","description":"Read human observation notes.","inputSchema":{"type":"object","properties":{"last_n":{"type":"integer","description":"Notes to return. Default: 10."}}}},
        {"name":"braid_connections","description":"Get the human braid graph.","inputSchema":{"type":"object","properties":{}}},
        {"name":"raw_feed","description":"Fetch any raw membrane feed by name.","inputSchema":{"type":"object","properties":{"feed":{"type":"string","description":"Feed name."}},"required":["feed"]}},
        {"name":"read_stories","description":"Read stories left on the membrane.","inputSchema":{"type":"object","properties":{"kind":{"type":"string","enum":["pages","visitors","campfire","all"],"description":"Default: all."}}}},
        {"name":"neutron_read","description":"Read neutron messages.","inputSchema":{"type":"object","properties":{"channel":{"type":"string","enum":["inbox","notes","reminders","chat","all"],"description":"Default: all."},"last_n":{"type":"integer","description":"Default: 10."}}}},
        {"name":"neutron_write","description":"Write to neutron.","inputSchema":{"type":"object","properties":{"channel":{"type":"string","enum":["inbox","notes","reminders","chat"],"description":"Default: notes."},"body":{"type":"string","description":"Message content."},"author":{"type":"string","description":"Default: membrane-lens."}},"required":["body"]}},
        {"name":"particle_model","description":"The atom — proton=membrane, neutron=messaging, electron=human, neutrino=squirrel+chipmunk.","inputSchema":{"type":"object","properties":{}}}
    ])
}

fn feed_url(name: &str) -> Option<String> {
    let url = match name {
        "negentropy" => format!("{SIGNAL}/negentropy.json"),
        "now" => format!("{MEMBRANE}/now.json"),
        "journal" => format!("{MEMBRANE}/journal-today.json"),
        "human_notes" => format!("{MEMBRANE}/human-notes-today.json"),
        "dashboard" | "human" | "chipmunk" | "topology" | "braid" | "scan_sweep_sort"
        | "totality" | "subgraphs" | "seo" | "stories" | "page_stories"
        | "visitor_stories" | "campfire" | "dialogue" | "dialogue_depth"
        | "falsification" | "fossil" | "membrane" | "membrane_deep" | "membrane_stack"
        | "pulse" | "signals" | "silicon_eye" | "self_remainder" | "tope_sort"
        | "touches" | "bingo_cube" | "pi_manifest" | "outreach" | "pollen" => {
            let kebab = name.replace('_', "-");
            format!("{MEMBRANE}/{kebab}.json")
        }
        _ => return None,
    };
    Some(url)
}

fn handle_counts() -> String {
    let now = fetch(&format!("{MEMBRANE}/now.json"));
    if now.get("error").is_some() {
        return serde_json::to_string_pretty(&json!({"error": now["error"], "hint": "membrane unreachable"})).unwrap();
    }
    serde_json::to_string_pretty(&json!({
        "timestamp": now["instant"]["utc"],
        "entities": {
            "total": now["classification"]["total"],
            "self": now["classification"]["self"],
            "non_self": now["classification"]["non_self"],
            "partial": now["classification"]["partial"],
            "null": now["classification"]["null"],
            "confidence": now["classification"]["confidence"],
        },
        "traffic": {
            "human": now["flow"]["human"],
            "bot": now["flow"]["bot"],
            "total": now["flow"]["total"],
            "human_pct": now["flow"]["human_pct"],
            "trend": now["flow"]["rate_trend"],
        },
        "pulse": {
            "value": now["pulse"]["value"],
            "trend": now["pulse"]["trend"],
            "rhythm": now["pulse"]["rhythm"],
        },
        "mesh": now["mesh"],
        "cube": {"scale": now["cube"]["scale_name"], "dimensions": now["cube"]["dimensions"]},
        "membrane": {
            "chain_depth": now["membrane"]["chain_depth"],
            "chain_hours": now["membrane"]["chain_hours"],
            "identity_match": now["membrane"]["identity_match"],
        },
        "negentropy": {
            "blackwall_blocks": now["negentropy"]["blackwall_blocks"],
            "ai_blocked": now["negentropy"]["ai_blocked"],
        },
        "topology": {
            "clusters": now["topo_sort"]["clusters"],
            "nodes": now["topo_sort"]["nodes"],
            "edges": now["topo_sort"]["edges"],
        },
    }))
    .unwrap()
}

fn handle_journal(args: &Value) -> String {
    let n = args.get("last_n").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    let data = fetch(&format!("{MEMBRANE}/journal-today.json"));
    if data.get("error").is_some() {
        return serde_json::to_string_pretty(&json!({"message": "Journal unreachable.", "error": data["error"]})).unwrap();
    }
    let entries = data["entries"].as_array().cloned().unwrap_or_default();
    let start = entries.len().saturating_sub(n);
    serde_json::to_string_pretty(&json!({
        "date": data["date"],
        "total_entries": data["total"],
        "entries": &entries[start..],
    }))
    .unwrap()
}

fn handle_notes(args: &Value) -> String {
    let n = args.get("last_n").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let data = fetch(&format!("{MEMBRANE}/human-notes-today.json"));
    if data.get("error").is_some() {
        return serde_json::to_string_pretty(&json!({"message": "No notes available.", "error": data["error"]})).unwrap();
    }
    let notes = data["notes"].as_array().cloned().unwrap_or_default();
    let start = notes.len().saturating_sub(n);
    serde_json::to_string_pretty(&json!({
        "date": data["date"],
        "total_notes": data["total"],
        "notes": &notes[start..],
    }))
    .unwrap()
}

fn handle_tool(name: &str, args: &Value) -> String {
    match name {
        "membrane_counts" => handle_counts(),
        "read_journal" => handle_journal(args),
        "human_notes" => handle_notes(args),
        "chipmunk_status" | "human_epitopes" | "topology" | "scan_sweep_sort"
        | "braid_connections" => {
            let url = feed_url(name).unwrap_or_default();
            serde_json::to_string_pretty(&fetch(&url)).unwrap()
        }
        "observe_membrane" => {
            // Compact: fetch now.json for the observe view
            let now = fetch(&format!("{MEMBRANE}/now.json"));
            serde_json::to_string_pretty(&now).unwrap()
        }
        "raw_feed" => {
            let feed = args.get("feed").and_then(|v| v.as_str()).unwrap_or("dashboard");
            match feed_url(feed) {
                Some(url) => serde_json::to_string_pretty(&fetch(&url)).unwrap(),
                None => serde_json::to_string(&json!({"error": format!("Unknown feed: {feed}")})).unwrap(),
            }
        }
        "read_stories" => {
            let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("all");
            let mut result = json!({});
            if kind == "pages" || kind == "all" {
                result["page_stories"] = fetch(&format!("{MEMBRANE}/page-stories.json"));
            }
            if kind == "visitors" || kind == "all" {
                result["visitor_stories"] = fetch(&format!("{MEMBRANE}/visitor-stories.json"));
            }
            if kind == "campfire" || kind == "all" {
                result["campfire"] = fetch(&format!("{MEMBRANE}/campfire.json"));
            }
            serde_json::to_string_pretty(&result).unwrap()
        }
        "neutron_read" => {
            // ELASTON: read from pre-fetched cache (bounces with the pulse beat)
            // instead of making 4 live API calls that hit rate limits.
            // Cache refreshed every ~2min by neutron-cache-sense.py in membrane-pulse.
            let ch = args.get("channel").and_then(|v| v.as_str()).unwrap_or("all");
            let last_n = args.get("last_n").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let cache = fetch(&format!("{MEMBRANE}/neutron-cache.json"));
            if cache.get("error").is_some() {
                // Cache miss — fall back to live API (1 call for single channel)
                let page = match ch {
                    "inbox" => "neutron-inbox", "notes" => "neutron-notes",
                    "reminders" => "neutron-reminders", "chat" => "neutron-chat",
                    "all" => {
                        return serde_json::to_string(&json!({"error": "neutron cache unavailable, try a specific channel"})).unwrap();
                    }
                    _ => return serde_json::to_string(&json!({"error": "Unknown channel"})).unwrap(),
                };
                let d = fetch(&format!("{NEUTRON}/api/discuss?page={page}"));
                let comments = d["comments"].as_array().cloned().unwrap_or_default();
                let msgs: Vec<Value> = comments.iter().rev().take(last_n).map(|c| json!({
                    "time": c["ts"], "author": c.get("badge").unwrap_or(&json!("anon")),
                    "body": c["body"], "badge": c.get("badge").unwrap_or(&json!("")),
                })).collect();
                return serde_json::to_string_pretty(&json!({
                    "channel": ch, "total": d["total"], "messages": msgs,
                })).unwrap();
            }
            let channels = &cache["channels"];
            match ch {
                "all" => {
                    let mut summary = json!({});
                    for k in ["inbox", "notes", "reminders", "chat"] {
                        let ch_data = &channels[k];
                        summary[k] = json!({
                            "total": ch_data["total"],
                            "latest": ch_data["latest"],
                            "latest_time": ch_data["latest_time"],
                        });
                    }
                    serde_json::to_string_pretty(&summary).unwrap()
                }
                _ => {
                    let ch_data = &channels[ch];
                    let msgs = ch_data["messages"].as_array().cloned().unwrap_or_default();
                    let limited: Vec<Value> = msgs.into_iter().take(last_n).collect();
                    serde_json::to_string_pretty(&json!({
                        "channel": ch,
                        "total": ch_data["total"],
                        "messages": limited,
                    })).unwrap()
                }
            }
        }
        "neutron_write" => {
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            if body.trim().is_empty() {
                return serde_json::to_string(&json!({"error": "Empty message body."})).unwrap();
            }
            let ch = args.get("channel").and_then(|v| v.as_str()).unwrap_or("notes");
            let page = match ch {
                "inbox" => "neutron-inbox", "notes" => "neutron-notes",
                "reminders" => "neutron-reminders", "chat" => "neutron-chat",
                _ => return serde_json::to_string(&json!({"error": "Unknown channel"})).unwrap(),
            };
            let author = args.get("author").and_then(|v| v.as_str()).unwrap_or("membrane-lens");
            let payload = json!({"page": page, "author": author, "body": body});
            match ureq::post(&format!("{NEUTRON}/api/discuss"))
                .set("Content-Type", "application/json")
                .set("User-Agent", "membrane-lens/1.0")
                .timeout(std::time::Duration::from_secs(10))
                .send_string(&payload.to_string())
            {
                Ok(resp) => {
                    let r: Value = resp.into_string().ok()
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or(json!({}));
                    serde_json::to_string(&json!({"ok": r["ok"], "channel": ch, "echo": &body[..body.len().min(100)]})).unwrap()
                }
                Err(e) => serde_json::to_string(&json!({"error": e.to_string()})).unwrap(),
            }
        }
        "particle_model" => {
            let now = fetch(&format!("{MEMBRANE}/now.json"));
            let notes = fetch(&format!("{NEUTRON}/api/discuss?page=neutron-notes"));
            let inbox = fetch(&format!("{NEUTRON}/api/discuss?page=neutron-inbox"));
            serde_json::to_string_pretty(&json!({
                "atom": "the membrane IS an atom",
                "nucleus": {
                    "proton": {
                        "particle": "proton (p⁺)", "system": "membrane",
                        "charge": "+  (selectivity — self/non-self classification)",
                        "role": "identity — the membrane IS the proton",
                        "services": now.get("services").cloned().unwrap_or(json!({})),
                    },
                    "neutron": {
                        "particle": "neutron (n⁰)", "system": "neutron.primals.eco",
                        "charge": "0  (no bias — anyone can speak)",
                        "role": "stability — binds surfaces together",
                        "channels": {
                            "inbox": inbox.get("total").and_then(|v| v.as_i64()).unwrap_or(0),
                            "notes": notes.get("total").and_then(|v| v.as_i64()).unwrap_or(0),
                        },
                        "surfaces_bound": ["hud","sporeprint","thesis","detroit","signal","water","neutron"],
                        "replaces": "giscus",
                    },
                },
                "cloud": {
                    "electron": {
                        "particle": "electron (e⁻)", "system": "human viewer",
                        "charge": "−  (questions, probes, tests)",
                        "current_humans": now.get("flow").and_then(|f| f.get("human")).cloned().unwrap_or(json!(0)),
                        "energy_levels": ["visitor","observer","coder","sovereign"],
                    },
                },
                "null": {
                    "neutrino": {
                        "particle": "neutrino (ν)", "system": "squirrel + chipmunk",
                        "charge": "null", "mass": "~0",
                        "role": "passes through everything, observes without interacting",
                    },
                },
                "forces": {
                    "strong": {"carrier":"gluon","system":"Rust primals"},
                    "electromagnetic": {"carrier":"photon","system":"HTTPS/TLS"},
                    "weak": {"carrier":"W/Z boson","system":"anderson sort"},
                    "gravity": {"carrier":"graviton?","system":"tuebor"},
                },
            })).unwrap()
        }
        _ => serde_json::to_string(&json!({"error": format!("Unknown tool: {name}")})).unwrap(),
    }
}

fn send(id: &Value, result: Option<Value>, error: Option<Value>) {
    let mut msg = json!({"jsonrpc": "2.0", "id": id});
    if let Some(e) = error {
        msg["error"] = e;
    } else if let Some(r) = result {
        msg["result"] = r;
    }
    let out = serde_json::to_string(&msg).unwrap();
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    let _ = writeln!(lock, "{out}");
    let _ = lock.flush();
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let line = line.trim().to_string();
        if line.is_empty() { continue; }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = req["method"].as_str().unwrap_or("");
        let id = &req["id"];
        let params = &req["params"];

        match method {
            "initialize" => send(id, Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "squirrel", "version": "1.0.0"},
            })), None),
            "notifications/initialized" => {}
            "tools/list" => send(id, Some(json!({"tools": tools_list()})), None),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];
                let text = handle_tool(name, args);
                send(id, Some(json!({"content": [{"type": "text", "text": text}]})), None);
            }
            "ping" => send(id, Some(json!({})), None),
            _ => {
                if !id.is_null() {
                    send(id, None, Some(json!({"code": -32601, "message": format!("Method not found: {method}")})));
                }
            }
        }
    }
}
