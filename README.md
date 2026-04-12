# uid-generator-rs

[![CI](https://github.com/zeayush/uid-generator-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/zeayush/uid-generator-rs/actions/workflows/rust-ci.yml)
![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)

Unique ID generation in Rust — Snowflake-style 64-bit IDs and ULID with
lexicographic sortability, built for hot paths.

Part of a distributed systems portfolio implementing every system from **Alex
Xu's System Design Interview (Vol. 1 & 2)**. This covers **Chapter 7 —
Design a Unique ID Generator in Distributed Systems**.

---

## What It Provides

- **Snowflake 64-bit IDs**: 41-bit timestamp, 10-bit machine ID, 12-bit sequence
- **ULID**: 48-bit timestamp + 80-bit randomness, string-sortable by time
- **Custom epoch**: choose a start date to push rollover horizon forward
- **Machine ID resolution**: env var, IP hash, or hostname hash
- **Clock drift protection**: backward clock movement parks until safe time
- **Sequence exhaustion handling**: parks at 4096 IDs/ms ceiling

---

## How It Works

### Snowflake (64-bit)

```text
  63        22        12        0
  ┌──────────┬─────────┬────────┐
  │  41-bit  │ 10-bit  │ 12-bit │
  │ timestamp│machine  │sequence│
  └──────────┴─────────┴────────┘
```

- `timestamp`: milliseconds since configured epoch (`DEFAULT_EPOCH_MS`)
- `machine`: stable node identifier in `[0, 1023]`
- `sequence`: per-millisecond counter in `[0, 4095]`

ID assembly:

```text
id = (timestamp_ms << 22) | (machine_id << 12) | sequence
```

Generation behavior:

- Same millisecond: increment sequence
- Sequence overflow: park until next millisecond
- Clock moves backward: park until clock is safe again

### ULID (128-bit)

```text
bytes 0..5   -> 48-bit timestamp (ms since Unix epoch)
bytes 6..15  -> 80-bit cryptographic randomness
```

- Encoded with Crockford Base32 into a 26-char string
- Lexicographic string order matches chronological order
- Parser accepts Crockford aliases (`I/L -> 1`, `O -> 0`)

### Machine ID Resolution

Resolution order:

1. `UID_MACHINE_ID` environment variable
2. Primary outbound IPv4 hash (FNV-1a)
3. Hostname hash (FNV-1a)

---

## Quick Start

```toml
[dependencies]
uid-generator-rs = { git = "https://github.com/zeayush/uid-generator-rs" }
```

```rust
use std::sync::Arc;
use uid_generator_rs::{decompose_id, resolve_machine_id, Snowflake, Ulid, DEFAULT_EPOCH_MS};

let machine_id = resolve_machine_id()?;
let sf = Arc::new(Snowflake::new(machine_id, DEFAULT_EPOCH_MS)?);

let id = sf.next_id()?;
let (ts_ms, machine, seq) = decompose_id(id);

let ulid = Ulid::new()?;
let parsed: Ulid = ulid.to_string().parse()?;
assert!(parsed >= ulid);
# Ok::<(), Box<dyn std::error::Error>>(())
```

---

## API

```rust
pub const DEFAULT_EPOCH_MS: u64;
pub const MAX_MACHINE_ID: u64; // 1023
pub const MAX_SEQUENCE: u64;   // 4095

pub struct Snowflake;
impl Snowflake {
  pub fn new(machine_id: u64, epoch_ms: u64) -> Result<Self, SnowflakeError>;
  pub fn next_id(&self) -> Result<u64, SnowflakeError>;
}

pub fn decompose_id(id: u64) -> (u64, u64, u64);

pub struct Ulid;
impl Ulid {
  pub fn new() -> Result<Self, UlidError>;
  pub fn with_ms(ms: u64) -> Result<Self, UlidError>;
  pub fn timestamp_ms(&self) -> u64;
}

pub fn machine_id_from_env() -> Result<u64, MachineIdError>;
pub fn machine_id_from_ip() -> Result<u64, MachineIdError>;
pub fn machine_id_from_hostname() -> Result<u64, MachineIdError>;
pub fn resolve_machine_id() -> Result<u64, MachineIdError>;
```

---

## Benchmarks

```sh
cargo bench
```

Current target:

| Benchmark | Target |
|---|---|
| `snowflake/next_id` | >= 4,000,000 IDs/sec |
| `ulid/new` | >= 1,000,000 IDs/sec |

Snowflake has a hard architectural ceiling of $4096 \times 1000 = 4{,}096{,}000$
IDs/sec from its 12-bit sequence field.

---

## Tests

```sh
cargo test
```

Test suite covers uniqueness, monotonicity, machine ID boundaries, concurrent
generation, sequence overflow behavior, ULID round-trips, parse validation,
and machine ID source resolution.

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
