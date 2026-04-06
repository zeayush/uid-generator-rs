//! ULID — Universally Unique Lexicographically Sortable Identifier.
//!
//! Raw byte layout (big-endian, 16 bytes total):
//!
//! ```text
//! bytes  0-5  :  48-bit millisecond timestamp since Unix epoch
//! bytes 6-15  :  80-bit cryptographically random data
//! ```
//!
//! String representation: 26 [Crockford Base32](https://www.crockford.com/base32.html) characters.

use std::str::FromStr;

// ── Alphabet ──────────────────────────────────────────────────────────────────

/// Crockford Base32 encoding alphabet — 32 characters, no I, L, O, U.
pub const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum UlidError {
    InvalidLength { got: usize },
    InvalidChar(char),
    TimestampOverflow,
    RandomnessError(String),
}

impl std::fmt::Display for UlidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength { got } => {
                write!(f, "expected 26 characters, got {got}")
            }
            Self::InvalidChar(c) => {
                write!(f, "invalid Crockford Base32 character: {c:?}")
            }
            Self::TimestampOverflow => write!(f, "timestamp overflows 48 bits"),
            Self::RandomnessError(e) => write!(f, "random source error: {e}"),
        }
    }
}

impl std::error::Error for UlidError {}

// ── ULID type ─────────────────────────────────────────────────────────────────

/// A 128-bit ULID stored as raw big-endian bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ulid(pub(crate) [u8; 16]);

impl std::fmt::Debug for Ulid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ulid({self})")
    }
}

impl Ulid {
    /// Creates a ULID using the current system time (millisecond precision).
    pub fn new() -> Result<Self, UlidError> {
        // Obtain current Unix timestamp in milliseconds, then delegate to with_ms.
        // Hint: use std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| UlidError::RandomnessError(e.to_string()))?
            .as_millis() as u64;
        Self::with_ms(ms)
    }

    /// Creates a ULID for a specific millisecond timestamp (ms since UNIX_EPOCH).
    ///
    /// Useful in tests to produce a predictable timestamp while keeping the
    /// random portion cryptographically random.
    pub fn with_ms(ms: u64) -> Result<Self, UlidError> {
        // TODO:
        //  1. Validate: ms must fit in 48 bits.
        //       const MAX_MS: u64 = (1u64 << 48) - 1;
        //       if ms > MAX_MS { return Err(UlidError::TimestampOverflow); }
        //
        //  2. Encode ms into the first 6 bytes (big-endian):
        //       let mut bytes = [0u8; 16];
        //       bytes[0] = (ms >> 40) as u8;
        //       bytes[1] = (ms >> 32) as u8;
        //       bytes[2] = (ms >> 24) as u8;
        //       bytes[3] = (ms >> 16) as u8;
        //       bytes[4] = (ms >>  8) as u8;
        //       bytes[5] =  ms        as u8;
        //
        //  3. Fill bytes[6..16] with 10 cryptographically random bytes:
        //       use getrandom::getrandom;   ← add this `use` at the top of the file
        //       getrandom(&mut bytes[6..])
        //           .map_err(|e| UlidError::RandomnessError(e.to_string()))?;
        //
        //  4. Return Ok(Ulid(bytes))
        let _ = ms;
        todo!("implement Ulid::with_ms")
    }

    /// Returns the embedded timestamp in milliseconds since Unix epoch.
    pub fn timestamp_ms(&self) -> u64 {
        // TODO: reconstruct ms from bytes 0–5 (big-endian):
        //   (self.0[0] as u64) << 40
        //   | (self.0[1] as u64) << 32
        //   | (self.0[2] as u64) << 24
        //   | (self.0[3] as u64) << 16
        //   | (self.0[4] as u64) <<  8
        //   | (self.0[5] as u64)
        todo!("implement Ulid::timestamp_ms")
    }
}

// ── Display — encodes to 26-char Crockford Base32 ────────────────────────────

impl std::fmt::Display for Ulid {
    /// Encodes the ULID as a 26-character Crockford Base32 string.
    ///
    /// Each output character holds exactly 5 bits of the 128-bit value.
    /// Full bit assignment (same algorithm as the reference oklog/ulid):
    ///
    /// ```text
    /// dst[ 0] = CROCKFORD[(bytes[0] & 0xE0) >> 5]
    /// dst[ 1] = CROCKFORD[ bytes[0] & 0x1F]
    /// dst[ 2] = CROCKFORD[(bytes[1] & 0xF8) >> 3]
    /// dst[ 3] = CROCKFORD[((bytes[1] & 0x07) << 2) | ((bytes[2] & 0xC0) >> 6)]
    /// dst[ 4] = CROCKFORD[(bytes[2] & 0x3E) >> 1]
    /// dst[ 5] = CROCKFORD[((bytes[2] & 0x01) << 4) | ((bytes[3] & 0xF0) >> 4)]
    /// dst[ 6] = CROCKFORD[((bytes[3] & 0x0F) << 1) | ((bytes[4] & 0x80) >> 7)]
    /// dst[ 7] = CROCKFORD[(bytes[4] & 0x7C) >> 2]
    /// dst[ 8] = CROCKFORD[((bytes[4] & 0x03) << 3) | ((bytes[5] & 0xE0) >> 5)]
    /// dst[ 9] = CROCKFORD[ bytes[5] & 0x1F]
    /// dst[10..25] — same pattern repeating for bytes[6..15]
    /// dst[25] = CROCKFORD[ bytes[15] & 0x1F]
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO:
        //  let b = &self.0;
        //  let mut dst = [0u8; 26];
        //  // fill dst[0..10] from b[0..6]  (timestamp portion)
        //  // fill dst[10..26] from b[6..16] (random portion)
        //  // Safety: CROCKFORD is ASCII-only; all indices are masked to 5 bits (0–31).
        //  let s = std::str::from_utf8(&dst).expect("CROCKFORD is valid UTF-8");
        //  f.write_str(s)
        todo!("implement Display for Ulid")
    }
}

// ── FromStr — decodes from 26-char Crockford Base32 ──────────────────────────

impl FromStr for Ulid {
    type Err = UlidError;

    /// Parses a 26-character Crockford Base32 string into a ULID.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO:
        //  1. if s.len() != 26 { return Err(UlidError::InvalidLength { got: s.len() }); }
        //  2. Reverse the Display encoding:
        //       - iterate over s.chars()
        //       - for each char call decode_base32_char(c)?  (returns a 5-bit u8)
        //       - pack the bits back into a [u8; 16]  (exact inverse of Display)
        //  3. Return Ok(Ulid(bytes))
        let _ = s;
        todo!("implement FromStr for Ulid")
    }
}

// ── Ordering ──────────────────────────────────────────────────────────────────

impl PartialOrd for Ulid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ulid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // TODO: compare self.0 and other.0 byte-by-byte.
        // Hint: slices implement Ord, so `self.0.cmp(&other.0)` works directly.
        todo!("implement Ord for Ulid")
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Maps a single Crockford Base32 character to its 5-bit value.
///
/// Case-insensitive. Applies Crockford's OCR-confusion substitutions:
///
/// | Input | Value | Reason |
/// |-------|-------|--------|
/// | `I`, `i`, `L`, `l` | 1 | Looks like `1` |
/// | `O`, `o` | 0 | Looks like `0` |
/// | `U`, `u` | error | Not in alphabet |
pub fn decode_base32_char(c: char) -> Result<u8, UlidError> {
    // TODO: implement with a match block covering all valid Crockford characters.
    //
    // Suggested skeleton:
    //   match c {
    //       '0' | 'O' | 'o'            => Ok(0),
    //       '1' | 'I' | 'i' | 'L' | 'l' => Ok(1),
    //       '2'                          => Ok(2),
    //       '3'                          => Ok(3),
    //       ...
    //       'Y' | 'y'                   => Ok(30),
    //       'Z' | 'z'                   => Ok(31),
    //       _                            => Err(UlidError::InvalidChar(c)),
    //   }
    //
    // Crockford alphabet for reference: 0123456789ABCDEFGHJKMNPQRSTVWXYZ
    //                                   (note: no I, L, O, U)
    let _ = c;
    todo!("implement decode_base32_char")
}
