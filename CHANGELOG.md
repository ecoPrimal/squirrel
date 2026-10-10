# squirrel — Evolution Record

The changelog is sorted by capability surface, not by date.
Each section tells the arc: what emerged, what it replaced, what it became.
The date-sorted history lives in `CHANGELOG_ARCHIVE.md`.

`H(data|epitope) < H(data|date) < H(data)`

---

## AI Routing & Inference — model coordination and provider dispatch

Squirrel began as a monolithic ecosystem scaffold; it became a focused AI coordination primal routing inference, embedding, and tool calls through registered providers.

- **AiRouter** (Wave 119→156): Central routing with `inference.complete`, `inference.embed`, `inference.models`; upsert semantics on `inference.register_provider`.
- **Remote provider adapters** (Wave 141→150u): UDS JSON-RPC and HTTP (Ollama-compatible REST) providers; `endpoint` param auto-promotion from legacy `socket` field.
- **Provider registration protocol** (Wave 156c): `provider.register/list/deregister` — springs register capabilities and socket paths at runtime via JSON-RPC.
- **Inference auto-discovery** (Wave 124): `INFERENCE_ENDPOINT`/`AI_INFERENCE_ENDPOINT` env auto-registers `RemoteInferenceAdapter` at startup.
- **Security-wrapped inference** (Wave 141): `SecurityProviderClient` purpose-key RPC foundation for encrypting prompts/responses via crypto capability provider.
- **Shared ContextManager** (Wave 119): `JsonRpcServer.context_manager` field — multi-request context flow persists across connection lifetime.
- **Signal graph dispatch** (Wave 155p): G18 signal graph routing wired into dispatch surface.

---

## MCP Server & Client — Model Context Protocol surface

MCP emerged as Squirrel's primary external agent interface, evolving from lying stubs to honest error semantics and full tool/resource/prompt routing.

- **MCP JSON-RPC server** (Wave 116→157): Full tool execution, resource listing, prompt handling over stdio and socket transports.
- **SDK honesty evolution** (Wave 150→156): `list_tools/resources/prompts` return `Err(McpError)` when transport unwired — callers distinguish "none" from "unavailable".
- **MCP auth crate** (Wave 157g): Dead deps excised (`blake3`, `rand`, `squirrel-mcp-config`); tokio aligned to workspace.
- **AtomicBool run flags** (Wave 157g): MCP server/client/listener mutex-guarded booleans → lock-free `AtomicBool` + `Ordering::AcqRel`.
- **Inference UDS timeout** (Wave 124): 120s read timeout for LLM calls vs 2s lifecycle heartbeat — prevents neuralSpring timeout failures.

---

## Context Management — session state and lifecycle

Context handling evolved from per-request instantiation to persistent shared state with plugin-aware permission enforcement.

- **Context lifecycle** (Wave 119→156): `context.create/update/summarize/get/delete` with UUID persistence across requests on same connection.
- **Permission enforcement** (Wave 114): `is_path_allowed` evolved from permissive stub to manifest-based `FileSystemRead`/`FileSystemWrite` validation — default-deny.
- **Context-learning deprecation** (Wave 157g): ~14.6k LOC RL/neural scaffold feature-gated OFF; RL belongs to compute primals, lifecycle stays in squirrel.
- **Replay buffer optimization** (Wave 157g): `sample_uniform()` uses `IteratorRandom::choose_multiple()` — O(k) sampling instead of O(n) buffer clone.

---

## BTSP & Encrypted Sessions — wire security

BTSP evolved from NULL-cipher stubs to full Phase 3 ChaCha20-Poly1305 encrypted framing with transport upgrade after negotiation.

- **BTSP Phase 3 FULL** (Wave 119→157): `btsp_encrypted_framing` module — HKDF-SHA256 key derivation, length-prefixed AEAD frames, seamless NDJSON→encrypted upgrade.
- **Client handshake** (Wave 151b): JSON-line `ClientHello` for bearDog strict mode; binary and JSON-line Phase 2 entry points auto-detected.
- **riboCipher compliance** (Wave 114): Inbound `0xEC`/`0xED`/`0xEE` acceptance + outbound `[0xEC, 0x01]` preamble on all UDS client paths.
- **Binary probe graceful handling** (Wave 141): Non-BTSP first bytes (HTTP, TLS) return `BinaryProbe` error at debug level — no reconnect storm.
- **PG-14 auto-detect** (Wave 107): Plain `{` JSON-RPC fallback on BTSP-guarded sockets; handshake timeout 5s→1.5s with structured error frame on failure.

---

## JSON-RPC Server & Method Gate — the dispatch surface

The server surface matured from scattered handlers to a gated, self-documenting RPC surface with 13/13 stadial compliance.

- **MethodGate JH-0** (Wave 101): Pre-dispatch capability gate — public vs protected method classification; permissive default, enforcing mode available.
- **JsonRpcServer refactor** (Wave 113→157b): Monolithic server split into `jsonrpc_dispatch.rs`, `jsonrpc_request_processing.rs`, `method_gate.rs` — zero files >800L.
- **Provider registry LIVE** (Wave 116): `provider.register/list/deregister` wired to `InMemoryServiceRegistry` with deterministic UUID upsert.
- **Provenance proxy** (Wave 119): `provenance.*`, `dag.*`, `anchoring.*` dynamically routed to discovered primals via capability probe.
- **capabilities.list envelope** (Wave 105): Canonical `{capabilities, count, primal}` shape per CAPABILITY_WIRE_STANDARD.
- **Null params acceptance** (Wave 149b): Handlers accept null params per wire standard — agent compatibility.

---

## Transport & Cross-Platform IPC — getting bytes to peers

Transport evolved from Unix-only hardcoded paths to platform-abstracted endpoints with Windows gating and launcher-injected bind modes.

- **TransportEndpoint injection** (Wave 100→157c): `TRANSPORT_ENDPOINT` env (Tier 0 priority); `connect_transport()` replaces raw `UnixStream` at all call sites.
- **G66 transport abstraction** (Wave 157c): All test `UnixStream`/`UnixListener` imports gated `#[cfg(unix)]` — Windows cross-compile clean.
- **G68 platform substrate** (Wave 157e): `create_capability_alias()` — symlink on Unix, discovery file on Windows; `AccessLevel` enum for permissions.
- **SQ-01 socket path fix** (Wave 54): Filesystem companion socket uses `--socket` CLI path, not env re-derivation — NUCLEUS launcher alignment.
- **Stale socket prevention** (Wave 106→118): Connect-probe liveness (50ms timeout), PID files, unlink-before-bind server hygiene.
- **Fake-async cleanup** (Wave 157g): 15 functions de-asynced; 53 trait-imposed async retained (correct contract).

---

## Capability Discovery & Provider Registry — finding who fulfills what

Discovery transformed from hardcoded primal names to runtime capability-based resolution with multi-method fallback chains.

- **TRUE PRIMAL compliance** (Wave 116): `niche::DEPENDENCIES` (primal IDs) → `niche::REQUIRED_CAPABILITIES` (capability domains); legacy deprecated.
- **Capability-first naming** (Wave 157g): `niche.rs` DEPENDENCIES evolved to `crypto`, `discovery`, `compute`, `storage`, `coordination`, `visualization`.
- **DISCOVERY_SOCKET resolution** (Wave 141): `discover_capability()` queries discovery service via `ipc.find_provider` as Method 2 in fallback chain.
- **Neural API announce** (Wave 43→44): `primal.announce` on startup with routing metadata; tiered socket discovery (`$NEURAL_API_SOCKET` → XDG → `/tmp`).
- **Env var centralization** (Wave 58→116): 316 env constants in `universal-constants/env_vars.rs`; incremental migration from raw `env::var()` calls.
- **Hardcoding elimination** (Wave 155n→157i): Zero cross-primal name literals in production; `niche::PRIMAL_ID` replaces 25+ `"squirrel"` strings.

---

## Security Integration — crypto delegation without coupling

Security evolved from inline BearDog coupling to capability-based crypto provider discovery with honest health probes.

- **Security health probes** (Wave 116): `check_discovered_endpoints` probes real capability-discovery registry for `crypto.*`/`auth.*` providers.
- **CredentialStore integration** (Wave 150u): BearDog `secrets.*` JSON-RPC for platform secret backends (Linux SecretService, Windows DPAPI, Android Keystore).
- **Auth delegation** (Wave 141): Squirrel intentionally delegates auth — `auth.mode` not exposed; callers use security capability provider.
- **DignityGuard** (Wave 116): `SQUIRREL_DIGNITY_ENFORCEMENT` env (warn/enforce/audit) — configurable request dignity enforcement.

---

## Monitoring & Metrics — honest system observation

Metrics evolved from simulated sine waves and hardcoded averages to real `/proc` data and atomic request tracking.

- **Real system metrics** (Wave 116): CPU from `/proc/self/stat`, memory from `/proc/self/statm`, disk/network from `/proc/self/io` and `/proc/net/dev`.
- **RequestTracker** (Wave 119): Atomic counters replace hardcoded `125.3` avg_response_time; `HttpMetrics` wired to real tracker.
- **Monitoring dep excision** (Wave 157g): `prometheus`, `metrics`, `metrics-exporter-prometheus` removed — squirrel uses own exposition format in `exporters.rs`.

---

## Architecture Excision & Sovereignty — becoming a true primal

Squirrel shed ~100k+ lines of absorbed scaffolding to become a focused coordination primal with 12 workspace crates.

- **C8 upstream absorption excision** (Wave 157b): ~67K lines, 216 files — ecosystem/, universal_primal_ecosystem/, compute_client/, storage_client/ removed from production path.
- **Orphan crate removal** (Wave 156y→156z): squirrel-plugins (15.5kL), squirrel-core (13.9kL), squirrel-sdk (11.9kL) — zero reverse dependencies.
- **Self-healing excision** (Wave 157g): 878 LOC always-compiled dead code; ecosystem health via capability IPC (`health.liveness`, `health.readiness`).
- **GPU detection excision** (Wave 157g): 704 LOC feature-gated hardware probe belongs to compute primals; squirrel consumes `compute.hardware.observe` via IPC.
- **G72 dependency pandemic** (Wave 157g): Cargo.lock 384→333 packages; tokio `full` → 9 explicit features; 18 dead deps removed across 6 crates.
- **Config crate excision** (Wave 156w): External `config` 0.14 replaced with hand-rolled TOML/YAML/env loader — 234 lockfile lines removed.
- **Test performance** (Wave 156b): 400s → 16s via sleep elimination, port-0 binds, injectable env readers.
- **Double-lock excision** (Wave 157g): Redundant `Arc<Mutex<CommandRegistry>>` wrapper removed — interior mutability already provided.

---

*Full date-sorted history: `CHANGELOG_ARCHIVE.md`*
*Last compressed: Wave 172*
