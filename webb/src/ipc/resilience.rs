// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC resilience primitives (neuralSpring / groundSpring pattern).
//!
//! Transport-agnostic [`RetryPolicy`] (exponential backoff) and
//! [`CircuitBreaker`] (Closed / Open / `HalfOpen`) for primal IPC calls.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Configurable exponential-backoff retry policy.
///
/// Delays grow as `initial_delay * multiplier^attempt`, capped at
/// `max_delay`. Configured via environment variables with sensible defaults.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum retry attempts (0 = no retries).
    pub max_retries: u32,
    /// Delay before the first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier applied per attempt.
    pub multiplier: f64,
}

impl RetryPolicy {
    /// Compute the delay for a given zero-based attempt index.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.initial_delay.as_secs_f64() * self.multiplier.powf(f64::from(attempt));
        let capped = base.min(self.max_delay.as_secs_f64());
        Duration::from_secs_f64(capped.max(0.0))
    }

    /// Load policy from environment variables with defaults.
    ///
    /// - `ESOTERICWEBB_IPC_RETRY_MAX` (default 2)
    /// - `ESOTERICWEBB_IPC_RETRY_INITIAL_MS` (default 50)
    /// - `ESOTERICWEBB_IPC_RETRY_MAX_MS` (default 2000)
    #[must_use]
    pub fn from_env() -> Self {
        let max_retries = env_parse(crate::env_keys::ESOTERICWEBB_IPC_RETRY_MAX, 2);
        let initial_ms: u64 = env_parse(crate::env_keys::ESOTERICWEBB_IPC_RETRY_INITIAL_MS, 50);
        let max_ms: u64 = env_parse(crate::env_keys::ESOTERICWEBB_IPC_RETRY_MAX_MS, 2000);
        Self {
            max_retries,
            initial_delay: Duration::from_millis(initial_ms),
            max_delay: Duration::from_millis(max_ms),
            multiplier: 2.0,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(2),
            multiplier: 2.0,
        }
    }
}

/// Three-state circuit breaker preventing cascading failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — requests pass through.
    Closed,
    /// Failures exceeded threshold — requests are short-circuited.
    Open,
    /// Cooldown elapsed — one probe request is allowed.
    HalfOpen,
}

/// Thread-safe circuit breaker with configurable failure threshold and
/// cooldown period.
///
/// State transitions:
/// - `Closed` -> `Open`: after `threshold` consecutive failures
/// - `Open` -> `HalfOpen`: after `cooldown` elapses
/// - `HalfOpen` -> `Closed`: on success
/// - `HalfOpen` -> `Open`: on failure
pub struct CircuitBreaker {
    threshold: u32,
    cooldown: Duration,
    consecutive_failures: AtomicU32,
    last_failure_epoch_ms: AtomicU64,
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("state", &self.state())
            .field("threshold", &self.threshold)
            .field("cooldown", &self.cooldown)
            .field(
                "consecutive_failures",
                &self.consecutive_failures.load(Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// - `threshold`: consecutive failures to open the circuit
    /// - `cooldown`: duration to wait before transitioning to `HalfOpen`
    #[must_use]
    pub const fn new(threshold: u32, cooldown: Duration) -> Self {
        Self {
            threshold,
            cooldown,
            consecutive_failures: AtomicU32::new(0),
            last_failure_epoch_ms: AtomicU64::new(0),
        }
    }

    /// Load configuration from environment variables.
    ///
    /// - `ESOTERICWEBB_IPC_CB_THRESHOLD` (default 5)
    /// - `ESOTERICWEBB_IPC_CB_COOLDOWN_SECS` (default 5)
    #[must_use]
    pub fn from_env() -> Self {
        let threshold = env_parse(crate::env_keys::ESOTERICWEBB_IPC_CB_THRESHOLD, 5);
        let cooldown_secs = env_parse::<u64>(crate::env_keys::ESOTERICWEBB_IPC_CB_COOLDOWN_SECS, 5);
        Self::new(threshold, Duration::from_secs(cooldown_secs))
    }

    /// Current circuit state.
    #[must_use]
    pub fn state(&self) -> CircuitState {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures < self.threshold {
            return CircuitState::Closed;
        }
        let last_failure = self.last_failure_epoch_ms.load(Ordering::Relaxed);
        let now = epoch_ms();
        let cooldown_ms = u64::try_from(self.cooldown.as_millis()).unwrap_or(u64::MAX);
        if now.saturating_sub(last_failure) >= cooldown_ms {
            CircuitState::HalfOpen
        } else {
            CircuitState::Open
        }
    }

    /// Whether the circuit allows a request through.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self.state(), CircuitState::Closed | CircuitState::HalfOpen)
    }

    /// Record a successful call — resets the failure counter.
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    /// Record a failed call — increments counter and updates timestamp.
    pub fn record_failure(&self) {
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        self.last_failure_epoch_ms
            .store(epoch_ms(), Ordering::Relaxed);
    }
}

/// Whether an [`IpcError`](super::envelope::IpcError) is transient and
/// worth retrying.
///
/// Delegates to [`IpcError::is_recoverable`](super::envelope::IpcError::is_recoverable)
/// for ecosystem-wide consistency.
#[must_use]
pub const fn is_recoverable(err: &super::envelope::IpcError) -> bool {
    err.is_recoverable()
}

fn epoch_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delay_grows_exponentially() {
        let policy = RetryPolicy::default();
        let d0 = policy.delay_for_attempt(0);
        let d1 = policy.delay_for_attempt(1);
        let d2 = policy.delay_for_attempt(2);
        assert!(d1 > d0);
        assert!(d2 > d1);
    }

    #[test]
    fn retry_delay_capped_at_max() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(500),
            multiplier: 10.0,
        };
        for attempt in 0..10 {
            assert!(policy.delay_for_attempt(attempt) <= policy.max_delay);
        }
    }

    #[test]
    fn circuit_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_allowed());
    }

    #[test]
    fn circuit_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_allowed());
    }

    #[test]
    fn circuit_resets_on_success() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_allowed());
    }

    #[test]
    fn circuit_half_open_after_cooldown() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(0));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(cb.is_allowed());
    }

    #[test]
    fn is_recoverable_aligns_with_ecosystem() {
        use super::super::envelope::IpcError;
        assert!(is_recoverable(&IpcError::ConnectionRefused(
            "refused".to_owned()
        )));
        assert!(is_recoverable(&IpcError::ConnectionReset(
            "reset".to_owned()
        )));
        assert!(is_recoverable(&IpcError::Timeout { ms: 5000 }));
        assert!(is_recoverable(&IpcError::ApplicationError {
            code: crate::ipc::envelope::ERROR_INTERNAL,
            message: "internal".to_owned(),
        }));
        assert!(!is_recoverable(&IpcError::MethodNotFound {
            method: "health.check".to_owned(),
        }));
        assert!(!is_recoverable(&IpcError::Serialization {
            detail: "parse".to_owned(),
        }));
        assert!(!is_recoverable(&IpcError::PrimalNotFound {
            domain: "ai".to_owned(),
        }));
    }
}
