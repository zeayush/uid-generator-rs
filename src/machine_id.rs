//! Machine ID resolution — identifies this host across three sources.

use crate::snowflake::MAX_MACHINE_ID;

/// Environment variable for explicit machine ID override.
/// Set to a decimal integer in `[0, MAX_MACHINE_ID]`, e.g. `UID_MACHINE_ID=7`.
pub const MACHINE_ID_ENV_VAR: &str = "UID_MACHINE_ID";

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum MachineIdError {
    EnvNotSet,
    InvalidValue(String),
    NoAddress,
    SystemError(String),
}

impl std::fmt::Display for MachineIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnvNotSet => write!(f, "{MACHINE_ID_ENV_VAR} is not set"),
            Self::InvalidValue(s) => {
                write!(f, "invalid machine ID '{s}' (expected integer in 0–{MAX_MACHINE_ID})")
            }
            Self::NoAddress => write!(f, "no suitable non-loopback IPv4 address found"),
            Self::SystemError(e) => write!(f, "system error: {e}"),
        }
    }
}

impl std::error::Error for MachineIdError {}

// ── Sources ───────────────────────────────────────────────────────────────────

/// Returns the machine ID from the `UID_MACHINE_ID` environment variable.
pub fn machine_id_from_env() -> Result<u64, MachineIdError> {
    // TODO:
    //  1. Read the env var:
    //       use std::env;
    //       let val = env::var(MACHINE_ID_ENV_VAR)
    //           .map_err(|_| MachineIdError::EnvNotSet)?;
    //
    //  2. Parse as u64:
    //       let n: u64 = val.parse()
    //           .map_err(|_| MachineIdError::InvalidValue(val.clone()))?;
    //
    //  3. Validate range:
    //       if n > MAX_MACHINE_ID { return Err(MachineIdError::InvalidValue(val)); }
    //
    //  4. Return Ok(n)
    todo!("implement machine_id_from_env")
}

/// Hashes the hostname with FNV-1a 32-bit and maps it to `[0, MAX_MACHINE_ID]`.
///
/// Reads the hostname from:
/// - `HOSTNAME` environment variable (common on Unix/Linux)
/// - `COMPUTERNAME` environment variable (Windows)
pub fn machine_id_from_hostname() -> Result<u64, MachineIdError> {
    // TODO:
    //  1. let hostname = std::env::var("HOSTNAME")
    //         .or_else(|_| std::env::var("COMPUTERNAME"))
    //         .map_err(|_| MachineIdError::SystemError("hostname not available".into()))?;
    //
    //  2. let hash = fnv1a_32(hostname.as_bytes());
    //
    //  3. Return Ok(hash as u64 & MAX_MACHINE_ID)
    todo!("implement machine_id_from_hostname")
}

/// Discovers this machine's primary outbound IPv4 and hashes it with FNV-1a 32-bit.
///
/// Uses the "UDP trick": connecting a UDP socket to a public address causes the
/// OS to select the best outbound interface — without sending any packets.
pub fn machine_id_from_ip() -> Result<u64, MachineIdError> {
    // TODO (you will need `use std::net::UdpSocket;`):
    //
    //  1. let socket = UdpSocket::bind("0.0.0.0:0")
    //         .map_err(|e| MachineIdError::SystemError(e.to_string()))?;
    //
    //  2. socket.connect("8.8.8.8:80")   // no packets sent
    //         .map_err(|e| MachineIdError::SystemError(e.to_string()))?;
    //
    //  3. let local = socket.local_addr()
    //         .map_err(|e| MachineIdError::SystemError(e.to_string()))?;
    //
    //  4. let ip = match local.ip() {
    //         std::net::IpAddr::V4(v4) => v4,
    //         _ => return Err(MachineIdError::NoAddress),
    //     };
    //
    //  5. let hash = fnv1a_32(&ip.octets());
    //
    //  6. Return Ok(hash as u64 & MAX_MACHINE_ID)
    todo!("implement machine_id_from_ip")
}

/// Returns a machine ID by trying sources in priority order:
///
/// 1. [`machine_id_from_env`]      — explicit override
/// 2. [`machine_id_from_ip`]       — primary IPv4 hash
/// 3. [`machine_id_from_hostname`] — hostname hash
///
/// Returns the first `Ok` result, or the last `Err` if all fail.
pub fn resolve_machine_id() -> Result<u64, MachineIdError> {
    // TODO: try each source in order; return the first Ok, or last Err.
    // Hint:
    //   if let Ok(id) = machine_id_from_env()      { return Ok(id); }
    //   if let Ok(id) = machine_id_from_ip()        { return Ok(id); }
    //   machine_id_from_hostname()
    todo!("implement resolve_machine_id")
}

// ── FNV-1a 32-bit hash ────────────────────────────────────────────────────────

/// FNV-1a 32-bit non-cryptographic hash.
///
/// Public domain. See <http://www.isthe.com/chongo/tech/comp/fnv/>.
fn fnv1a_32(data: &[u8]) -> u32 {
    let mut hash: u32 = 2_166_136_261;
    for &byte in data {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}
