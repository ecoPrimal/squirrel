// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security Types
//!
//! This module contains all security-related enums, types, and basic data structures
//! used throughout the security system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Security capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityCapability {
    /// Authentication and authorization
    Authentication,
    /// Encryption services
    Encryption,
    /// Access control
    AccessControl,
    /// Security auditing
    Auditing,
    /// Threat detection
    ThreatDetection,
    /// Compliance monitoring
    ComplianceMonitoring,
    /// Security analytics
    SecurityAnalytics,
}

/// Security levels — canonical type from cellmembrane-types.
///
/// Migration mapping from former local variants:
///   Low → Standard (canonical alias), Standard → Standard,
///   High → High, Critical → Maximum (canonical alias).
pub use cellmembrane_types::SecurityLevel;

/// Authorization levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorizationLevel {
    /// No authorization
    None,
    /// Basic user authorization
    User,
    /// Elevated authorization
    Elevated,
    /// Administrator authorization
    Admin,
    /// System-level authorization
    System,
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User identifier
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Operation context
    pub operation: String,
    /// Resource being accessed
    pub resource: String,
    /// Client information
    pub client_info: HashMap<String, String>,
    /// Environment information
    pub environment: HashMap<String, String>,
    /// Timestamp of the operation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Authentication handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationHandler {
    /// Handler name
    pub name: String,
    /// Handler type
    pub handler_type: String,
    /// Handler configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Priority (lower numbers = higher priority)
    pub priority: u32,
    /// Enabled status
    pub enabled: bool,
}

/// Universal security request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequest {
    /// Request identifier
    pub request_id: String,
    /// Request type
    pub request_type: SecurityRequestType,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Security context
    pub context: SecurityContext,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Security request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRequestType {
    /// Authentication request
    Authentication,
    /// Authorization request
    Authorization,
    /// Encryption request
    Encryption,
    /// Decryption request
    Decryption,
    /// Audit request
    Audit,
    /// Policy check request
    PolicyCheck,
    /// Health check request
    HealthCheck,
    /// Token validation request
    TokenValidation,
}

/// Universal security response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResponse {
    /// Request identifier
    pub request_id: String,
    /// Response status
    pub status: SecurityResponseStatus,
    /// Response payload
    pub payload: serde_json::Value,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Processing time
    pub processing_time: Duration,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Security response status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityResponseStatus {
    /// Success
    Success,
    /// Authentication failed
    AuthenticationFailed,
    /// Authorization failed
    AuthorizationFailed,
    /// Policy violation
    PolicyViolation,
    /// Threat detected
    ThreatDetected,
    /// Error with code and message
    Error {
        /// Error code identifier.
        code: String,
        /// Human-readable error message.
        message: String,
    },
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            operation: "unknown".to_string(),
            resource: "unknown".to_string(),
            client_info: HashMap::new(),
            environment: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl SecurityContext {
    /// Create a new security context
    #[must_use]
    pub fn new(operation: &str, resource: &str) -> Self {
        Self {
            user_id: None,
            session_id: None,
            operation: operation.to_string(),
            resource: resource.to_string(),
            client_info: HashMap::new(),
            environment: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Set user ID
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session ID
    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add client information
    #[must_use]
    pub fn with_client_info(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.client_info.insert(key.into(), value.into());
        self
    }

    /// Add environment information
    #[must_use]
    pub fn with_environment(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SecurityCapability tests ---

    #[test]
    fn test_security_capability_serde_roundtrip() {
        let capabilities = vec![
            SecurityCapability::Authentication,
            SecurityCapability::Encryption,
            SecurityCapability::AccessControl,
            SecurityCapability::Auditing,
            SecurityCapability::ThreatDetection,
            SecurityCapability::ComplianceMonitoring,
            SecurityCapability::SecurityAnalytics,
        ];
        for cap in capabilities {
            let json = serde_json::to_string(&cap).expect("should succeed");
            let deserialized: SecurityCapability =
                serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, cap);
        }
    }

    // --- SecurityLevel tests ---

    #[test]
    fn test_security_level_serde_roundtrip() {
        let levels = vec![
            SecurityLevel::None,
            SecurityLevel::Standard,
            SecurityLevel::High,
            SecurityLevel::Maximum,
            SecurityLevel::QuantumResistant,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).expect("should succeed");
            let deserialized: SecurityLevel = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, level);
        }
    }

    // --- AuthorizationLevel tests ---

    #[test]
    fn test_authorization_level_serde_roundtrip() {
        let levels = vec![
            AuthorizationLevel::None,
            AuthorizationLevel::User,
            AuthorizationLevel::Elevated,
            AuthorizationLevel::Admin,
            AuthorizationLevel::System,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).expect("should succeed");
            let deserialized: AuthorizationLevel =
                serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, level);
        }
    }

    // --- SecurityContext tests ---

    #[test]
    fn test_security_context_default() {
        let ctx = SecurityContext::default();
        assert!(ctx.user_id.is_none());
        assert!(ctx.session_id.is_none());
        assert_eq!(ctx.operation, "unknown");
        assert_eq!(ctx.resource, "unknown");
        assert!(ctx.client_info.is_empty());
        assert!(ctx.environment.is_empty());
    }

    #[test]
    fn test_security_context_new() {
        let ctx = SecurityContext::new("read", "database");
        assert!(ctx.user_id.is_none());
        assert_eq!(ctx.operation, "read");
        assert_eq!(ctx.resource, "database");
    }

    #[test]
    fn test_security_context_builder_chain() {
        let ctx = SecurityContext::new("write", "config")
            .with_user_id("user-123".to_string())
            .with_session_id("sess-456".to_string())
            .with_client_info("ip".to_string(), "127.0.0.1".to_string())
            .with_environment("env".to_string(), "production".to_string());

        assert_eq!(ctx.user_id.as_deref(), Some("user-123"));
        assert_eq!(ctx.session_id.as_deref(), Some("sess-456"));
        assert_eq!(
            ctx.client_info.get("ip").expect("should succeed"),
            "127.0.0.1"
        );
        assert_eq!(
            ctx.environment.get("env").expect("should succeed"),
            "production"
        );
    }

    #[test]
    fn test_security_context_serde_roundtrip() {
        let ctx = SecurityContext::new("delete", "record").with_user_id("admin".to_string());
        let json = serde_json::to_string(&ctx).expect("should succeed");
        let deserialized: SecurityContext = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.operation, "delete");
        assert_eq!(deserialized.resource, "record");
        assert_eq!(deserialized.user_id.as_deref(), Some("admin"));
    }

    // --- AuthenticationHandler tests ---

    #[test]
    fn test_authentication_handler_serde() {
        let handler = AuthenticationHandler {
            name: "jwt_handler".to_string(),
            handler_type: "jwt".to_string(),
            config: HashMap::from([("issuer".to_string(), serde_json::json!("biome-os"))]),
            priority: 1,
            enabled: true,
        };
        let json = serde_json::to_string(&handler).expect("should succeed");
        let deserialized: AuthenticationHandler =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.name, "jwt_handler");
        assert_eq!(deserialized.priority, 1);
        assert!(deserialized.enabled);
    }

    // --- SecurityRequestType tests ---

    #[test]
    fn test_security_request_type_serde() {
        let types = vec![
            SecurityRequestType::Authentication,
            SecurityRequestType::Authorization,
            SecurityRequestType::Encryption,
            SecurityRequestType::Decryption,
            SecurityRequestType::Audit,
            SecurityRequestType::PolicyCheck,
            SecurityRequestType::HealthCheck,
            SecurityRequestType::TokenValidation,
        ];
        for rt in types {
            let json = serde_json::to_string(&rt).expect("should succeed");
            let deserialized: SecurityRequestType =
                serde_json::from_str(&json).expect("should succeed");
            let json2 = serde_json::to_string(&deserialized).expect("should succeed");
            assert_eq!(json, json2);
        }
    }

    // --- SecurityRequest tests ---

    #[test]
    fn test_security_request_serde() {
        let request = SecurityRequest {
            request_id: "req-001".to_string(),
            request_type: SecurityRequestType::Authentication,
            payload: serde_json::json!({"token": "abc123"}),
            metadata: HashMap::new(),
            context: SecurityContext::new("login", "auth_service"),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&request).expect("should succeed");
        let deserialized: SecurityRequest = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.request_id, "req-001");
    }

    // --- SecurityResponseStatus tests ---

    #[test]
    fn test_security_response_status_serde() {
        let statuses = vec![
            SecurityResponseStatus::Success,
            SecurityResponseStatus::AuthenticationFailed,
            SecurityResponseStatus::AuthorizationFailed,
            SecurityResponseStatus::PolicyViolation,
            SecurityResponseStatus::ThreatDetected,
            SecurityResponseStatus::Error {
                code: "E001".to_string(),
                message: "Internal error".to_string(),
            },
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).expect("should succeed");
            let deserialized: SecurityResponseStatus =
                serde_json::from_str(&json).expect("should succeed");
            let json2 = serde_json::to_string(&deserialized).expect("should succeed");
            assert_eq!(json, json2);
        }
    }

    // --- SecurityResponse tests ---

    #[test]
    fn test_security_response_serde() {
        let response = SecurityResponse {
            request_id: "req-001".to_string(),
            status: SecurityResponseStatus::Success,
            payload: serde_json::json!({"authenticated": true}),
            metadata: HashMap::new(),
            processing_time: Duration::from_millis(42),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&response).expect("should succeed");
        let deserialized: SecurityResponse = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.request_id, "req-001");
    }
}
