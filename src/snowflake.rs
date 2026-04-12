//! Snowflake 64-bit unique ID generator.
//!
//! Bit layout:
//!
//! ```text
//!  63        22        12        0
//!  ┌──────────┬─────────┬────────┐
//!  │  41-bit  │ 10-bit  │ 12-bit │
//!  │ timestamp│machine  │sequence│
//!  └──────────┴─────────┴────────┘
//! ```

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Bit-layout constants ──────────────────────────────────────────────────────

pub const SEQUENCE_BITS: u64 = 12;
pub const MACHINE_ID_BITS: u64 = 10;
pub const TIMESTAMP_BITS: u64 = 41;

pub const MACHINE_ID_SHIFT: u64 = SEQUENCE_BITS;
pub const TIMESTAMP_SHIFT: u64 = SEQUENCE_BITS + MACHINE_ID_BITS;

/// Largest valid machine ID (2^10 − 1 = 1 023).
pub const MAX_MACHINE_ID: u64 = (1 << MACHINE_ID_BITS) - 1;
/// Largest valid sequence number (2^12 − 1 = 4 095).
pub const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;

/// Default epoch: 2020-01-01 00:00:00 UTC as milliseconds since UNIX_EPOCH.
pub const DEFAULT_EPOCH_MS: u64 = 1_577_836_800_000;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SnowflakeError {
    InvalidMachineId(u64),
    ClockDrift { last_ms: u64, now_ms: u64 },
    SystemTime(String),
    PoisonedLock,
}

impl std::fmt::Display for SnowflakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMachineId(id) => {
                write!(f, "machine ID {id} exceeds maximum {MAX_MACHINE_ID}")
            }
            Self::ClockDrift { last_ms, now_ms } => {
                write!(f, "clock went backward: last={last_ms} now={now_ms}")
            }
            Self::SystemTime(e) => write!(f, "system clock error: {e}"),
            Self::PoisonedLock => write!(f, "internal mutex was poisoned"),
        }
    }
}

impl std::error::Error for SnowflakeError {}

// ── Internal state (kept behind the mutex) ────────────────────────────────────

#[derive(Debug)]
struct State {
    epoch_ms: u64,
    machine_id: u64,
    sequence: u64,
    last_ms: u64,
}

// ── Generator ─────────────────────────────────────────────────────────────────

/// Thread-safe Snowflake ID generator.
///
/// Create once per process/service and share via `Arc<Snowflake>`.
#[derive(Debug)]
pub struct Snowflake {
    inner: Mutex<State>,
}

impl Snowflake {
    /// Creates a new generator.
    ///
    /// - `machine_id` must be in `[0, MAX_MACHINE_ID]`.
    /// - `epoch_ms` is milliseconds since UNIX_EPOCH for your custom epoch;
    ///   pass [`DEFAULT_EPOCH_MS`] to use the built-in 2020-01-01 default.
    pub fn new(machine_id: u64, epoch_ms: u64) -> Result<Self, SnowflakeError> {
        if machine_id > MAX_MACHINE_ID {
            return Err(SnowflakeError::InvalidMachineId(machine_id));
        }
        Ok(Snowflake {
            inner: Mutex::new(State {
                epoch_ms,
                machine_id,
                sequence: 0,
                last_ms: 0,
            }),
        })
    }

    /// Returns milliseconds elapsed since the configured epoch.
    ///
    /// Hints:
    /// ```ignore
    /// use std::time::{SystemTime, UNIX_EPOCH};
    /// let dur = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .map_err(|e| SnowflakeError::SystemTime(e.to_string()))?;
    /// Ok(dur.as_millis() as u64 - epoch_ms)
    /// ```
    fn current_ms(epoch_ms: u64) -> Result<u64, SnowflakeError> {
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SnowflakeError::SystemTime(e.to_string()))?;
        Ok(dur.as_millis() as u64 - epoch_ms)
    }

    /// Spins until `current_ms()` returns a value strictly greater than `last`.
    /// Call this when the sequence counter overflows `MAX_SEQUENCE`.
    fn wait_next_ms(epoch_ms: u64, last: u64) -> Result<u64, SnowflakeError> {
        loop {
            let ms = Self::current_ms(epoch_ms)?;
            if ms > last {
                return Ok(ms);
            }
        }
    }

    /// Generates the next unique 64-bit Snowflake ID.
    ///
    /// Thread-safe — blocks on the internal mutex for the duration of the call.
    pub fn next_id(&self) -> Result<u64, SnowflakeError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| SnowflakeError::PoisonedLock)?;

        let mut now = Self::current_ms(inner.epoch_ms)?;

        // Clock drift protection: if the wall clock moved backward, park until
        // we observe a millisecond strictly greater than the last emitted one.
        if now < inner.last_ms {
            now = Self::wait_next_ms(inner.epoch_ms, inner.last_ms)?;
        }

        if now == inner.last_ms {
            inner.sequence = (inner.sequence + 1) & MAX_SEQUENCE;
            if inner.sequence == 0 {
                // sequence wrapped — wait for the clock to advance
                now = Self::wait_next_ms(inner.epoch_ms, inner.last_ms)?;
            }
        }

        if now > inner.last_ms {
            inner.sequence = 0;
            inner.last_ms = now;
        }

        let id = (now << TIMESTAMP_SHIFT) | (inner.machine_id << MACHINE_ID_SHIFT) | inner.sequence;
        Ok(id)
    }
}

// ── Decompose ─────────────────────────────────────────────────────────────────

/// Decomposes a Snowflake ID into `(timestamp_ms_since_epoch, machine_id, sequence)`.
pub fn decompose_id(id: u64) -> (u64, u64, u64) {
    let sequence = id & MAX_SEQUENCE;
    let machine_id = (id >> MACHINE_ID_SHIFT) & MAX_MACHINE_ID;
    let ts_ms = id >> TIMESTAMP_SHIFT;
    (ts_ms, machine_id, sequence)
}
