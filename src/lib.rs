//! # uid-generator-rs
//!
//! Snowflake and ULID unique ID generators — implemented from scratch as a learning exercise.
//!
//! ## Quick start (once you implement the stubs)
//!
//! ```rust,ignore
//! use uid_generator_rs::{Snowflake, Ulid, DEFAULT_EPOCH_MS, resolve_machine_id};
//! use std::str::FromStr;
//!
//! // ── Snowflake ─────────────────────────────────────────────────────────────
//! let machine_id = resolve_machine_id().unwrap();
//! let sf = Snowflake::new(machine_id, DEFAULT_EPOCH_MS).unwrap();
//! let id: u64 = sf.next_id().unwrap();
//! println!("Snowflake: {id}");
//!
//! // ── ULID ─────────────────────────────────────────────────────────────────
//! let ulid = Ulid::new().unwrap();
//! println!("ULID: {ulid}");                            // 01ARZ3NDEKTSV4RRFFQ69G5FAV
//! let parsed = Ulid::from_str(&ulid.to_string()).unwrap();
//! assert_eq!(ulid, parsed);
//! ```

pub mod machine_id;
pub mod snowflake;
pub mod ulid;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use machine_id::{
    machine_id_from_env, machine_id_from_hostname, machine_id_from_ip, resolve_machine_id,
    MachineIdError, MACHINE_ID_ENV_VAR,
};
pub use snowflake::{
    decompose_id, Snowflake, SnowflakeError, DEFAULT_EPOCH_MS, MAX_MACHINE_ID, MAX_SEQUENCE,
};
pub use ulid::{decode_base32_char, Ulid, UlidError};
