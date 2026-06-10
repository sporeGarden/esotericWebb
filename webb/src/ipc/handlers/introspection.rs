// SPDX-License-Identifier: AGPL-3.0-or-later
//! Runtime method introspection — `method.describe` handler.
//!
//! Follows the barraCuda pattern (Wave 107): any consumer can query a primal
//! for structured metadata about its methods without prior knowledge. Enables
//! self-correcting distributed compositions.

use serde_json::Value;

use super::super::envelope::JsonRpcError;

/// Method descriptor returned by `method.describe`.
struct MethodDescriptor {
    method: &'static str,
    description: &'static str,
    domain: &'static str,
    stability: &'static str,
    params: Option<&'static str>,
    access: &'static str,
}

/// Complete method catalog — compiled from `capability_registry.toml`.
static METHODS: &[MethodDescriptor] = &[
    // sourDough / health
    MethodDescriptor {
        method: "health.liveness",
        description: "Kubernetes-style liveness probe — process is alive",
        domain: "health",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "health.readiness",
        description: "Kubernetes-style readiness probe — content loaded, session can start",
        domain: "health",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "health.check",
        description: "Health check (sourDough alias)",
        domain: "health",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "health.version",
        description: "Detailed version, build target, and composition tier information",
        domain: "health",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "health.drain",
        description: "Acknowledge graceful shutdown intent",
        domain: "health",
        stability: "stable",
        params: None,
        access: "public",
    },
    // Identity
    MethodDescriptor {
        method: "identity.get",
        description: "Self-identification — primal name, version, domain",
        domain: "identity",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "capabilities.list",
        description: "Return canonical capability registry (Wave 20 envelope)",
        domain: "capabilities",
        stability: "stable",
        params: None,
        access: "public",
    },
    // Lifecycle
    MethodDescriptor {
        method: "primal.announce",
        description: "Accept inbound primal registration announcements",
        domain: "lifecycle",
        stability: "stable",
        params: Some("{ primal: string, version: string, capabilities: [string] }"),
        access: "public",
    },
    MethodDescriptor {
        method: "primal.info",
        description: "Return niche metadata — version, capabilities, composition tiers",
        domain: "lifecycle",
        stability: "stable",
        params: None,
        access: "public",
    },
    // Webb domain
    MethodDescriptor {
        method: "webb.health",
        description: "Webb-specific health check",
        domain: "webb",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "webb.liveness",
        description: "Webb liveness check",
        domain: "webb",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "webb.readiness",
        description: "Webb readiness check",
        domain: "webb",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "webb.scene.current",
        description: "Current game scene description and metadata",
        domain: "webb",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "webb.narrative.status",
        description: "Narrative DAG status — node count, edge count, start/end nodes",
        domain: "webb",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "webb.content.list",
        description: "List loaded content (scenes, abilities, NPCs)",
        domain: "webb",
        stability: "stable",
        params: None,
        access: "public",
    },
    // Session
    MethodDescriptor {
        method: "session.start",
        description: "Initialize a new game session with content bundle",
        domain: "session",
        stability: "stable",
        params: Some("{ content_path: string }"),
        access: "public",
    },
    MethodDescriptor {
        method: "session.state",
        description: "Full game state snapshot — node, knowledge, inventory, flags, trust",
        domain: "session",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "session.actions",
        description: "List available actions from the current game state",
        domain: "session",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "session.act",
        description: "Perform a player action — returns outcome and narration context",
        domain: "session",
        stability: "stable",
        params: Some("{ kind: string, id: string }"),
        access: "public",
    },
    MethodDescriptor {
        method: "session.history",
        description: "Full action history for the current session",
        domain: "session",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "session.narrate",
        description: "Narration context for AI-as-generator",
        domain: "session",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "session.graph",
        description: "Narrative DAG as DOT with live session overlay",
        domain: "session",
        stability: "stable",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "session.metrics",
        description: "Session engagement metrics for game science / DDA",
        domain: "session",
        stability: "stable",
        params: None,
        access: "public",
    },
    // Introspection
    MethodDescriptor {
        method: "method.describe",
        description: "Runtime method introspection — describe any exposed method",
        domain: "introspection",
        stability: "stable",
        params: Some("{ method: string }"),
        access: "public",
    },
    // MCP (evolving)
    MethodDescriptor {
        method: "tools.list",
        description: "MCP tool enumeration for AI agent discovery",
        domain: "tools",
        stability: "evolving",
        params: None,
        access: "public",
    },
    MethodDescriptor {
        method: "tools.call",
        description: "MCP tool invocation for AI agent interaction",
        domain: "tools",
        stability: "evolving",
        params: Some("{ name: string, arguments: object }"),
        access: "public",
    },
];

/// Handle `method.describe` — return structured metadata for a named method.
pub(super) fn handle_method_describe(params: Option<&Value>) -> Result<Value, JsonRpcError> {
    let method_name = params
        .and_then(|p| p.get("method"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            JsonRpcError::invalid_params("missing required parameter: method".to_owned())
        })?;

    METHODS
        .iter()
        .find(|m| m.method == method_name)
        .map_or_else(
            || {
                Ok(serde_json::json!({
                    "method": method_name,
                    "found": false,
                    "primal": crate::niche::NICHE_NAME,
                    "available_methods": crate::niche::CAPABILITIES.len(),
                }))
            },
            |desc| {
                let mut map = serde_json::Map::with_capacity(7);
                map.insert("method".to_owned(), Value::String(desc.method.to_owned()));
                map.insert(
                    "description".to_owned(),
                    Value::String(desc.description.to_owned()),
                );
                map.insert("domain".to_owned(), Value::String(desc.domain.to_owned()));
                map.insert(
                    "stability".to_owned(),
                    Value::String(desc.stability.to_owned()),
                );
                map.insert("access".to_owned(), Value::String(desc.access.to_owned()));
                map.insert(
                    "primal".to_owned(),
                    Value::String(crate::niche::NICHE_NAME.to_owned()),
                );
                if let Some(params_schema) = desc.params {
                    map.insert("params".to_owned(), Value::String(params_schema.to_owned()));
                }
                Ok(Value::Object(map))
            },
        )
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn describe_known_method() {
        let params = serde_json::json!({"method": "health.liveness"});
        let result = handle_method_describe(Some(&params)).unwrap();
        assert_eq!(result["method"], "health.liveness");
        assert_eq!(result["domain"], "health");
        assert_eq!(result["stability"], "stable");
        assert_eq!(result["access"], "public");
        assert!(result["description"].as_str().unwrap().contains("liveness"));
    }

    #[test]
    fn describe_method_with_params() {
        let params = serde_json::json!({"method": "session.act"});
        let result = handle_method_describe(Some(&params)).unwrap();
        assert_eq!(result["method"], "session.act");
        assert!(result.get("params").is_some());
        assert!(result["params"].as_str().unwrap().contains("kind"));
    }

    #[test]
    fn describe_unknown_method() {
        let params = serde_json::json!({"method": "nonexistent.foo"});
        let result = handle_method_describe(Some(&params)).unwrap();
        assert_eq!(result["found"], false);
        assert_eq!(result["method"], "nonexistent.foo");
    }

    #[test]
    fn describe_missing_params() {
        let result = handle_method_describe(None);
        assert!(result.is_err());
    }

    #[test]
    fn describe_missing_method_field() {
        let params = serde_json::json!({"wrong_key": "health.liveness"});
        let result = handle_method_describe(Some(&params));
        assert!(result.is_err());
    }

    #[test]
    fn describe_evolving_method() {
        let params = serde_json::json!({"method": "tools.list"});
        let result = handle_method_describe(Some(&params)).unwrap();
        assert_eq!(result["stability"], "evolving");
        assert_eq!(result["domain"], "tools");
    }

    #[test]
    fn describe_method_describe_itself() {
        let params = serde_json::json!({"method": "method.describe"});
        let result = handle_method_describe(Some(&params)).unwrap();
        assert_eq!(result["method"], "method.describe");
        assert_eq!(result["domain"], "introspection");
        assert_eq!(result["stability"], "stable");
        assert!(result.get("params").is_some());
    }

    #[test]
    fn all_niche_capabilities_have_descriptors() {
        for cap in crate::niche::CAPABILITIES {
            let params = serde_json::json!({"method": cap});
            let result = handle_method_describe(Some(&params)).unwrap();
            assert!(
                result.get("found").is_none(),
                "capability '{cap}' missing from METHODS catalog"
            );
            assert_eq!(result["method"], *cap);
        }
    }

    #[test]
    fn descriptor_count_matches_capabilities() {
        assert_eq!(METHODS.len(), crate::niche::CAPABILITIES.len());
    }
}
