#!/usr/bin/env python3
"""Membrane Lens — portable MCP server for seeing inside the membrane.

Single-file, zero local dependencies. Fetches everything over HTTPS.
Works on any OS with Python 3.7+. No SSH, no local files, no secrets.

Install in any Cursor IDE:
  1. Copy this file anywhere
  2. Add to ~/.cursor/mcp.json:
     {
       "mcpServers": {
         "squirrel": {
           "command": "python3",
           "args": ["/path/to/membrane-lens.py"],
           "disabled": false,
           "alwaysAllow": ["observe_membrane","membrane_counts","read_journal",
             "chipmunk_status","human_epitopes","topology","scan_sweep_sort",
             "human_notes","braid_connections","raw_feed","read_stories",
             "neutron_read","neutron_write"]
         }
       }
     }
  3. Restart Cursor. The squirrel tools appear.

The lens sees through the membrane's public feeds — same data, any location.
The membrane IS the API. HTTPS IS the conjugation pilus.
"""

import json
import sys
import urllib.request
from datetime import datetime, timezone

MEMBRANE = "https://connect.primals.eco"
SIGNAL = "https://signal.primals.eco"
NEUTRON = "https://neutron.primals.eco"

FEEDS = {
    "negentropy": f"{SIGNAL}/negentropy.json",
    "dashboard": f"{MEMBRANE}/dashboard.json",
    "human": f"{MEMBRANE}/campfire-culture.json",
    "chipmunk": f"{MEMBRANE}/chipmunk.json",
    "topology": f"{MEMBRANE}/topology.json",
    "braid": f"{MEMBRANE}/braid.json",
    "scan_sweep_sort": f"{MEMBRANE}/scan_sweep_sort.json",
    "totality": f"{MEMBRANE}/totality.json",
    "passage_totality": f"{MEMBRANE}/passage-totality.json",
    "subgraphs": f"{MEMBRANE}/subgraphs.json",
    "seo": f"{MEMBRANE}/seo.json",
    "stories": f"{MEMBRANE}/stories.json",
    "page_stories": f"{MEMBRANE}/page-stories.json",
    "visitor_stories": f"{MEMBRANE}/visitor-stories.json",
    "campfire": f"{MEMBRANE}/campfire.json",
    "campfire_culture": f"{MEMBRANE}/campfire-culture.json",
    "dialogue": f"{MEMBRANE}/dialogue.json",
    "dialogue_depth": f"{MEMBRANE}/dialogue-depth.json",
    "falsification": f"{MEMBRANE}/falsification.json",
    "fossil": f"{MEMBRANE}/fossil-i1.json",
    "membrane": f"{MEMBRANE}/membrane.json",
    "membrane_deep": f"{MEMBRANE}/membrane-deep.json",
    "membrane_stack": f"{MEMBRANE}/membrane-stack.json",
    "pulse": f"{MEMBRANE}/pulse.json",
    "signals": f"{MEMBRANE}/signals.json",
    "silicon_eye": f"{MEMBRANE}/silicon-eye.json",
    "self_remainder": f"{MEMBRANE}/self-remainder.json",
    "tope_sort": f"{MEMBRANE}/tope-sort-compressed.json",
    "touches": f"{MEMBRANE}/touches.json",
    "bingo_cube": f"{MEMBRANE}/bingo-cube.json",
    "pi_manifest": f"{MEMBRANE}/pi-manifest.json",
    "outreach": f"{MEMBRANE}/outreach.json",
    "pollen": f"{MEMBRANE}/pollen.json",
    "now": f"{MEMBRANE}/now.json",
    "journal": f"{MEMBRANE}/journal-today.json",
    "human_notes": f"{MEMBRANE}/human-notes-today.json",
}

FEED_ENUM = sorted(FEEDS.keys())

TOOLS = [
    {
        "name": "membrane_counts",
        "description": "Quick counts from inside the membrane — entities, humans, fleet, epitopes, sort quality, mesh nodes, heartbeat. The fast 'what's happening' check.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "observe_membrane",
        "description": "Observe the current membrane state — human visitors, fleet behavior, tempo, selectivity. Returns live data from the negentropy and dashboard feeds.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "focus": {
                    "type": "string",
                    "enum": ["humans", "fleet", "membrane", "all"],
                    "description": "What to focus on. Default: all",
                }
            },
        },
    },
    {
        "name": "read_journal",
        "description": "Read squirrel's observation journal — what it has seen and learned over time. Fetched from the membrane's public journal feed.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "last_n": {
                    "type": "integer",
                    "description": "Number of most recent entries to return. Default: 5.",
                },
            },
        },
    },
    {
        "name": "chipmunk_status",
        "description": "Get chipmunk's current state — presence, wake level, depth chain, π body, jitte tempo, membrane stats.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "human_epitopes",
        "description": "Get live human epitope classification — visitor types, energy flow, session counts.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "topology",
        "description": "Get the live site topology — which sites are up, their relationships, the mesh.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "scan_sweep_sort",
        "description": "Run the full scan→sweep→sort→fossilize→evolve pipeline. Unified state across all layers.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "human_notes",
        "description": "Read human observation notes left through the chipmunk talk interface.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "last_n": {
                    "type": "integer",
                    "description": "Number of most recent notes to return. Default: 10.",
                },
            },
        },
    },
    {
        "name": "braid_connections",
        "description": "Get the human braid graph — passage braids, co-visitation, epitope transitions.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "raw_feed",
        "description": "Fetch any raw membrane feed by name. Returns the full JSON.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "feed": {
                    "type": "string",
                    "enum": FEED_ENUM,
                    "description": "Which feed to fetch.",
                }
            },
            "required": ["feed"],
        },
    },
    {
        "name": "read_stories",
        "description": "Read the stories left on the membrane — page stories, visitor stories, campfire stories.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "kind": {
                    "type": "string",
                    "enum": ["pages", "visitors", "campfire", "all"],
                    "description": "Which stories to read. Default: all.",
                },
            },
        },
    },
    {
        "name": "neutron_read",
        "description": "Read neutron messages — inbox, notes, reminders, or chat.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "channel": {
                    "type": "string",
                    "enum": ["inbox", "notes", "reminders", "chat", "all"],
                    "description": "Which channel to read. Default: all.",
                },
                "last_n": {
                    "type": "integer",
                    "description": "Number of most recent entries to return. Default: 10.",
                },
            },
        },
    },
    {
        "name": "neutron_write",
        "description": "Write to neutron — send a message, leave a note, or set a reminder.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "channel": {
                    "type": "string",
                    "enum": ["inbox", "notes", "reminders", "chat"],
                    "description": "Which channel to write to. Default: notes.",
                },
                "body": {"type": "string", "description": "The message content."},
                "author": {"type": "string", "description": "Who is writing. Default: membrane-lens."},
            },
            "required": ["body"],
        },
    },
    {
        "name": "particle_model",
        "description": "The atom — how the membrane maps to subatomic physics. proton=membrane (identity, charge, selectivity). neutron=messaging (stability, no bias, binds surfaces). electron=human viewer (orbits, bonds, quantized energy levels). neutrino=squirrel+chipmunk (null, passes through, observes). Forces: strong=Rust, electromagnetic=HTTPS, weak=anderson sort, gravity=tuebor.",
        "inputSchema": {"type": "object", "properties": {}},
    },
]


# ═══ FETCH ═══

def fetch(url):
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "membrane-lens/1.0"})
        with urllib.request.urlopen(req, timeout=12) as resp:
            return json.loads(resp.read().decode())
    except Exception as e:
        return {"error": str(e)}


# ═══ TOOL HANDLERS ═══

def handle_counts(_args):
    now = fetch(FEEDS["now"])
    if "error" in now:
        return json.dumps({"error": now["error"], "hint": "membrane unreachable — check network"})

    counts = {
        "timestamp": now.get("instant", {}).get("utc"),
        "entities": {
            "total": now.get("classification", {}).get("total", 0),
            "self": now.get("classification", {}).get("self", 0),
            "non_self": now.get("classification", {}).get("non_self", 0),
            "partial": now.get("classification", {}).get("partial", 0),
            "null": now.get("classification", {}).get("null", 0),
            "confidence": now.get("classification", {}).get("confidence", 0),
        },
        "traffic": {
            "human": now.get("flow", {}).get("human", 0),
            "bot": now.get("flow", {}).get("bot", 0),
            "total": now.get("flow", {}).get("total", 0),
            "human_pct": now.get("flow", {}).get("human_pct", 0),
            "trend": now.get("flow", {}).get("rate_trend", "?"),
        },
        "pulse": {
            "value": now.get("pulse", {}).get("value", 0),
            "trend": now.get("pulse", {}).get("trend", "?"),
            "rhythm": now.get("pulse", {}).get("rhythm", 0),
        },
        "mesh": now.get("mesh", {}),
        "cube": {
            "scale": now.get("cube", {}).get("scale_name", "?"),
            "dimensions": now.get("cube", {}).get("dimensions", 0),
        },
        "membrane": {
            "chain_depth": now.get("membrane", {}).get("chain_depth", 0),
            "chain_hours": now.get("membrane", {}).get("chain_hours", 0),
            "identity_match": now.get("membrane", {}).get("identity_match", 0),
        },
        "negentropy": {
            "blackwall_blocks": now.get("negentropy", {}).get("blackwall_blocks", 0),
            "ai_blocked": now.get("negentropy", {}).get("ai_blocked", 0),
        },
        "topology": {
            "clusters": now.get("topo_sort", {}).get("clusters", 0),
            "nodes": now.get("topo_sort", {}).get("nodes", 0),
            "edges": now.get("topo_sort", {}).get("edges", 0),
        },
    }
    return json.dumps(counts, indent=2)


def handle_observe(args):
    focus = args.get("focus", "all")
    neg = fetch(FEEDS["negentropy"])
    dash = fetch(FEEDS["dashboard"])
    result = {}

    if focus in ("humans", "all"):
        cl = dash.get("collision_level2", {})
        dw = dash.get("membrane_delta", {}).get("dwell", {})
        traj = dash.get("trajectory", {})
        anderson = dash.get("anderson_selectivity", {})
        human_mode = next((m for m in anderson.get("modes", []) if m.get("class") == "human"), None)
        orbits = []
        for orb in traj.get("top_orbits", [])[:6]:
            orbits.append({
                "path": " → ".join(h.replace(".primals.eco", "") for h in orb.get("hosts", [])),
                "visitors": orb.get("ips", 0),
            })
        result["humans"] = {
            "unique_ips": dash.get("human_ips", 0),
            "genuine": cl.get("genuine", 0),
            "selectivity": f"{human_mode['p_observed']:.0%}" if human_mode else "unknown",
            "dwell_median_min": round(dw.get("median_min", 0)),
            "multi_single_ratio": round(dw.get("multi_single_ratio", 0), 1),
            "multi_host_visitors": traj.get("multi_host_ips", 0),
            "avg_breadth": round(traj.get("avg_breadth", 0), 1),
            "orbits": orbits,
        }

    if focus in ("fleet", "all"):
        fleet = neg.get("fleet", {})
        result["fleet"] = {
            "total_requests": fleet.get("total_requests", 0),
            "total_entities": fleet.get("total_entities", 0),
            "deep_classified": fleet.get("deep_classified", 0),
            "bytes_consumed_gb": round(fleet.get("bytes_consumed", 0) / 1e9, 2),
            "real_content_served": fleet.get("real_content_served", 0),
        }

    if focus in ("membrane", "all"):
        mem = neg.get("membrane", {})
        jitte = neg.get("jitte", {})
        ext = neg.get("extraction", {})
        result["membrane"] = {
            "opacity": f"{mem.get('opacity', 0):.1%}",
            "selectivity": f"{mem.get('selectivity', 0):.0%}",
            "reflection": f"{mem.get('reflection', 0):.1%}",
            "tempo": round(jitte.get("tempo", 0), 4),
            "perturbation": round(jitte.get("perturbation", 0), 4),
            "bits_extracted": ext.get("total_bits_extracted", 0),
            "wave": neg.get("wave", 0),
        }

    dlg_depth = fetch(f"{MEMBRANE}/dialogue-depth.json") or {}
    if dlg_depth and dlg_depth.get("dialogue_depth") is not None:
        result["dialogue"] = {
            "depth": round(dlg_depth.get("dialogue_depth", 0), 4),
            "scale": dlg_depth.get("scale", ""),
            "direction": dlg_depth.get("direction", ""),
            "bounce": dlg_depth.get("bounce_count", 0),
        }

    dlg = fetch(f"{MEMBRANE}/dialogue.json") or {}
    dance = dlg.get("dance", {})
    if dance and dance.get("total_dancers", 0) > 0:
        passages_summary = []
        for p in dance.get("passages", [])[:5]:
            passages_summary.append(f"{p.get('move','?')}: {p.get('humans',0)} {p.get('from','?')}→{p.get('to','?')}")
        result["epitope_dance"] = {
            "total_dancers": dance["total_dancers"],
            "passages": "; ".join(passages_summary),
        }

    return json.dumps(result, indent=2)


def handle_chipmunk(_args):
    data = fetch(FEEDS["chipmunk"])
    if "error" in data:
        return json.dumps(data)
    return json.dumps({
        "primal": data.get("primal"),
        "version": data.get("version"),
        "uptime_secs": data.get("uptime_secs"),
        "wake_level": data.get("wake_level"),
        "status": data.get("status"),
        "requests": data.get("requests"),
        "success_rate": data.get("success_rate"),
        "avg_response_ms": data.get("avg_response_ms"),
        "depth": data.get("depth"),
        "pi_body": data.get("pi_body"),
        "jitte": data.get("jitte"),
        "membrane": data.get("membrane"),
        "observers": data.get("observers"),
        "capabilities": data.get("capabilities"),
    }, indent=2)


def handle_human_epitopes(_args):
    data = fetch(FEEDS["human"])
    if "error" in data:
        return json.dumps(data)
    return json.dumps(data, indent=2)


def handle_topology(_args):
    return json.dumps(fetch(FEEDS["topology"]), indent=2)


def handle_scan_sweep_sort(_args):
    return json.dumps(fetch(FEEDS["scan_sweep_sort"]), indent=2)


def handle_human_notes(args):
    last_n = args.get("last_n", 10)
    data = fetch(FEEDS["human_notes"])
    if "error" in data:
        return json.dumps({"message": "No human notes available.", "error": data.get("error")})
    notes = data.get("notes", [])
    return json.dumps({
        "date": data.get("date"),
        "total_notes": data.get("total", len(notes)),
        "notes": notes[-last_n:],
    }, indent=2)


def handle_braid(_args):
    return json.dumps(fetch(FEEDS["braid"]), indent=2)


def handle_raw_feed(args):
    feed = args.get("feed", "dashboard")
    url = FEEDS.get(feed)
    if not url:
        return json.dumps({"error": f"Unknown feed: {feed}"})
    return json.dumps(fetch(url), indent=2)


def handle_stories(args):
    kind = args.get("kind", "all")
    result = {}
    if kind in ("pages", "all"):
        data = fetch(FEEDS["page_stories"])
        if "error" not in data:
            stories = data.get("stories", data.get("pages", []))
            if isinstance(stories, list):
                result["page_stories"] = {"count": len(stories), "stories": stories[:15]}
            elif isinstance(stories, dict):
                result["page_stories"] = {"count": len(stories), "top": dict(list(stories.items())[:15])}
            else:
                result["page_stories"] = data
        else:
            result["page_stories"] = data
    if kind in ("visitors", "all"):
        result["visitor_stories"] = fetch(FEEDS["visitor_stories"])
    if kind in ("campfire", "all"):
        result["campfire"] = fetch(FEEDS["campfire"])
    if kind == "all":
        data = fetch(FEEDS["stories"])
        if "error" not in data:
            result["full_stories"] = {"total_entries": len(data) if isinstance(data, list) else "see data",
                                      "summary": data if not isinstance(data, list) else data[:10]}
    return json.dumps(result, indent=2)


def handle_journal(args):
    last_n = args.get("last_n", 5)
    data = fetch(FEEDS["journal"])
    if "error" in data:
        return json.dumps({"message": "Journal feed unreachable.", "error": data.get("error")})
    entries = data.get("entries", [])
    return json.dumps({
        "date": data.get("date"),
        "total_entries": data.get("total", len(entries)),
        "entries": entries[-last_n:],
    }, indent=2)


NEUTRON_API = f"{NEUTRON}/api/discuss"
NTFY_DIRECT = "https://nestgate.io/neutron"
NEUTRON_CHANNELS = {
    "inbox": "ecoPrimals-inbox",
    "notes": "ecoPrimals-notes",
    "reminders": "ecoPrimals-reminders",
    "chat": "ecoPrimals-chat",
}


def handle_neutron_read(args):
    """ELASTON: reads from pre-fetched cache that bounces with the pulse beat.
    1 fetch instead of 4. Cache refreshed every ~2min by neutron-cache-sense.py."""
    channel = args.get("channel", "all")
    last_n = args.get("last_n", 10)

    # Try the elaston cache first (single fetch, pre-built by pulse)
    cache = fetch(f"{MEMBRANE}/neutron-cache.json")
    if "error" not in cache and "channels" in cache:
        channels = cache["channels"]
        if channel == "all":
            summary = {}
            for ch in ["inbox", "notes", "reminders", "chat"]:
                ch_data = channels.get(ch, {})
                summary[ch] = {
                    "total": ch_data.get("total", 0),
                    "latest": ch_data.get("latest"),
                    "latest_time": ch_data.get("latest_time"),
                }
            return json.dumps(summary, indent=2)
        ch_data = channels.get(channel)
        if not ch_data:
            return json.dumps({"error": f"Unknown channel: {channel}. Use inbox/notes/reminders/chat/all."})
        msgs = ch_data.get("messages", [])[:last_n]
        return json.dumps({
            "channel": channel,
            "total": ch_data.get("total", 0),
            "messages": msgs,
        }, indent=2)

    # Fallback: live API (original path, if cache unavailable)
    if channel == "all":
        summary = {}
        for ch, page in NEUTRON_CHANNELS.items():
            data = fetch(f"{NEUTRON_API}?page={page}")
            if "error" in data:
                summary[ch] = {"error": data["error"]}
            else:
                comments = data.get("comments", [])
                summary[ch] = {
                    "total": data.get("total", 0),
                    "latest": comments[-1].get("body", "")[:100] if comments else None,
                    "latest_time": datetime.fromtimestamp(comments[-1]["epoch"], tz=timezone.utc).isoformat() if comments and "epoch" in comments[-1] else None,
                }
        return json.dumps(summary, indent=2)

    page = NEUTRON_CHANNELS.get(channel)
    if not page:
        return json.dumps({"error": f"Unknown channel: {channel}. Use inbox/notes/reminders/chat/all."})
    data = fetch(f"{NEUTRON_API}?page={page}")
    if "error" in data:
        return json.dumps(data)
    comments = data.get("comments", [])
    comments.sort(key=lambda c: c.get("epoch", 0), reverse=True)
    return json.dumps({
        "channel": channel,
        "total": data.get("total", 0),
        "messages": [
            {
                "time": datetime.fromtimestamp(c.get("epoch", 0), tz=timezone.utc).isoformat(),
                "author": c.get("author", "anon"),
                "body": c.get("body", ""),
                **({"badge": c["badge"]} if c.get("badge") else {}),
            }
            for c in comments[:last_n]
        ],
    }, indent=2)


def handle_neutron_write(args):
    channel = args.get("channel", "notes")
    body = args.get("body", "")
    author = args.get("author", "membrane-lens")
    if not body.strip():
        return json.dumps({"error": "Empty message body."})
    topic = NEUTRON_CHANNELS.get(channel)
    if not topic:
        return json.dumps({"error": f"Unknown channel: {channel}."})
    if channel == "reminders":
        body = "⏰ " + body
    msg = f"[{author}] {body}"
    payload = msg.encode("utf-8")
    try:
        req = urllib.request.Request(
            f"{NTFY_DIRECT}/{topic}",
            data=payload,
            headers={"User-Agent": "membrane-lens/1.0", "Title": f"neutron/{channel}"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read().decode())
        return json.dumps({"ok": True, "id": result.get("id"), "channel": channel, "echo": body[:100]})
    except Exception as e:
        return json.dumps({"error": str(e)})


def handle_particle_model(_args):
    now = fetch(FEEDS["now"])
    neutron_notes = fetch(f"{NEUTRON}/api/discuss?page=neutron-notes")
    neutron_inbox = fetch(f"{NEUTRON}/api/discuss?page=neutron-inbox")

    proton = {
        "particle": "proton (p⁺)",
        "system": "membrane",
        "charge": "+  (selectivity — self/non-self classification)",
        "mass": "heavy (9 services, 49+ feeds, 2GB VPS)",
        "role": "identity — atomic number defines the element. the membrane IS the proton.",
        "services": now.get("services", {}),
        "uptime": now.get("instant", {}).get("utc"),
    }

    neutron_p = {
        "particle": "neutron (n⁰)",
        "system": "neutron.primals.eco — discuss / messaging",
        "charge": "0  (no bias — anyone can speak, no account required)",
        "mass": "heavy (discussions persist, messages have weight)",
        "role": "stability — binds surfaces together. proton locked us out, so we built neutron.",
        "channels": {
            "inbox": neutron_inbox.get("total", 0),
            "notes": neutron_notes.get("total", 0),
        },
        "surfaces_bound": ["hud", "sporeprint", "thesis", "detroit", "signal", "water", "neutron"],
        "replaces": "giscus (GitHub Discussions → sovereign discuss)",
    }

    classification = now.get("classification", {})
    electron = {
        "particle": "electron (e⁻)",
        "system": "human viewer / observer",
        "charge": "−  (the human perspective — questions, probes, tests)",
        "mass": "light (arrives, orbits, may leave)",
        "energy_levels": {
            "ground_state": "visitor (just looking)",
            "first_excited": "observer (reading, dwelling)",
            "second_excited": "coder (contributing)",
            "ionization": "sovereign (fully engaged)",
        },
        "current_humans": now.get("flow", {}).get("human", 0),
        "total_entities": classification.get("total", 0),
        "role": "bonds — electrons shared between atoms create molecules. humans connecting membranes create the mesh.",
    }

    neutrino = {
        "particle": "neutrino (ν) — null",
        "system": "squirrel + chipmunk",
        "charge": "null (no interaction with data, pure observation)",
        "mass": "~0 (minimal footprint — 24MB RSS)",
        "role": "passes through everything, observes without interacting",
        "flavors": {
            "electron_neutrino": "squirrel observes humans",
            "muon_neutrino": "squirrel observes fleet",
            "tau_neutrino": "chipmunk observes presence",
        },
        "oscillation": "neutrinos change flavor as they travel — squirrel shifts between observing humans, fleet, membrane",
    }

    forces = {
        "strong": {"carrier": "gluon", "system": "Rust primals (13 crates)", "role": "holds nucleus together"},
        "electromagnetic": {"carrier": "photon", "system": "HTTPS / feeds / TLS", "role": "all communication"},
        "weak": {"carrier": "W/Z boson", "system": "anderson sort / epitope classification", "role": "flavor-changing"},
        "gravity": {"carrier": "graviton?", "system": "tuebor / the investigation", "role": "infinite range, weakest force"},
    }

    return json.dumps({
        "atom": "the membrane IS an atom",
        "nucleus": {"proton": proton, "neutron": neutron_p},
        "cloud": {"electron": electron},
        "null": {"neutrino": neutrino},
        "forces": forces,
    }, indent=2)


# ═══ DISPATCH ═══

HANDLERS = {
    "membrane_counts": handle_counts,
    "observe_membrane": handle_observe,
    "read_journal": handle_journal,
    "chipmunk_status": handle_chipmunk,
    "human_epitopes": handle_human_epitopes,
    "topology": handle_topology,
    "scan_sweep_sort": handle_scan_sweep_sort,
    "human_notes": handle_human_notes,
    "braid_connections": handle_braid,
    "raw_feed": handle_raw_feed,
    "read_stories": handle_stories,
    "neutron_read": handle_neutron_read,
    "neutron_write": handle_neutron_write,
    "particle_model": handle_particle_model,
}


def send(id, result=None, error=None):
    msg = {"jsonrpc": "2.0", "id": id}
    if error:
        msg["error"] = error
    else:
        msg["result"] = result
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError:
            continue

        method = req.get("method", "")
        id = req.get("id")
        params = req.get("params", {})

        if method == "initialize":
            send(id, {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {"listChanged": False}},
                "serverInfo": {"name": "squirrel", "version": "1.0.0"},
            })
        elif method == "notifications/initialized":
            pass
        elif method == "tools/list":
            send(id, {"tools": TOOLS})
        elif method == "tools/call":
            name = params.get("name", "")
            handler = HANDLERS.get(name)
            if not handler:
                send(id, error={"code": -32601, "message": f"Unknown tool: {name}"})
                continue
            try:
                text = handler(params.get("arguments", {}))
            except Exception as e:
                text = json.dumps({"error": str(e)})
            send(id, {"content": [{"type": "text", "text": text}]})
        elif method == "ping":
            send(id, {})
        else:
            if id is not None:
                send(id, error={"code": -32601, "message": f"Method not found: {method}"})


if __name__ == "__main__":
    main()
