// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 wire types, error codes, and server metrics.
//!
//! Extracted from `jsonrpc_server.rs` so the server module focuses on
//! transport / dispatch logic while these types remain reusable across
//! handlers and tests.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

// ── Arc<str> serde helpers (zero-copy for hot-path jsonrpc/method fields) ────

pub(crate) fn serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

pub(crate) fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

// ── JSON-RPC 2.0 DTOs ───────────────────────────────────────────────────────

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0") — `Arc<str>` for zero-copy (always "2.0")
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub jsonrpc: Arc<str>,

    /// Method name — `Arc<str>` for zero-copy (method names reused constantly)
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub method: Arc<str>,

    /// Parameters (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    /// Request ID (optional for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version — `Arc<str>` for zero-copy (always "2.0")
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub jsonrpc: Arc<str>,

    /// Result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request ID (echoed from request)
    pub id: Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("{message} (code {code})")]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// ── Method normalisation ─────────────────────────────────────────────────────

/// Strip known legacy prefixes for ecosystem compatibility
/// (per BearDog v0.9.0, barraCuda v0.3.7 pattern).
///
/// Tracks how many calls arrive with legacy prefixes for migration metrics.
pub(crate) fn normalize_method(method: &str) -> &str {
    match method
        .strip_prefix("squirrel.")
        .or_else(|| method.strip_prefix("mcp."))
    {
        Some(stripped) => {
            LEGACY_PREFIX_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            stripped
        }
        None => method,
    }
}

/// Count of JSON-RPC calls that arrived with a legacy `squirrel.`/`mcp.` prefix.
pub(crate) static LEGACY_PREFIX_HITS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

// ── Error codes ──────────────────────────────────────────────────────────────

/// JSON-RPC error codes (standard + reserved server range)
pub mod error_codes {
    /// Invalid JSON was received by the server.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist or is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Start of reserved implementation-defined server-error range (-32000..=-32099).
    pub const SERVER_ERROR_MIN: i32 = -32099;
    /// End of reserved implementation-defined server-error range.
    pub const SERVER_ERROR_MAX: i32 = -32000;

    /// Request denied by security orchestrator (rate limit, input validation, threat).
    pub const SECURITY_DENIED: i32 = -32003;
}

// ── Server metrics ───────────────────────────────────────────────────────────

/// Server metrics
#[derive(Debug, Clone)]
pub struct ServerMetrics {
    /// Total requests handled
    pub requests_handled: u64,

    /// Total errors
    pub errors: u64,

    /// Server start time
    pub start_time: Instant,

    /// Total response time (for averaging)
    pub total_response_time_ms: u64,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerMetrics {
    /// Creates a new server metrics instance with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            requests_handled: 0,
            errors: 0,
            start_time: Instant::now(),
            total_response_time_ms: 0,
        }
    }

    /// Returns the server uptime in seconds.
    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Returns the average response time in milliseconds, if any requests were handled.
    #[must_use]
    pub fn avg_response_time_ms(&self) -> Option<f64> {
        if self.requests_handled > 0 {
            Some(self.total_response_time_ms as f64 / self.requests_handled as f64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_request_serde_roundtrip() {
        let req = JsonRpcRequest {
            jsonrpc: Arc::from("2.0"),
            method: Arc::from("ai.query"),
            params: Some(serde_json::json!({"prompt": "hello"})),
            id: Some(serde_json::json!(1)),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let back: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(&*back.jsonrpc, "2.0");
        assert_eq!(&*back.method, "ai.query");
    }

    #[test]
    fn jsonrpc_response_serde_roundtrip() {
        let resp = JsonRpcResponse {
            jsonrpc: Arc::from("2.0"),
            result: Some(serde_json::json!({"text": "hi"})),
            error: None,
            id: serde_json::json!(1),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: JsonRpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.result.is_some());
        assert!(back.error.is_none());
    }

    #[test]
    fn jsonrpc_error_display() {
        let err = JsonRpcError {
            code: error_codes::METHOD_NOT_FOUND,
            message: "Method not found".into(),
            data: None,
        };
        assert!(err.to_string().contains("Method not found"));
        assert!(err.to_string().contains("-32601"));
    }

    #[test]
    fn normalize_method_strips_prefixes() {
        assert_eq!(normalize_method("squirrel.ai.query"), "ai.query");
        assert_eq!(normalize_method("mcp.health.check"), "health.check");
        assert_eq!(normalize_method("ai.query"), "ai.query");
    }

    #[test]
    fn server_metrics_defaults_and_uptime() {
        let m = ServerMetrics::new();
        assert_eq!(m.requests_handled, 0);
        assert!(m.avg_response_time_ms().is_none());
        assert!(m.uptime_seconds() < 2);
    }

    #[test]
    fn server_metrics_avg_response_time() {
        let mut m = ServerMetrics::new();
        m.requests_handled = 10;
        m.total_response_time_ms = 100;
        let avg = m.avg_response_time_ms().expect("should have avg");
        assert!((avg - 10.0).abs() < f64::EPSILON);
    }
}
