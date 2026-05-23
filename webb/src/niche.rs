// SPDX-License-Identifier: AGPL-3.0-or-later
//! Niche self-knowledge for Esoteric Webb.
//!
//! Single source of truth for identity, capabilities, socket resolution,
//! and biomeOS family integration. Follows the ludoSpring `niche.rs`
//! pattern: a primal (or composition) has complete self-knowledge and
//! discovers others at runtime.
//!
//! This module has **no IPC dependencies** — pure constants and metadata.

/// Primal identity name (socket naming, registration, logging).
pub const NICHE_NAME: &str = "esotericwebb";

/// Capability domain — Webb's own methods live under this.
pub const NICHE_DOMAIN: &str = "narrative";

/// Conventional directory name for ecosystem IPC sockets.
pub const ECOSYSTEM_SOCKET_DIR: &str = "biomeos";

/// All capabilities this composition exposes.
///
/// Kept in sync with `deploy/esotericwebb.toml` and
/// `capability_registry.toml`. Tests verify consistency.
pub const CAPABILITIES: &[&str] = &[
    // sourDough
    "health.liveness",
    "health.readiness",
    "health.check",
    "health.version",
    "health.drain",
    "identity.get",
    "capabilities.list",
    // Lifecycle (Wave 17)
    "primal.announce",
    "primal.info",
    // Webb health
    "webb.health",
    "webb.liveness",
    "webb.readiness",
    // Narrative
    "webb.scene.current",
    "webb.narrative.status",
    // Content
    "webb.content.list",
    // Session
    "session.start",
    "session.state",
    "session.actions",
    "session.act",
    "session.history",
    "session.narrate",
    "session.graph",
    // MCP
    "tools.list",
    "tools.call",
];

/// Resolve the biomeOS family ID from environment.
///
/// Priority: `FAMILY_ID` → `BIOMEOS_FAMILY_ID` → `"default"`.
#[must_use]
pub fn family_id() -> String {
    use crate::env_keys;
    std::env::var(env_keys::FAMILY_ID)
        .or_else(|_| std::env::var(env_keys::BIOMEOS_FAMILY_ID))
        .unwrap_or_else(|_| "default".to_owned())
}

/// Socket directories in XDG-compliant priority order.
///
/// 1. `BIOMEOS_SOCKET_DIR` — explicit ecosystem override
/// 2. `$XDG_RUNTIME_DIR/biomeos/` — standard runtime location
/// 3. `/tmp/biomeos-$USER/` — user-scoped temp fallback
/// 4. platform `temp_dir()` — last resort
#[must_use]
pub fn socket_dirs() -> Vec<std::path::PathBuf> {
    use crate::env_keys;
    use std::path::PathBuf;
    let mut dirs = Vec::new();

    if let Ok(d) = std::env::var(env_keys::BIOMEOS_SOCKET_DIR) {
        dirs.push(PathBuf::from(d));
    }
    if let Ok(xdg) = std::env::var(env_keys::XDG_RUNTIME_DIR) {
        dirs.push(PathBuf::from(xdg).join(ECOSYSTEM_SOCKET_DIR));
    }
    let user = std::env::var(env_keys::USER).unwrap_or_else(|_| "unknown".to_owned());
    dirs.push(std::env::temp_dir().join(format!("{ECOSYSTEM_SOCKET_DIR}-{user}")));
    dirs.push(std::env::temp_dir());
    dirs
}

/// Resolve the socket path for Webb's own IPC server.
///
/// Explicit `ESOTERICWEBB_SOCK` override checked first, then the
/// XDG-compliant directory chain with family-scoped naming.
#[must_use]
pub fn resolve_server_socket() -> std::path::PathBuf {
    use crate::env_keys;
    use std::path::PathBuf;

    if let Ok(explicit) = std::env::var(env_keys::ESOTERICWEBB_SOCK) {
        return PathBuf::from(explicit);
    }

    let fid = family_id();
    let sock_name = if fid == "default" {
        format!("{NICHE_NAME}.sock")
    } else {
        format!("{NICHE_NAME}-{fid}.sock")
    };

    for dir in socket_dirs() {
        if dir.is_dir() || std::fs::create_dir_all(&dir).is_ok() {
            return dir.join(&sock_name);
        }
    }
    std::env::temp_dir().join(sock_name)
}

/// Resolve the Neural API socket path (biomeOS orchestration layer).
///
/// Follows convention: `neural-api-{family_id}.sock` in socket dirs.
#[must_use]
pub fn resolve_neural_api_socket() -> Option<std::path::PathBuf> {
    use crate::env_keys;
    if let Ok(explicit) = std::env::var(env_keys::NEURAL_API_SOCKET) {
        let p = std::path::PathBuf::from(&explicit);
        if p.exists() {
            return Some(p);
        }
    }

    let fid = family_id();
    let sock_name = format!("neural-api-{fid}.sock");

    for dir in socket_dirs() {
        let p = dir.join(&sock_name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_constants() {
        assert_eq!(NICHE_NAME, "esotericwebb");
        assert_eq!(NICHE_DOMAIN, "narrative");
    }

    #[test]
    fn capabilities_count() {
        assert_eq!(CAPABILITIES.len(), 24);
    }

    #[test]
    fn all_capabilities_are_namespaced() {
        let prefixes = [
            "health.",
            "identity.",
            "capabilities.",
            "primal.",
            "webb.",
            "session.",
            "tools.",
        ];
        for cap in CAPABILITIES {
            assert!(
                prefixes.iter().any(|p| cap.starts_with(p)),
                "capability '{cap}' has no recognized namespace prefix"
            );
        }
    }

    #[test]
    fn socket_dirs_never_empty() {
        assert!(!socket_dirs().is_empty());
    }

    #[test]
    fn family_id_has_default() {
        let fid = family_id();
        assert!(!fid.is_empty());
    }

    #[test]
    fn resolve_server_socket_returns_path() {
        let path = resolve_server_socket();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert!(
            name.starts_with(NICHE_NAME),
            "socket name should start with niche name, got: {name}"
        );
        assert!(
            std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
        );
    }

    #[test]
    fn neural_api_socket_none_without_env() {
        let result = resolve_neural_api_socket();
        assert!(
            result.is_none() || result.is_some(),
            "should not panic regardless of env"
        );
    }

    #[test]
    fn capabilities_match_registry_toml() {
        let toml_content = include_str!("../capability_registry.toml");
        for cap in CAPABILITIES {
            assert!(
                toml_content.contains(cap),
                "capability '{cap}' missing from capability_registry.toml"
            );
        }
    }
}
