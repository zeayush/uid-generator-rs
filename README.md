# uid-generator-rs

A learning-focused implementation of two production-grade unique ID systems written in Rust.

> **Learning exercise** — the function bodies are left as `todo!()` stubs for you to implement. Tests are pre-written and will all pass once your implementations are correct.

---

## Features

| Feature | Snowflake | ULID |
|---|---|---|
| Size | 64-bit `u64` | 128-bit (26-char string) |
| Timestamp precision | millisecond | millisecond |
| Lexicographic sort | ✓ (as `u64`) | ✓ (`Ord` implemented) |
| Uniqueness guarantee | machine ID + sequence | cryptographic randomness |
| Throughput target | ≥ 4 000 000 / sec | ≥ 1 000 000 / sec |
| Custom epoch | ✓ | — |
| Clock drift protection | ✓ | — |

---

## ID Formats

### Snowflake — 64-bit `u64`

```
 63        22        12        0
 ┌──────────┬─────────┬────────┐
 │  41-bit  │ 10-bit  │ 12-bit │
 │ timestamp│machine  │sequence│
 └──────────┴─────────┴────────┘
```

- **timestamp** — milliseconds since `DEFAULT_EPOCH_MS` (2020-01-01 UTC). 41 bits → ~69 years.
- **machine ID** — unique ID for this generator instance (0–1023). Resolved from env var, IP hash, or hostname hash.
- **sequence** — resets each ms. 12 bits → 4096 values/ms → 4 096 000 IDs/sec before parking.

### ULID — 26-character string

```
 01ARZ3NDEKTSV4RRFFQ69G5FAV
 │          │               │
 10 chars   16 chars
 48-bit ms  80-bit random
```

- **timestamp** — 48-bit millisecond Unix timestamp.
- **random** — 80 bits from the OS entropy source via `getrandom`.
- Encoded in [Crockford Base32](https://www.crockford.com/base32.html): 26 chars, case-insensitive, no ambiguous characters.

---

## Project Structure

```
uid-generator-rs/
├── src/
│   ├── lib.rs           ← crate root + re-exports
│   ├── snowflake.rs     ← Snowflake generator   (YOUR IMPLEMENTATION)
│   ├── ulid.rs          ← ULID generator         (YOUR IMPLEMENTATION)
│   └── machine_id.rs    ← Machine ID sources     (YOUR IMPLEMENTATION)
├── tests/
│   ├── snowflake_tests.rs   ← pre-written
│   ├── ulid_tests.rs        ← pre-written
│   └── machine_id_tests.rs  ← pre-written
├── benches/
│   └── benchmark.rs     ← criterion benchmarks
├── .github/
│   └── workflows/
│       └── rust-ci.yml
├── Cargo.toml
└── README.md
```

---

## Getting Started

### Prerequisites

- Rust 1.70+ (stable toolchain)
- Install via [rustup](https://rustup.rs): `rustup toolchain install stable`

### Run the tests (all will fail until you implement the stubs)

```bash
cargo test
```

### Run a single test

```bash
cargo test next_id_is_monotonically_increasing
```

### Run benchmarks (requires implementation)

```bash
# Full suite
cargo bench

# Single benchmark
cargo bench snowflake/next_id
```

---

## Implementation Guide

Work through the files in this order:

### Step 1 — `src/snowflake.rs`

1. `Snowflake::new` — validate `machine_id`, construct the generator.
2. `current_ms` — use `SystemTime::now().duration_since(UNIX_EPOCH)`.
3. `wait_next_ms` — spin until `current_ms() > last`.
4. `next_id` — handle clock drift, same-ms increment, sequence overflow, then compose the 64-bit ID.
5. `decompose_id` — extract each field with bit masks and shifts.

### Step 2 — `src/ulid.rs`

1. `Ulid::with_ms` — validate 48-bit range, encode ms into bytes 0–5, fill bytes 6–15 with `getrandom`.
2. `Display` — encode 128 bits as 26 Crockford Base32 characters (see bit layout in the doc comment).
3. `FromStr` — reverse the `Display` encoding.
4. `timestamp_ms` — reconstruct ms from bytes 0–5.
5. `Ord` — compare `self.0` and `other.0` byte-by-byte (Hint: slices implement `Ord`).
6. `decode_base32_char` — map Crockford chars (including I/L/O aliases) to 5-bit values.

### Step 3 — `src/machine_id.rs`

1. `machine_id_from_env` — parse `UID_MACHINE_ID`, validate range.
2. `machine_id_from_hostname` — read `HOSTNAME` or `COMPUTERNAME`; apply `fnv1a_32`.
3. `machine_id_from_ip` — use the UDP-socket trick to get the primary IPv4; apply `fnv1a_32`.
4. `resolve_machine_id` — try sources in priority order, return first `Ok`.

---

## Traits to implement

| Trait | Type | Effect |
|---|---|---|
| `Display` | `Ulid` | `ulid.to_string()` → 26-char string |
| `FromStr` | `Ulid` | `"01ARZ…".parse::<Ulid>()` |
| `Ord` / `PartialOrd` | `Ulid` | `u1 < u2`, `.sort()`, `BTreeMap` keys |
| `Display` | `SnowflakeError` | human-readable error messages |
| `Display` | `UlidError` | human-readable error messages |
| `Display` | `MachineIdError` | human-readable error messages |

All `Error` trait implementations are already provided for you.

---

## Useful References

- [Snowflake original announcement (Twitter)](https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake)
- [ULID specification](https://github.com/ulid/spec)
- [Crockford Base32](https://www.crockford.com/base32.html)
- [FNV hash](http://www.isthe.com/chongo/tech/comp/fnv/)
- [`getrandom` crate docs](https://docs.rs/getrandom)
- [`criterion` benchmarking guide](https://bheisler.github.io/criterion.rs/book/)

---

## Machine ID Resolution Order

| Priority | Source | Mechanism |
|---|---|---|
| 1 | `UID_MACHINE_ID` env var | Explicit integer override |
| 2 | Primary IPv4 address | FNV-1a hash via UDP socket trick |
| 3 | Hostname | FNV-1a hash of `HOSTNAME` / `COMPUTERNAME` |
