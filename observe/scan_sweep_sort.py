#!/usr/bin/env python3
"""Scan → Sweep → Sort → Fossilize → Evolve

Integrates all data sources into the membrane's 5-stage pipeline:

SCAN:   Discover what exists — entities, datasets, game models, traffic
SWEEP:  Compress, deduplicate, find structure — braids, patterns, overlaps
SORT:   Classify self/non-self/partial/null — Anderson localization
FOSSILIZE: Record what we learned — journal, history, fossil record
EVOLVE: Feed learning back — stronger braids, better classification

Data sources:
  - detroit investigation graph (341 nodes, 957 edges, 295 classified)
  - braid manifest datasets (127+ scientific datasets)
  - human epitopes (visitor patterns, energy flow)
  - chipmunk passages (category-to-category movement)
  - game science models (Fitts, game trees, flow theory)
  - membrane traffic (fleet/human/ghost classification)

Outputs scan_sweep_sort.json — unified pipeline state.
"""

import json
import math
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.request import Request, urlopen

PI = math.pi
OUTPUT = Path("/opt/membrane/live-terminal/scan_sweep_sort.json")
JOURNAL = Path("/opt/membrane/squirrel-journal/scan-sweep-sort")

FEEDS = {
    "chipmunk": "https://hud.primals.eco/chipmunk.json",
    "human": "https://hud.primals.eco/human.json",
    "braid": "https://hud.primals.eco/braid.json",
    "dashboard": "https://hud.primals.eco/dashboard.json",
}

# Classification from detroit — served by the site
CLASSIFICATION_URL = "https://detroit.primals.eco/classification.json"
GRAPH_URL = "https://detroit.primals.eco/graph.json"


def fetch(url):
    try:
        req = Request(url, headers={"User-Agent": "squirrel-scan/0.1"})
        with urlopen(req, timeout=15) as r:
            return json.loads(r.read().decode())
    except Exception as e:
        return {"_error": str(e)}


def scan(classification, graph, human, chipmunk, braid):
    """SCAN: discover what exists across all sources."""

    # Detroit entities
    entity_count = len(classification) if isinstance(classification, dict) else 0
    edge_count = len(graph.get("edges", graph.get("links", []))) if isinstance(graph, dict) else 0
    node_count = len(graph.get("nodes", [])) if isinstance(graph, dict) else 0

    # Sort classes
    sort_classes = {}
    methods = {}
    confidences = []
    for k, v in (classification.items() if isinstance(classification, dict) else []):
        cls = v.get("sort_class", "unknown")
        meth = v.get("method", "unknown")
        sort_classes[cls] = sort_classes.get(cls, 0) + 1
        methods[meth] = methods.get(meth, 0) + 1
        if v.get("confidence"):
            confidences.append(v["confidence"])

    # Edge types
    edge_types = {}
    for e in (graph.get("edges", graph.get("links", [])) if isinstance(graph, dict) else []):
        t = e.get("type", e.get("label", "unknown"))
        edge_types[t] = edge_types.get(t, 0) + 1

    # Human traffic scan
    human_sessions = 0
    epitope_types = {}
    if isinstance(human, dict) and human.get("epitopes"):
        human_sessions = human.get("energy", {}).get("total_sessions", 0)
        for name, info in human["epitopes"].items():
            epitope_types[name] = info.get("count", 0)

    # Membrane modes from chipmunk
    anderson_modes = {}
    if isinstance(chipmunk, dict):
        for mem in chipmunk.get("membrane", []):
            md = mem.get("metadata", {})
            if md.get("anderson"):
                for mode in md["anderson"].get("modes", []):
                    anderson_modes[mode["class"]] = mode["count"]

    # Braid routes
    braid_routes = 0
    braid_density = 0
    shifters = 0
    if isinstance(braid, dict) and braid.get("graph"):
        g = braid["graph"]
        braid_routes = g.get("active_routes", 0)
        braid_density = g.get("density", 0)
        shifters = g.get("shifters", 0)

    return {
        "entities_discovered": entity_count,
        "edges_discovered": edge_count,
        "nodes_discovered": node_count,
        "sort_classes": sort_classes,
        "classification_methods": len(methods),
        "method_breakdown": dict(sorted(methods.items(), key=lambda x: -x[1])[:10]),
        "avg_confidence": round(sum(confidences) / max(1, len(confidences)), 3) if confidences else 0,
        "edge_types": dict(sorted(edge_types.items(), key=lambda x: -x[1])[:10]),
        "human_sessions": human_sessions,
        "epitope_types": epitope_types,
        "anderson_modes": anderson_modes,
        "braid_routes": braid_routes,
        "braid_density": round(braid_density, 3),
        "shifters": shifters,
    }


def sweep(scan_data, classification, graph, human, braid):
    """SWEEP: compress, find structure, identify patterns."""

    patterns = []

    # Pattern 1: Classification coverage
    total = scan_data["entities_discovered"]
    classified = total - scan_data["sort_classes"].get("unknown", 0)
    coverage = classified / max(1, total)
    patterns.append({
        "name": "classification_coverage",
        "value": round(coverage, 3),
        "description": f"{classified}/{total} entities classified ({coverage*100:.0f}%)",
    })

    # Pattern 2: Sort confidence distribution
    if scan_data["avg_confidence"] > 0:
        patterns.append({
            "name": "confidence_mean",
            "value": scan_data["avg_confidence"],
            "description": f"average sort confidence {scan_data['avg_confidence']:.1%}",
        })

    # Pattern 3: Edge density — how connected is the graph
    if scan_data["nodes_discovered"] > 0:
        max_edges = scan_data["nodes_discovered"] * (scan_data["nodes_discovered"] - 1) / 2
        edge_density = scan_data["edges_discovered"] / max(1, max_edges)
        patterns.append({
            "name": "graph_density",
            "value": round(edge_density, 4),
            "description": f"{scan_data['edges_discovered']} edges across {scan_data['nodes_discovered']} nodes",
        })

    # Pattern 4: Human-to-entity ratio — how much human attention per entity
    if scan_data["human_sessions"] > 0 and total > 0:
        attention_ratio = scan_data["human_sessions"] / total
        patterns.append({
            "name": "human_attention_ratio",
            "value": round(attention_ratio, 2),
            "description": f"{scan_data['human_sessions']} human sessions across {total} entities",
        })

    # Pattern 5: Cascade efficiency — how many methods were needed
    patterns.append({
        "name": "cascade_methods",
        "value": scan_data["classification_methods"],
        "description": f"{scan_data['classification_methods']} distinct classification methods used",
    })

    # Pattern 6: Self/non-self ratio (π target)
    self_count = scan_data["sort_classes"].get("self", 0)
    nonself_count = scan_data["sort_classes"].get("non-self", 0)
    partial_count = scan_data["sort_classes"].get("partial", 0)
    null_count = scan_data["sort_classes"].get("null", 0)

    if total > 0:
        pi_ratio = classified / max(1, total)
        patterns.append({
            "name": "pi_sort_ratio",
            "value": round(pi_ratio, 3),
            "target": round(1 / PI, 3),
            "description": f"sort ratio {pi_ratio:.3f} (target 1/π = {1/PI:.3f})",
        })

    # Pattern 7: Braid-entity alignment — do braids connect sorted categories?
    if braid and isinstance(braid, dict) and braid.get("braids"):
        total_braid_humans = sum(b.get("humans_total", 0) for b in braid["braids"])
        patterns.append({
            "name": "braid_flow",
            "value": total_braid_humans,
            "description": f"{total_braid_humans} humans flowing through {scan_data['braid_routes']} braid routes",
        })

    # Compression: how much structure was found
    compression = len(patterns)

    return {
        "patterns_found": compression,
        "patterns": patterns,
        "compression_summary": {
            "sort_classes": 4,
            "edge_types_found": len(scan_data["edge_types"]),
            "epitope_types": len(scan_data["epitope_types"]),
            "anderson_modes": len(scan_data["anderson_modes"]),
        },
    }


def sort_phase(scan_data, sweep_data):
    """SORT: classify everything — self/non-self/partial/null."""

    sort_summary = {
        "detroit_entities": scan_data["sort_classes"],
        "membrane_traffic": scan_data["anderson_modes"],
        "human_epitopes": scan_data["epitope_types"],
    }

    # Overall self/non-self ratio across all domains
    total_self = (
        scan_data["sort_classes"].get("self", 0) +
        scan_data["anderson_modes"].get("human", 0) +
        scan_data["anderson_modes"].get("sovereign", 0)
    )
    total_nonself = (
        scan_data["sort_classes"].get("non-self", 0) +
        scan_data["anderson_modes"].get("chorus", 0) +
        scan_data["anderson_modes"].get("spectrum", 0)
    )
    total_partial = (
        scan_data["sort_classes"].get("partial", 0) +
        scan_data["anderson_modes"].get("ghost", 0)
    )
    total_null = scan_data["sort_classes"].get("null", 0)

    grand_total = total_self + total_nonself + total_partial + total_null
    selectivity = total_self / max(1, total_self + total_nonself) if grand_total > 0 else 0

    # Unsorted: what still needs classification
    unsorted_entities = scan_data["entities_discovered"] - sum(scan_data["sort_classes"].values())

    return {
        "summary": sort_summary,
        "unified_sort": {
            "self": total_self,
            "non_self": total_nonself,
            "partial": total_partial,
            "null": total_null,
            "total": grand_total,
            "selectivity": round(selectivity, 3),
        },
        "unsorted": unsorted_entities,
        "sort_quality": round(scan_data["avg_confidence"], 3),
    }


def fossilize(scan_data, sweep_data, sort_data):
    """FOSSILIZE: record what we learned.

    Squirrel observes from 8 perspectives relative to the membrane:

    Spatial (where):
      ON      — what's touching the surface right now (live sort, current traffic)
      INSIDE  — within the layers (entities partially sorted, in transit)
      ABOVE   — system view (patterns across all sites, the solar system)
      BELOW   — raw unsorted data (null body, unclassified)
      BESIDE  — lateral connections (braids, site co-visitation)

    Temporal (when):
      BEFORE  — what WAS (trends fading, history)
      AFTER   — what WILLBE (emerging patterns, predictions)

    Topological (how):
      THROUGH — what completes the passage (self that made it all the way through)
               Life makes holes. Ears, pores, ion channels, gut tubes.
               The membrane has integrity — passthrough doesn't break it.
               Self passes through. Non-self bounces off. The hole is the sort.
               Different from granular splitting (digging through dirt) —
               membrane passthrough preserves the boundary.
    """

    observations = []

    def obs(perspective, text):
        observations.append({"perspective": perspective, "text": text})

    # ON — what's touching the membrane surface right now
    sel = sort_data["unified_sort"]["selectivity"]
    obs("on", f"Selectivity {sel:.1%} — self vs non-self at the surface")
    for p in sweep_data["patterns"]:
        if p["name"] == "human_attention_ratio":
            obs("on", f"Human attention: {p['description']}")

    # INSIDE — what's passing through, partially sorted
    for p in sweep_data["patterns"]:
        if p["name"] == "classification_coverage":
            obs("inside", f"Entity coverage: {p['description']}")
        if p["name"] == "confidence_mean":
            obs("inside", f"Sort confidence: {p['description']}")
    u = sort_data["unified_sort"]
    if u["partial"] > 0:
        obs("inside", f"{u['partial']} entities partially sorted — still in transit through layers")

    # ABOVE — the system view, looking down
    for p in sweep_data["patterns"]:
        if p["name"] == "pi_sort_ratio":
            obs("above", f"π ratio: {p['description']}")
        if p["name"] == "graph_density":
            obs("above", f"Graph structure: {p['description']}")
    total = u["total"]
    if total > 0:
        obs("above", f"System total: {total:,} entities across detroit + membrane + human")

    # BELOW — what's still raw, unsorted, in the null body
    if u["null"] > 0:
        obs("below", f"{u['null']} entities at null — not yet reached by any classifier")
    if sort_data["unsorted"] > 0:
        obs("below", f"Unsorted gap: {sort_data['unsorted']} entities await classification")
    ghost = scan_data["anderson_modes"].get("ghost", 0)
    if ghost > 0:
        obs("below", f"{ghost} ghost entities — ambiguous, neither self nor non-self")

    # BESIDE — lateral, the braids and connections
    for p in sweep_data["patterns"]:
        if p["name"] == "braid_flow":
            obs("beside", f"Braid flow: {p['description']}")
        if p["name"] == "cascade_methods":
            obs("beside", f"Classification diversity: {p['description']}")
    if scan_data["shifters"] > 0:
        obs("beside", f"{scan_data['shifters']} shifters crossing category boundaries")

    # BEFORE — what was, trends from history
    if scan_data["avg_confidence"] < 0.85:
        obs("before", f"Confidence {scan_data['avg_confidence']:.1%} still below 85% target — was it higher?")
    if scan_data["braid_routes"] < 8:
        obs("before", f"Only {scan_data['braid_routes']} active braid routes — some may have faded")

    # THROUGH — what completed the passage (topological, not positional)
    # Self passes through. Non-self bounces. The holes ARE the sort.
    human_through = scan_data["anderson_modes"].get("human", 0)
    sovereign_through = scan_data["anderson_modes"].get("sovereign", 0)
    total_through = human_through + sovereign_through
    total_bounce = scan_data["anderson_modes"].get("chorus", 0) + scan_data["anderson_modes"].get("spectrum", 0)

    if total_through > 0:
        obs("through", f"{total_through:,} entities passed through the membrane (human + sovereign)")
    if total_bounce > 0:
        ratio = total_bounce / max(1, total_through)
        obs("through", f"{total_bounce:,} bounced off — {ratio:.0f}× more non-self than self at the boundary")

    # The passages themselves — braids are the holes, the topology
    if scan_data["braid_routes"] > 0:
        obs("through", f"{scan_data['braid_routes']} passage routes — these are the holes in the membrane")

    # Epitopes are passthrough types — each one is a different kind of hole
    ep_count = len(scan_data["epitope_types"])
    if ep_count > 0:
        obs("through", f"{ep_count} epitope types — each is a different pore shape. "
            f"visitors enter one shape, emerge as investigators through another")

    # Detroit classification passthrough — what made it through northGate's sort
    detroit_self = scan_data["sort_classes"].get("self", 0)
    detroit_partial = scan_data["sort_classes"].get("partial", 0)
    if detroit_self > 0:
        obs("through", f"{detroit_self} detroit entities passed through all 3 classification phases")
    if detroit_partial > 0:
        obs("through", f"{detroit_partial} detroit entities stuck in the membrane wall — partially through")

    # AFTER — what's coming, predictions
    humans = scan_data["human_sessions"]
    if humans > 50:
        obs("after", f"{humans} humans present — classification will improve with their sort votes")
    if u["partial"] > u["self"]:
        obs("after", f"Partial ({u['partial']}) > self ({u['self']}) — resolution wave expected")
    if scan_data["braid_density"] > 0.5:
        obs("after", f"Braid density {scan_data['braid_density']:.0%} — new cross-category routes likely")

    return {
        "observations": observations,
        "flat": [o["text"] for o in observations],
        "by_perspective": _group_perspectives(observations),
        "fossil_count": len(observations),
    }


def _group_perspectives(observations):
    groups = {}
    for o in observations:
        p = o["perspective"]
        if p not in groups:
            groups[p] = []
        groups[p].append(o["text"])
    return groups


def evolve(scan_data, sweep_data, sort_data, fossil_data):
    """EVOLVE: what should happen next — feed learning back."""

    recommendations = []

    # Low-confidence entities need re-sort
    if scan_data["avg_confidence"] < 0.85:
        recommendations.append({
            "action": "rescan_low_confidence",
            "reason": f"Average confidence {scan_data['avg_confidence']:.1%} below 85% target",
            "priority": "medium",
        })

    # Unsorted entities
    if sort_data["unsorted"] > 0:
        recommendations.append({
            "action": "classify_remaining",
            "reason": f"{sort_data['unsorted']} entities still unsorted",
            "priority": "high",
        })

    # Braid strengthening
    if scan_data["braid_routes"] < 8:
        recommendations.append({
            "action": "discover_braids",
            "reason": f"Only {scan_data['braid_routes']} active braid routes — cross-category exploration needed",
            "priority": "medium",
        })

    # Human participation
    if scan_data["human_sessions"] > 0:
        recommendations.append({
            "action": "human_sort_assist",
            "reason": f"{scan_data['human_sessions']} humans present — their observations can improve classification",
            "priority": "high",
        })

    # Game tree integration
    recommendations.append({
        "action": "game_tree_sort",
        "reason": "Game science models (Fitts law, game tree complexity) can model sort decision spaces",
        "priority": "future",
    })

    return {
        "recommendations": recommendations,
        "evolution_count": len(recommendations),
    }


def main():
    now = datetime.now(timezone.utc)

    # Fetch all data sources
    chipmunk = fetch(FEEDS["chipmunk"])
    human = fetch(FEEDS["human"])
    braid = fetch(FEEDS["braid"])
    classification = fetch(CLASSIFICATION_URL)
    graph = fetch(GRAPH_URL)

    # 5-stage pipeline
    scan_data = scan(classification, graph, human, chipmunk, braid)
    sweep_data = sweep(scan_data, classification, graph, human, braid)
    sort_data = sort_phase(scan_data, sweep_data)
    fossil_data = fossilize(scan_data, sweep_data, sort_data)
    evolve_data = evolve(scan_data, sweep_data, sort_data, fossil_data)

    result = {
        "timestamp": now.isoformat(),
        "pipeline": "scan → sweep → sort → fossilize → evolve",
        "scan": scan_data,
        "sweep": sweep_data,
        "sort": sort_data,
        "fossilize": fossil_data,
        "evolve": evolve_data,
    }

    # Write output
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(result, indent=2))

    # Journal
    JOURNAL.mkdir(parents=True, exist_ok=True)
    today = now.strftime("%Y-%m-%d")
    journal_file = JOURNAL / f"{today}.jsonl"
    journal_entry = {
        "ts": now.isoformat(),
        "type": "scan_sweep_sort",
        "entities": scan_data["entities_discovered"],
        "edges": scan_data["edges_discovered"],
        "confidence": scan_data["avg_confidence"],
        "selectivity": sort_data["unified_sort"]["selectivity"],
        "observations": fossil_data["observations"],
        "perspectives": fossil_data["by_perspective"],
    }
    with open(journal_file, "a") as f:
        f.write(json.dumps(journal_entry) + "\n")

    # Print summary
    ARROWS = {
        "on": "⊙", "inside": "⊕", "above": "↑", "below": "↓",
        "beside": "↔", "before": "←", "after": "→", "through": "⊘",
    }
    print("scan → sweep → sort → fossilize → evolve")
    print(f"  SCAN:      {scan_data['entities_discovered']} entities, {scan_data['edges_discovered']} edges, {scan_data['human_sessions']} humans")
    print(f"  SWEEP:     {sweep_data['patterns_found']} patterns found")
    print(f"  SORT:      self={sort_data['unified_sort']['self']} non-self={sort_data['unified_sort']['non_self']} partial={sort_data['unified_sort']['partial']} null={sort_data['unified_sort']['null']}")
    print(f"  FOSSILIZE: {fossil_data['fossil_count']} observations across {len(fossil_data['by_perspective'])} perspectives")
    print(f"  EVOLVE:    {evolve_data['evolution_count']} recommendations")
    for o in fossil_data["observations"]:
        arrow = ARROWS.get(o["perspective"], "·")
        print(f"  {arrow} [{o['perspective']:6s}] {o['text']}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
