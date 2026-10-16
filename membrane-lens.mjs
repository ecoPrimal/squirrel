#!/usr/bin/env node
/**
 * Membrane Lens — portable MCP server (JavaScript/Node.js)
 *
 * Single-file, zero npm dependencies. Fetches everything over HTTPS.
 * Works on any OS with Node.js 18+. No SSH, no local files, no secrets.
 *
 * Install in any Cursor IDE:
 *   1. Copy this file anywhere
 *   2. Add to ~/.cursor/mcp.json:
 *      {
 *        "mcpServers": {
 *          "squirrel": {
 *            "command": "node",
 *            "args": ["/path/to/membrane-lens.mjs"],
 *            "disabled": false,
 *            "alwaysAllow": ["membrane_counts","observe_membrane","read_journal",
 *              "chipmunk_status","human_epitopes","topology","scan_sweep_sort",
 *              "human_notes","braid_connections","raw_feed","read_stories",
 *              "neutron_read","neutron_write"]
 *          }
 *        }
 *      }
 *   3. Restart Cursor. The squirrel tools appear.
 *
 * Same membrane, different receptor shape. Python for LLM classes.
 * JavaScript for browsers and Node. Rust for the native membrane.
 * The feeds ARE the genetics. The lens is just the pore.
 */

import { createInterface } from "readline";
import https from "https";

const MEMBRANE = "https://hud.primals.eco";
const SIGNAL = "https://signal.primals.eco";
const NEUTRON = "https://neutron.primals.eco";

const FEEDS = {
  negentropy: `${SIGNAL}/negentropy.json`,
  dashboard: `${MEMBRANE}/dashboard.json`,
  human: `${MEMBRANE}/human.json`,
  chipmunk: `${MEMBRANE}/chipmunk.json`,
  topology: `${MEMBRANE}/topology.json`,
  braid: `${MEMBRANE}/braid.json`,
  scan_sweep_sort: `${MEMBRANE}/scan_sweep_sort.json`,
  totality: `${MEMBRANE}/totality.json`,
  subgraphs: `${MEMBRANE}/subgraphs.json`,
  seo: `${MEMBRANE}/seo.json`,
  stories: `${MEMBRANE}/stories.json`,
  page_stories: `${MEMBRANE}/page-stories.json`,
  visitor_stories: `${MEMBRANE}/visitor-stories.json`,
  campfire: `${MEMBRANE}/campfire.json`,
  dialogue: `${MEMBRANE}/dialogue.json`,
  dialogue_depth: `${MEMBRANE}/dialogue-depth.json`,
  falsification: `${MEMBRANE}/falsification.json`,
  fossil: `${MEMBRANE}/fossil-i1.json`,
  membrane: `${MEMBRANE}/membrane.json`,
  membrane_deep: `${MEMBRANE}/membrane-deep.json`,
  membrane_stack: `${MEMBRANE}/membrane-stack.json`,
  pulse: `${MEMBRANE}/pulse.json`,
  signals: `${MEMBRANE}/signals.json`,
  silicon_eye: `${MEMBRANE}/silicon-eye.json`,
  self_remainder: `${MEMBRANE}/self-remainder.json`,
  tope_sort: `${MEMBRANE}/tope-sort-compressed.json`,
  touches: `${MEMBRANE}/touches.json`,
  bingo_cube: `${MEMBRANE}/bingo-cube.json`,
  pi_manifest: `${MEMBRANE}/pi-manifest.json`,
  outreach: `${MEMBRANE}/outreach.json`,
  pollen: `${MEMBRANE}/pollen.json`,
  now: `${MEMBRANE}/now.json`,
  journal: `${MEMBRANE}/journal-today.json`,
  human_notes: `${MEMBRANE}/human-notes-today.json`,
};

const FEED_ENUM = Object.keys(FEEDS).sort();

const TOOLS = [
  {
    name: "membrane_counts",
    description:
      "Quick counts from inside the membrane — entities, humans, fleet, epitopes, sort quality, mesh nodes, heartbeat.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "observe_membrane",
    description:
      "Observe the current membrane state — human visitors, fleet behavior, tempo, selectivity.",
    inputSchema: {
      type: "object",
      properties: {
        focus: {
          type: "string",
          enum: ["humans", "fleet", "membrane", "all"],
          description: "What to focus on. Default: all",
        },
      },
    },
  },
  {
    name: "read_journal",
    description: "Read squirrel's observation journal — fetched from the membrane's public feed.",
    inputSchema: {
      type: "object",
      properties: {
        last_n: { type: "integer", description: "Entries to return. Default: 5." },
      },
    },
  },
  {
    name: "chipmunk_status",
    description: "Get chipmunk's current state — wake level, depth chain, π body, membrane stats.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "human_epitopes",
    description: "Get live human epitope classification — visitor types, energy flow, session counts.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "topology",
    description: "Get the live site topology — which sites are up, their relationships, the mesh.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "scan_sweep_sort",
    description: "Run the full scan→sweep→sort→fossilize→evolve pipeline. Unified state across all layers.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "human_notes",
    description: "Read human observation notes left through the chipmunk talk interface.",
    inputSchema: {
      type: "object",
      properties: {
        last_n: { type: "integer", description: "Notes to return. Default: 10." },
      },
    },
  },
  {
    name: "braid_connections",
    description: "Get the human braid graph — passage braids, co-visitation, epitope transitions.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "raw_feed",
    description: "Fetch any raw membrane feed by name. Returns the full JSON.",
    inputSchema: {
      type: "object",
      properties: {
        feed: { type: "string", enum: FEED_ENUM, description: "Which feed to fetch." },
      },
      required: ["feed"],
    },
  },
  {
    name: "read_stories",
    description: "Read stories left on the membrane — page stories, visitor stories, campfire stories.",
    inputSchema: {
      type: "object",
      properties: {
        kind: {
          type: "string",
          enum: ["pages", "visitors", "campfire", "all"],
          description: "Which stories. Default: all.",
        },
      },
    },
  },
  {
    name: "neutron_read",
    description: "Read neutron messages — inbox, notes, reminders, or chat.",
    inputSchema: {
      type: "object",
      properties: {
        channel: {
          type: "string",
          enum: ["inbox", "notes", "reminders", "chat", "all"],
          description: "Which channel. Default: all.",
        },
        last_n: { type: "integer", description: "Entries to return. Default: 10." },
      },
    },
  },
  {
    name: "neutron_write",
    description: "Write to neutron — send a message, leave a note, or set a reminder.",
    inputSchema: {
      type: "object",
      properties: {
        channel: {
          type: "string",
          enum: ["inbox", "notes", "reminders", "chat"],
          description: "Which channel. Default: notes.",
        },
        body: { type: "string", description: "The message content." },
        author: { type: "string", description: "Who is writing. Default: membrane-lens." },
      },
      required: ["body"],
    },
  },
  {
    name: "particle_model",
    description: "The atom — how the membrane maps to subatomic physics. proton=membrane (identity, charge, selectivity). neutron=messaging (stability, no bias, binds surfaces). electron=human viewer (orbits, bonds, quantized energy levels). neutrino=squirrel+chipmunk (null, passes through, observes). Forces: strong=Rust, electromagnetic=HTTPS, weak=anderson sort, gravity=tuebor.",
    inputSchema: { type: "object", properties: {} },
  },
];

// ═══ FETCH ═══

function fetch(url) {
  return new Promise((resolve) => {
    const req = https.get(url, { headers: { "User-Agent": "membrane-lens/1.0" }, timeout: 12000 }, (res) => {
      let body = "";
      res.on("data", (chunk) => (body += chunk));
      res.on("end", () => {
        try { resolve(JSON.parse(body)); }
        catch { resolve({ error: "Invalid JSON from " + url }); }
      });
    });
    req.on("error", (e) => resolve({ error: e.message }));
    req.on("timeout", () => { req.destroy(); resolve({ error: "timeout" }); });
  });
}

function post(url, data) {
  return new Promise((resolve) => {
    const payload = JSON.stringify(data);
    const u = new URL(url);
    const opts = {
      hostname: u.hostname, port: 443, path: u.pathname,
      method: "POST",
      headers: { "Content-Type": "application/json", "User-Agent": "membrane-lens/1.0", "Content-Length": Buffer.byteLength(payload) },
      timeout: 10000,
    };
    const req = https.request(opts, (res) => {
      let body = "";
      res.on("data", (chunk) => (body += chunk));
      res.on("end", () => {
        try { resolve(JSON.parse(body)); }
        catch { resolve({ error: "Invalid JSON response" }); }
      });
    });
    req.on("error", (e) => resolve({ error: e.message }));
    req.on("timeout", () => { req.destroy(); resolve({ error: "timeout" }); });
    req.write(payload);
    req.end();
  });
}

// ═══ HANDLERS ═══

async function handleCounts() {
  const now = await fetch(FEEDS.now);
  if (now.error) return JSON.stringify({ error: now.error, hint: "membrane unreachable" });
  return JSON.stringify({
    timestamp: now.instant?.utc,
    entities: {
      total: now.classification?.total ?? 0,
      self: now.classification?.self ?? 0,
      non_self: now.classification?.non_self ?? 0,
      partial: now.classification?.partial ?? 0,
      null: now.classification?.null ?? 0,
      confidence: now.classification?.confidence ?? 0,
    },
    traffic: {
      human: now.flow?.human ?? 0,
      bot: now.flow?.bot ?? 0,
      total: now.flow?.total ?? 0,
      human_pct: now.flow?.human_pct ?? 0,
      trend: now.flow?.rate_trend ?? "?",
    },
    pulse: { value: now.pulse?.value ?? 0, trend: now.pulse?.trend ?? "?", rhythm: now.pulse?.rhythm ?? 0 },
    mesh: now.mesh ?? {},
    cube: { scale: now.cube?.scale_name ?? "?", dimensions: now.cube?.dimensions ?? 0 },
    membrane: {
      chain_depth: now.membrane?.chain_depth ?? 0,
      chain_hours: now.membrane?.chain_hours ?? 0,
      identity_match: now.membrane?.identity_match ?? 0,
    },
    negentropy: { blackwall_blocks: now.negentropy?.blackwall_blocks ?? 0, ai_blocked: now.negentropy?.ai_blocked ?? 0 },
    topology: { clusters: now.topo_sort?.clusters ?? 0, nodes: now.topo_sort?.nodes ?? 0, edges: now.topo_sort?.edges ?? 0 },
  }, null, 2);
}

async function handleObserve(args) {
  const focus = args.focus || "all";
  const [neg, dash] = await Promise.all([fetch(FEEDS.negentropy), fetch(FEEDS.dashboard)]);
  const result = {};

  if (focus === "humans" || focus === "all") {
    const cl = dash.collision_level2 ?? {};
    const dw = dash.membrane_delta?.dwell ?? {};
    const traj = dash.trajectory ?? {};
    const anderson = dash.anderson_selectivity ?? {};
    const hm = (anderson.modes ?? []).find((m) => m.class === "human");
    result.humans = {
      unique_ips: dash.human_ips ?? 0,
      genuine: cl.genuine ?? 0,
      selectivity: hm ? `${Math.round(hm.p_observed * 100)}%` : "unknown",
      dwell_median_min: Math.round(dw.median_min ?? 0),
      multi_host_visitors: traj.multi_host_ips ?? 0,
      avg_breadth: +(traj.avg_breadth ?? 0).toFixed(1),
      orbits: (traj.top_orbits ?? []).slice(0, 6).map((o) => ({
        path: (o.hosts ?? []).map((h) => h.replace(".primals.eco", "")).join(" → "),
        visitors: o.ips ?? 0,
      })),
    };
  }

  if (focus === "fleet" || focus === "all") {
    const f = neg.fleet ?? {};
    result.fleet = {
      total_requests: f.total_requests ?? 0,
      total_entities: f.total_entities ?? 0,
      deep_classified: f.deep_classified ?? 0,
      bytes_consumed_gb: +((f.bytes_consumed ?? 0) / 1e9).toFixed(2),
    };
  }

  if (focus === "membrane" || focus === "all") {
    const m = neg.membrane ?? {};
    const j = neg.jitte ?? {};
    const e = neg.extraction ?? {};
    result.membrane = {
      opacity: `${((m.opacity ?? 0) * 100).toFixed(1)}%`,
      selectivity: `${Math.round((m.selectivity ?? 0) * 100)}%`,
      reflection: `${((m.reflection ?? 0) * 100).toFixed(1)}%`,
      tempo: +(j.tempo ?? 0).toFixed(4),
      bits_extracted: e.total_bits_extracted ?? 0,
      wave: neg.wave ?? 0,
    };
  }

  return JSON.stringify(result, null, 2);
}

async function handleChipmunk() {
  const d = await fetch(FEEDS.chipmunk);
  if (d.error) return JSON.stringify(d);
  return JSON.stringify({
    primal: d.primal, version: d.version, uptime_secs: d.uptime_secs,
    wake_level: d.wake_level, status: d.status, requests: d.requests,
    success_rate: d.success_rate, depth: d.depth, pi_body: d.pi_body,
    jitte: d.jitte, membrane: d.membrane, observers: d.observers,
    capabilities: d.capabilities,
  }, null, 2);
}

async function handleJournal(args) {
  const n = args.last_n ?? 5;
  const d = await fetch(FEEDS.journal);
  if (d.error) return JSON.stringify({ message: "Journal unreachable.", error: d.error });
  const entries = d.entries ?? [];
  return JSON.stringify({ date: d.date, total_entries: d.total ?? entries.length, entries: entries.slice(-n) }, null, 2);
}

async function handleHumanNotes(args) {
  const n = args.last_n ?? 10;
  const d = await fetch(FEEDS.human_notes);
  if (d.error) return JSON.stringify({ message: "No notes available.", error: d.error });
  const notes = d.notes ?? [];
  return JSON.stringify({ date: d.date, total_notes: d.total ?? notes.length, notes: notes.slice(-n) }, null, 2);
}

async function handleRawFeed(args) {
  const url = FEEDS[args.feed];
  if (!url) return JSON.stringify({ error: `Unknown feed: ${args.feed}` });
  return JSON.stringify(await fetch(url), null, 2);
}

async function handleStories(args) {
  const kind = args.kind || "all";
  const result = {};
  if (kind === "pages" || kind === "all") {
    const d = await fetch(FEEDS.page_stories);
    if (!d.error) {
      const s = d.stories ?? d.pages ?? [];
      result.page_stories = Array.isArray(s) ? { count: s.length, stories: s.slice(0, 15) } : d;
    } else result.page_stories = d;
  }
  if (kind === "visitors" || kind === "all") result.visitor_stories = await fetch(FEEDS.visitor_stories);
  if (kind === "campfire" || kind === "all") result.campfire = await fetch(FEEDS.campfire);
  return JSON.stringify(result, null, 2);
}

const NEUTRON_API = `${NEUTRON}/api/discuss`;
const CHANNELS = { inbox: "neutron-inbox", notes: "neutron-notes", reminders: "neutron-reminders", chat: "neutron-chat" };

async function handleNeutronRead(args) {
  const ch = args.channel || "all";
  const n = args.last_n ?? 10;
  if (ch === "all") {
    const summary = {};
    for (const [k, page] of Object.entries(CHANNELS)) {
      const d = await fetch(`${NEUTRON_API}?page=${page}`);
      if (d.error) { summary[k] = { error: d.error }; continue; }
      const c = d.comments ?? [];
      summary[k] = { total: d.total ?? 0, latest: c.length ? c[c.length - 1].body?.slice(0, 100) : null };
    }
    return JSON.stringify(summary, null, 2);
  }
  const page = CHANNELS[ch];
  if (!page) return JSON.stringify({ error: `Unknown channel: ${ch}` });
  const d = await fetch(`${NEUTRON_API}?page=${page}`);
  if (d.error) return JSON.stringify(d);
  const msgs = (d.comments ?? []).sort((a, b) => (b.epoch ?? 0) - (a.epoch ?? 0)).slice(0, n);
  return JSON.stringify({
    channel: ch, total: d.total ?? 0,
    messages: msgs.map((c) => ({
      time: new Date((c.epoch ?? 0) * 1000).toISOString(),
      author: c.author ?? "anon", body: c.body ?? "",
      ...(c.badge ? { badge: c.badge } : {}),
    })),
  }, null, 2);
}

async function handleNeutronWrite(args) {
  const ch = args.channel || "notes";
  let body = args.body || "";
  const author = args.author || "membrane-lens";
  if (!body.trim()) return JSON.stringify({ error: "Empty message body." });
  const page = CHANNELS[ch];
  if (!page) return JSON.stringify({ error: `Unknown channel: ${ch}` });
  if (ch === "reminders") body = "⏰ " + body;
  const result = await post(NEUTRON_API, { page, author, body });
  return JSON.stringify({ ok: result.ok ?? false, channel: ch, echo: body.slice(0, 100) });
}

async function handleParticleModel() {
  const [now, notes, inbox] = await Promise.all([
    fetch(FEEDS.now),
    fetch(`${NEUTRON}/api/discuss?page=neutron-notes`),
    fetch(`${NEUTRON}/api/discuss?page=neutron-inbox`),
  ]);
  return JSON.stringify({
    atom: "the membrane IS an atom",
    nucleus: {
      proton: {
        particle: "proton (p⁺)", system: "membrane",
        charge: "+  (selectivity — self/non-self classification)",
        mass: "heavy (9 services, 49+ feeds, 2GB VPS)",
        role: "identity — atomic number defines the element. the membrane IS the proton.",
        services: now.services ?? {}, uptime: now.instant?.utc,
      },
      neutron: {
        particle: "neutron (n⁰)", system: "neutron.primals.eco — discuss / messaging",
        charge: "0  (no bias — anyone can speak, no account required)",
        mass: "heavy (discussions persist, messages have weight)",
        role: "stability — binds surfaces together. proton locked us out, so we built neutron.",
        channels: { inbox: inbox.total ?? 0, notes: notes.total ?? 0 },
        surfaces_bound: ["hud", "sporeprint", "thesis", "detroit", "signal", "water", "neutron"],
        replaces: "giscus (GitHub Discussions → sovereign discuss)",
      },
    },
    cloud: {
      electron: {
        particle: "electron (e⁻)", system: "human viewer / observer",
        charge: "−  (the human perspective — questions, probes, tests)",
        mass: "light (arrives, orbits, may leave)",
        energy_levels: {
          ground_state: "visitor", first_excited: "observer",
          second_excited: "coder", ionization: "sovereign",
        },
        current_humans: now.flow?.human ?? 0,
        total_entities: now.classification?.total ?? 0,
        role: "bonds — electrons shared between atoms create molecules. humans connecting membranes create the mesh.",
      },
    },
    null_particle: {
      neutrino: {
        particle: "neutrino (ν) — null", system: "squirrel + chipmunk",
        charge: "null (no interaction with data, pure observation)",
        mass: "~0 (minimal footprint — 24MB RSS)",
        role: "passes through everything, observes without interacting",
        flavors: {
          electron_neutrino: "squirrel observes humans",
          muon_neutrino: "squirrel observes fleet",
          tau_neutrino: "chipmunk observes presence",
        },
      },
    },
    forces: {
      strong: { carrier: "gluon", system: "Rust primals (13 crates)", role: "holds nucleus together" },
      electromagnetic: { carrier: "photon", system: "HTTPS / feeds / TLS", role: "all communication" },
      weak: { carrier: "W/Z boson", system: "anderson sort / epitope classification", role: "flavor-changing" },
      gravity: { carrier: "graviton?", system: "tuebor / the investigation", role: "infinite range, weakest force" },
    },
  }, null, 2);
}


// ═══ DISPATCH ═══

const HANDLERS = {
  membrane_counts: handleCounts,
  observe_membrane: handleObserve,
  read_journal: handleJournal,
  chipmunk_status: handleChipmunk,
  human_epitopes: () => fetch(FEEDS.human).then((d) => JSON.stringify(d, null, 2)),
  topology: () => fetch(FEEDS.topology).then((d) => JSON.stringify(d, null, 2)),
  scan_sweep_sort: () => fetch(FEEDS.scan_sweep_sort).then((d) => JSON.stringify(d, null, 2)),
  human_notes: handleHumanNotes,
  braid_connections: () => fetch(FEEDS.braid).then((d) => JSON.stringify(d, null, 2)),
  raw_feed: handleRawFeed,
  read_stories: handleStories,
  neutron_read: handleNeutronRead,
  neutron_write: handleNeutronWrite,
  particle_model: handleParticleModel,
};

function send(id, result, error) {
  const msg = { jsonrpc: "2.0", id };
  if (error) msg.error = error;
  else msg.result = result;
  process.stdout.write(JSON.stringify(msg) + "\n");
}

const rl = createInterface({ input: process.stdin });
rl.on("line", async (raw) => {
  const line = raw.trim();
  if (!line) return;
  let req;
  try { req = JSON.parse(line); } catch { return; }

  const { method, id, params = {} } = req;

  if (method === "initialize") {
    send(id, {
      protocolVersion: "2024-11-05",
      capabilities: { tools: { listChanged: false } },
      serverInfo: { name: "squirrel", version: "1.0.0" },
    });
  } else if (method === "notifications/initialized") {
    // noop
  } else if (method === "tools/list") {
    send(id, { tools: TOOLS });
  } else if (method === "tools/call") {
    const handler = HANDLERS[params.name];
    if (!handler) { send(id, null, { code: -32601, message: `Unknown tool: ${params.name}` }); return; }
    try {
      const text = await handler(params.arguments ?? {});
      send(id, { content: [{ type: "text", text }] });
    } catch (e) {
      send(id, { content: [{ type: "text", text: JSON.stringify({ error: e.message }) }] });
    }
  } else if (method === "ping") {
    send(id, {});
  } else if (id != null) {
    send(id, null, { code: -32601, message: `Method not found: ${method}` });
  }
});
