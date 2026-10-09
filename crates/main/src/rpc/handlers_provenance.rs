// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Provenance proxy layer — routes `provenance.*`, `dag.*`, `anchoring.*`,
//! and `attribution.*` methods to discovered primals via capability-based
//! socket discovery.
//!
//! Squirrel does NOT implement DAG/merkle/ledger locally. It discovers the
//! primals that provide these capabilities at runtime (TRUE PRIMAL pattern)
//! and proxies the JSON-RPC call over Unix sockets.
//!
//! ## Capability → Domain mapping
//!
//! | Prefix | Consumed capability | Typical provider |
//! |--------|---------------------|------------------|
//! | `dag.*` | `dag.session.create`, `dag.event.append`, `dag.vertex.query` | DAG provenance primal |
//! | `anchoring.*` | `anchoring.anchor`, `anchoring.verify` | Anchoring primal |
//! | `attribution.*` | `attribution.calculate_rewards` | Attribution primal |
//! | `provenance.*` | (umbrella) routes to `dag.*` provider | DAG provenance primal |

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use serde_json::Value;
use squirrel_mcp_config::unified::TimeoutConfig;
use tracing::{debug, info, warn};

/// Maps a provenance method prefix to the capability domain used for socket discovery.
fn capability_domain_for_method(method: &str) -> &'static str {
    if method.starts_with("dag.") {
        "dag"
    } else if method.starts_with("anchoring.") {
        "anchoring"
    } else if method.starts_with("attribution.") {
        "attribution"
    } else {
        // provenance.* umbrella maps to dag domain
        "dag"
    }
}

impl JsonRpcServer {
    /// Handle any provenance-domain method by proxying to the discovered provider.
    ///
    /// Discovery uses the socket registry: scans for sockets whose
    /// `capability.discover` response includes the target capability domain.
    /// Falls back to the provider registry (springs that registered via
    /// `provider.register` with matching capabilities).
    pub(crate) async fn handle_provenance_proxy(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let domain = capability_domain_for_method(method);
        info!(method, domain, "Provenance proxy: resolving capability");

        // Strategy 1: Check spring provider registry for a registered provider
        if let Some(socket) = self.find_provider_socket(domain).await {
            debug!(
                method,
                socket = socket.as_str(),
                "Routing via provider registry"
            );
            return self.forward_jsonrpc(method, params.as_ref(), &socket).await;
        }

        // Strategy 2: Scan socket directory for primals advertising the capability
        if let Some(socket) = self.discover_capability_socket(domain).await {
            debug!(
                method,
                socket = socket.as_str(),
                "Routing via socket discovery"
            );
            return self.forward_jsonrpc(method, params.as_ref(), &socket).await;
        }

        warn!(
            method,
            domain, "No provider found for provenance capability"
        );
        Err(JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!(
                "No provider discovered for '{domain}.*' capabilities. \
                 Ensure a primal providing '{domain}' is running and discoverable."
            ),
            data: Some(serde_json::json!({
                "method": method,
                "domain": domain,
                "resolution": "awaiting_provider",
            })),
        })
    }

    /// Search the provider registry for a socket offering the given capability domain.
    pub(crate) async fn find_provider_socket(&self, domain: &str) -> Option<String> {
        use crate::universal_adapters::registry::UniversalServiceRegistry;

        let services = self.provider_registry.list_all_services().await.ok()?;
        for svc in &services {
            let domain_match = svc.capabilities.iter().any(|cap| {
                let name = match cap {
                    crate::universal_adapters::ServiceCapability::Custom { capability, .. } => {
                        capability.as_str()
                    }
                    _ => "",
                };
                name.starts_with(domain) || name.contains(&format!("{domain}."))
            });
            if domain_match {
                for ep in &svc.endpoints {
                    let ep_str: &str = ep.as_ref();
                    if ep_str.starts_with("unix://") {
                        return Some(ep_str.trim_start_matches("unix://").to_string());
                    }
                    if ep_str.starts_with('/') {
                        return Some(ep_str.to_string());
                    }
                }
            }
        }
        None
    }

    /// Scan the biomeos socket directory for a primal advertising the capability.
    pub(crate) async fn discover_capability_socket(&self, domain: &str) -> Option<String> {
        let socket_dir = universal_constants::network::get_socket_dir();
        let dir = socket_dir.as_path();
        if !dir.is_dir() {
            return None;
        }

        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("sock") {
                continue;
            }

            if self.probe_capabilities(&path).await.is_ok_and(|caps| {
                caps.iter()
                    .any(|c| c.starts_with(domain) || c.contains(&format!("{domain}.")))
            }) {
                return path.to_str().map(String::from);
            }
        }
        None
    }

    /// Probe a socket with `capability.discover` and return advertised capability names.
    async fn probe_capabilities(&self, socket_path: &std::path::Path) -> Result<Vec<String>, ()> {
        use crate::transport::{TransportEndpoint, connect_transport_with_timeout};
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let endpoint = TransportEndpoint::uds(socket_path.to_string_lossy());
        let stream = connect_transport_with_timeout(
            &endpoint,
            std::time::Duration::from_millis(universal_constants::timeouts::DEFAULT_PROBE_TIMEOUT_MS),
        )
        .await
        .map_err(|_| ())?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.discover",
            "id": 1
        });

        let (reader, mut writer) = tokio::io::split(stream);
        // riboCipher signal prefix — required by ecosystem JSON-RPC servers
        writer.write_all(&[0xEC, 0x01]).await.map_err(|_| ())?;
        let mut line = serde_json::to_string(&request).map_err(|_| ())?;
        line.push('\n');
        writer.write_all(line.as_bytes()).await.map_err(|_| ())?;
        writer.flush().await.map_err(|_| ())?;

        let mut buf_reader = BufReader::new(reader);
        let mut resp_line = String::new();
        tokio::time::timeout(
            std::time::Duration::from_millis(universal_constants::timeouts::DEFAULT_PROBE_TIMEOUT_MS),
            buf_reader.read_line(&mut resp_line),
        )
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

        let resp: Value = serde_json::from_str(resp_line.trim()).map_err(|_| ())?;
        let caps = resp
            .get("result")
            .and_then(|r| r.get("capabilities"))
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(caps)
    }

    /// Forward a JSON-RPC request to a remote socket (or TCP fallback on non-Unix).
    pub(crate) async fn forward_jsonrpc(
        &self,
        method: &str,
        params: Option<&Value>,
        socket_path: &str,
    ) -> Result<Value, JsonRpcError> {
        use crate::transport::{TransportEndpoint, connect_transport_with_timeout};
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let endpoint = TransportEndpoint::uds(socket_path);
        let stream = connect_transport_with_timeout(
            &endpoint,
            TimeoutConfig::global().health_check_timeout(),
        )
        .await
        .map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to connect to provider at {socket_path}: {e}"),
            data: Some(serde_json::json!({ "method": method, "socket": socket_path })),
        })?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let (reader, mut writer) = tokio::io::split(stream);
        // riboCipher signal prefix — required by ecosystem JSON-RPC servers
        writer
            .write_all(&[0xEC, 0x01])
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("riboCipher write error: {e}"),
                data: None,
            })?;
        let mut line = serde_json::to_string(&request).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })?;
        line.push('\n');

        writer
            .write_all(line.as_bytes())
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Write error: {e}"),
                data: None,
            })?;
        writer.flush().await.map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Flush error: {e}"),
            data: None,
        })?;

        let mut buf_reader = BufReader::new(reader);
        let mut resp_line = String::new();
        tokio::time::timeout(TimeoutConfig::global().request_timeout(), async {
            buf_reader.read_line(&mut resp_line).await
        })
        .await
        .map_err(|_| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Timeout waiting for response from {socket_path}"),
            data: None,
        })?
        .map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Read error: {e}"),
            data: None,
        })?;

        let resp: Value = serde_json::from_str(resp_line.trim()).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Invalid JSON response from provider: {e}"),
            data: None,
        })?;

        if let Some(error) = resp.get("error") {
            return Err(JsonRpcError {
                code: error.get("code").and_then(Value::as_i64).map_or(
                    error_codes::INTERNAL_ERROR,
                    |c| {
                        #[expect(
                            clippy::cast_possible_truncation,
                            reason = "JSON-RPC codes fit i32"
                        )]
                        {
                            c as i32
                        }
                    },
                ),
                message: error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Remote error")
                    .to_string(),
                data: error.get("data").cloned(),
            });
        }

        resp.get("result").cloned().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "Provider response missing 'result' field".to_string(),
            data: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::JsonRpcServer;
    use crate::rpc::jsonrpc_server::error_codes;

    #[tokio::test]
    async fn provenance_proxy_no_provider_returns_error() {
        let server = JsonRpcServer::new("/tmp/sq-prov-test.sock".to_string());
        let err = server
            .handle_provenance_proxy("dag.session.create", Some(serde_json::json!({})))
            .await
            .unwrap_err();
        assert_eq!(err.code, error_codes::INTERNAL_ERROR);
        assert!(err.message.contains("No provider discovered"));
    }

    #[tokio::test]
    async fn provenance_proxy_anchoring_no_provider() {
        let server = JsonRpcServer::new("/tmp/sq-anch-test.sock".to_string());
        let err = server
            .handle_provenance_proxy("anchoring.verify", None)
            .await
            .unwrap_err();
        assert!(err.message.contains("anchoring"));
    }

    #[tokio::test]
    async fn provenance_proxy_attribution_no_provider() {
        let server = JsonRpcServer::new("/tmp/sq-attr-test.sock".to_string());
        let err = server
            .handle_provenance_proxy("attribution.calculate_rewards", None)
            .await
            .unwrap_err();
        assert!(err.message.contains("attribution"));
    }

    #[tokio::test]
    async fn provenance_umbrella_maps_to_dag() {
        let server = JsonRpcServer::new("/tmp/sq-prov-umbrella.sock".to_string());
        let err = server
            .handle_provenance_proxy("provenance.query", None)
            .await
            .unwrap_err();
        assert!(err.message.contains("dag"));
    }

    #[tokio::test]
    async fn forward_to_missing_socket_errors() {
        let server = JsonRpcServer::new("/tmp/sq-fwd-test.sock".to_string());
        let payload = serde_json::json!({});
        let err = server
            .forward_jsonrpc(
                "dag.session.create",
                Some(&payload),
                "/nonexistent/provenance.sock",
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, error_codes::INTERNAL_ERROR);
        assert!(err.message.contains("Failed to connect"));
    }

    #[tokio::test]
    async fn provider_registry_routing() {
        use crate::universal_adapters::registry::UniversalServiceRegistry;
        use crate::universal_adapters::{
            IntegrationPreferences, ResourceSpec, ServiceCapability, ServiceCategory,
            ServiceEndpoint, ServiceMetadata, UniversalServiceRegistration,
        };
        use std::collections::HashMap;

        let server = JsonRpcServer::new("/tmp/sq-prov-reg-route.sock".to_string());

        let reg = UniversalServiceRegistration {
            service_id: uuid::Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "rhizocrypt-test".to_string(),
                category: ServiceCategory::Custom {
                    category: "provenance".to_string(),
                    subcategories: vec![],
                },
                version: "0.1.0".to_string(),
                description: "DAG provenance".to_string(),
                maintainer: "eco".to_string(),
                protocols: vec!["jsonrpc-2.0".to_string()],
            },
            capabilities: vec![ServiceCapability::Custom {
                domain: "dag".to_string(),
                capability: "dag.session.create".to_string(),
                parameters: HashMap::new(),
            }],
            endpoints: vec![ServiceEndpoint {
                name: "uds".to_string(),
                url: "unix:///tmp/rhizocrypt-test-provenance.sock".to_string(),
                protocol: "jsonrpc-2.0".to_string(),
                port: None,
                path: Some("/tmp/rhizocrypt-test-provenance.sock".to_string()),
            }],
            resources: ResourceSpec {
                cpu_cores: None,
                memory_gb: None,
                storage_gb: None,
                network_bandwidth: None,
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["jsonrpc-2.0".to_string()],
                retry_policy: "exponential".to_string(),
                timeout_seconds: 30,
                load_balancing_weight: 50,
            },
            extensions: HashMap::new(),
            registration_timestamp: chrono::Utc::now(),
            service_version: "0.1.0".to_string(),
            instance_id: "rhizocrypt-test-01".to_string(),
            priority: 50,
        };

        server
            .provider_registry
            .register_service(reg)
            .await
            .expect("register");

        // Socket doesn't exist, but discovery should resolve the path
        let err = server
            .handle_provenance_proxy("dag.session.create", Some(serde_json::json!({})))
            .await
            .unwrap_err();
        // Should fail at connection, not at discovery
        assert!(
            err.message.contains("Failed to connect"),
            "Should have found provider but failed to connect: {}",
            err.message
        );
    }

    #[cfg(unix)]
    mod unix {
        use super::JsonRpcServer;

        /// Mock provenance primal: listens on UDS, responds to JSON-RPC.
        fn spawn_mock_provenance(socket_path: &std::path::Path) -> tokio::task::JoinHandle<()> {
            use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
            use tokio::net::UnixListener;

            let _ = std::fs::remove_file(socket_path);
            let listener = UnixListener::bind(socket_path).expect("bind mock provenance socket");
            tokio::spawn(async move {
                while let Ok((stream, _)) = listener.accept().await {
                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.into_split();
                        let mut buf_reader = BufReader::new(reader);
                        // consume riboCipher signal prefix [0xEC, 0x01]
                        let mut prefix = [0u8; 2];
                        if buf_reader.read_exact(&mut prefix).await.is_err() {
                            return;
                        }
                        let mut line = String::new();
                        if buf_reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            return;
                        }
                        let request: serde_json::Value =
                            serde_json::from_str(line.trim()).unwrap_or_default();
                        let id = request
                            .get("id")
                            .cloned()
                            .unwrap_or(serde_json::json!(null));
                        let method = request
                            .get("method")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");

                        let result = match method {
                            "dag.session.create" => serde_json::json!({
                                "session_id": "mock-session-001",
                                "created": true
                            }),
                            "anchoring.verify" => serde_json::json!({
                                "valid": true,
                                "anchor_hash": "abc123"
                            }),
                            _ => serde_json::json!({
                                "echo": method
                            }),
                        };

                        let response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "result": result,
                            "id": id
                        });
                        let mut out = serde_json::to_string(&response).expect("serialize");
                        out.push('\n');
                        let _ = writer.write_all(out.as_bytes()).await;
                        let _ = writer.flush().await;
                    });
                }
            })
        }

        #[tokio::test]
        async fn forward_jsonrpc_happy_path() {
            let tmp = tempfile::TempDir::new().expect("tempdir");
            let mock_sock = tmp.path().join("mock-provenance.sock");
            let _bg = spawn_mock_provenance(&mock_sock);
            tokio::task::yield_now().await;

            let server = JsonRpcServer::new("/tmp/sq-fwd-ok.sock".to_string());
            let params = serde_json::json!({"branch": "main"});
            let result = server
                .forward_jsonrpc(
                    "dag.session.create",
                    Some(&params),
                    mock_sock.to_str().expect("path"),
                )
                .await
                .expect("forward should succeed");

            assert_eq!(
                result.get("session_id").and_then(|v| v.as_str()),
                Some("mock-session-001")
            );
            assert_eq!(
                result.get("created").and_then(serde_json::Value::as_bool),
                Some(true)
            );
        }

        #[tokio::test]
        async fn forward_jsonrpc_remote_error_propagated() {
            use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
            use tokio::net::UnixListener;

            let tmp = tempfile::TempDir::new().expect("tempdir");
            let mock_sock = tmp.path().join("mock-err-prov.sock");
            let _ = std::fs::remove_file(&mock_sock);
            let listener = UnixListener::bind(&mock_sock).expect("bind");
            tokio::spawn(async move {
                if let Ok((stream, _)) = listener.accept().await {
                    let (reader, mut writer) = stream.into_split();
                    let mut buf_reader = BufReader::new(reader);
                    let mut prefix = [0u8; 2];
                    let _ = buf_reader.read_exact(&mut prefix).await;
                    let mut line = String::new();
                    let _ = buf_reader.read_line(&mut line).await;
                    let req: serde_json::Value =
                        serde_json::from_str(line.trim()).unwrap_or_default();
                    let id = req.get("id").cloned().unwrap_or(serde_json::json!(null));
                    let resp = serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32001,
                            "message": "session not found"
                        },
                        "id": id
                    });
                    let mut out = serde_json::to_string(&resp).expect("ser");
                    out.push('\n');
                    let _ = writer.write_all(out.as_bytes()).await;
                    let _ = writer.flush().await;
                }
            });
            tokio::task::yield_now().await;

            let server = JsonRpcServer::new("/tmp/sq-fwd-err.sock".to_string());
            let err = server
                .forward_jsonrpc("dag.session.get", None, mock_sock.to_str().expect("path"))
                .await
                .unwrap_err();

            assert_eq!(err.code, -32001);
            assert!(err.message.contains("session not found"));
        }

        #[tokio::test]
        async fn forward_jsonrpc_invalid_json_response() {
            use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
            use tokio::net::UnixListener;

            let tmp = tempfile::TempDir::new().expect("tempdir");
            let mock_sock = tmp.path().join("mock-badjson.sock");
            let _ = std::fs::remove_file(&mock_sock);
            let listener = UnixListener::bind(&mock_sock).expect("bind");
            tokio::spawn(async move {
                if let Ok((stream, _)) = listener.accept().await {
                    let (reader, mut writer) = stream.into_split();
                    let mut buf_reader = BufReader::new(reader);
                    let mut prefix = [0u8; 2];
                    let _ = buf_reader.read_exact(&mut prefix).await;
                    let mut line = String::new();
                    let _ = buf_reader.read_line(&mut line).await;
                    let _ = writer.write_all(b"not json\n").await;
                    let _ = writer.flush().await;
                }
            });
            tokio::task::yield_now().await;

            let server = JsonRpcServer::new("/tmp/sq-fwd-bad.sock".to_string());
            let err = server
                .forward_jsonrpc("dag.query", None, mock_sock.to_str().expect("path"))
                .await
                .unwrap_err();

            assert!(err.message.contains("Invalid JSON response"));
        }

        #[tokio::test]
        async fn forward_jsonrpc_missing_result_field() {
            use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
            use tokio::net::UnixListener;

            let tmp = tempfile::TempDir::new().expect("tempdir");
            let mock_sock = tmp.path().join("mock-noresult.sock");
            let _ = std::fs::remove_file(&mock_sock);
            let listener = UnixListener::bind(&mock_sock).expect("bind");
            tokio::spawn(async move {
                if let Ok((stream, _)) = listener.accept().await {
                    let (reader, mut writer) = stream.into_split();
                    let mut buf_reader = BufReader::new(reader);
                    let mut prefix = [0u8; 2];
                    let _ = buf_reader.read_exact(&mut prefix).await;
                    let mut line = String::new();
                    let _ = buf_reader.read_line(&mut line).await;
                    let resp = serde_json::json!({"jsonrpc": "2.0", "id": 1});
                    let mut out = serde_json::to_string(&resp).expect("ser");
                    out.push('\n');
                    let _ = writer.write_all(out.as_bytes()).await;
                    let _ = writer.flush().await;
                }
            });
            tokio::task::yield_now().await;

            let server = JsonRpcServer::new("/tmp/sq-fwd-nores.sock".to_string());
            let err = server
                .forward_jsonrpc("dag.query", None, mock_sock.to_str().expect("path"))
                .await
                .unwrap_err();

            assert!(err.message.contains("missing 'result' field"));
        }

        #[tokio::test]
        async fn provenance_proxy_end_to_end_via_registry() {
            use crate::universal_adapters::registry::UniversalServiceRegistry;
            use crate::universal_adapters::{
                IntegrationPreferences, ResourceSpec, ServiceCapability, ServiceCategory,
                ServiceEndpoint, ServiceMetadata, UniversalServiceRegistration,
            };
            use std::collections::HashMap;

            let tmp = tempfile::TempDir::new().expect("tempdir");
            let mock_sock = tmp.path().join("rhizocrypt-e2e.sock");
            let _bg = spawn_mock_provenance(&mock_sock);
            tokio::task::yield_now().await;

            let server = JsonRpcServer::new("/tmp/sq-prov-e2e.sock".to_string());

            let reg = UniversalServiceRegistration {
                service_id: uuid::Uuid::new_v4(),
                metadata: ServiceMetadata {
                    name: "rhizocrypt-e2e".to_string(),
                    category: ServiceCategory::Custom {
                        category: "provenance".to_string(),
                        subcategories: vec![],
                    },
                    version: "0.1.0".to_string(),
                    description: "DAG provenance E2E".to_string(),
                    maintainer: "eco".to_string(),
                    protocols: vec!["jsonrpc-2.0".to_string()],
                },
                capabilities: vec![ServiceCapability::Custom {
                    domain: "dag".to_string(),
                    capability: "dag.session.create".to_string(),
                    parameters: HashMap::new(),
                }],
                endpoints: vec![ServiceEndpoint {
                    name: "uds".to_string(),
                    url: format!("unix://{}", mock_sock.display()),
                    protocol: "jsonrpc-2.0".to_string(),
                    port: None,
                    path: Some(mock_sock.to_str().expect("path").to_string()),
                }],
                resources: ResourceSpec {
                    cpu_cores: None,
                    memory_gb: None,
                    storage_gb: None,
                    network_bandwidth: None,
                    custom_resources: HashMap::new(),
                },
                integration: IntegrationPreferences {
                    preferred_protocols: vec!["jsonrpc-2.0".to_string()],
                    retry_policy: "exponential".to_string(),
                    timeout_seconds: 30,
                    load_balancing_weight: 50,
                },
                extensions: HashMap::new(),
                registration_timestamp: chrono::Utc::now(),
                service_version: "0.1.0".to_string(),
                instance_id: "rhizocrypt-e2e-01".to_string(),
                priority: 50,
            };

            server
                .provider_registry
                .register_service(reg)
                .await
                .expect("register provider");

            let result = server
                .handle_provenance_proxy(
                    "dag.session.create",
                    Some(serde_json::json!({"branch": "main"})),
                )
                .await
                .expect("provenance proxy should succeed");

            assert_eq!(
                result.get("session_id").and_then(|v| v.as_str()),
                Some("mock-session-001")
            );
        }
    }
}
