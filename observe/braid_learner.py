#!/usr/bin/env python3
"""Squirrel braid learner.

Watches chipmunk's passage data and human epitopes over time.
Builds a braid graph: which paths do humans follow between sites/categories?
Learns which connections strengthen, which fade, which are emerging.

Writes braid.json — the living connection map.
Accumulates learning in /opt/membrane/squirrel-journal/braids/

The braid is three strands twisted:
  1. Category passages (INVESTIGATION → TOPOLOGY)
  2. Site orbits (git → lansing → sporeprint)
  3. Epitope transitions (visitor → reader → investigator)

Each strand reinforces the others. The twist IS the learning.
"""

import json
import math
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.request import Request, urlopen

PI = math.pi

FEEDS = {
    "chipmunk": "https://hud.primals.eco/chipmunk.json",
    "human": "https://hud.primals.eco/human.json",
    "dashboard": "https://hud.primals.eco/dashboard.json",
}

OUTPUT = Path("/opt/membrane/live-terminal/braid.json")
JOURNAL_DIR = Path("/opt/membrane/squirrel-journal/braids")
HISTORY_FILE = Path("/opt/membrane/squirrel-journal/braid-history.jsonl")

# Category → sites mapping (which sites belong to which epitope category)
CATEGORY_SITES = {
    "INVESTIGATION": ["detroit", "epitope", "antibody", "antigen", "evidence"],
    "TOPOLOGY": ["lansing", "git", "relay", "beacon", "layer2"],
    "SCIENCE": ["sporeprint", "hypothesis"],
    "MEMBRANE": ["hud", "signal", "macrophage"],
    "IMMUNE": ["tuebor", "clutch"],
    "OTHER": [],
}

# Reverse: site → category
SITE_CATEGORY = {}
for cat, sites in CATEGORY_SITES.items():
    for s in sites:
        SITE_CATEGORY[s] = cat


def fetch(url):
    try:
        req = Request(url, headers={"User-Agent": "squirrel-braid/0.1"})
        with urlopen(req, timeout=10) as r:
            return json.loads(r.read().decode())
    except Exception as e:
        return {"error": str(e)}


def load_history():
    """Load previous braid observations for trend detection."""
    if not HISTORY_FILE.exists():
        return []
    entries = []
    for line in HISTORY_FILE.read_text().splitlines():
        if line.strip():
            try:
                entries.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    return entries[-48:]  # Keep last 48 observations (4 hours at 5min intervals)


def compute_braids(chipmunk, human, dashboard):
    now = datetime.now(timezone.utc)
    braids = []
    observations = []  # flat list kept for backward compat
    perspectives = []  # structured: {perspective, text}

    def obs(perspective, text):
        observations.append(text)
        perspectives.append({"perspective": perspective, "text": text})

    # ═══ STRAND 1: Category passages from chipmunk ═══
    presence_obs = None
    passages = []
    epitope_tense = {}

    for mem in chipmunk.get("membrane", []):
        md = mem.get("metadata", {})
        if "passages" in md:
            passages = md["passages"]
            epitope_tense = md.get("epitope_tense", {})
            presence_obs = md.get("presence", {})

    # Build passage graph
    passage_graph = {}
    for p in passages:
        key = (p["from"], p["to"])
        passage_graph[key] = p["humans"]

    # Compute braid strength for each category pair
    categories = set()
    for p in passages:
        categories.add(p["from"])
        categories.add(p["to"])

    category_pairs = set()
    for p in passages:
        pair = tuple(sorted([p["from"], p["to"]]))
        category_pairs.add(pair)

    for pair in category_pairs:
        a, b = pair
        forward = passage_graph.get((a, b), 0)
        reverse = passage_graph.get((b, a), 0)
        total = forward + reverse

        # Tense weighting: IS connections are π× stronger than WAS
        a_is = epitope_tense.get(a, {}).get("is", 0)
        b_is = epitope_tense.get(b, {}).get("is", 0)
        a_was = epitope_tense.get(a, {}).get("was", 0)
        b_was = epitope_tense.get(b, {}).get("was", 0)

        # Vitality: ratio of IS to (IS + WAS) — how alive is this connection
        is_total = a_is + b_is
        was_total = a_was + b_was
        vitality = is_total / max(1, is_total + was_total)

        # Strength = passage count × vitality boost
        strength = total * (1 + vitality * PI)

        # Direction
        if forward > 0 and reverse > 0:
            ratio = max(forward, reverse) / min(forward, reverse)
            if ratio > 3:
                direction = "dominant"
                dominant_dir = a + " → " + b if forward > reverse else b + " → " + a
            else:
                direction = "bidirectional"
                dominant_dir = a + " ↔ " + b
        elif forward > 0:
            direction = "one-way"
            dominant_dir = a + " → " + b
        else:
            direction = "one-way"
            dominant_dir = b + " → " + a

        # Sites involved in this braid
        sites_a = CATEGORY_SITES.get(a, [])
        sites_b = CATEGORY_SITES.get(b, [])

        braids.append({
            "from": a,
            "to": b,
            "strength": round(strength, 2),
            "humans_forward": forward,
            "humans_reverse": reverse,
            "humans_total": total,
            "direction": direction,
            "dominant_flow": dominant_dir,
            "vitality": round(vitality, 3),
            "sites_from": sites_a[:3],
            "sites_to": sites_b[:3],
        })

    # Sort by strength
    braids.sort(key=lambda b: b["strength"], reverse=True)

    # ═══ STRAND 2: Site orbits from human.json ═══
    site_braids = []
    host_dist = human.get("host_distribution", {})
    top_pages = human.get("top_pages", [])

    # Find which sites are co-visited (proxy from host distribution)
    active_sites = [s for s, c in host_dist.items() if c > 10]
    for i, site_a in enumerate(active_sites):
        for site_b in active_sites[i+1:]:
            cat_a = SITE_CATEGORY.get(site_a, "OTHER")
            cat_b = SITE_CATEGORY.get(site_b, "OTHER")
            # Sites in same category = internal braid, different = cross-braid
            cross = cat_a != cat_b
            # Co-visitation strength from traffic overlap
            traffic_a = host_dist.get(site_a, 0)
            traffic_b = host_dist.get(site_b, 0)
            co_strength = min(traffic_a, traffic_b) / max(1, max(traffic_a, traffic_b))

            if co_strength > 0.01:
                site_braids.append({
                    "site_a": site_a,
                    "site_b": site_b,
                    "co_strength": round(co_strength, 3),
                    "cross_category": cross,
                    "category_a": cat_a,
                    "category_b": cat_b,
                })

    site_braids.sort(key=lambda b: b["co_strength"], reverse=True)

    # ═══ STRAND 3: Epitope depth transitions ═══
    epitopes = human.get("epitopes", {})
    # Ordered by depth: visitor → reader → coder → investigator → explorer → scholar
    DEPTH_ORDER = ["visitor", "observer", "witness", "reader", "coder", "investigator", "explorer", "scholar"]
    depth_map = {e: i for i, e in enumerate(DEPTH_ORDER)}

    epitope_transitions = []
    ep_list = list(epitopes.items())
    for i, (name_a, info_a) in enumerate(ep_list):
        for name_b, info_b in ep_list[i+1:]:
            da = depth_map.get(name_a, 0)
            db = depth_map.get(name_b, 0)
            depth_gap = abs(da - db)
            # Transition strength: nearby epitopes with traffic = strong braid
            combined = info_a["count"] + info_b["count"]
            if combined > 0 and depth_gap <= 3:
                transition_strength = combined / (1 + depth_gap)
                epitope_transitions.append({
                    "from_epitope": name_a if da < db else name_b,
                    "to_epitope": name_b if da < db else name_a,
                    "depth_gap": depth_gap,
                    "strength": round(transition_strength, 1),
                    "combined_humans": combined,
                })

    epitope_transitions.sort(key=lambda t: t["strength"], reverse=True)

    # ═══ LEARNING: detect patterns and trends ═══
    history = load_history()

    # Compare with last observation
    if history:
        last = history[-1]
        last_braids = {(b["from"], b["to"]): b for b in last.get("braids", [])}

        for braid in braids:
            key = (braid["from"], braid["to"])
            prev = last_braids.get(key)
            if prev:
                delta = braid["strength"] - prev.get("strength", 0)
                if delta > 0:
                    braid["trend"] = "growing"
                elif delta < 0:
                    braid["trend"] = "fading"
                else:
                    braid["trend"] = "stable"
                braid["delta"] = round(delta, 2)
            else:
                braid["trend"] = "emerging"
                braid["delta"] = braid["strength"]
                obs("after",
                    f"New braid emerging: {braid['dominant_flow']} "
                    f"(strength {braid['strength']:.1f})"
                )

        # Check for braids that disappeared
        for key, prev_braid in last_braids.items():
            current_keys = {(b["from"], b["to"]) for b in braids}
            if key not in current_keys and prev_braid.get("strength", 0) > 5:
                obs("before",
                    f"Braid faded: {prev_braid['dominant_flow']} "
                    f"(was strength {prev_braid['strength']:.1f})"
                )
    else:
        for braid in braids:
            braid["trend"] = "initial"
            braid["delta"] = 0
        obs("below", "First braid observation — establishing baseline")

    # Density: how interconnected is the graph?
    possible_pairs = len(categories) * (len(categories) - 1) / 2
    actual_pairs = len(braids)
    density = actual_pairs / max(1, possible_pairs)

    # Strongest braid observation
    if braids:
        strongest = braids[0]
        obs("on",
            f"Strongest braid: {strongest['dominant_flow']} "
            f"({strongest['humans_total']} humans, "
            f"vitality {strongest['vitality']:.1%}, "
            f"trend: {strongest.get('trend', 'unknown')})"
        )

    # Shifter density
    total_humans = sum(
        epitope_tense.get(c, {}).get("is", 0) for c in categories
    )
    shifters = chipmunk.get("membrane", [{}])
    shifter_count = 0
    for mem in chipmunk.get("membrane", []):
        md = mem.get("metadata", {})
        if "shifters" in md:
            shifter_count = md["shifters"]

    if total_humans > 0:
        shift_rate = shifter_count / max(1, total_humans)
        obs("beside",
            f"Shift rate: {shift_rate:.1%} of humans cross category boundaries "
            f"({shifter_count} shifters / {total_humans} present)"
        )

    # Multi-host exploration from human.json
    multi = human.get("output_signals", {}).get("multi_host_explorers", 0)
    avg_surf = human.get("output_signals", {}).get("avg_surfaces_explored", 0)
    if multi > 0:
        obs("inside",
            f"Exploration: {multi} humans crossing {avg_surf:.1f} surfaces on average"
        )

    # Energy balance
    energy = human.get("energy", {})
    sn = energy.get("signal_to_noise", 0)
    if sn > 100:
        obs("above", f"S/N ratio {sn:.0f} — human signal dominates fleet noise")

    # THROUGH — braids ARE the passages, the holes in the membrane
    # Each braid is a topological feature — a controlled opening where things pass
    total_passages = sum(b["humans_total"] for b in braids)
    if total_passages > 0:
        obs("through", f"{total_passages} humans passed through {len(braids)} membrane passages this cycle")

    # Bidirectional braids = open pores (pass both ways, like a tube)
    # One-way braids = valves (directed flow, like a heart valve)
    bidir = [b for b in braids if b["direction"] == "bidirectional"]
    oneway = [b for b in braids if b["direction"] in ("dominant", "one-way")]
    if bidir:
        obs("through", f"{len(bidir)} open pores (bidirectional) — passage in both directions")
    if oneway:
        obs("through", f"{len(oneway)} valves (one-way) — directed flow only")

    # Epitope transitions = transformations during passage
    # You enter as a visitor, you exit as an investigator — the passage changes you
    if epitope_transitions:
        deepest = max(epitope_transitions, key=lambda t: t.get("depth_gap", 0))
        if deepest.get("depth_gap", 0) > 1:
            obs("through",
                f"Deepest transformation: {deepest['from_epitope']} → {deepest['to_epitope']} "
                f"(gap {deepest['depth_gap']}) — passage through {deepest['depth_gap']} π layers")

    # Cross-category site braids = holes connecting different organs
    cross = [sb for sb in site_braids if sb.get("cross_category")]
    if cross:
        obs("through", f"{len(cross)} cross-organ passages — holes connecting different membrane regions")

    # Build output
    result = {
        "timestamp": now.isoformat(),
        "wave": human.get("wave", 0),
        "braids": braids,
        "site_braids": site_braids[:10],
        "epitope_transitions": epitope_transitions[:6],
        "graph": {
            "categories": sorted(categories),
            "density": round(density, 3),
            "total_passages": sum(b["humans_total"] for b in braids),
            "shifters": shifter_count,
            "active_routes": len(braids),
        },
        "learning": {
            "observations": observations,
            "perspectives": perspectives,
            "by_perspective": _group_perspectives(perspectives),
            "observation_count": len(history) + 1,
            "history_depth_hours": round(len(history) * 5 / 60, 1),
        },
    }

    return result


def _group_perspectives(perspectives):
    groups = {}
    for o in perspectives:
        p = o["perspective"]
        if p not in groups:
            groups[p] = []
        groups[p].append(o["text"])
    return groups


def save_history(braid_data):
    """Append to history for trend detection."""
    HISTORY_FILE.parent.mkdir(parents=True, exist_ok=True)
    entry = {
        "ts": braid_data["timestamp"],
        "braids": [
            {"from": b["from"], "to": b["to"], "strength": b["strength"]}
            for b in braid_data["braids"]
        ],
        "density": braid_data["graph"]["density"],
        "shifters": braid_data["graph"]["shifters"],
    }
    with open(HISTORY_FILE, "a") as f:
        f.write(json.dumps(entry) + "\n")

    # Also write daily journal
    JOURNAL_DIR.mkdir(parents=True, exist_ok=True)
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    journal_file = JOURNAL_DIR / f"{today}.jsonl"
    journal_entry = {
        "ts": braid_data["timestamp"],
        "type": "braid_observation",
        "observations": braid_data["learning"]["observations"],
        "perspectives": braid_data["learning"].get("by_perspective", {}),
        "strongest": braid_data["braids"][0] if braid_data["braids"] else None,
        "density": braid_data["graph"]["density"],
        "routes": braid_data["graph"]["active_routes"],
    }
    with open(journal_file, "a") as f:
        f.write(json.dumps(journal_entry) + "\n")


def main():
    chipmunk = fetch(FEEDS["chipmunk"])
    human = fetch(FEEDS["human"])
    dashboard = fetch(FEEDS["dashboard"])

    if "error" in chipmunk or "error" in human:
        print(f"Feed error: chipmunk={chipmunk.get('error')}, human={human.get('error')}", file=sys.stderr)
        return 1

    braid_data = compute_braids(chipmunk, human, dashboard)

    # Write output
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(braid_data, indent=2))

    # Save history
    save_history(braid_data)

    # Print summary
    ARROWS = {
        "on": "⊙", "inside": "⊕", "above": "↑", "below": "↓",
        "beside": "↔", "before": "←", "after": "→", "through": "⊘",
    }
    persp = braid_data["learning"].get("perspectives", [])
    print(f"Braid observation #{braid_data['learning']['observation_count']}")
    print(f"  Routes: {braid_data['graph']['active_routes']}")
    print(f"  Density: {braid_data['graph']['density']:.1%}")
    print(f"  Shifters: {braid_data['graph']['shifters']}")
    for o in persp:
        arrow = ARROWS.get(o["perspective"], "·")
        print(f"  {arrow} [{o['perspective']:6s}] {o['text']}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
