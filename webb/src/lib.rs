// SPDX-License-Identifier: AGPL-3.0-or-later
//! Esoteric Webb — working tool for CRPG composition.
//!
//! A standalone cross-evolution substrate that composes deployed primals
//! (from `plasmidBin/`) into a Disco Elysium-inspired CRPG with DAG-traced
//! narrative, deep NPCs, emergent ability interactions, and meaningful
//! bounded endings. Webb is informed by the science in the springs but
//! consumes only primal capabilities — never spring source code.
//!
//! ## Architecture
//!
//! Webb consumes primals via JSON-RPC IPC — zero Rust crate dependencies on
//! any spring. Primal binaries resolved from `plasmidBin/` or discovered
//! via Songbird. The runtime loop:
//!
//! 1. Receive player input (visualization primal poll)
//! 2. Resolve against current scene and valid exits
//! 3. Evaluate state predicates against [`state::WorldState`]
//! 4. Apply state effects
//! 5. Record provenance vertex (provenance primal)
//! 6. Request narration (game science primal + AI primal)
//! 7. Push scene (visualization primal)
//! 8. Evaluate metrics (flow, engagement, DDA via game science primal)
//!
//! ## Content
//!
//! Creative teams author YAML files (worlds, NPCs, abilities, scenes,
//! narrative graphs) that the runtime loads, validates, and traverses.

pub mod autoplay;
pub mod content;
pub mod director;
pub mod experiment;
pub mod ipc;
pub mod narrative;
pub mod niche;
pub mod session;
pub mod state;
