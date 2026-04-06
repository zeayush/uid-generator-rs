# uid-generator-rs

![CI](https://github.com/zeayush/uid-generator-rs/actions/workflows/rust-ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)

Two production-grade unique ID generators implemented from scratch in Rust — a **Snowflake** 64-bit integer generator and a **ULID** 128-bit lexicographically sortable identifier.

> **Learning exercise** — function bodies are left as `todo!()` stubs for you to implement. Tests are pre-written and pass once your implementations are correct. Once complete, this is a production-ready library.

---

## What are these?

**Snowflake** IDs were invented by Twitter in 2010 to replace database auto-increment in distributed systems. A single `u64` embeds a millisecond timestamp, a machine ID, and a per-millisecond sequence counter — so IDs generated across thousands of machines are unique without any coordination or central authority.

**ULID** (Universally Unique Lexicographically Sortable Identifier) solves a different problem: UUID is globally unique but sorts randomly in indexes, causing B-tree fragmentation and poor write locality. ULID packs a 48-bit timestamp into the high bits so string-sorted order equals time-sorted order.

---

## How it Works

### Snowflake — bit packing

```
  63        22        12        0
  ┌──────────┬─────────┬────────┐
  │  41-bit  │ 10-bit  │ 12-bit │
  │ timestamp│machine  │sequence│
  └──────────┴─────────┴────────┘

  epoch: 2020-01-01 00:00:00 UTC  (DEFAULT_EPOCH_MS, configurable)
```

- **timestamp** — milliseconds since the custom epoch. 41 bits ≈ 69 years before rollover (valid past 2089).
- **machine ID** — unique per generator instance (0–1023). Resolved from env var, IP hash, or hostname hash.
- **sequence** — wraps at 4095 per millisecond; generator parks (busy-waits) until the next ms tick when exhausted.

Assembly and decomposition:

```
id = (timestamp_ms << 22) | (machine_id << 12) | sequence
```

The generator is wrapped in a `Mutex<State>` — share it across threads via `Arc<Snowflake>`.

### ULID — timestamp prefix + entropy suffix

```
  01ARZ3NDEKTSV4RRFFQ69G5FAV
  ├──────────┤├──────────────┤
   10 chars    16 chars
   48-bit ms   80-bit random
```

128 raw bytes are encoded as 26 Crockford Base32 characters (omits `I L O U` to prevent transcription errors). Two ULIDs from different milliseconds sort correctly as plain strings. `Ord` is implemented so `Ulid` values sort naturally in `BTreeMap`s and with `.sort()`.

---

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
uid-generator-rs = { path = "." }   # update once published to crates.io
```

#### Snowflake

```rust
use std::sync::Arc;
use uid_generator_rs::{Snowflake, DEFAULT_EPOCH_MS, resolve_machine_id, decompose_id};

let machine_id = resolve_machine_id().unwrap();
let sf = Arc::new(Snowflake::new(machine_id, DEFAULT_EPOCH_MS).unwrap());

let id: u64 = sf.next_id().unwrap();          // monotonically increasing
let (ts_ms, machine, seq) = decompose_id(id); // inspect each field
```

#### ULID

```rust
use uid_generator_rs::Ulid;

let u: Ulid  = Ulid::new().unwrap();
let s: String = u.to_string();                  // "01HXK7P9ZQABCDEFGHJKMNPQ"

let u2: Ulid = s.parse::<Ulid>().unwrap();     // FromStr
let ms: u64  = u2.timestamp_ms();              // milliseconds since Unix epoch
assert!(u <= u2);                              // Ord — sorts by time
```

#### Machine ID

```rust
use uid_generator_rs::{machine_id_from_env, machine_id_from_ip, machine_id_from_hostname, resolve_machine_id};

let mid = resolve_machine_id().unwrap();          // env → IP → hostname

let mid = machine_id_from_env().unwrap();         // reads UID_MACHINE_ID env var
let mid = machine_id_from_ip().unwrap();          // FNV-1a hash via UDP socket trick
let mid = machine_id_from_hostname().unwrap();    // FNV-1a hash of HOSTNAME
```

---

## API

### Snowflake

```rust
pub const DEFAULT_EPOCH_MS: u64;
pub const MAX_MACHINE_ID:   u64;   // 1023
pub const MAX_SEQUENCE:     u64;   // 4095

impl Snowflake {
    pub fn new(machine_id: u64, epoch_ms: u64) -> Result<Self, SnowflakeError>;
    pub fn next_id(&self)                       -> Result<u64,  SnowflakeError>;
}

pub fn decompose_id(id: u64) -> (u64, u64, u64); // (timestamp_ms, machine_id, sequence)
```

`next_id` is safe for concurrent use (`&self` via internal `Mutex`). Use `Arc<Snowflake>` to share across threads.

### ULID

```rust
impl Ulid {
    pub fn new()             -> Result<Self, UlidError>;
    pub fn with_ms(ms: u64)  -> Result<Self, UlidError>;
    pub fn timestamp_ms(&self) -> u64;
}

impl Display  for Ulid { /* 26-char Crockford Base32 */ }
impl FromStr  for Ulid { type Err = UlidError; }
impl Ord      for Ulid { /* lexicographic == time order */ }
impl PartialOrd for Ulid {}
```

`parse::<Ulid>()` accepts upper- and lower-case input and maps `I→1`, `L→1`, `O→0` per the Crockford spec.

### Machine ID

```rust
pub const MACHINE_ID_ENV_VAR: &str = "UID_MACHINE_ID";

pub fn machine_id_from_env()      -> Result<u64, MachineIdError>;
pub fn machine_id_from_hostname() -> Result<u64, MachineIdError>;
pub fn machine_id_from_ip()       -> Result<u64, MachineIdError>;
pub fn resolve_machine_id()       -> Result<u64, MachineIdError>; // tries in priority order
```

---

## Traits to Implement

| Trait | Type | Effect |
|---|---|---|
| `Display` | `Ulid` | `ulid.to_string()` → 26-char string |
| `FromStr` | `Ulid` | `"01ARZ…".parse::<Ulid>()` |
| `Ord` / `PartialOrd` | `Ulid` | `u1 < u2`, `.sort()`, `BTreeMap` keys |
| `Display` | `SnowflakeError` | human-readable error messages |
| `Display` | `UlidError` | human-readable error messages |
| `Display` | `MachineIdError` | human-readable error messages |

All `Error` trait implementations are already provided.

---

## Key Design Decisions

| Decision | Rationale |
|---|---|
| Custom epoch (2020-01-01) | Shifts the 41-bit counter forward — IDs stay valid past 2089 vs 2039 with a Unix epoch |
| `Mutex<State>` inside `Snowflake` | Safe interior mutability; share via `Arc<Snowflake>` across threads with no extra ceremony |
| Busy-wait on sequence exhaustion | Avoids `thread::sleep` jitter — the spin completes in < 1 ms in practice |
| `getrandom` for ULID entropy | OS entropy via `getrandom` syscall; zero transitive dependencies; same source as `rand_core` |
| FNV-1a 32-bit for machine ID hash | No external deps; deterministic; single-pass; hardware-friendly on x86 and ARM |
| Crockford Base32 for ULID encoding | 5 bits/character is maximally efficient; omitting `I L O U` prevents hand-transcription errors |
| `[u8; 16]` as the ULID backing type | Zero-cost `Ord` — slice comparison is already lexicographic; no custom `PartialOrd` logic needed |

---

## Benchmarks

```bash
cargo bench
```

Target performance (single-threaded, post-implementation):

| Benchmark | Target |
|---|---|
| `snowflake/next_id` | ≥ 4 000 000 IDs/sec |
| `snowflake/next_id_parallel` | scales with threads (mutex-bound) |
| `ulid/new` | ≥ 1 000 000 IDs/sec |
| `ulid/parse` | ≥ 5 000 000 parses/sec |

The Snowflake ceiling of 4 096 000 IDs/sec is architectural (12-bit sequence field) — not a code quality issue. The ULID ceiling is bounded by `getrandom` throughput. Benchmarks are run with `criterion` and produce an HTML report in `target/criterion/`.

---

## Tests

```bash
cargo test
```

```bash
# With output visible
cargo test -- --nocapture

# Single test
cargo test next_id_is_monotonically_increasing
```

**Snowflake** (`tests/snowflake_tests.rs`, 10 tests): out-of-range machine ID, zero machine ID, max machine ID, 10k unique IDs, 5k monotonic, concurrent no-duplicates, machine ID embedded in output, decompose round-trip, sequence overflow parking, custom epoch timestamp field.

**ULID** (`tests/ulid_tests.rs`, 9 tests): 26-char output, 1k unique, sort order, display-then-parse round-trip, invalid length error, invalid char error, `timestamp_ms` round-trip, `Ord` ordering matches string order, case-insensitive parse.

**Machine ID** (`tests/machine_id_tests.rs`, 5 tests): valid env var, unset env var error, out-of-range error, hostname result in `[0, MAX_MACHINE_ID]`, fallthrough to hostname when env unset.

---

## Project Structure

```
uid-generator-rs/
├── src/
│   ├── lib.rs              ← crate root + re-exports
│   ├── snowflake.rs        ← Snowflake generator   (your implementation)
│   ├── ulid.rs             ← ULID generator        (your implementation)
│   └── machine_id.rs       ← Machine ID sources    (your implementation)
├── tests/
│   ├── snowflake_tests.rs  ← pre-written
│   ├── ulid_tests.rs       ← pre-written
│   └── machine_id_tests.rs ← pre-written
├── benches/
│   └── benchmark.rs        ← criterion benchmarks
├── .github/
│   └── workflows/
│       └── rust-ci.yml     ← CI: test + clippy + bench smoke
├── Cargo.toml
└── README.md
```

---

## Implementation Guide

Work through the files in this order — each step builds on the previous one.

### Step 1 — `src/machine_id.rs`

Start here because `Snowflake` depends on a valid machine ID.

1. `machine_id_from_env` — `std::env::var(MACHINE_ID_ENV_VAR)`, parse as `u64`, validate `[0, MAX_MACHINE_ID]`.
2. `machine_id_from_hostname` — read `HOSTNAME` / `COMPUTERNAME` env var, apply `fnv1a_32`, mask with `MAX_MACHINE_ID`.
3. `machine_id_from_ip` — UDP-socket trick: connect to `8.8.8.8:80`, call `local_addr()`, apply `fnv1a_32` to the 4-byte IP.
4. `resolve_machine_id` — try each source in priority order, return the first `Ok`.

### Step 2 — `src/snowflake.rs`

1. `Snowflake::new` — validate `machine_id`, construct `Mutex<State>`.
2. `current_ms` — `SystemTime::now().duration_since(UNIX_EPOCH)`, subtract `epoch_ms`.
3. `wait_next_ms` — spin calling `current_ms()` until result exceeds `last`.
4. `next_id` — lock the mutex, implement clock drift → same-ms → new-ms → compose, unlock.
5. `decompose_id` — free function: right-shift and mask each field.

### Step 3 — `src/ulid.rs`

1. `Ulid::with_ms` — validate 48-bit range, big-endian encode ms into bytes 0–5, fill bytes 6–15 with `getrandom::getrandom`.
2. `Display` — pack 128 bits into 26 × 5-bit Crockford characters (bit table is in the source comment).
3. `FromStr` — validate 26-char length, map each char through `decode_base32_char`, reassemble the `[u8; 16]`.
4. `timestamp_ms` — reconstruct ms from bytes 0–5 (reverse of encoding).
5. `Ord` — compare `self.0` and `other.0` as slices (slices implement `Ord` lexicographically).
6. `decode_base32_char` — Crockford lookup table with aliases `I→1`, `L→1`, `O→0`.

---

## Machine ID Resolution Order

| Priority | Source | Mechanism |
|---|---|---|
| 1 | `UID_MACHINE_ID` env var | Explicit integer override — use this in production |
| 2 | Primary non-loopback IPv4 | FNV-1a hash via UDP socket trick (`connect` to `8.8.8.8:80`, read `local_addr`) |
| 3 | `HOSTNAME` / `COMPUTERNAME` | FNV-1a hash of the hostname env var |

Set `UID_MACHINE_ID=<n>` to guarantee the same machine ID across restarts and redeployments.

---

## References

- [Announcing Snowflake — Twitter Engineering Blog (2010)](https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake)
- [ULID Specification — github.com/ulid/spec](https://github.com/ulid/spec)
- [Crockford Base32 — crockford.com](https://www.crockford.com/base32.html)
- [FNV Hash — Fowler, Noll, Vo](http://www.isthe.com/chongo/tech/comp/fnv/)
- [getrandom crate docs](https://docs.rs/getrandom)
- [criterion benchmarking guide](https://bheisler.github.io/criterion.rs/book/)
- [ulid-rs — reference ULID implementation in Rust](https://github.com/dylanhart/ulid-rs)
- [System Design Interview Vol. 1, Ch. 7 — Design a Unique ID Generator in Distributed Systems](https://www.amazon.com/System-Design-Interview-insiders-Second/dp/B08CMF2CQF)
