// SPDX-License-Identifier: AGPL-3.0-or-later
//! MCP `tools.list` / `tools.call` handlers.
//!
//! `tools.list` returns MCP-compliant descriptors with typed `input_schema`
//! (JSON Schema) for each tool. `tools.call` dispatches to the same handler
//! functions used by JSON-RPC — zero duplicate logic (ludoSpring pattern).

use serde_json::Value;

use crate::ipc::envelope::JsonRpcError;

use super::SharedSession;
use super::lifecycle::{
    handle_capabilities_list, handle_health, handle_identity, handle_readiness,
};
use super::narrative::{handle_content_list, handle_narrative_status, handle_scene_current};
use super::session::{
    handle_session_act, handle_session_actions, handle_session_graph, handle_session_history,
    handle_session_narrate, handle_session_start, handle_session_state,
};

/// `tools.list` — MCP tool descriptors with JSON Schema `input_schema`.
pub(super) fn handle_tools_list() -> Value {
    serde_json::json!({
        "tools": mcp_tool_descriptors()
    })
}

/// `tools.call` — dispatch by tool name into existing handlers.
pub(super) fn handle_tools_call(
    params: Option<&Value>,
    session: &SharedSession,
) -> Result<Value, JsonRpcError> {
    let name = params
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let arguments = params.and_then(|p| p.get("arguments"));

    match name {
        "health.liveness" | "webb.health" | "health.check" => Ok(handle_health()),
        "health.readiness" => Ok(handle_readiness(session)),
        "identity.get" => Ok(handle_identity()),
        "capabilities.list" => Ok(handle_capabilities_list()),
        "webb.scene.current" => Ok(handle_scene_current(session)),
        "webb.narrative.status" => Ok(handle_narrative_status(session)),
        "webb.content.list" => Ok(handle_content_list(session)),
        "session.start" => handle_session_start(arguments, session),
        "session.state" => handle_session_state(session),
        "session.actions" => handle_session_actions(session),
        "session.act" => handle_session_act(arguments, session),
        "session.history" => handle_session_history(session),
        "session.narrate" => handle_session_narrate(session),
        "session.graph" => handle_session_graph(session),
        _ => Err(JsonRpcError {
            code: -32602,
            message: format!("unknown tool: {name}"),
            data: None,
        }),
    }
}

/// MCP tool descriptors array with `input_schema` per tool.
fn mcp_tool_descriptors() -> Value {
    serde_json::json!([
        {
            "name": "health.liveness",
            "description": "Liveness probe — process is alive (sourDough)",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "health.readiness",
            "description": "Readiness probe — content loaded, session can start (sourDough)",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "identity.get",
            "description": "Self-identification: primal name, version, domain (sourDough)",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "capabilities.list",
            "description": "List all capabilities from the registry (sourDough)",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "webb.scene.current",
            "description": "Get the current game scene — node, description, NPCs, ending state",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "webb.narrative.status",
            "description": "Narrative DAG status — current node, turn, knowledge, actions",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "webb.content.list",
            "description": "List loaded content — worlds, NPCs, abilities, scenes, nodes",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "session.start",
            "description": "Initialize a new game session with a content bundle path",
            "input_schema": {
                "type": "object",
                "properties": {
                    "content_path": {
                        "type": "string",
                        "description": "Path to the content directory (defaults to 'content')"
                    }
                }
            }
        },
        {
            "name": "session.state",
            "description": "Get the full game state snapshot — node, knowledge, inventory, flags, trust",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "session.actions",
            "description": "List available actions from the current game state",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "session.act",
            "description": "Perform a player action — returns outcome text and narration context",
            "input_schema": {
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "description": "Action kind: go, talk, use, examine, ability, rest, or lie"
                    },
                    "id": {
                        "type": "string",
                        "description": "Target ID for the action (node ID, NPC name, item name, etc.)"
                    }
                },
                "required": ["kind", "id"]
            }
        },
        {
            "name": "session.history",
            "description": "Get the full action history for the current session",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "session.narrate",
            "description": "Get narration context for AI-as-generator — scene, mood, NPCs, history",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "session.graph",
            "description": "Get the narrative DAG as DOT format with live session overlay",
            "input_schema": { "type": "object", "properties": {} }
        }
    ])
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn empty_session() -> SharedSession {
        Arc::new(Mutex::new(None))
    }

    #[test]
    fn tools_list_has_input_schema() {
        let v = handle_tools_list();
        let tools = v["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
        for tool in tools {
            assert!(tool.get("name").is_some(), "tool missing name");
            assert!(
                tool.get("description").is_some(),
                "tool missing description"
            );
            assert!(
                tool.get("input_schema").is_some(),
                "tool {} missing input_schema",
                tool["name"]
            );
        }
    }

    #[test]
    fn tools_list_session_act_has_required_fields() {
        let v = handle_tools_list();
        let tools = v["tools"].as_array().unwrap();
        let act = tools
            .iter()
            .find(|t| t["name"] == "session.act")
            .expect("session.act missing");
        let schema = &act["input_schema"];
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&Value::String("kind".to_owned())));
        assert!(required.contains(&Value::String("id".to_owned())));
    }

    #[test]
    fn tools_call_health() {
        let session = empty_session();
        let params = serde_json::json!({"name": "health.liveness"});
        let result = handle_tools_call(Some(&params), &session);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["status"], "healthy");
    }

    #[test]
    fn tools_call_identity() {
        let session = empty_session();
        let params = serde_json::json!({"name": "identity.get"});
        let result = handle_tools_call(Some(&params), &session);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["primal"], "esotericwebb");
    }

    #[test]
    fn tools_call_unknown_returns_error() {
        let session = empty_session();
        let params = serde_json::json!({"name": "nonexistent.tool"});
        let result = handle_tools_call(Some(&params), &session);
        assert!(result.is_err());
    }

    #[test]
    fn tools_call_session_state_without_session_errors() {
        let session = empty_session();
        let params = serde_json::json!({"name": "session.state"});
        let result = handle_tools_call(Some(&params), &session);
        assert!(result.is_err());
    }
}
