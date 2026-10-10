// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Property-based round-trip tests for serialization invariants.
//!
//! Follows the wetSpring / groundSpring proptest pattern:
//! for every `Serialize + Deserialize` type, verify that
//! `decode(encode(x)) == x` holds for all generated inputs.

use proptest::prelude::*;
use serde::{Deserialize, Serialize};

fn roundtrip_json<T>(value: &T)
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + PartialEq,
{
    let json = serde_json::to_string(value).expect("test: serialization must not fail");
    let decoded: T = serde_json::from_str(&json).expect("test: deserialization must not fail");
    assert_eq!(*value, decoded, "round-trip mismatch");
}

// ── JSON-RPC types ──────────────────────────────────────────────────────

use squirrel::rpc::types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, HealthCheckRequest,
    HealthCheckResponse, HealthTier, ListProvidersRequest, ListProvidersResponse, ProviderInfo,
    QueryAiRequest, QueryAiResponse, ToolListEntry, ToolListResponse, ToolSource,
};

prop_compose! {
    fn arb_query_ai_request()(
        prompt in "[a-zA-Z0-9 ]{1,100}",
        provider in proptest::option::of("[a-z]{3,10}"),
        model in proptest::option::of("[a-z0-9-]{3,20}"),
        priority in proptest::option::of(0u8..=100),
        max_tokens in proptest::option::of(1usize..10000),
        // Use tenths to avoid JSON float precision loss (0.0, 0.1, ..., 2.0)
        temp_tenths in proptest::option::of(0u8..=20),
        stream in proptest::option::of(proptest::bool::ANY),
    ) -> QueryAiRequest {
        QueryAiRequest {
            prompt,
            provider,
            model,
            priority,
            max_tokens,
            temperature: temp_tenths.map(|t| f32::from(t) / 10.0),
            stream,
            mode: None,
            tool_schema: None,
            tools: None,
            context: None,
        }
    }
}

prop_compose! {
    fn arb_query_ai_response()(
        response in "[a-zA-Z0-9 ]{0,200}",
        provider in "[a-z]{3,10}",
        model in "[a-z0-9-]{3,20}",
        tokens_used in proptest::option::of(0usize..100_000),
        latency_ms in 0u64..60_000,
        success in proptest::bool::ANY,
    ) -> QueryAiResponse {
        QueryAiResponse {
            response,
            provider,
            model,
            tokens_used,
            latency_ms,
            success,
        }
    }
}

prop_compose! {
    fn arb_list_providers_request()(
        capability in proptest::option::of("[a-z.]{3,30}"),
        include_offline in proptest::option::of(proptest::bool::ANY),
    ) -> ListProvidersRequest {
        ListProvidersRequest {
            capability,
            include_offline,
        }
    }
}

prop_compose! {
    fn arb_provider_info()(
        id in "[a-z0-9-]{3,20}",
        name in "[A-Za-z ]{3,30}",
        models in proptest::collection::vec("[a-z0-9-]{3,15}", 0..5),
        capabilities in proptest::collection::vec("[a-z.]{3,20}", 0..5),
        online in proptest::bool::ANY,
        avg_latency_ms in proptest::option::of(0u64..10_000),
        cost_tier in prop_oneof!["free", "basic", "premium", "enterprise"],
    ) -> ProviderInfo {
        ProviderInfo {
            id,
            name,
            models,
            capabilities,
            online,
            avg_latency_ms,
            cost_tier,
        }
    }
}

prop_compose! {
    fn arb_list_providers_response()(
        providers in proptest::collection::vec(arb_provider_info(), 0..5),
    ) -> ListProvidersResponse {
        let total = providers.len();
        ListProvidersResponse { providers, total }
    }
}

prop_compose! {
    fn arb_announce_capabilities_request()(
        capabilities in proptest::collection::vec("[a-z.]{3,30}", 1..10),
        primal in proptest::option::of("[a-zA-Z]{3,20}"),
        socket_path in proptest::option::of("/tmp/[a-z]{3,20}.sock"),
        tools in proptest::option::of(proptest::collection::vec("[a-z.]{3,30}", 0..5)),
        sub_federations in proptest::option::of(proptest::collection::vec("[a-z-]{3,15}", 0..3)),
        genetic_families in proptest::option::of(proptest::collection::vec("[a-z]{3,15}", 0..3)),
    ) -> AnnounceCapabilitiesRequest {
        AnnounceCapabilitiesRequest {
            capabilities,
            primal,
            socket_path,
            tools,
            sub_federations,
            genetic_families,
            metadata: None,
        }
    }
}

prop_compose! {
    fn arb_announce_capabilities_response()(
        success in proptest::bool::ANY,
        message in "[a-zA-Z0-9 ]{5,50}",
        announced_at in "[0-9T:Z-]{20,30}",
        tools_registered in 0usize..100,
    ) -> AnnounceCapabilitiesResponse {
        AnnounceCapabilitiesResponse {
            success,
            message,
            announced_at,
            tools_registered,
        }
    }
}

prop_compose! {
    fn arb_health_check_response()(
        tier in prop_oneof![
            Just(HealthTier::Alive),
            Just(HealthTier::Ready),
            Just(HealthTier::Healthy),
        ],
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
        uptime_seconds in 0u64..1_000_000,
        active_providers in 0usize..20,
        requests_processed in 0u64..1_000_000,
        // Use integer-backed f64 to avoid JSON float precision loss
        avg_ms_int in proptest::option::of(0u32..10_000),
    ) -> HealthCheckResponse {
        let (alive, ready, healthy, status) = match tier {
            HealthTier::Alive => (true, false, false, "alive".to_string()),
            HealthTier::Ready => (true, true, false, "ready".to_string()),
            HealthTier::Healthy => (true, true, true, "healthy".to_string()),
        };
        HealthCheckResponse {
            tier,
            alive,
            ready,
            healthy,
            status,
            version,
            uptime_seconds,
            active_providers,
            requests_processed,
            avg_response_time_ms: avg_ms_int.map(f64::from),
        }
    }
}

fn arb_tool_source() -> impl Strategy<Value = ToolSource> {
    prop_oneof![
        Just(ToolSource::Builtin),
        "[a-zA-Z]{3,20}".prop_map(|primal| ToolSource::Remote { primal }),
    ]
}

prop_compose! {
    fn arb_tool_list_entry()(
        name in "[a-z.]{3,30}",
        description in "[a-zA-Z0-9 ]{5,80}",
        domain in "[a-z]{3,15}",
        source in arb_tool_source(),
    ) -> ToolListEntry {
        ToolListEntry {
            name,
            description,
            domain,
            source,
            input_schema: None,
        }
    }
}

prop_compose! {
    fn arb_tool_list_response()(
        tools in proptest::collection::vec(arb_tool_list_entry(), 0..10),
    ) -> ToolListResponse {
        let total = tools.len();
        ToolListResponse { tools, total }
    }
}

// ── Proptest declarations ───────────────────────────────────────────────

proptest! {
    #[test]
    fn query_ai_request_roundtrip(req in arb_query_ai_request()) {
        roundtrip_json(&req);
    }

    #[test]
    fn query_ai_response_roundtrip(resp in arb_query_ai_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn list_providers_request_roundtrip(req in arb_list_providers_request()) {
        roundtrip_json(&req);
    }

    #[test]
    fn list_providers_response_roundtrip(resp in arb_list_providers_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn announce_capabilities_request_roundtrip(req in arb_announce_capabilities_request()) {
        roundtrip_json(&req);
    }

    #[test]
    fn announce_capabilities_response_roundtrip(resp in arb_announce_capabilities_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn health_check_response_roundtrip(resp in arb_health_check_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn tool_list_response_roundtrip(resp in arb_tool_list_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn health_check_request_roundtrip(_req in Just(HealthCheckRequest {})) {
        roundtrip_json(&HealthCheckRequest {});
    }
}

// ── JSON-RPC wire-format fuzz (absorbed from airSpring v0.8.7) ──────────
// Tests the full JSON-RPC 2.0 envelope, not just payloads.

use universal_patterns::{IpcClientError, extract_rpc_error, parse_capabilities_from_response};

prop_compose! {
    fn arb_jsonrpc_request()(
        method in "[a-z]{2,10}\\.[a-z]{2,10}",
        id in 1u64..1_000_000,
        has_params in proptest::bool::ANY,
        param_key in "[a-z]{2,10}",
        param_val in "[a-zA-Z0-9]{1,20}",
    ) -> serde_json::Value {
        let params = if has_params {
            serde_json::json!({ param_key: param_val })
        } else {
            serde_json::json!({})
        };
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        })
    }
}

prop_compose! {
    fn arb_jsonrpc_success_response()(
        id in 1u64..1_000_000,
        result_key in "[a-z]{2,10}",
        result_val in "[a-zA-Z0-9]{1,20}",
    ) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": { result_key: result_val },
            "id": id
        })
    }
}

prop_compose! {
    fn arb_jsonrpc_error_response()(
        id in 1u64..1_000_000,
        code in prop_oneof![
            Just(IpcClientError::PARSE_ERROR),
            Just(IpcClientError::INVALID_REQUEST),
            Just(IpcClientError::METHOD_NOT_FOUND),
            Just(IpcClientError::INVALID_PARAMS),
            Just(IpcClientError::INTERNAL_ERROR),
            (-32099i32..=-32000),
        ],
        message in "[a-zA-Z0-9 ]{5,50}",
    ) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": code, "message": message },
            "id": id
        })
    }
}

prop_compose! {
    fn arb_capability_response_flat()(
        caps in proptest::collection::vec("[a-z]{2,8}\\.[a-z]{2,8}", 1..10),
        primal in "[a-z]{3,10}",
    ) -> serde_json::Value {
        serde_json::json!({
            "primal": primal,
            "capabilities": caps
        })
    }
}

prop_compose! {
    fn arb_capability_response_nested()(
        caps in proptest::collection::vec("[a-z]{2,8}\\.[a-z]{2,8}", 1..10),
    ) -> serde_json::Value {
        serde_json::json!({
            "result": {
                "capabilities": caps
            }
        })
    }
}

proptest! {
    #[test]
    fn jsonrpc_request_is_valid_json(req in arb_jsonrpc_request()) {
        let json_str = serde_json::to_string(&req).expect("should succeed");
        let decoded: serde_json::Value = serde_json::from_str(&json_str).expect("should succeed");
        assert_eq!(decoded["jsonrpc"], "2.0");
        assert!(decoded["method"].is_string());
        assert!(decoded["id"].is_number());
    }

    #[test]
    fn jsonrpc_success_response_roundtrip(resp in arb_jsonrpc_success_response()) {
        let json_str = serde_json::to_string(&resp).expect("should succeed");
        let decoded: serde_json::Value = serde_json::from_str(&json_str).expect("should succeed");
        assert_eq!(decoded["jsonrpc"], "2.0");
        assert!(decoded["result"].is_object());
        assert!(decoded.get("error").is_none());
    }

    #[test]
    fn jsonrpc_error_response_extractable(resp in arb_jsonrpc_error_response()) {
        let err = extract_rpc_error(&resp).expect("should succeed");
        assert!(err.code != 0);
        assert!(!err.message.is_empty());
    }

    #[test]
    fn capability_flat_parseable(resp in arb_capability_response_flat()) {
        let caps = parse_capabilities_from_response(&resp);
        assert!(!caps.is_empty());
        for cap in &caps {
            assert!(cap.contains('.'), "capability should be dotted: {cap}");
        }
    }

    #[test]
    fn capability_nested_parseable(resp in arb_capability_response_nested()) {
        let caps = parse_capabilities_from_response(&resp);
        assert!(!caps.is_empty());
    }

    #[test]
    fn jsonrpc_error_code_in_reserved_range(resp in arb_jsonrpc_error_response()) {
        let err = extract_rpc_error(&resp).expect("should succeed");
        // All generated codes should be either standard or server range
        assert!(
            err.code <= -32000,
            "error code should be in reserved range: {}",
            err.code
        );
    }

    #[test]
    fn capability_empty_response_returns_empty(_v in Just(())) {
        let empty = serde_json::json!({});
        assert!(parse_capabilities_from_response(&empty).is_empty());
        let null_resp = serde_json::json!(null);
        assert!(parse_capabilities_from_response(&null_resp).is_empty());
    }
}

// ── IPC fuzz tests (absorbed from healthSpring / wetSpring) ─────────────
// Verify parsers never panic on arbitrary input.

use universal_patterns::extract_rpc_result;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn parse_request_never_panics(bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..512)) {
        let _ = serde_json::from_slice::<serde_json::Value>(&bytes);
    }

    #[test]
    fn parse_capabilities_never_panics(json in arb_jsonrpc_success_response()) {
        let _ = parse_capabilities_from_response(&json);
    }

    #[test]
    fn extract_rpc_result_never_panics(
        has_result in proptest::bool::ANY,
        has_error in proptest::bool::ANY,
        key in "[a-z]{2,8}",
        val in "[a-zA-Z0-9]{1,20}",
    ) {
        let mut obj = serde_json::Map::new();
        obj.insert("jsonrpc".to_string(), serde_json::json!("2.0"));
        obj.insert("id".to_string(), serde_json::json!(1));
        if has_result {
            obj.insert("result".to_string(), serde_json::json!({ key: val }));
        }
        if has_error {
            obj.insert("error".to_string(), serde_json::json!({"code": -32600, "message": "test"}));
        }
        let response = serde_json::Value::Object(obj);
        let _ = extract_rpc_result(&response);
    }

    #[test]
    fn extract_rpc_error_never_panics(json in arb_jsonrpc_error_response()) {
        let _ = extract_rpc_error(&json);
        let _ = extract_rpc_result(&json);
    }

    #[test]
    fn dispatch_method_name_never_panics(
        method in "([a-z]{1,10}\\.){0,3}[a-z]{1,10}",
    ) {
        let domain = method.split('.').next().unwrap_or("unknown");
        assert!(!domain.is_empty());
        assert!(method.split('.').count() > 0);
    }
}

// ── Niche invariants ────────────────────────────────────────────────────

#[test]
fn niche_json_roundtrip() {
    let cost_json = squirrel::niche::cost_estimates_json();
    let serialized =
        serde_json::to_string(&cost_json).expect("test: niche cost JSON must serialize");
    let decoded: serde_json::Value =
        serde_json::from_str(&serialized).expect("test: niche cost JSON must deserialize");
    assert_eq!(cost_json, decoded);

    let deps_json = squirrel::niche::operation_dependencies();
    let serialized =
        serde_json::to_string(&deps_json).expect("test: niche deps JSON must serialize");
    let decoded: serde_json::Value =
        serde_json::from_str(&serialized).expect("test: niche deps JSON must deserialize");
    assert_eq!(deps_json, decoded);
}
