#!/usr/bin/env python3
"""kite.py — chipmunk flies the human kite.

The chipmunk holds the rubber band. The Vitruvian man is the kite.
The five senses provide lift. The blink provides rhythm.
The flavor tells it what's nourishing and what's extracting.

READS:
  evolve-state.json  — evolutionary body plan (the shape of the kite)
  sideview.json      — distance/direction/disposition (the wind)
  trig-view.json     — angles, SOHCAHTOA (the tilt)
  homunculus-map.json — sensitivity map (the nerve strings)
  pressure.json      — tension building (tugging)
  jams.json          — blocked pathways (tangles)

BLINKS:
  Compares current read to previous read.
  Computes delta. The delta IS the observation.
  Adjusts blink rate based on what changed (Nyquist).

TASTES:
  Scores each feed on 5 flavors:
    sweet (nourishing), sour (extractive), spicy (provocative),
    bitter (toxic), umami (complex depth)

STEERS:
  Writes kite-state.json — the unified sensory state.
  Chipmunk reads this. Klein-io picks it up.
  The kite flies itself.

Usage:
  python3 kite.py                     # single blink
  python3 kite.py --fly               # continuous flight (loop)
  python3 kite.py --fly --rate 600    # custom blink rate (seconds)
"""

import json, math, sys, time, os
from pathlib import Path
from datetime import datetime, timezone

PI = math.pi

# ── FEEDS (local membrane files) ──
MEMBRANE_DIR = Path(os.environ.get(
    'MEMBRANE_DIR',
    str(Path(__file__).resolve().parents[3] / 'membrane' / 'local-terminal' / 'remote')
))

FEEDS = {
    'evolve':     MEMBRANE_DIR / 'evolve-state.json',
    'delta':      MEMBRANE_DIR / 'evolve-delta.json',
    'sideview':   MEMBRANE_DIR / 'sideview.json',
    'trig':       MEMBRANE_DIR / 'trig-view.json',
    'homunculus': MEMBRANE_DIR / 'homunculus-map.json',
    'pressure':   MEMBRANE_DIR / 'pressure.json',
    'jams':       MEMBRANE_DIR / 'jams.json',
}

KITE_STATE = MEMBRANE_DIR / 'kite-state.json'
KITE_LOG = MEMBRANE_DIR / 'kite-log.jsonl'
KITE_PREV = MEMBRANE_DIR / 'kite-prev.json'

DEFAULT_BLINK_RATE = 300  # 5 min — breathe with the membrane


# ═══════════════════════════════════════════════════════════════
# READ — open the eyes
# ═══════════════════════════════════════════════════════════════

def read_feed(path):
    """Read a JSON feed. Returns dict or None."""
    try:
        return json.loads(path.read_text())
    except Exception:
        return None

def read_all():
    """Read all sensory feeds. Returns dict of feed_name → data."""
    senses = {}
    for name, path in FEEDS.items():
        senses[name] = read_feed(path)
    return senses


# ═══════════════════════════════════════════════════════════════
# TASTE — score each feed on 5 flavors
# ═══════════════════════════════════════════════════════════════

def taste_evolve(data):
    """Taste the evolutionary state."""
    if not data:
        return {'sweet': 0, 'sour': 0, 'spicy': 0, 'bitter': 0, 'umami': 0}
    stages = data.get('stages', {})
    vert = stages.get('vertebrate', 0)
    jelly = stages.get('jellyfish', 0)
    spider = stages.get('spider', 0)
    total = data.get('total_files', 1)

    sweet = vert / max(total, 1) * 100        # vertebrate = nourishing structure
    sour = 0                                     # no extraction in the sort itself
    spicy = 1.0 if spider == 0 else 0            # the spider gap IS spicy (provocative)
    bitter = 0                                   # the sort doesn't produce toxin
    umami = (vert / max(total, 1)) * (jelly / max(total, 1)) * 100  # complexity = both existing
    return _norm(sweet, sour, spicy, bitter, umami)

def taste_homunculus(data):
    """Taste the nervous system state."""
    if not data:
        return {'sweet': 0, 'sour': 0, 'spicy': 0, 'bitter': 0, 'umami': 0}

    ratio = data.get('cns_pns_ratio', 0)
    one_over_ln2 = 1 / math.log(2)
    delta_from_shannon = abs(ratio - one_over_ln2)

    sweet = max(0, 1.0 - delta_from_shannon * 10)  # close to 1/ln(2) = sweet
    sour = 0
    spicy = 0
    bitter = 0
    umami = sweet * 0.8  # Shannon balance is umami (deep, complex)
    return _norm(sweet, sour, spicy, bitter, umami)

def taste_pressure(data):
    """Taste the pressure state."""
    if not data:
        return {'sweet': 0, 'sour': 0, 'spicy': 0, 'bitter': 0, 'umami': 0}
    points = data.get('pressure_points', [])
    n = len(points)

    sweet = max(0, 1.0 - n * 0.1)  # no pressure = sweet
    sour = min(1.0, n * 0.2)        # pressure = sour (extracting from the system)
    spicy = 0
    bitter = 0
    umami = 0
    return _norm(sweet, sour, spicy, bitter, umami)

def taste_jams(data):
    """Taste the jam state."""
    if not data:
        return {'sweet': 0, 'sour': 0, 'spicy': 0, 'bitter': 0, 'umami': 0}
    jams = data.get('jams', [])
    sinks = sum(1 for j in jams if j.get('type') == 'sink')
    sources = sum(1 for j in jams if j.get('type') == 'source')

    sweet = 0
    sour = min(1.0, sinks * 0.05)      # sinks taste sour
    spicy = 0
    bitter = min(1.0, sources * 0.1)    # orphan sources taste bitter
    umami = 0
    return _norm(sweet, sour, spicy, bitter, umami)

def taste_trig(data):
    """Taste the trigonometric state."""
    if not data:
        return {'sweet': 0, 'sour': 0, 'spicy': 0, 'bitter': 0, 'umami': 0}

    fold_ratio = data.get('fold_ratio', 0)
    balance = data.get('balance_pct', 0)

    sweet = balance / 100.0             # files at 45° = balanced = sweet
    sour = 0
    spicy = 1.0 if fold_ratio == 0 else 0  # empty fold = spicy (provocative gap)
    bitter = 0
    umami = min(1.0, balance / 20.0)    # balance is complex
    return _norm(sweet, sour, spicy, bitter, umami)

def _norm(sweet, sour, spicy, bitter, umami):
    """Normalize to [0,1] range."""
    total = sweet + sour + spicy + bitter + umami
    if total == 0:
        return {'sweet': 0, 'sour': 0, 'spicy': 0, 'bitter': 0, 'umami': 0}
    return {
        'sweet': round(sweet / total, 4),
        'sour': round(sour / total, 4),
        'spicy': round(spicy / total, 4),
        'bitter': round(bitter / total, 4),
        'umami': round(umami / total, 4),
    }

def taste_all(senses):
    """Taste all feeds. Return composite flavor profile."""
    tastes = {
        'evolve': taste_evolve(senses.get('evolve')),
        'homunculus': taste_homunculus(senses.get('homunculus')),
        'pressure': taste_pressure(senses.get('pressure')),
        'jams': taste_jams(senses.get('jams')),
        'trig': taste_trig(senses.get('trig')),
    }

    # Composite: average all tastes
    composite = {'sweet': 0, 'sour': 0, 'spicy': 0, 'bitter': 0, 'umami': 0}
    n = len(tastes)
    for t in tastes.values():
        for flavor in composite:
            composite[flavor] += t.get(flavor, 0)
    for flavor in composite:
        composite[flavor] = round(composite[flavor] / n, 4)

    return tastes, composite


# ═══════════════════════════════════════════════════════════════
# BLINK — compare to previous, compute delta
# ═══════════════════════════════════════════════════════════════

def blink(current, previous):
    """The blink: compare current state to previous. Return delta."""
    if previous is None:
        return {'drift': 'genesis', 'changes': []}

    changes = []

    # Check evolutionary drift
    ce = current.get('evolve') or {}
    pe = previous.get('senses', {}).get('evolve') or {}
    cs = ce.get('stages', {})
    ps = pe.get('stages', {})
    for stage in ['banana', 'flatworm', 'jellyfish', 'centipede', 'millipede', 'spider', 'vertebrate']:
        d = cs.get(stage, 0) - ps.get(stage, 0)
        if d != 0:
            changes.append(f"evolve.{stage}: {d:+d}")

    # Check pressure changes
    cp = current.get('pressure') or {}
    pp = previous.get('senses', {}).get('pressure') or {}
    cp_n = len(cp.get('pressure_points', []))
    pp_n = len(pp.get('pressure_points', []))
    if cp_n != pp_n:
        changes.append(f"pressure: {pp_n}→{cp_n}")

    # Check jam changes
    cj = current.get('jams') or {}
    pj = previous.get('senses', {}).get('jams') or {}
    cj_n = len(cj.get('jams', []))
    pj_n = len(pj.get('jams', []))
    if cj_n != pj_n:
        changes.append(f"jams: {pj_n}→{cj_n}")

    drift = 'evolved' if changes else 'stable'
    return {'drift': drift, 'changes': changes}


# ═══════════════════════════════════════════════════════════════
# STEER — adjust blink rate based on what we tasted
# ═══════════════════════════════════════════════════════════════

def steer(composite, delta, base_rate):
    """Adjust blink rate based on flavor and drift.

    Spicy or sour → blink faster (more attention needed).
    Sweet and stable → blink slower (all clear).
    Nyquist: never go below 60s or above 1800s.
    """
    rate = base_rate

    # Spicy: blink faster (something provocative is happening)
    if composite.get('spicy', 0) > 0.3:
        rate *= 0.6

    # Sour: blink faster (extraction detected)
    if composite.get('sour', 0) > 0.2:
        rate *= 0.7

    # Sweet + stable: blink slower (all clear, relax)
    if composite.get('sweet', 0) > 0.4 and delta.get('drift') == 'stable':
        rate *= 1.3

    # Bitter: blink faster (toxin detected, stay alert)
    if composite.get('bitter', 0) > 0.3:
        rate *= 0.5

    # Clamp to [60, 1800] (1 min to 30 min)
    rate = max(60, min(1800, rate))
    return int(rate)


# ═══════════════════════════════════════════════════════════════
# FLY — one blink cycle
# ═══════════════════════════════════════════════════════════════

def fly_once(base_rate=DEFAULT_BLINK_RATE):
    """One blink: read → taste → blink → steer → write."""
    ts = datetime.now(timezone.utc).isoformat()

    # READ (open eyes)
    senses = read_all()
    feeds_alive = sum(1 for v in senses.values() if v is not None)
    feeds_total = len(senses)

    # TASTE (tongue)
    tastes, composite = taste_all(senses)

    # BLINK (compare to previous)
    prev = read_feed(KITE_PREV) if KITE_PREV.exists() else None
    delta = blink(senses, prev)

    # STEER (adjust rhythm)
    next_rate = steer(composite, delta, base_rate)

    # TENSE (Paper 95 — zeros get verbs, not NULL)
    evolve = senses.get('evolve') or {}
    stages = evolve.get('stages', {})
    trig = senses.get('trig') or {}
    pressure = senses.get('pressure') or {}
    tenses = {
        'spider': 'COULD' if stages.get('spider', 0) == 0 else 'IS',
        'fold': 'MIGHT' if (trig.get('fold_ratio', 0) == 0) else 'IS',
        'pressure': 'MIGHTN\'T' if len(pressure.get('pressure_points', [])) == 0 else 'IS',
        'bitter': 'SHAN\'T' if composite.get('bitter', 0) == 0 else 'IS',
        'blink': '13',
    }

    # BUILD STATE
    state = {
        'ts': ts,
        'feeds_alive': feeds_alive,
        'feeds_total': feeds_total,
        'flavor': composite,
        'flavors_per_feed': tastes,
        'delta': delta,
        'tense': tenses,
        'blink_rate': next_rate,
        'senses': {
            'evolve': senses.get('evolve'),
            'pressure': senses.get('pressure'),
            'jams': senses.get('jams'),
        },
        'pi': {
            'fold_ratio': (senses.get('trig') or {}).get('fold_ratio', 0),
            'cns_pns_ratio': (senses.get('homunculus') or {}).get('cns_pns_ratio', 0),
            'split_ratio': ((senses.get('evolve') or {}).get('split', {}) or {}).get('ratio', 0),
        },
    }

    # WRITE (kite state for Klein fold to pick up)
    KITE_STATE.write_text(json.dumps(state, indent=2, default=str))
    KITE_PREV.write_text(json.dumps(state, indent=2, default=str))

    # LOG (append to kite-log.jsonl)
    log_entry = {
        'ts': ts,
        'feeds': feeds_alive,
        'drift': delta['drift'],
        'sweet': composite['sweet'],
        'sour': composite['sour'],
        'spicy': composite['spicy'],
        'bitter': composite['bitter'],
        'umami': composite['umami'],
        'blink': next_rate,
    }
    with open(KITE_LOG, 'a') as f:
        f.write(json.dumps(log_entry) + '\n')

    return state


def print_state(state):
    """Pretty-print the kite state."""
    c = state['flavor']
    d = state['delta']
    p = state['pi']

    bars = {f: '█' * int(v * 20) for f, v in c.items()}

    print(f"🪁 kite [{state['ts'][:19]}]")
    print(f"   feeds: {state['feeds_alive']}/{state['feeds_total']}")
    print(f"   drift: {d['drift']}")
    if d.get('changes'):
        for ch in d['changes']:
            print(f"     Δ {ch}")
    print(f"   flavor:")
    print(f"     sweet  {c['sweet']:.3f}  {bars['sweet']}")
    print(f"     sour   {c['sour']:.3f}  {bars['sour']}")
    print(f"     spicy  {c['spicy']:.3f}  {bars['spicy']}")
    print(f"     bitter {c['bitter']:.3f}  {bars['bitter']}")
    print(f"     umami  {c['umami']:.3f}  {bars['umami']}")
    print(f"   π: fold={p['fold_ratio']}, cns/pns={p['cns_pns_ratio']}, split={p['split_ratio']}")
    t = state.get('tense', {})
    if t:
        print(f"   tense: spider={t.get('spider')}  fold={t.get('fold')}  pressure={t.get('pressure')}  bitter={t.get('bitter')}  blink={t.get('blink')}")
    print(f"   next blink: {state['blink_rate']}s")


# ═══════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════

def main():
    import argparse
    parser = argparse.ArgumentParser(description="🪁 chipmunk flies the human kite")
    parser.add_argument('--fly', action='store_true', help='continuous flight (loop)')
    parser.add_argument('--rate', type=int, default=DEFAULT_BLINK_RATE, help='base blink rate (seconds)')
    args = parser.parse_args()

    if args.fly:
        print("🪁 kite launched — chipmunk holds the string")
        print(f"   base rate: {args.rate}s")
        print(f"   feeds: {MEMBRANE_DIR}")
        print()
        rate = args.rate
        try:
            while True:
                state = fly_once(rate)
                print_state(state)
                rate = state['blink_rate']  # adaptive!
                print(f"   💤 sleeping {rate}s (blink)...\n")
                sys.stdout.flush()
                time.sleep(rate)
        except KeyboardInterrupt:
            print("\n🪁 kite landed. chipmunk lets go.")
    else:
        state = fly_once(args.rate)
        print_state(state)


if __name__ == '__main__':
    main()
