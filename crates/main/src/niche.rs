// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Niche self-knowledge for the Squirrel AI primal.
//!
//! Follows the ecoPrimals niche pattern established by groundSpring, wetSpring,
//! and airSpring. Every primal defines its self-knowledge in a single module so
//! that biomeOS, the service mesh, and the Pathway Learner can reason about it without
//! hardcoded primal names or port numbers.
//!
//! This module holds:
//! - Identity (who am I?)
//! - Capabilities (what do I expose via biomeOS?)
//! - Semantic mappings (capability domain → handler methods)
//! - Consumed capabilities (what do I need from other primals?)
//! - Dependencies (what primals must be running for me to function?)
//! - Cost estimates (scheduling hints for biomeOS Pathway Learner)
//! - Operation dependencies (parallelization DAG for Pathway Learner)
//!
//! Other modules reference these constants rather than duplicating string
//! literals. Squirrel only knows itself — it discovers other primals at
//! runtime via capability-based discovery.


/// Primal identity — used in all JSON-RPC, IPC, and biomeOS interactions.
pub const PRIMAL_ID: &str = "squirrel";

/// Human-readable description for biomeOS registration.
pub const PRIMAL_DESCRIPTION: &str = "Universal AI coordination and MCP routing primal";

/// Primary capability domain.
pub const DOMAIN: &str = "ai";

/// Primal version (tracks crate version).
pub const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// SPDX license identifier.
pub const LICENSE: &str = "AGPL-3.0-or-later";

/// IPC transport mechanism.
pub const TRANSPORT: &str = "unix_socket";

/// Wire protocol.
pub const PROTOCOL: &str = universal_constants::protocol::JSONRPC_PROTOCOL_ID;

/// Human-readable descriptions for each capability domain group.
pub const CAPABILITY_GROUP_DESCRIPTIONS: &[(&str, &str)] = &[
    ("ai", "AI inference and text generation"),
    ("signal", "Signal graph planning and dispatch"),
    ("inference", "Vendor-agnostic model inference"),
    ("capabilities", "Capability announcement and discovery"),
    ("capability", "Capability announcement and discovery"),
    ("health", "Health probes and readiness checks"),
    ("system", "System monitoring and diagnostics"),
    ("identity", "Primal identity"),
    ("discovery", "Peer discovery"),
    ("tool", "Tool orchestration and execution"),
    ("context", "Context management"),
    ("provider", "Spring provider registration"),
    ("btsp", "Transport security negotiation"),
    ("lifecycle", "biomeOS lifecycle management"),
    ("graph", "Deployment graph parsing and validation"),
    ("membrane", "Membrane observation from announced peers"),
    ("provenance", "Provenance DAG proxy"),
    ("dag", "DAG provenance operations"),
    ("anchoring", "Merkle anchoring operations"),
    ("attribution", "Attribution calculations"),
];

/// All capabilities this primal exposes to biomeOS.
///
/// Each string is a fully qualified capability name (`{domain}.{method}`)
/// that biomeOS can route via `capability.call`.
pub const CAPABILITIES: &[&str] = &[
    // AI inference (legacy domain)
    "ai.query",
    "ai.complete",
    "ai.chat",
    "ai.list_providers",
    // Inference domain — vendor-agnostic wire standard (ecoPrimal)
    "inference.complete",
    "inference.embed",
    "inference.models",
    "inference.register_provider",
    "inference.unregister_provider",
    // Capability routing (capabilities.list is canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.1)
    "capabilities.announce",
    "capabilities.list",
    "capability.announce",
    "capability.discover",
    "capability.list",
    "primal.announce",
    // Health probes — canonical per PRIMAL_IPC_PROTOCOL v3.0
    // NOTE: bare "health" is in the dispatch table and capability_registry.toml
    // but excluded here because CAPABILITIES require domain.method format.
    "health.check",
    "health.liveness",
    "health.readiness",
    // System monitoring (backward-compat aliases — prefer health.*)
    "system.health",
    "system.status",
    "system.metrics",
    "system.ping",
    // Identity (CAPABILITY_BASED_DISCOVERY_STANDARD v1.0)
    "identity.get",
    // Peer discovery
    "discovery.peers",
    // Signal graph dispatch — G18 spring orchestration
    "signal.plan",
    "signal.dispatch",
    // Tool orchestration
    "tool.execute",
    "tool.list",
    // Context management
    "context.create",
    "context.update",
    "context.summarize",
    // Provider registration — springs register capabilities with Squirrel
    "provider.register",
    "provider.list",
    "provider.deregister",
    // BTSP Phase 3
    "btsp.negotiate",
    // Lifecycle (biomeOS)
    "lifecycle.register",
    "lifecycle.status",
    // Graph introspection (primalSpring BYOB)
    "graph.parse",
    "graph.validate",
    // Membrane observation — read-only view of announced peer observations
    "membrane.status",
];

/// Semantic mappings: short operation name → fully qualified capability.
///
/// biomeOS uses these during domain registration so
/// `capability.call { domain: "ai", operation: "query" }` routes to
/// the correct JSON-RPC method on our socket.
pub const SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    ("query", "ai.query"),
    ("complete", "ai.complete"),
    ("chat", "ai.chat"),
    ("list_providers", "ai.list_providers"),
    ("plan", "signal.plan"),
    ("dispatch", "signal.dispatch"),
    ("announce", "capability.announce"),
    ("discover", "capability.discover"),
    ("list_capabilities", "capabilities.list"),
    ("health_check", "health.check"),
    ("liveness", "health.liveness"),
    ("readiness", "health.readiness"),
    ("health", "system.health"),
    ("status", "system.status"),
    ("metrics", "system.metrics"),
    ("ping", "system.ping"),
    ("identity", "identity.get"),
    ("peers", "discovery.peers"),
    ("execute", "tool.execute"),
    ("list", "tool.list"),
    ("create", "context.create"),
    ("update", "context.update"),
    ("summarize", "context.summarize"),
    ("register", "lifecycle.register"),
    ("parse_graph", "graph.parse"),
    ("validate_graph", "graph.validate"),
];

/// Consumed capabilities — what Squirrel calls on other primals.
///
/// Squirrel discovers these at runtime via capability discovery; it never hardcodes
/// which primal provides them. The Pathway Learner uses this list to
/// ensure required capabilities are available before routing to Squirrel.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    // Security domain (crypto, auth, secrets)
    "crypto.sign",
    "crypto.verify",
    "auth.validate_token",
    "secrets.store",
    "secrets.retrieve",
    "secrets.list",
    "secrets.delete",
    // Discovery domain (service mesh)
    "discovery.register",
    "discovery.find_primals",
    "discovery.query",
    // Compute domain (GPU dispatch, hardware)
    "compute.execute",
    "compute.submit",
    "compute.dispatch.submit",
    "compute.dispatch.status",
    "compute.dispatch.result",
    "compute.dispatch.capabilities",
    "compute.dispatch.cancel",
    "compute.hardware.observe",
    // Storage domain (persistence, model cache)
    "storage.put",
    "storage.get",
    "storage.list",
    "model.register",
    "model.locate",
    "model.metadata",
    "model.exists",
    // MCP tool discovery
    "mcp.tools.list",
    // Health probes (verify peer readiness before routing)
    "health.liveness",
    "health.readiness",
    // Relay domain (authorization, status)
    "relay.authorize",
    "relay.status",
    // DAG domain (sessions, events, vertices)
    "dag.session.create",
    "dag.event.append",
    "dag.vertex.query",
    // Provenance domain (anchoring, attribution)
    "anchoring.anchor",
    "anchoring.verify",
    "attribution.calculate_rewards",
    // Coordination domain (composition, deployment)
    "coordination.validate_composition",
    "coordination.deploy_atomic",
    "composition.nucleus_health",
    // IPC registration (biomeOS service mesh)
    "ipc.register",
];

/// Capability-domain dependencies for deployment.
///
/// Each entry: `(capability_domain, required, description)`.
/// `required = true` means Squirrel cannot function without it.
/// `required = false` means graceful degradation is supported.
///
/// Squirrel discovers providers for each capability domain at runtime via
/// `capabilities.discover` — it does not encode which primal fulfills each
/// domain. The same domain may be served by different primals per gate.
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    (
        "crypto",
        true,
        "cryptographic identity and trust",
    ),
    (
        "discovery",
        true,
        "service discovery and IPC mesh",
    ),
    (
        "compute",
        false,
        "GPU compute dispatch (graceful fallback to CPU-only inference)",
    ),
    (
        "storage",
        false,
        "persistent storage (graceful fallback to in-memory cache)",
    ),
    (
        "coordination",
        false,
        "coordination validation and BYOB graph execution",
    ),
    (
        "visualization",
        false,
        "visualization and user interface rendering",
    ),
];

/// Cost estimates for biomeOS Pathway Learner scheduling.
///
/// Each entry: `(capability, estimated_ms, gpu_beneficial)`.
/// Times are representative for typical workloads. The Pathway Learner
/// uses these to make intelligent routing decisions.
pub const COST_ESTIMATES: &[(&str, u32, bool)] = &[
    ("ai.query", 500, true),
    ("ai.complete", 500, true),
    ("ai.chat", 800, true),
    ("ai.list_providers", 1, false),
    ("inference.complete", 500, true),
    ("inference.embed", 300, true),
    ("inference.models", 1, false),
    ("inference.register_provider", 10, false),
    ("inference.unregister_provider", 5, false),
    ("capabilities.announce", 2, false),
    ("capabilities.list", 1, false),
    ("capability.announce", 2, false),
    ("capability.discover", 1, false),
    ("capability.list", 1, false),
    ("primal.announce", 2, false),
    ("health.check", 1, false),
    ("health.liveness", 1, false),
    ("health.readiness", 2, false),
    ("system.health", 1, false),
    ("system.status", 1, false),
    ("system.metrics", 5, false),
    ("system.ping", 1, false),
    ("identity.get", 1, false),
    ("discovery.peers", 50, false),
    ("signal.plan", 3000, true),
    ("signal.dispatch", 500, false),
    ("tool.execute", 200, false),
    ("tool.list", 1, false),
    ("context.create", 5, false),
    ("context.update", 5, false),
    ("context.summarize", 300, true),
    ("provider.register", 10, false),
    ("provider.list", 1, false),
    ("provider.deregister", 5, false),
    ("btsp.negotiate", 50, false),
    ("lifecycle.register", 10, false),
    ("lifecycle.status", 1, false),
    ("graph.parse", 5, false),
    ("graph.validate", 50, false),
    ("membrane.status", 1, false),
];

/// Operation dependency hints for biomeOS Pathway Learner parallelization.
///
/// Maps each operation to the data inputs it requires, enabling the Pathway
/// Learner to build a DAG and parallelize independent operations.
#[must_use]
pub fn operation_dependencies() -> serde_json::Value {
    serde_json::json!({
        "ai.query": ["prompt"],
        "ai.complete": ["prompt"],
        "ai.chat": ["prompt"],
        "ai.list_providers": [],
        "inference.complete": ["prompt"],
        "inference.embed": ["input"],
        "inference.models": [],
        "inference.register_provider": ["provider_id"],
        "inference.unregister_provider": ["provider_id"],
        "signal.plan": ["prompt", "tools"],
        "signal.dispatch": ["signal", "params"],
        "capabilities.announce": ["capabilities"],
        "capabilities.list": [],
        "capability.announce": ["capabilities", "primal"],
        "capability.discover": [],
        "capability.list": [],
        "primal.announce": ["capabilities"],
        "health.check": [],
        "health.liveness": [],
        "health.readiness": [],
        "system.health": [],
        "system.status": [],
        "system.metrics": [],
        "system.ping": [],
        "identity.get": [],
        "discovery.peers": [],
        "tool.execute": ["tool", "args"],
        "tool.list": [],
        "context.create": [],
        "context.update": ["id", "data"],
        "context.summarize": ["id"],
        "provider.register": ["provider_id", "capabilities"],
        "provider.list": [],
        "provider.deregister": ["provider_id"],
        "btsp.negotiate": ["session_id"],
        "lifecycle.register": [],
        "lifecycle.status": [],
        "graph.parse": ["graph_toml"],
        "graph.validate": ["graph_toml"],
        "membrane.status": [],
    })
}

/// Structured cost estimates as JSON for `capability.list` responses.
///
/// Derived from `COST_ESTIMATES` with CPU load heuristics.
#[must_use]
pub fn cost_estimates_json() -> serde_json::Value {
    let mut map = serde_json::Map::with_capacity(COST_ESTIMATES.len());
    for &(cap, ms, gpu) in COST_ESTIMATES {
        let cpu = if ms >= 300 { "medium" } else { "low" };
        let mem: u64 = match ms {
            0..=2 => 256,
            3..=10 => 512,
            11..=100 => 2048,
            101..=500 => 8192,
            _ => 32768,
        };
        map.insert(
            cap.to_string(),
            serde_json::json!({
                "latency_ms": ms,
                "cpu": cpu,
                "memory_bytes": mem,
                "gpu_beneficial": gpu,
            }),
        );
    }
    serde_json::Value::Object(map)
}

/// Semantic mappings as JSON for biomeOS Neural API routing.
#[must_use]
pub fn semantic_mappings_json() -> serde_json::Value {
    serde_json::json!({
        "query":          "ai.query",
        "complete":       "ai.complete",
        "chat":           "ai.chat",
        "list_providers": "ai.list_providers",
        "plan":           "signal.plan",
        "dispatch":       "signal.dispatch",
        "announce":       "capability.announce",
        "discover":       "capability.discover",
        "list_capabilities": "capabilities.list",
        "health_check":   "health.check",
        "liveness":       "health.liveness",
        "readiness":      "health.readiness",
        "health":         "system.health",
        "status":         "system.status",
        "metrics":        "system.metrics",
        "ping":           "system.ping",
        "identity":       "identity.get",
        "peers":          "discovery.peers",
        "execute":        "tool.execute",
        "list":           "tool.list",
        "create":         "context.create",
        "update":         "context.update",
        "summarize":      "context.summarize",
        "register":       "lifecycle.register",
        "parse_graph":    "graph.parse",
        "validate_graph": "graph.validate",
    })
}

/// Number of required dependencies.
#[must_use]
pub const fn required_dependency_count() -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < DEPENDENCIES.len() {
        if DEPENDENCIES[i].1 {
            count += 1;
        }
        i += 1;
    }
    count
}

/// Feature gates that expand primal capabilities.
pub const FEATURE_GATES: &[(&str, &str)] = &[
    (
        "direct-http",
        "Direct HTTP AI provider access (dev/testing)",
    ),
    ("marketplace", "Plugin marketplace integration"),
    ("monitoring", "Prometheus-compatible metrics export"),
    ("ecosystem", "Full ecosystem manager with federation"),
    ("nvml", "NVIDIA GPU detection via NVML"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_are_fully_qualified() {
        for cap in CAPABILITIES {
            assert!(
                cap.contains('.'),
                "capability {cap} must be domain.method format"
            );
        }
    }

    #[test]
    fn semantic_mappings_reference_valid_capabilities() {
        for (_, fqn) in SEMANTIC_MAPPINGS {
            assert!(
                CAPABILITIES.contains(fqn),
                "semantic mapping target {fqn} not in CAPABILITIES"
            );
        }
    }

    #[test]
    fn cost_estimates_cover_all_capabilities() {
        let costed: Vec<&str> = COST_ESTIMATES.iter().map(|(c, _, _)| *c).collect();
        for cap in CAPABILITIES {
            assert!(
                costed.contains(cap),
                "capability {cap} missing from COST_ESTIMATES"
            );
        }
    }

    #[test]
    fn required_dependency_count_is_correct() {
        let manual = DEPENDENCIES.iter().filter(|(_, req, _)| *req).count();
        assert_eq!(required_dependency_count(), manual);
    }

    #[test]
    fn operation_dependencies_covers_capabilities() {
        let deps = operation_dependencies();
        let map = deps.as_object().expect("should be an object");
        for cap in CAPABILITIES {
            assert!(
                map.contains_key(*cap),
                "capability {cap} missing from operation_dependencies()"
            );
        }
    }

    #[test]
    fn cost_estimates_json_covers_capabilities() {
        let costs = cost_estimates_json();
        let map = costs.as_object().expect("should be an object");
        for cap in CAPABILITIES {
            assert!(
                map.contains_key(*cap),
                "capability {cap} missing from cost_estimates_json()"
            );
        }
    }

    #[test]
    fn consumed_capabilities_are_fully_qualified() {
        for cap in CONSUMED_CAPABILITIES {
            assert!(
                cap.contains('.'),
                "consumed capability {cap} must be domain.method format"
            );
        }
    }

    #[test]
    fn primal_identity_constants() {
        assert_eq!(PRIMAL_ID, "squirrel");
        assert_eq!(DOMAIN, "ai");
        assert_eq!(TRANSPORT, "unix_socket");
        assert_eq!(PROTOCOL, universal_constants::protocol::JSONRPC_PROTOCOL_ID);
        assert_eq!(LICENSE, "AGPL-3.0-or-later");
    }

    #[test]
    fn identity_primal_domain_matches_niche_domain() {
        assert_eq!(
            universal_constants::identity::PRIMAL_DOMAIN,
            DOMAIN,
            "identity::PRIMAL_DOMAIN must match niche::DOMAIN"
        );
    }

    #[test]
    fn capability_registry_toml_sync() {
        let toml_str = include_str!("../../../config/capability_registry.toml");
        let toml: toml::Value = toml_str.parse().expect("valid TOML");
        let caps_table = toml
            .get("capabilities")
            .and_then(toml::Value::as_table)
            .expect("capabilities table");

        let registry_methods: std::collections::BTreeSet<String> = caps_table
            .values()
            .filter_map(|v: &toml::Value| {
                v.get("method")
                    .and_then(toml::Value::as_str)
                    .filter(|m| m.contains('.'))
                    .map(String::from)
            })
            .collect();

        let niche_methods: std::collections::BTreeSet<String> =
            CAPABILITIES.iter().map(|s| (*s).to_string()).collect();

        let missing_from_toml: Vec<_> = niche_methods.difference(&registry_methods).collect();
        let missing_from_niche: Vec<_> = registry_methods.difference(&niche_methods).collect();

        assert!(
            missing_from_toml.is_empty(),
            "niche::CAPABILITIES has methods not in capability_registry.toml: {missing_from_toml:?}"
        );
        assert!(
            missing_from_niche.is_empty(),
            "capability_registry.toml has methods not in niche::CAPABILITIES: {missing_from_niche:?}"
        );
    }

    #[test]
    fn semantic_mappings_json_matches_semantic_mappings_table() {
        let json = semantic_mappings_json();
        let map = json.as_object().expect("object");
        for (short, expected) in SEMANTIC_MAPPINGS {
            assert_eq!(
                map.get(*short).and_then(|v| v.as_str()),
                Some(*expected),
                "short key {short} should map to {expected}"
            );
        }
    }

    #[test]
    fn feature_gates_are_non_empty_descriptions() {
        for (gate, desc) in FEATURE_GATES {
            assert!(!gate.is_empty());
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn cost_estimates_json_has_latency_and_gpu_keys() {
        let costs = cost_estimates_json();
        let map = costs.as_object().expect("object");
        let q = map.get("ai.query").expect("ai.query");
        let obj = q.as_object().expect("inner object");
        assert!(obj.contains_key("latency_ms"));
        assert!(obj.contains_key("gpu_beneficial"));
    }

    #[test]
    fn consumed_capabilities_include_compute_and_storage() {
        let joined = CONSUMED_CAPABILITIES.join(" ");
        assert!(joined.contains("compute.execute"));
        assert!(joined.contains("storage.get"));
    }

    #[test]
    fn dependencies_crypto_and_discovery_required() {
        let crypto = DEPENDENCIES.iter().find(|(id, _, _)| *id == "crypto");
        let discovery = DEPENDENCIES.iter().find(|(id, _, _)| *id == "discovery");
        assert_eq!(crypto.map(|(_, r, _)| *r), Some(true));
        assert_eq!(discovery.map(|(_, r, _)| *r), Some(true));
    }
}
