// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filesystem path resolution for primal discovery.
//!
//! Resolves candidate socket directories and `plasmidBin/` paths using
//! XDG conventions, environment overrides, and UID detection
//! (pure Rust, no libc).

use std::path::PathBuf;

/// Candidate directories where primal UDS sockets may be found.
pub(super) fn socket_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(dir) = std::env::var(crate::env_keys::BIOMEOS_SOCKET_DIR) {
        dirs.push(PathBuf::from(dir));
    }

    if let Ok(xdg) = std::env::var(crate::env_keys::XDG_RUNTIME_DIR) {
        dirs.push(PathBuf::from(xdg).join("biomeos"));
    }

    let uid = process_uid();
    dirs.push(PathBuf::from(format!("/run/user/{uid}/biomeos")));
    dirs.push(PathBuf::from(format!("/tmp/biomeos-{uid}")));

    dirs
}

/// Candidate `plasmidBin/` directories.
pub(super) fn plasmid_bin_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(dir) = std::env::var(crate::env_keys::ECOPRIMALS_PLASMID_BIN) {
        dirs.push(PathBuf::from(dir));
    }

    dirs.push(PathBuf::from("./plasmidBin"));
    dirs.push(PathBuf::from("../plasmidBin"));
    dirs.push(PathBuf::from("../../plasmidBin"));
    dirs.push(PathBuf::from("../../../plasmidBin"));

    dirs
}

/// Resolve the current user's UID for socket path construction.
///
/// Reads `/proc/self/status` (pure Rust, no libc) to get the real UID.
/// Falls back to `$UID` env var, then 0 on non-Unix.
pub(super) fn process_uid() -> u32 {
    #[cfg(unix)]
    {
        uid_from_proc_status().or_else(uid_from_env).unwrap_or(0)
    }
    #[cfg(not(unix))]
    {
        uid_from_env().unwrap_or(0)
    }
}

/// Parse real UID from `/proc/self/status` — toadStool sysmon pattern.
#[cfg(unix)]
pub(super) fn uid_from_proc_status() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            return rest.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

/// Fall back to `$UID` environment variable.
fn uid_from_env() -> Option<u32> {
    std::env::var(crate::env_keys::UID).ok()?.parse().ok()
}
