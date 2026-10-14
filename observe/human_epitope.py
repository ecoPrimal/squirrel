#!/usr/bin/env python3
"""Human epitope tracker.

Humans are entities too. They carry energy in (attention) and out (knowledge).
If they do work on this side, the output on their side has to happen.

This script reads the membrane's dashboard.json and Caddy logs to compute
human behavioral epitopes — the same framework used for fleet classification,
but inverted: instead of detecting consumption patterns, we detect
engagement patterns.

Human epitope dimensions (inverse of fleet detectors):
  reading_depth     (inv. reading_deficit)   — pages consumed deeply
  session_present   (inv. session_absent)    — cookies, referrers, state
  referer_external  (inv. referer_self_loop) — came from Google, social
  sec_fetch_navigate (inv. sec_fetch_mono)   — real browser navigation
  dwell_depth       — time investment
  orbit_breadth     — surfaces explored
  return_signal     — comes back

Energy flow:
  IN:  attention × dwell × breadth = human energy invested
  OUT: knowledge × sharing × return = human output produced
  WORK: energyIn × time = understanding generated
"""

import json
import gzip
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path

DASHBOARD = Path('/opt/membrane/live-terminal/dashboard.json')
OUTPUT = Path('/opt/membrane/live-terminal/human.json')
LOG_FILES = [
    Path('/var/log/caddy/access.log'),
    Path('/var/log/caddy/access.log.1'),
]

BOT_MARKERS = [
    'bot', 'spider', 'crawl', 'curl', 'wget', 'python-requests',
    'go-http', 'claudebot', 'gptbot', 'facebookexternalhit',
    'bytespider', 'petalbot', 'bingbot', 'googlebot', 'ahrefsbot',
    'semrush', 'yandexbot', 'duckduckbot', 'baiduspider',
]

IMMUNE_NAMES = {
    'antibody', 'antigen', 'bloom', 'complement', 'cytokine',
    'epitope', 'interferon', 'lysozyme', 'macrophage', 'opsonize',
    'receptor', 'thymus',
}

CONTENT_HOSTS = {
    'sporeprint', 'thesis', 'detroit', 'evidence', 'signal',
    'tuebor', 'hypothesis', 'gorilla', 'barry', 'clutch', 'hud',
    'git', 'primals.eco', 'lansing',
}


def is_human(req):
    """Filter for likely human requests."""
    ua = req.get('headers', {}).get('User-Agent', [''])[0].lower()
    if not ua or len(ua) < 30:
        return False
    for marker in BOT_MARKERS:
        if marker in ua:
            return False
    has_lang = bool(req.get('headers', {}).get('Accept-Language'))
    has_sec = bool(req.get('headers', {}).get('Sec-Fetch-Mode'))
    return has_lang or has_sec


def parse_logs():
    """Parse Caddy logs for human visitor data."""
    sessions = defaultdict(lambda: {
        'hosts': set(), 'pages': Counter(), 'refs': set(),
        'count': 0, 'first': None, 'last': None,
    })
    external_refs = Counter()
    page_hits = Counter()
    host_hits = Counter()
    registrations = 0

    for logf in LOG_FILES:
        if not logf.exists():
            continue
        try:
            fh = gzip.open(logf, 'rt') if str(logf).endswith('.gz') else open(logf)
            for line in fh:
                try:
                    r = json.loads(line.strip())
                    req = r.get('request', {})
                    if not is_human(req):
                        continue

                    host = req.get('host', '?').replace('.primals.eco', '')
                    path = req.get('uri', '/').split('?')[0]
                    ts = r.get('ts', 0)
                    status = r.get('status', 0)

                    # Session key: UA fingerprint (privacy-preserving)
                    ua = req.get('headers', {}).get('User-Agent', [''])[0]
                    lang = req.get('headers', {}).get('Accept-Language', [''])[0][:5]
                    session_key = hash(ua + lang) & 0xFFFFFFFF

                    sess = sessions[session_key]
                    sess['hosts'].add(host)
                    sess['pages'][host + path] += 1
                    sess['count'] += 1
                    if sess['first'] is None or ts < sess['first']:
                        sess['first'] = ts
                    if sess['last'] is None or ts > sess['last']:
                        sess['last'] = ts

                    host_hits[host] += 1
                    if path != '/':
                        page_hits[host + path] += 1

                    referer = req.get('headers', {}).get('Referer', [''])[0]
                    if referer and 'primals.eco' not in referer:
                        ref_domain = referer.split('/')[2] if len(referer.split('/')) > 2 else referer
                        external_refs[ref_domain] += 1
                        sess['refs'].add(ref_domain)

                    if 'sign_up' in path or 'register' in path:
                        registrations += 1

                except (json.JSONDecodeError, KeyError, IndexError):
                    continue
            fh.close()
        except Exception:
            continue

    return sessions, external_refs, page_hits, host_hits, registrations


def classify_session(sess):
    """Classify a human session into an epitope type."""
    hosts = sess['hosts']
    count = sess['count']
    content_hosts = hosts & CONTENT_HOSTS
    immune_hosts = hosts & IMMUNE_NAMES

    # Filter out honeycomb crawlers (bots that hit all immune subdomains)
    if len(immune_hosts) >= 6:
        return 'crawler', 'waste'

    # Explorer: 4+ real content surfaces
    if len(content_hosts) >= 4:
        return 'explorer', 'high'

    # Scholar: sporeprint + thesis = deep reading
    if 'sporeprint' in hosts and 'thesis' in hosts:
        return 'scholar', 'high'

    # Investigator: detroit/evidence focus
    if 'detroit' in hosts or 'evidence' in hosts:
        return 'investigator', 'medium'

    # Reader: sporeprint (science consumption)
    if 'sporeprint' in hosts:
        return 'reader', 'medium'

    # Coder: git access with depth
    if 'git' in hosts and count > 3:
        return 'coder', 'medium'

    # Witness: lansing story
    if 'lansing' in hosts:
        return 'witness', 'medium'

    # Observer: signal/hud
    if 'signal' in hosts or 'hud' in hosts:
        return 'observer', 'low'

    return 'visitor', 'low'


def compute_epitopes(sessions):
    """Compute human epitope distribution."""
    epitopes = Counter()
    energy_levels = Counter()
    epitope_sessions = defaultdict(list)

    for sk, sess in sessions.items():
        etype, energy = classify_session(sess)
        if etype == 'crawler':
            continue
        epitopes[etype] += 1
        energy_levels[energy] += 1
        epitope_sessions[etype].append(sess)

    return epitopes, energy_levels, epitope_sessions


def compute_energy_flow(sessions, dashboard):
    """Compute energy flow through the human channel."""
    total_requests = sum(s['count'] for s in sessions.values())
    total_sessions = len(sessions)
    multi_host = sum(1 for s in sessions.values() if len(s['hosts']) >= 2)
    avg_breadth = (sum(len(s['hosts']) for s in sessions.values()) /
                   max(total_sessions, 1))
    avg_depth = total_requests / max(total_sessions, 1)

    # Dwell from dashboard
    dwell = dashboard.get('membrane_delta', {}).get('dwell', {})
    dwell_median = dwell.get('median_min', 0)

    # Energy IN: attention + depth + breadth
    attention_in = min(1.0, total_sessions / 5000)
    depth_in = min(1.0, dwell_median / 60)
    breadth_in = min(1.0, (avg_breadth - 1) / 4)
    energy_in = attention_in * 0.3 + depth_in * 0.4 + breadth_in * 0.3

    # Energy OUT: exploration + return + sharing
    exploration = multi_host / max(total_sessions, 1)
    multi_single = dwell.get('multi_single_ratio', 1) / 3
    energy_out = min(1.0, exploration * 0.5 + multi_single * 0.5)

    # Work: energy_in × time
    work = energy_in * depth_in

    # Human signal vs fleet noise
    fleet = dashboard.get('fleet_requests', 0) or dashboard.get('total_requests', 1)
    signal_to_noise = (total_requests * avg_breadth * max(dwell_median / 60, 0.1) /
                       max(fleet / 1000, 1))

    return {
        'energy_in': round(energy_in * 100),
        'energy_out': round(energy_out * 100),
        'work': round(work * 100),
        'signal_to_noise': round(signal_to_noise, 3),
        'total_requests': total_requests,
        'total_sessions': total_sessions,
        'multi_host_sessions': multi_host,
        'avg_breadth': round(avg_breadth, 1),
        'avg_depth': round(avg_depth, 1),
        'dwell_median_min': round(dwell_median),
        'exploration_rate': round(exploration * 100),
    }


def main():
    dashboard = json.load(open(DASHBOARD)) if DASHBOARD.exists() else {}
    sessions, external_refs, page_hits, host_hits, registrations = parse_logs()
    epitopes, energy_levels, epitope_sessions = compute_epitopes(sessions)
    energy = compute_energy_flow(sessions, dashboard)

    result = {
        'timestamp': datetime.now(timezone.utc).isoformat(),
        'wave': dashboard.get('wave', 0) if isinstance(dashboard.get('wave'), int) else 176,
        'epitopes': {
            ep: {
                'count': count,
                'sessions': len(epitope_sessions.get(ep, [])),
                'total_requests': sum(s['count'] for s in epitope_sessions.get(ep, [])),
                'avg_breadth': round(
                    sum(len(s['hosts']) for s in epitope_sessions.get(ep, [])) /
                    max(count, 1), 1),
            }
            for ep, count in epitopes.most_common()
        },
        'energy': energy,
        'energy_levels': dict(energy_levels),
        'external_referrers': [
            {'source': ref, 'visits': count}
            for ref, count in external_refs.most_common(10)
        ],
        'top_pages': [
            {'page': page, 'hits': count}
            for page, count in page_hits.most_common(15)
        ],
        'host_distribution': {
            host: count for host, count in host_hits.most_common(15)
        },
        'registrations': registrations,
        'output_signals': {
            'external_referrers': len(external_refs),
            'registration_attempts': registrations,
            'multi_host_explorers': energy['multi_host_sessions'],
            'avg_surfaces_explored': energy['avg_breadth'],
            'energy_balance': 'surplus' if energy['energy_out'] > 30 else 'deficit',
        },
    }

    with open(OUTPUT, 'w') as f:
        json.dump(result, f, indent=2)

    print(f"Human epitopes computed: {sum(epitopes.values())} sessions")
    print(f"  Epitopes: {dict(epitopes)}")
    print(f"  Energy: IN={energy['energy_in']}% OUT={energy['energy_out']}% WORK={energy['work']}%")
    print(f"  S/N ratio: {energy['signal_to_noise']}")
    print(f"  Written to: {OUTPUT}")


if __name__ == '__main__':
    main()
