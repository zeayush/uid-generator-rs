// Note: std::env::set_var / remove_var require synchronisation across tests
// because Rust runs test functions in parallel threads within the same process.
// The ENV_LOCK mutex ensures env-var tests don't race each other.

use std::env;
use std::sync::Mutex;
use uid_generator_rs::{
    MACHINE_ID_ENV_VAR, MAX_MACHINE_ID, MachineIdError, machine_id_from_env,
    machine_id_from_hostname, machine_id_from_ip, resolve_machine_id,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_env(val: &str, f: impl FnOnce()) {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { env::set_var(MACHINE_ID_ENV_VAR, val) }; // safe: serialised by ENV_LOCK
    f();
    unsafe { env::remove_var(MACHINE_ID_ENV_VAR) };
}

fn without_env(f: impl FnOnce()) {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { env::remove_var(MACHINE_ID_ENV_VAR) };
    f();
}

// ── machine_id_from_env ───────────────────────────────────────────────────────

#[test]
fn env_valid_value() {
    with_env("42", || {
        assert_eq!(machine_id_from_env().unwrap(), 42);
    });
}

#[test]
fn env_zero_is_valid() {
    with_env("0", || {
        assert_eq!(machine_id_from_env().unwrap(), 0);
    });
}

#[test]
fn env_max_is_valid() {
    with_env(&MAX_MACHINE_ID.to_string(), || {
        assert_eq!(machine_id_from_env().unwrap(), MAX_MACHINE_ID);
    });
}

#[test]
fn env_unset_returns_error() {
    without_env(|| {
        let err = machine_id_from_env().unwrap_err();
        assert!(matches!(err, MachineIdError::EnvNotSet));
    });
}

#[test]
fn env_out_of_range_returns_error() {
    with_env("9999", || {
        let err = machine_id_from_env().unwrap_err();
        assert!(
            matches!(err, MachineIdError::InvalidValue(_)),
            "expected InvalidValue, got {err:?}"
        );
    });
}

#[test]
fn env_not_a_number_returns_error() {
    with_env("abc", || {
        let err = machine_id_from_env().unwrap_err();
        assert!(matches!(err, MachineIdError::InvalidValue(_)));
    });
}

// ── machine_id_from_hostname ──────────────────────────────────────────────────

#[test]
fn hostname_result_in_valid_range() {
    let id = machine_id_from_hostname().unwrap();
    assert!(
        id <= MAX_MACHINE_ID,
        "id {id} > MAX_MACHINE_ID {MAX_MACHINE_ID}"
    );
}

#[test]
fn hostname_is_deterministic() {
    let id1 = machine_id_from_hostname().unwrap();
    let id2 = machine_id_from_hostname().unwrap();
    assert_eq!(id1, id2, "hostname ID should be deterministic");
}

// ── machine_id_from_ip ────────────────────────────────────────────────────────

#[test]
fn ip_result_in_valid_range() {
    match machine_id_from_ip() {
        Ok(id) => assert!(
            id <= MAX_MACHINE_ID,
            "id {id} > MAX_MACHINE_ID {MAX_MACHINE_ID}"
        ),
        Err(e) => eprintln!("machine_id_from_ip skipped (no suitable IP in CI): {e}"),
    }
}

// ── resolve_machine_id ────────────────────────────────────────────────────────

#[test]
fn resolve_returns_valid_id_without_env() {
    without_env(|| {
        let id = resolve_machine_id().expect("resolve_machine_id failed");
        assert!(id <= MAX_MACHINE_ID);
    });
}

#[test]
fn resolve_env_takes_priority() {
    with_env("7", || {
        let id = resolve_machine_id().unwrap();
        assert_eq!(id, 7, "env-var value should take priority");
    });
}
