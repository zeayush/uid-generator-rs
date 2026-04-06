use std::collections::HashSet;
use uid_generator_rs::{
    decompose_id, Snowflake, SnowflakeError, DEFAULT_EPOCH_MS, MAX_MACHINE_ID, MAX_SEQUENCE,
};

#[test]
fn new_snowflake_rejects_out_of_range_machine_id() {
    let err = Snowflake::new(MAX_MACHINE_ID + 1, DEFAULT_EPOCH_MS).unwrap_err();
    assert!(
        matches!(err, SnowflakeError::InvalidMachineId(_)),
        "expected InvalidMachineId, got {err}"
    );
}

#[test]
fn new_snowflake_accepts_zero_machine_id() {
    Snowflake::new(0, DEFAULT_EPOCH_MS).expect("machine_id=0 should be valid");
}

#[test]
fn new_snowflake_accepts_max_machine_id() {
    Snowflake::new(MAX_MACHINE_ID, DEFAULT_EPOCH_MS)
        .expect("MAX_MACHINE_ID should be valid");
}

#[test]
fn next_id_produces_unique_ids() {
    let sf = Snowflake::new(1, DEFAULT_EPOCH_MS).unwrap();
    let mut seen = HashSet::with_capacity(10_000);
    for _ in 0..10_000 {
        let id = sf.next_id().unwrap();
        assert!(seen.insert(id), "duplicate ID: {id}");
    }
}

#[test]
fn next_id_is_monotonically_increasing() {
    let sf = Snowflake::new(1, DEFAULT_EPOCH_MS).unwrap();
    let mut prev = sf.next_id().unwrap();
    for _ in 0..5_000 {
        let id = sf.next_id().unwrap();
        assert!(id > prev, "ID not monotonic: {id} <= {prev}");
        prev = id;
    }
}

#[test]
fn next_id_concurrent_no_duplicates() {
    use std::sync::Arc;

    let sf = Arc::new(Snowflake::new(1, DEFAULT_EPOCH_MS).unwrap());
    let mut handles = Vec::with_capacity(10);

    for _ in 0..10 {
        let sf = Arc::clone(&sf);
        handles.push(std::thread::spawn(move || {
            (0..1_000).map(|_| sf.next_id().unwrap()).collect::<Vec<_>>()
        }));
    }

    let all: Vec<u64> = handles
        .into_iter()
        .flat_map(|h| h.join().expect("thread panicked"))
        .collect();

    let mut seen = HashSet::with_capacity(all.len());
    for id in all {
        assert!(seen.insert(id), "duplicate concurrent ID: {id}");
    }
}

#[test]
fn decompose_id_extracts_correct_machine_id() {
    let machine_id = 42;
    let sf = Snowflake::new(machine_id, DEFAULT_EPOCH_MS).unwrap();
    let id = sf.next_id().unwrap();
    let (_, mid, _) = decompose_id(id);
    assert_eq!(mid, machine_id, "machine_id mismatch");
}

#[test]
fn decompose_id_sequence_in_range() {
    let sf = Snowflake::new(1, DEFAULT_EPOCH_MS).unwrap();
    let id = sf.next_id().unwrap();
    let (_, _, seq) = decompose_id(id);
    assert!(
        seq <= MAX_SEQUENCE,
        "sequence {seq} exceeds MAX_SEQUENCE {MAX_SEQUENCE}"
    );
}

#[test]
fn decompose_id_timestamp_in_range() {
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        - DEFAULT_EPOCH_MS;

    let sf = Snowflake::new(1, DEFAULT_EPOCH_MS).unwrap();
    let id = sf.next_id().unwrap();

    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        - DEFAULT_EPOCH_MS;

    let (ts_ms, _, _) = decompose_id(id);
    assert!(
        ts_ms >= before && ts_ms <= after + 1,
        "timestamp {ts_ms} outside window [{before}, {after}]"
    );
}

#[test]
fn sequence_exhaustion_ids_remain_unique_and_monotonic() {
    let sf = Snowflake::new(1, DEFAULT_EPOCH_MS).unwrap();
    // Generate 3× the sequence capacity, forcing the generator to park and
    // advance the millisecond counter at least twice.
    let n = (MAX_SEQUENCE as usize + 1) * 3;
    let mut prev = sf.next_id().unwrap();
    let mut seen = HashSet::with_capacity(n);
    seen.insert(prev);

    for i in 0..n {
        let id = sf.next_id().unwrap();
        assert!(id > prev, "non-monotonic at i={i}: {id} <= {prev}");
        assert!(seen.insert(id), "duplicate at i={i}: {id}");
        prev = id;
    }
}
