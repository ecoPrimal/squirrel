#!/usr/bin/env python3
"""squirrel-watch: membrane observation daemon.

Squirrel begins watching. Slowly. Learning.

Polls the membrane's negentropy and dashboard feeds, observes human visitors,
tracks patterns over time, and writes observations to a local journal.

The journal is squirrel's memory — what it saw, when, and what changed.
"""

import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

FEEDS = {
    "negentropy": "https://signal.primals.eco/negentropy.json",
    "dashboard": "https://hud.primals.eco/dashboard.json",
}

JOURNAL_DIR = Path(__file__).parent / "journal"
JOURNAL_DIR.mkdir(exist_ok=True)

POLL_INTERVAL = 300  # 5 minutes — breathe with the membrane


def fetch_feed(url):
    """Fetch a JSON feed. No dependencies beyond stdlib."""
    import urllib.request
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "squirrel-watch/0.1"})
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode())
    except Exception as e:
        return {"error": str(e)}


def extract_human_signal(dashboard):
    """Extract human-focused observations from the dashboard."""
    if "error" in dashboard:
        return {"error": dashboard["error"]}

    humans = dashboard.get("human_ips", 0)
    genuine = dashboard.get("collision_level2", {}).get("genuine", 0)
    dwell = dashboard.get("membrane_delta", {}).get("dwell", {})
    trajectory = dashboard.get("trajectory", {})
    anderson = dashboard.get("anderson_selectivity", {})

    human_mode = None
    for m in anderson.get("modes", []):
        if m.get("class") == "human":
            human_mode = m
            break

    orbits = []
    for orb in trajectory.get("top_orbits", [])[:8]:
        orbits.append({
            "path": " → ".join(h.replace(".primals.eco", "") for h in orb.get("hosts", [])),
            "visitors": orb.get("ips", 0),
        })

    cross_hosts = {}
    for ch in trajectory.get("top_cross_hosts", []):
        host = ch.get("host", "").replace(".primals.eco", "")
        cross_hosts[host] = ch.get("cross_visits", 0)

    return {
        "unique_ips": humans,
        "genuine": genuine,
        "selectivity": human_mode.get("p_observed", 0) if human_mode else 0,
        "dwell_median_min": dwell.get("median_min", 0),
        "multi_single_ratio": dwell.get("multi_single_ratio", 0),
        "multi_host_visitors": trajectory.get("multi_host_ips", 0),
        "avg_breadth": trajectory.get("avg_breadth", 0),
        "orbits": orbits,
        "cross_hosts": cross_hosts,
    }


def extract_membrane_state(negentropy):
    """Extract membrane breathing state."""
    if "error" in negentropy:
        return {"error": negentropy["error"]}

    return {
        "opacity": negentropy.get("membrane", {}).get("opacity", 0),
        "selectivity": negentropy.get("membrane", {}).get("selectivity", 0),
        "reflection": negentropy.get("membrane", {}).get("reflection", 0),
        "tempo": negentropy.get("jitte", {}).get("tempo", 0),
        "perturbation": negentropy.get("jitte", {}).get("perturbation", 0),
        "bits_extracted": negentropy.get("extraction", {}).get("total_bits_extracted", 0),
        "fleet_requests": negentropy.get("fleet", {}).get("total_requests", 0),
        "fleet_entities": negentropy.get("fleet", {}).get("total_entities", 0),
        "real_content_served": negentropy.get("fleet", {}).get("real_content_served", 0),
        "wave": negentropy.get("wave", 0),
    }


def compute_delta(prev, curr, key):
    """Compute change between two observation snapshots."""
    if prev is None:
        return None
    p = prev.get(key, {})
    c = curr.get(key, {})
    delta = {}
    for k in c:
        if isinstance(c[k], (int, float)) and isinstance(p.get(k), (int, float)):
            d = c[k] - p[k]
            if d != 0:
                delta[k] = d
    return delta if delta else None


def observe_once(prev_observation=None):
    """Take one observation. Compare with previous. Write journal entry."""
    ts = datetime.now(timezone.utc)

    neg = fetch_feed(FEEDS["negentropy"])
    dash = fetch_feed(FEEDS["dashboard"])

    humans = extract_human_signal(dash)
    membrane = extract_membrane_state(neg)

    observation = {
        "timestamp": ts.isoformat(),
        "humans": humans,
        "membrane": membrane,
    }

    # Compute deltas from previous observation
    if prev_observation:
        h_delta = compute_delta(prev_observation, observation, "humans")
        m_delta = compute_delta(prev_observation, observation, "membrane")
        if h_delta:
            observation["human_delta"] = h_delta
        if m_delta:
            observation["membrane_delta"] = m_delta

    # Write journal entry
    journal_file = JOURNAL_DIR / f"{ts.strftime('%Y-%m-%d')}.jsonl"
    with open(journal_file, "a") as f:
        f.write(json.dumps(observation) + "\n")

    return observation


def summarize(obs):
    """Print a human-readable summary of the observation."""
    ts = obs["timestamp"][:19].replace("T", " ")
    h = obs.get("humans", {})
    m = obs.get("membrane", {})

    print(f"\n🐿️  [{ts}Z]")
    print(f"  humans: {h.get('unique_ips', '?')} IPs, {h.get('genuine', '?')} genuine, "
          f"selectivity {h.get('selectivity', 0):.0%}")
    print(f"  dwell: {h.get('dwell_median_min', 0):.0f}m median, "
          f"multi/single {h.get('multi_single_ratio', 0):.1f}")
    print(f"  journeys: {h.get('multi_host_visitors', 0)} multi-surface, "
          f"avg {h.get('avg_breadth', 0):.1f} sites")

    orbits = h.get("orbits", [])
    if orbits:
        top = orbits[0]
        print(f"  top orbit: {top['path']} ({top['visitors']} visitors)")

    print(f"  membrane: opacity {m.get('opacity', 0):.1%}, "
          f"selectivity {m.get('selectivity', 0):.0%}, "
          f"reflection {m.get('reflection', 0):.1%}")
    print(f"  tempo: {m.get('tempo', 0):.4f}, "
          f"bits: {m.get('bits_extracted', 0):,}")
    print(f"  fleet: {m.get('fleet_requests', 0):,} reqs, "
          f"{m.get('fleet_entities', 0):,} entities, "
          f"real content: {m.get('real_content_served', 0)}")

    hd = obs.get("human_delta")
    md = obs.get("membrane_delta")
    if hd or md:
        print("  Δ", end="")
        if hd:
            for k, v in sorted(hd.items()):
                sign = "+" if v > 0 else ""
                print(f"  {k}:{sign}{v}", end="")
        if md:
            for k, v in sorted(md.items()):
                sign = "+" if v > 0 else ""
                print(f"  {k}:{sign}{v}", end="")
        print()


def main():
    print("🐿️  squirrel-watch starting")
    print(f"  journal: {JOURNAL_DIR}")
    print(f"  poll interval: {POLL_INTERVAL}s")
    print(f"  feeds: {', '.join(FEEDS.keys())}")
    print()

    prev = None
    try:
        while True:
            obs = observe_once(prev)
            summarize(obs)
            prev = obs
            sys.stdout.flush()
            time.sleep(POLL_INTERVAL)
    except KeyboardInterrupt:
        print("\n🐿️  squirrel-watch stopping. Journal preserved.")


if __name__ == "__main__":
    main()
