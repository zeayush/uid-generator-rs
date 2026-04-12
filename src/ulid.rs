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

use getrandom::getrandom;
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
        const MAX_MS: u64 = (1u64 << 48) - 1;
        if ms > MAX_MS {
            return Err(UlidError::TimestampOverflow);
        }
        let mut bytes = [0u8; 16];
        bytes[0] = (ms >> 40) as u8;
        bytes[1] = (ms >> 32) as u8;
        bytes[2] = (ms >> 24) as u8;
        bytes[3] = (ms >> 16) as u8;
        bytes[4] = (ms >> 8) as u8;
        bytes[5] = ms as u8;
        getrandom(&mut bytes[6..]).map_err(|e| UlidError::RandomnessError(e.to_string()))?;
        Ok(Ulid(bytes))
    }

    /// Returns the embedded timestamp in milliseconds since Unix epoch.
    pub fn timestamp_ms(&self) -> u64 {
        (self.0[0] as u64) << 40
            | (self.0[1] as u64) << 32
            | (self.0[2] as u64) << 24
            | (self.0[3] as u64) << 16
            | (self.0[4] as u64) << 8
            | (self.0[5] as u64)
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
        let b = &self.0;
        let mut dst = [0u8; 26];
        dst[0] = CROCKFORD[((b[0] & 0xE0) >> 5) as usize];
        dst[1] = CROCKFORD[(b[0] & 0x1F) as usize];
        dst[2] = CROCKFORD[((b[1] & 0xF8) >> 3) as usize];
        dst[3] = CROCKFORD[(((b[1] & 0x07) << 2) | ((b[2] & 0xC0) >> 6)) as usize];
        dst[4] = CROCKFORD[((b[2] & 0x3E) >> 1) as usize];
        dst[5] = CROCKFORD[(((b[2] & 0x01) << 4) | ((b[3] & 0xF0) >> 4)) as usize];
        dst[6] = CROCKFORD[(((b[3] & 0x0F) << 1) | ((b[4] & 0x80) >> 7)) as usize];
        dst[7] = CROCKFORD[((b[4] & 0x7C) >> 2) as usize];
        dst[8] = CROCKFORD[(((b[4] & 0x03) << 3) | ((b[5] & 0xE0) >> 5)) as usize];
        dst[9] = CROCKFORD[(b[5] & 0x1F) as usize];
        dst[10] = CROCKFORD[((b[6] & 0xF8) >> 3) as usize];
        dst[11] = CROCKFORD[(((b[6] & 0x07) << 2) | ((b[7] & 0xC0) >> 6)) as usize];
        dst[12] = CROCKFORD[((b[7] & 0x3E) >> 1) as usize];
        dst[13] = CROCKFORD[(((b[7] & 0x01) << 4) | ((b[8] & 0xF0) >> 4)) as usize];
        dst[14] = CROCKFORD[(((b[8] & 0x0F) << 1) | ((b[9] & 0x80) >> 7)) as usize];
        dst[15] = CROCKFORD[((b[9] & 0x7C) >> 2) as usize];
        dst[16] = CROCKFORD[(((b[9] & 0x03) << 3) | ((b[10] & 0xE0) >> 5)) as usize];
        dst[17] = CROCKFORD[(b[10] & 0x1F) as usize];
        dst[18] = CROCKFORD[((b[11] & 0xF8) >> 3) as usize];
        dst[19] = CROCKFORD[(((b[11] & 0x07) << 2) | ((b[12] & 0xC0) >> 6)) as usize];
        dst[20] = CROCKFORD[((b[12] & 0x3E) >> 1) as usize];
        dst[21] = CROCKFORD[(((b[12] & 0x01) << 4) | ((b[13] & 0xF0) >> 4)) as usize];
        dst[22] = CROCKFORD[(((b[13] & 0x0F) << 1) | ((b[14] & 0x80) >> 7)) as usize];
        dst[23] = CROCKFORD[((b[14] & 0x7C) >> 2) as usize];
        dst[24] = CROCKFORD[(((b[14] & 0x03) << 3) | ((b[15] & 0xE0) >> 5)) as usize];
        dst[25] = CROCKFORD[(b[15] & 0x1F) as usize];
        // Safety: CROCKFORD is ASCII-only; all indices are masked to 5 bits (0–31).
        let s = std::str::from_utf8(&dst).expect("CROCKFORD is valid UTF-8");
        f.write_str(s)
    }
}

// ── FromStr — decodes from 26-char Crockford Base32 ──────────────────────────

impl FromStr for Ulid {
    type Err = UlidError;

    /// Parses a 26-character Crockford Base32 string into a ULID.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 26 {
            return Err(UlidError::InvalidLength { got: s.len() });
        }
        let mut v = [0u8; 26];
        for (i, c) in s.chars().enumerate() {
            v[i] = decode_base32_char(c)?;
        }
        let mut bytes = [0u8; 16];
        bytes[0] = (v[0] << 5) | v[1];
        bytes[1] = (v[2] << 3) | (v[3] >> 2);
        bytes[2] = ((v[3] & 0x03) << 6) | (v[4] << 1) | (v[5] >> 4);
        bytes[3] = ((v[5] & 0x0F) << 4) | (v[6] >> 1);
        bytes[4] = ((v[6] & 0x01) << 7) | (v[7] << 2) | (v[8] >> 3);
        bytes[5] = ((v[8] & 0x07) << 5) | v[9];
        bytes[6] = (v[10] << 3) | (v[11] >> 2);
        bytes[7] = ((v[11] & 0x03) << 6) | (v[12] << 1) | (v[13] >> 4);
        bytes[8] = ((v[13] & 0x0F) << 4) | (v[14] >> 1);
        bytes[9] = ((v[14] & 0x01) << 7) | (v[15] << 2) | (v[16] >> 3);
        bytes[10] = ((v[16] & 0x07) << 5) | v[17];
        bytes[11] = (v[18] << 3) | (v[19] >> 2);
        bytes[12] = ((v[19] & 0x03) << 6) | (v[20] << 1) | (v[21] >> 4);
        bytes[13] = ((v[21] & 0x0F) << 4) | (v[22] >> 1);
        bytes[14] = ((v[22] & 0x01) << 7) | (v[23] << 2) | (v[24] >> 3);
        bytes[15] = ((v[24] & 0x07) << 5) | v[25];
        Ok(Ulid(bytes))
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
        self.0.cmp(&other.0)
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
    match c {
        '0' | 'O' | 'o' => Ok(0),
        '1' | 'I' | 'i' | 'L' | 'l' => Ok(1),
        '2' => Ok(2),
        '3' => Ok(3),
        '4' => Ok(4),
        '5' => Ok(5),
        '6' => Ok(6),
        '7' => Ok(7),
        '8' => Ok(8),
        '9' => Ok(9),
        'A' | 'a' => Ok(10),
        'B' | 'b' => Ok(11),
        'C' | 'c' => Ok(12),
        'D' | 'd' => Ok(13),
        'E' | 'e' => Ok(14),
        'F' | 'f' => Ok(15),
        'G' | 'g' => Ok(16),
        'H' | 'h' => Ok(17),
        'J' | 'j' => Ok(18),
        'K' | 'k' => Ok(19),
        'M' | 'm' => Ok(20),
        'N' | 'n' => Ok(21),
        'P' | 'p' => Ok(22),
        'Q' | 'q' => Ok(23),
        'R' | 'r' => Ok(24),
        'S' | 's' => Ok(25),
        'T' | 't' => Ok(26),
        'V' | 'v' => Ok(27),
        'W' | 'w' => Ok(28),
        'X' | 'x' => Ok(29),
        'Y' | 'y' => Ok(30),
        'Z' | 'z' => Ok(31),
        _ => Err(UlidError::InvalidChar(c)),
    }
}
