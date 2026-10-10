// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Membrane domain handler — read-only observation from announced peers.
//!
//! chipMonk's first sense: the ability to report what the membrane sees.
//! Returns stored metadata from `capabilities.announce` calls where the
//! announcing primal registered `membrane.*` tools.
//!
//! Pure read-only. No side effects, no disk writes, no network calls.

use super::jsonrpc_server::JsonRpcServer;
use super::jsonrpc_types::JsonRpcError;
use serde_json::Value;
use tracing::debug;

impl JsonRpcServer {
    /// Handle `membrane.status` — return latest membrane observations.
    ///
    /// Reads the announced_tools registry, filters for `membrane.*` entries,
    /// and returns the metadata each announcing primal provided.
    pub(crate) async fn handle_membrane_status(&self) -> Result<Value, JsonRpcError> {
        debug!("membrane.status request");

        let registry = self.announced_tools.read().await;

        // Collect unique primals that announced membrane.* tools
        let mut seen_primals = std::collections::HashSet::new();
        let mut observations = Vec::new();

        for (tool_name, announced) in registry.iter() {
            if !tool_name.starts_with("membrane.") {
                continue;
            }
            let primal_key = announced.primal.as_ref();
            if !seen_primals.insert(primal_key.to_string()) {
                continue;
            }

            observations.push(serde_json::json!({
                "primal": primal_key,
                "announced_at": announced.announced_at.to_rfc3339(),
                "capabilities": announced.capabilities.iter()
                    .map(|c| c.as_ref())
                    .collect::<Vec<&str>>(),
                "metadata": announced.metadata,
            }));
        }

        Ok(serde_json::json!({
            "status": if observations.is_empty() { "no_observations" } else { "observing" },
            "observers": observations.len(),
            "observations": observations,
        }))
    }
}
