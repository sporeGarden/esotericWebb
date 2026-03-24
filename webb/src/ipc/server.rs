// SPDX-License-Identifier: AGPL-3.0-or-later
//! Webb's own JSON-RPC server — re-exports from [`super::handlers`].
//!
//! This module exists for backward compatibility. New code should import
//! directly from `ipc::handlers`.

pub use super::handlers::{
    METHOD_SESSION_ACT, METHOD_SESSION_ACTIONS, METHOD_SESSION_GRAPH, METHOD_SESSION_HISTORY,
    METHOD_SESSION_NARRATE, METHOD_SESSION_START, METHOD_SESSION_STATE, SharedSession, dispatch,
    dispatch_with_session, new_shared_session,
};
