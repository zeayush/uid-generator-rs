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
                write!(
                    f,
                    "invalid machine ID '{s}' (expected integer in 0–{MAX_MACHINE_ID})"
                )
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
    let val = std::env::var(MACHINE_ID_ENV_VAR).map_err(|_| MachineIdError::EnvNotSet)?;
    let n: u64 = val
        .parse()
        .map_err(|_| MachineIdError::InvalidValue(val.clone()))?;
    if n > MAX_MACHINE_ID {
        return Err(MachineIdError::InvalidValue(val));
    }
    Ok(n)
}

/// Hashes the hostname with FNV-1a 32-bit and maps it to `[0, MAX_MACHINE_ID]`.
///
/// Reads the hostname from:
/// - `HOSTNAME` environment variable (common on Unix/Linux)
/// - `COMPUTERNAME` environment variable (Windows)
pub fn machine_id_from_hostname() -> Result<u64, MachineIdError> {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| {
            std::process::Command::new("hostname")
                .output()
                .map_err(|_| std::env::VarError::NotPresent)
                .and_then(|out| {
                    if out.status.success() {
                        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if s.is_empty() {
                            Err(std::env::VarError::NotPresent)
                        } else {
                            Ok(s)
                        }
                    } else {
                        Err(std::env::VarError::NotPresent)
                    }
                })
        })
        .map_err(|_| MachineIdError::SystemError("hostname not available".into()))?;
    let hash = fnv1a_32(hostname.as_bytes());
    Ok(hash as u64 & MAX_MACHINE_ID)
}

/// Discovers this machine's primary outbound IPv4 and hashes it with FNV-1a 32-bit.
///
/// Uses the "UDP trick": connecting a UDP socket to a public address causes the
/// OS to select the best outbound interface — without sending any packets.
pub fn machine_id_from_ip() -> Result<u64, MachineIdError> {
    use std::net::UdpSocket;
    let socket =
        UdpSocket::bind("0.0.0.0:0").map_err(|e| MachineIdError::SystemError(e.to_string()))?;
    socket
        .connect("8.8.8.8:80")
        .map_err(|e| MachineIdError::SystemError(e.to_string()))?;
    let local = socket
        .local_addr()
        .map_err(|e| MachineIdError::SystemError(e.to_string()))?;
    let ip = match local.ip() {
        std::net::IpAddr::V4(v4) => v4,
        _ => return Err(MachineIdError::NoAddress),
    };
    let hash = fnv1a_32(&ip.octets());
    Ok(hash as u64 & MAX_MACHINE_ID)
}

/// Returns a machine ID by trying sources in priority order:
///
/// 1. [`machine_id_from_env`]      — explicit override
/// 2. [`machine_id_from_ip`]       — primary IPv4 hash
/// 3. [`machine_id_from_hostname`] — hostname hash
///
/// Returns the first `Ok` result, or the last `Err` if all fail.
pub fn resolve_machine_id() -> Result<u64, MachineIdError> {
    if let Ok(id) = machine_id_from_env() {
        return Ok(id);
    }
    if let Ok(id) = machine_id_from_ip() {
        return Ok(id);
    }
    machine_id_from_hostname()
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
