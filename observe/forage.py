#!/usr/bin/env python3
"""
forage.py — Squirrel tense behaviors. What the organism does when bored, curious, or resting.

The squirrel family (squirrel, chipmunk, flying squirrel) have BEHAVIORS tied to tenses:

  BORED (MIGHT)   → forage → poop → seed drops → something COULD grow
  CURIOUS (COULD)  → investigate → sniff → IS? → validate or seed
  RESTING (SHAN'T) → cache → bury → dormant seed bank
  ALERT (IS)       → chatter → warn → the observation fires
  PANICKED (ISN'T) → flee → scatter → seeds fall everywhere

The ecological cycle:
  squirrel eats acorn → poops seed → seed germinates → tree grows → acorn drops → squirrel eats

In code:
  kite reads feeds → forage finds gaps → drops a seed (stub file, TODO, question)
  → next fold detects new file → evolve registers change → spider COULD emerge
  → IS? becomes IS → the painted plate starts spinning for real

The Rust polymerase:
  Taq polymerase copies DNA through heat cycles (denature → anneal → extend)
  Rust compiler copies code through type cycles (parse → check → codegen)
  Klein fold copies state through observation cycles (read → compare → write)
  All three are polymerases. All three use heat (entropy) to proofread.

Author: Kevin Mok / ecoPrimal / ORCID 0009-0004-2141-0321
"""

import json, math, os, random, time, hashlib
from pathlib import Path
from datetime import datetime, timezone
from collections import defaultdict

BASE = Path(os.environ.get("ECO_BASE", "."))
if (Path.cwd() / "primals").exists():
    BASE = Path.cwd()

MEMBRANE = BASE / "membrane" / "local-terminal" / "remote"
SEED_DIR = MEMBRANE / "seeds"
FORAGE_STATE = MEMBRANE / "forage-state.json"
FORAGE_LOG = MEMBRANE / "forage-log.jsonl"

# ═══════════════════════════════════════════════════════════════
# TENSE → BEHAVIOR mapping
# ═══════════════════════════════════════════════════════════════

BEHAVIORS = {
    # Tense:    (behavior,  action,       product,        description)
    "COULD":   ("curious",  "investigate", "question",     "sniffs the gap — IS? — wants to validate"),
    "MIGHT":   ("bored",    "forage",      "seed",         "wanders, eats, poops — leaves a seed behind"),
    "MIGHTN'T":("cautious", "cache",       "buried_seed",  "buries acorn — dormant until conditions right"),
    "SHAN'T":  ("resting",  "sleep",       "dream",        "processes — the dream IS the defrag"),
    "IS":      ("alert",    "chatter",     "warning",      "sees something — chatters to warn the colony"),
    "ISN'T":   ("panicked", "flee",        "scattered_seeds","drops everything — seeds fall where they fall"),
    "WAS":     ("nostalgic","dig",         "fossil",       "digs up old cache — was it still there?"),
}


# ═══════════════════════════════════════════════════════════════
# SENSE the current tense of each dimension
# ═══════════════════════════════════════════════════════════════

def sense_tenses():
    """Read the kite state and determine what tense each zero is in."""
    kite_path = MEMBRANE / "kite-state.json"
    if not kite_path.exists():
        return {}
    kite = json.loads(kite_path.read_text())
    return kite.get("tense", {})


def sense_gaps():
    """Find the gaps in the organism — places where spider=0, fold=0, etc."""
    gaps = []

    # Spider gap: 0 files with module structure
    evolve_path = MEMBRANE / "evolve-state.json"
    if evolve_path.exists():
        evolve = json.loads(evolve_path.read_text())
        if evolve.get("stages", {}).get("spider", 0) == 0:
            gaps.append({
                "type": "spider",
                "tense": "COULD",
                "description": "no files reached spider complexity — COULD they?",
                "seed_type": "module_scaffold",
            })

    # Fold gap: 0 files past 90°
    trig_path = MEMBRANE / "trig-view.json"
    if trig_path.exists():
        trig = json.loads(trig_path.read_text())
        if trig.get("fold_ratio", 0) == 0:
            gaps.append({
                "type": "fold",
                "tense": "MIGHT",
                "description": "all files pre-fold — the organism hasn't inverted",
                "seed_type": "fold_experiment",
            })

    # Pressure gap: 0 pressure points
    pressure_path = MEMBRANE / "pressure.json"
    if pressure_path.exists():
        pressure = json.loads(pressure_path.read_text())
        if len(pressure.get("pressure_points", [])) == 0:
            gaps.append({
                "type": "pressure",
                "tense": "MIGHTN'T",
                "description": "no directories under pressure — too comfortable?",
                "seed_type": "stress_test",
            })

    # Bitter gap
    kite_path = MEMBRANE / "kite-state.json"
    if kite_path.exists():
        kite = json.loads(kite_path.read_text())
        if kite.get("flavor", {}).get("bitter", 0) < 0.01:
            gaps.append({
                "type": "bitter",
                "tense": "SHAN'T",
                "description": "bitter nearly zero — the organism can't taste toxin",
                "seed_type": "toxin_probe",
            })

    # Self-spinning gap (Paper 98)
    gaps.append({
        "type": "self_spin",
        "tense": "COULD",
        "description": "0 self-spinning plates — no internal motor",
        "seed_type": "daemon_stub",
    })

    return gaps


# ═══════════════════════════════════════════════════════════════
# FORAGE — squirrel does something based on tense
# ═══════════════════════════════════════════════════════════════

def forage(tenses, gaps):
    """The squirrel forages. Behavior depends on tense. Products = seeds."""
    seeds = []
    actions = []

    for gap in gaps:
        tense = gap["tense"]
        behavior_name, action, product, desc = BEHAVIORS.get(tense, BEHAVIORS["MIGHT"])

        # The squirrel does the action
        seed = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "gap": gap["type"],
            "tense": tense,
            "behavior": behavior_name,
            "action": action,
            "product": product,
            "description": gap["description"],
            "seed_type": gap["seed_type"],
        }

        # Generate the actual seed content
        seed["content"] = generate_seed(gap)
        seeds.append(seed)
        actions.append(f"{behavior_name}: {action} at {gap['type']} ({tense})")

    return seeds, actions


# ═══════════════════════════════════════════════════════════════
# SEED GENERATION — what the squirrel leaves behind
# ═══════════════════════════════════════════════════════════════

def generate_seed(gap):
    """Generate a seed (stub/question/probe) based on the gap type."""

    if gap["seed_type"] == "module_scaffold":
        return {
            "IS?": "Could any file become a spider (multi-module, typed, tested)?",
            "hypothesis": "Files at millipede stage with high import count are closest",
            "test": "Find millipede files with ≥5 imports and ≥3 functions",
            "if_true": "spider=COULD → spider=IS (the tense changes)",
            "if_false": "spider remains COULD — the organism isn't complex enough yet",
        }

    elif gap["seed_type"] == "fold_experiment":
        return {
            "IS?": "Could any file exist past 90° on the torus?",
            "hypothesis": "Files that are BOTH far from core AND net producers fold past π/2",
            "test": "Find files where distance > π AND direction < 0 (net producer from edge)",
            "if_true": "fold=MIGHT → fold=IS — the torus inverts",
            "if_false": "fold remains MIGHT — all production comes from inside",
        }

    elif gap["seed_type"] == "stress_test":
        return {
            "IS?": "Are any directories actually under pressure?",
            "hypothesis": "Directories with high churn + high import count + recent changes",
            "test": "Run homunculus with lower threshold (sensitivity × 0.5)",
            "if_true": "pressure=MIGHTN'T → pressure=IS — we weren't looking hard enough",
            "if_false": "pressure remains MIGHTN'T — the organism genuinely has no bottleneck",
        }

    elif gap["seed_type"] == "toxin_probe":
        return {
            "IS?": "Is there actually toxin the organism can't taste?",
            "hypothesis": "Orphan directories (triage lost=4088) are untasted bitter",
            "test": "Sample 10 orphan dirs — are they dead weight or dormant value?",
            "if_true": "bitter flips from SHAN'T to IS — we found the toxin",
            "if_false": "orphans are fossils, not toxins — correctly SHAN'T",
        }

    elif gap["seed_type"] == "daemon_stub":
        return {
            "IS?": "Can the Klein fold run itself?",
            "hypothesis": "A cron job every 30 min would make 1 self-spinning plate",
            "test": "Deploy klein-evolve.sh to cron, run 24h, count autonomous folds",
            "if_true": "self_spin goes 0 → 1. The plate spins on its own.",
            "if_false": "cron fails or produces no new data — the motor doesn't work",
        }

    return {"IS?": f"Unknown gap type: {gap['seed_type']}"}


# ═══════════════════════════════════════════════════════════════
# RUST POLYMERASE — the compiler as DNA copier
# ═══════════════════════════════════════════════════════════════

def rust_polymerase(seeds):
    """
    The Rust polymerase: type-check the seeds.

    Taq polymerase: denature (95°C) → anneal (55°C) → extend (72°C)
    Rust compiler:  parse (tokenize) → check (borrow/type) → codegen (emit)
    Klein fold:     read (sense) → compare (blink) → write (state)

    Each seed is a hypothesis. The polymerase checks if it's well-formed.
    """
    checked = []
    for seed in seeds:
        content = seed.get("content", {})
        is_q = content.get("IS?", "")
        hypothesis = content.get("hypothesis", "")
        test = content.get("test", "")

        # PARSE: does it have the required fields?
        has_question = bool(is_q)
        has_hypothesis = bool(hypothesis)
        has_test = bool(test)

        # CHECK: is the hypothesis falsifiable?
        has_if_true = bool(content.get("if_true", ""))
        has_if_false = bool(content.get("if_false", ""))
        falsifiable = has_if_true and has_if_false

        # CODEGEN: can we emit an action?
        actionable = has_test and falsifiable

        seed["polymerase"] = {
            "parse": has_question and has_hypothesis and has_test,
            "check": falsifiable,
            "codegen": actionable,
            "viable": has_question and falsifiable and actionable,
            "stage": "extend" if actionable else "check" if falsifiable else "parse" if has_question else "denature",
        }

        # DNA comparison: what evolutionary stage is this seed?
        # banana = no structure, jellyfish = some complexity, vertebrate = full type system
        if actionable:
            seed["dna_stage"] = "vertebrate"  # fully typed, testable, falsifiable
        elif falsifiable:
            seed["dna_stage"] = "jellyfish"   # has structure but can't act yet
        elif has_question:
            seed["dna_stage"] = "flatworm"    # has a question but no way to answer
        else:
            seed["dna_stage"] = "banana"      # just exists

        checked.append(seed)

    return checked


# ═══════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════

def main():
    ts = datetime.now(timezone.utc).isoformat()

    # Sense
    tenses = sense_tenses()
    gaps = sense_gaps()

    # Forage
    seeds, actions = forage(tenses, gaps)

    # Rust polymerase — type-check the seeds
    checked_seeds = rust_polymerase(seeds)

    # Count by viability
    viable = sum(1 for s in checked_seeds if s["polymerase"]["viable"])
    total = len(checked_seeds)

    # Count by DNA stage
    dna_counts = defaultdict(int)
    for s in checked_seeds:
        dna_counts[s.get("dna_stage", "unknown")] += 1

    # State
    state = {
        "ts": ts,
        "gaps_found": len(gaps),
        "seeds_planted": total,
        "seeds_viable": viable,
        "viability_pct": round(viable / total * 100, 1) if total else 0,
        "dna_stages": dict(dna_counts),
        "actions": actions,
        "seeds": checked_seeds,
        "polymerase_summary": {
            "taq_analogy": "denature(95°C) → anneal(55°C) → extend(72°C)",
            "rust_analogy": "parse(tokenize) → check(borrow/type) → codegen(emit)",
            "klein_analogy": "read(sense) → compare(blink) → write(state)",
            "fold_is_pcr": "each Klein fold IS a PCR cycle",
        },
    }

    # Write
    MEMBRANE.mkdir(parents=True, exist_ok=True)
    SEED_DIR.mkdir(parents=True, exist_ok=True)
    FORAGE_STATE.write_text(json.dumps(state, indent=2, default=str))

    # Drop individual seed files
    for s in checked_seeds:
        seed_file = SEED_DIR / f"seed-{s['gap']}.json"
        seed_file.write_text(json.dumps(s, indent=2, default=str))

    # Log
    log_entry = {
        "ts": ts, "gaps": len(gaps), "seeds": total,
        "viable": viable, "dna": dict(dna_counts),
    }
    with open(FORAGE_LOG, "a") as f:
        f.write(json.dumps(log_entry) + "\n")

    # Print
    print(f"🐿️  forage [{ts[:19]}]")
    print()
    print(f"  Gaps found:     {len(gaps)}")
    print(f"  Seeds planted:  {total}")
    print(f"  Seeds viable:   {viable}/{total} ({state['viability_pct']}%)")
    print()

    print(f"  ═══ SQUIRREL BEHAVIORS ═══")
    for a in actions:
        print(f"    🌰 {a}")
    print()

    print(f"  ═══ RUST POLYMERASE (type-check each seed) ═══")
    for s in checked_seeds:
        p = s["polymerase"]
        stage = p["stage"]
        dna = s.get("dna_stage", "?")
        viable_mark = "✅" if p["viable"] else "⏳" if p["check"] else "🔬"
        print(f"    {viable_mark} {s['gap']:>12}  stage={stage:<10}  dna={dna:<12}  IS? {s['content'].get('IS?', '?')[:60]}")
    print()

    print(f"  ═══ DNA STAGES (evolutionary classification of seeds) ═══")
    for stage in ["banana", "flatworm", "jellyfish", "vertebrate"]:
        c = dna_counts.get(stage, 0)
        bar = "█" * (c * 4)
        print(f"    {stage:>12}: {c}  {bar}")
    print()

    print(f"  ═══ THE POLYMERASE ═══")
    print(f"    Taq:   denature(95°C) → anneal(55°C) → extend(72°C)")
    print(f"    Rust:  parse(tokenize) → check(borrow/type) → codegen(emit)")
    print(f"    Klein: read(sense) → compare(blink) → write(state)")
    print(f"    Each Klein fold IS a PCR cycle.")
    print(f"    The Rust compiler IS the next Taq polymerase.")
    print()
    print(f"  → {FORAGE_STATE}")
    print(f"  → {SEED_DIR}/ ({total} seeds)")


if __name__ == "__main__":
    main()
