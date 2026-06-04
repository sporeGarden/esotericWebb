// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized environment variable name constants.
//!
//! Every `std::env::var(...)` in this crate references a constant from this
//! module. This ensures discoverability, prevents typos, and aligns with the
//! primalSpring `env_keys.rs` ecosystem convention (Wave 46).
//!
//! Categories follow the primalSpring layout:
//! - Identity & genetics (`FAMILY_ID`, `BIOMEOS_FAMILY_ID`)
//! - XDG / OS paths (`XDG_RUNTIME_DIR`, `HOME`)
//! - Socket / discovery (`BIOMEOS_SOCKET_DIR`, `NEURAL_API_SOCKET`)
//! - Per-primal port overrides (`<PRIMAL>_ADDRESS`, `<PRIMAL>_JSONRPC_PORT`)
//! - Webb-specific configuration (`ESOTERICWEBB_*`)
//! - Deployment (`ECOPRIMALS_PLASMID_BIN`, `BIOMEOS_PLASMID_BIN_DIR`)

// ── Identity & genetics ─────────────────────────────────────────────────────

/// Family identity for ecosystem membership resolution.
pub const FAMILY_ID: &str = "FAMILY_ID";
/// biomeOS-specific family ID (fallback for `FAMILY_ID`).
pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";

// ── XDG / OS paths ─────────────────────────────────────────────────────────

/// XDG base directory for runtime data (sockets, pid files).
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";
/// Current user name (fallback for socket directory naming).
pub const USER: &str = "USER";
/// Current user ID (numeric, for `/run/user/<uid>` paths).
pub const UID: &str = "UID";

// ── Socket / discovery ──────────────────────────────────────────────────────

/// Explicit biomeOS socket directory override.
pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";
/// Explicit Neural API socket path override.
pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";
/// Gate identity for mesh registration (default: `"ironGate"`).
pub const BIOMEOS_GATE_ID: &str = "BIOMEOS_GATE_ID";

// ── Per-primal address / port overrides ─────────────────────────────────────
// Used by discovery: `<PRIMAL>_ADDRESS` for host:port,
// `<PRIMAL>_JSONRPC_PORT` for port-only, `<PRIMAL>_HTTP_ADDRESS` as fallback.
// These are dynamic — constructed from primal slug at runtime.
// The suffixes are documented here for reference:

/// Suffix for full address override (e.g. `RHIZOCRYPT_ADDRESS=127.0.0.1:9401`).
pub const ADDR_SUFFIX: &str = "_ADDRESS";
/// Suffix for port-only override (e.g. `RHIZOCRYPT_JSONRPC_PORT=9401`).
pub const PORT_SUFFIX: &str = "_JSONRPC_PORT";
/// Suffix for HTTP address fallback.
pub const HTTP_ADDR_SUFFIX: &str = "_HTTP_ADDRESS";

// ── Webb-specific configuration ─────────────────────────────────────────────

/// Explicit esotericWebb UDS socket path override.
pub const ESOTERICWEBB_SOCK: &str = "ESOTERICWEBB_SOCK";
/// IPC call timeout in seconds (default: 5).
pub const ESOTERICWEBB_IPC_TIMEOUT_SECS: &str = "ESOTERICWEBB_IPC_TIMEOUT_SECS";
/// Readiness probe timeout in seconds (default: 10).
pub const ESOTERICWEBB_READINESS_TIMEOUT_SECS: &str = "ESOTERICWEBB_READINESS_TIMEOUT_SECS";
/// Port base for TCP listeners (default: 9401).
pub const ESOTERICWEBB_PORT_BASE: &str = "ESOTERICWEBB_PORT_BASE";
/// Maximum characters for degraded AI summaries.
pub const ESOTERICWEBB_SUMMARY_LIMIT: &str = "ESOTERICWEBB_SUMMARY_LIMIT";
/// JSON output mode flag (`true` / `1` enables machine-readable output).
pub const ESOTERICWEBB_JSON: &str = "ESOTERICWEBB_JSON";
/// IPC retry maximum attempts (default: 2).
pub const ESOTERICWEBB_IPC_RETRY_MAX: &str = "ESOTERICWEBB_IPC_RETRY_MAX";
/// IPC retry initial backoff in milliseconds (default: 50).
pub const ESOTERICWEBB_IPC_RETRY_INITIAL_MS: &str = "ESOTERICWEBB_IPC_RETRY_INITIAL_MS";
/// IPC retry maximum backoff in milliseconds (default: 2000).
pub const ESOTERICWEBB_IPC_RETRY_MAX_MS: &str = "ESOTERICWEBB_IPC_RETRY_MAX_MS";
/// Circuit breaker failure threshold (default: 5).
pub const ESOTERICWEBB_IPC_CB_THRESHOLD: &str = "ESOTERICWEBB_IPC_CB_THRESHOLD";
/// Circuit breaker cooldown in seconds (default: 5).
pub const ESOTERICWEBB_IPC_CB_COOLDOWN_SECS: &str = "ESOTERICWEBB_IPC_CB_COOLDOWN_SECS";

// ── Deployment / plasmidBin ─────────────────────────────────────────────────

/// Path to ecosystem plasmidBin directory (primal binary artifacts).
pub const ECOPRIMALS_PLASMID_BIN: &str = "ECOPRIMALS_PLASMID_BIN";
/// biomeOS-specific plasmidBin directory override.
pub const BIOMEOS_PLASMID_BIN_DIR: &str = "BIOMEOS_PLASMID_BIN_DIR";
