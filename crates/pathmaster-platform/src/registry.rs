//! The registry adapter: the only code that touches the registry (spec §4).
//!
//! Reads and writes a Scope's `PATH` value raw — bytes and Value Type
//! preserved, never expanded, never normalised (`winreg` goes through
//! `RegQueryValueExW` / `RegSetValueExW`, which have no expansion behaviour).
//! Absent (`ERROR_FILE_NOT_FOUND`) is a distinct state from an empty value
//! and from a read failure. External changes are detected by comparing a
//! re-read `RawValue` — `(vtype, bytes)` — never the key's timestamp.

use std::io;

use pathmaster_core::logfmt;
use pathmaster_core::session::{ScopeValue, ValueType};
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
};
use winreg::{RegKey, RegValue};

const ERROR_FILE_NOT_FOUND: i32 = 2;

/// The registry roots a Scope can live under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hive {
    CurrentUser,
    LocalMachine,
}

impl Hive {
    fn key(self) -> RegKey {
        match self {
            Hive::CurrentUser => RegKey::predef(HKEY_CURRENT_USER),
            Hive::LocalMachine => RegKey::predef(HKEY_LOCAL_MACHINE),
        }
    }
}

/// A Scope's value as stored: Absent, or the exact `(vtype, bytes)` pair.
/// Equality on this type is the external-change detection primitive —
/// re-read and compare (spec §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawValue {
    Absent,
    Present {
        value_type: ValueType,
        bytes: Vec<u8>,
    },
}

impl RawValue {
    /// The value [`ScopeKey::write`] leaves in the registry for `value_type`
    /// and `value` — the same encoder, so the two cannot drift.
    ///
    /// An Apply needs this: what it just wrote becomes the value the *next*
    /// Apply compares against, and re-reading to learn it would put a second
    /// failure point after the write that mattered, at the one moment nothing
    /// can be done about it (spec §4, ADR-0008).
    pub fn written(value_type: ValueType, value: &str) -> RawValue {
        RawValue::Present {
            value_type,
            bytes: encode_utf16le_one_nul(value),
        }
    }

    /// The stored bytes as the editing model's `ScopeValue`: UTF-16LE decoded
    /// up to the first NUL — where every Windows reader of a registry string
    /// stops. The raw bytes stay authoritative for comparison and write-back;
    /// this decoded view is what the Session edits (spec §5, ADR-0006).
    pub fn decode(&self) -> ScopeValue {
        match self {
            RawValue::Absent => ScopeValue::Absent,
            RawValue::Present { value_type, bytes } => ScopeValue::Present {
                value_type: *value_type,
                raw: decode_utf16le_to_first_nul(bytes),
            },
        }
    }
}

/// A read or write failure — distinct from Absent, which is a value.
#[derive(Debug)]
pub enum RegistryError {
    Io(io::Error),
    /// The value exists but is neither `REG_SZ` nor `REG_EXPAND_SZ`.
    UnsupportedType(u32),
}

impl RegistryError {
    /// The derived fact the startup log line carries — the raw OS error code
    /// or the raw vtype, never this error's display text (spec §14: records
    /// are built from derived facts, not free-form messages).
    pub fn log_cause(&self) -> logfmt::FailureCause {
        match self {
            RegistryError::Io(err) => logfmt::FailureCause::Io {
                os_error: err.raw_os_error(),
            },
            RegistryError::UnsupportedType(vtype) => {
                logfmt::FailureCause::UnsupportedType { vtype: *vtype }
            }
        }
    }
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Io(err) => write!(f, "registry access failed: {err}"),
            RegistryError::UnsupportedType(vtype) => {
                write!(f, "value has unsupported registry type {vtype}")
            }
        }
    }
}

impl std::error::Error for RegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RegistryError::Io(err) => Some(err),
            RegistryError::UnsupportedType(_) => None,
        }
    }
}

/// The registry location of one Scope's `PATH` value. The key path is a
/// constructor parameter so tests point the same adapter at a temporary key.
#[derive(Debug, Clone)]
pub struct ScopeKey {
    hive: Hive,
    key_path: String,
    value_name: String,
}

impl ScopeKey {
    /// The User Scope: `HKCU\Environment`, value `Path` (TC-registry-keys).
    pub fn user() -> Self {
        ScopeKey::at(Hive::CurrentUser, r"Environment", "Path")
    }

    /// The System Scope: `HKLM\SYSTEM\CurrentControlSet\Control\Session
    /// Manager\Environment`, value `Path` (TC-registry-keys). Reading opens
    /// with `KEY_READ` and succeeds unelevated; only a write needs elevation.
    pub fn system() -> Self {
        ScopeKey::at(
            Hive::LocalMachine,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
            "Path",
        )
    }

    pub fn at(hive: Hive, key_path: impl Into<String>, value_name: impl Into<String>) -> Self {
        ScopeKey {
            hive,
            key_path: key_path.into(),
            value_name: value_name.into(),
        }
    }

    /// Writes `value` raw as `value_type` — UTF-16LE with exactly one
    /// trailing NUL, the shape Windows itself stores (research/05 §3.2).
    /// Creates the key and value when Absent; an empty string writes a
    /// lone NUL, never deletes the value.
    pub fn write(&self, value_type: ValueType, value: &str) -> Result<(), RegistryError> {
        let (key, _) = self
            .hive
            .key()
            .create_subkey_with_flags(&self.key_path, KEY_SET_VALUE)
            .map_err(RegistryError::Io)?;
        let raw = RegValue {
            bytes: encode_utf16le_one_nul(value).into(),
            vtype: match value_type {
                ValueType::RegSz => REG_SZ,
                ValueType::RegExpandSz => REG_EXPAND_SZ,
            },
        };
        key.set_raw_value(&self.value_name, &raw)
            .map_err(RegistryError::Io)
    }

    /// Reads the value raw. An absent key or value is `RawValue::Absent`,
    /// never an error and never an empty string.
    pub fn read(&self) -> Result<RawValue, RegistryError> {
        let key = match self
            .hive
            .key()
            .open_subkey_with_flags(&self.key_path, KEY_READ)
        {
            Ok(key) => key,
            Err(err) if is_not_found(&err) => return Ok(RawValue::Absent),
            Err(err) => return Err(RegistryError::Io(err)),
        };
        let raw = match key.get_raw_value(&self.value_name) {
            Ok(raw) => raw,
            Err(err) if is_not_found(&err) => return Ok(RawValue::Absent),
            Err(err) => return Err(RegistryError::Io(err)),
        };
        let value_type = match raw.vtype {
            REG_SZ => ValueType::RegSz,
            REG_EXPAND_SZ => ValueType::RegExpandSz,
            other => return Err(RegistryError::UnsupportedType(other as u32)),
        };
        Ok(RawValue::Present {
            value_type,
            bytes: raw.bytes.into_owned(),
        })
    }
}

fn is_not_found(err: &io::Error) -> bool {
    err.raw_os_error() == Some(ERROR_FILE_NOT_FOUND)
}

/// UTF-16LE with exactly one trailing NUL: `cbData = 2 × (chars + 1)`.
/// `RegSetValueExW` stores exactly the bytes it is handed — no terminator is
/// added for us, and a probe-padded size would grow the value on every save
/// (research/05 §3.2, H6).
fn encode_utf16le_one_nul(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// Decodes stored UTF-16LE bytes up to the first NUL unit. A trailing odd
/// byte cannot be part of any UTF-16 unit and is dropped; unpaired
/// surrogates decode lossily rather than failing the read.
fn decode_utf16le_to_first_nul(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|&unit| unit != 0)
        .collect();
    String::from_utf16_lossy(&units)
}
