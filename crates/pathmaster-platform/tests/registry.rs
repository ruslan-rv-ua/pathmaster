//! Registry adapter integration tests, against the live registry under a
//! temporary key (spec §18: no opt-in gate, no mocks — ticket 05's hazards
//! are about real API behaviour). Each test owns a unique subkey of
//! `HKCU\Software\PathMasterTest` and deletes it on drop.

#![cfg(windows)]

use pathmaster_core::session::{ScopeValue, ValueType};
use pathmaster_platform::registry::{Hive, RawValue, RegistryError, ScopeKey};

const TEST_ROOT: &str = r"Software\PathMasterTest";

/// One test's private subkey: created lazily by the adapter's first write,
/// deleted (with the shared parent, once empty) when the test finishes.
struct TestKey {
    path: String,
}

impl TestKey {
    fn new(name: &str) -> Self {
        TestKey {
            path: format!(r"{}\{}-{}", TEST_ROOT, name, std::process::id()),
        }
    }

    fn scope_key(&self) -> ScopeKey {
        ScopeKey::at(Hive::CurrentUser, &self.path, "Path")
    }
}

impl Drop for TestKey {
    fn drop(&mut self) {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(&self.path);
        // The shared parent only deletes once the last concurrent test is done.
        let _ = hkcu.delete_subkey(TEST_ROOT);
    }
}

#[test]
fn never_written_key_reads_as_absent() {
    let key = TestKey::new("absent");
    assert_eq!(key.scope_key().read().unwrap(), RawValue::Absent);
}

/// UTF-16LE with exactly one trailing NUL — `cbData = 2 × (chars + 1)`, the
/// shape Windows itself stores (research/05 §0, §3.2). Deliberately re-derived
/// here rather than imported: an independent oracle, so the byte-shape
/// assertions cannot pass by construction against the adapter's own encoder.
fn utf16le_one_nul(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

#[test]
fn reg_sz_round_trips_with_its_type_preserved() {
    let key = TestKey::new("reg-sz");
    let scope_key = key.scope_key();
    let text = r"C:\literal\%NOT_EXPANDED%\bin";

    scope_key.write(ValueType::RegSz, text).unwrap();

    assert_eq!(
        scope_key.read().unwrap(),
        RawValue::Present {
            value_type: ValueType::RegSz,
            bytes: utf16le_one_nul(text),
        }
    );
}

#[test]
fn zero_entries_over_a_present_scope_writes_empty_never_deletes() {
    let key = TestKey::new("empty");
    let scope_key = key.scope_key();
    scope_key.write(ValueType::RegExpandSz, r"C:\bin").unwrap();

    scope_key.write(ValueType::RegExpandSz, "").unwrap();

    // Present with a lone NUL (cbData = 2) — distinct from Absent.
    assert_eq!(
        scope_key.read().unwrap(),
        RawValue::Present {
            value_type: ValueType::RegExpandSz,
            bytes: vec![0, 0],
        }
    );
}

#[test]
fn read_value_decodes_to_the_core_scope_value() {
    let key = TestKey::new("decode");
    let scope_key = key.scope_key();
    let text = r"C:\a;%SystemRoot%\b;";
    scope_key.write(ValueType::RegExpandSz, text).unwrap();

    assert_eq!(
        scope_key.read().unwrap().decode(),
        ScopeValue::Present {
            value_type: ValueType::RegExpandSz,
            raw: text.to_string(),
        }
    );
    assert_eq!(RawValue::Absent.decode(), ScopeValue::Absent);
}

/// Opens (creating if needed) the test's key with winreg directly, to plant
/// values the adapter under test did not write.
fn plant_raw(key: &TestKey, vtype: winreg::enums::RegType, bytes: Vec<u8>) {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (reg_key, _) = hkcu.create_subkey(&key.path).unwrap();
    reg_key
        .set_raw_value(
            "Path",
            &winreg::RegValue {
                bytes: bytes.into(),
                vtype,
            },
        )
        .unwrap();
}

#[test]
fn external_bytes_are_preserved_exactly_and_decode_stops_at_the_first_nul() {
    let key = TestKey::new("double-nul");
    // A double-terminated value, as another tool might store it (H6): the
    // read must return the stored bytes untouched, never re-terminated.
    let mut bytes = utf16le_one_nul(r"C:\bin");
    bytes.extend_from_slice(&[0, 0]);
    plant_raw(&key, winreg::enums::RegType::REG_EXPAND_SZ, bytes.clone());

    let read = key.scope_key().read().unwrap();

    assert_eq!(
        read,
        RawValue::Present {
            value_type: ValueType::RegExpandSz,
            bytes,
        }
    );
    assert_eq!(
        read.decode(),
        ScopeValue::Present {
            value_type: ValueType::RegExpandSz,
            raw: r"C:\bin".to_string(),
        }
    );
}

#[test]
fn external_change_detection_compares_vtype_and_bytes() {
    let key = TestKey::new("external-change");
    let scope_key = key.scope_key();
    let text = r"C:\bin";
    scope_key.write(ValueType::RegExpandSz, text).unwrap();
    let baseline = scope_key.read().unwrap();

    // A byte-identical external rewrite is not a change (unlike the key's
    // timestamp, which any write bumps — H13).
    plant_raw(
        &key,
        winreg::enums::RegType::REG_EXPAND_SZ,
        utf16le_one_nul(text),
    );
    assert_eq!(scope_key.read().unwrap(), baseline);

    // A type flip with identical text is a change (the .NET round-trip bug).
    plant_raw(&key, winreg::enums::RegType::REG_SZ, utf16le_one_nul(text));
    assert_ne!(scope_key.read().unwrap(), baseline);
}

#[test]
fn value_absent_in_an_existing_key_reads_as_absent() {
    let key = TestKey::new("value-absent");
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    hkcu.create_subkey(&key.path).unwrap();

    assert_eq!(key.scope_key().read().unwrap(), RawValue::Absent);
}

#[test]
fn a_non_string_value_is_a_read_failure_not_absent_and_not_garbage() {
    let key = TestKey::new("binary");
    plant_raw(&key, winreg::enums::RegType::REG_BINARY, vec![1, 2, 3]);

    assert!(matches!(
        key.scope_key().read(),
        Err(RegistryError::UnsupportedType(3))
    ));
}

/// The adapter creates an Absent value with whatever Value Type the caller
/// passes — deliberately: Restore may legitimately apply a `REG_SZ` Snapshot
/// over an Absent Scope. The spec's "first Apply creates `REG_EXPAND_SZ`"
/// default lives in core, where a Session over an Absent Scope loads typed
/// `REG_EXPAND_SZ` (session.rs, tested there).
#[test]
fn first_write_over_absent_creates_the_value_as_written() {
    let key = TestKey::new("create");
    let scope_key = key.scope_key();
    let text = r"C:\tools\bin;%SystemRoot%\system32";

    scope_key.write(ValueType::RegExpandSz, text).unwrap();

    assert_eq!(
        scope_key.read().unwrap(),
        RawValue::Present {
            value_type: ValueType::RegExpandSz,
            bytes: utf16le_one_nul(text),
        }
    );
}

/// The startup log line's cause is derived from the error, never free text:
/// an I/O failure carries its raw OS error code, an unsupported type its raw
/// vtype (spec §14 — records are built from derived facts).
#[test]
fn a_registry_error_maps_to_the_log_cause_that_carries_its_raw_code() {
    use pathmaster_core::logfmt::FailureCause;
    let denied = RegistryError::Io(std::io::Error::from_raw_os_error(5));
    assert_eq!(denied.log_cause(), FailureCause::Io { os_error: Some(5) });
    let codeless = RegistryError::Io(std::io::Error::other("no os code"));
    assert_eq!(codeless.log_cause(), FailureCause::Io { os_error: None });
    assert_eq!(
        RegistryError::UnsupportedType(3).log_cause(),
        FailureCause::UnsupportedType { vtype: 3 }
    );
}
