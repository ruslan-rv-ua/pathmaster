# 05 — PATH registry I/O semantics

Research findings for [issues/05-registry-io-semantics.md](../issues/05-registry-io-semantics.md).
Status: resolved except where marked **UNKNOWN — needs a spike**.

Every claim below is backed by either a URL to a first-party source or by output from a command run on this
machine. All registry access performed for this research was **read-only**; nothing was written, and `setx`
was never invoked. The one system-wide side effect performed was an *idempotent* `WM_SETTINGCHANGE`
broadcast (section 6) — receivers re-read an environment that had not changed.

## 0. The machine under test, and what it already proves

Windows 11 Pro 10.0.26200, x64, 64-bit PowerShell 7. Probes made with direct P/Invoke to
`advapi32!RegOpenKeyExW` / `RegQueryValueExW` / `RegGetValueW` / `RegQueryInfoKeyW`.

```
=== HKCU\Environment :: Path
RegQueryValueExW size-probe rc=0 type=2 cbData=3036 bytes (1518 UTF-16 units)
  last 4 bytes: 65 00 00 00
  RAW string length (chars, NUL stripped) = 1517
  RAW contains '%' = False

=== HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment :: Path
RegQueryValueExW size-probe rc=0 type=2 cbData=1382 bytes (691 UTF-16 units)
  last 4 bytes: 5C 00 00 00
  RAW string length (chars, NUL stripped) = 690
  RAW contains '%' = True
```

Facts established before any theory:

- Both `Path` values here are **type 2 = `REG_EXPAND_SZ`**.
- The System `Path` **really does contain `%VAR%` references** — its raw text ends
  `…;%SystemRoot%\system32;%SystemRoot%;%SystemRoot%\System32\Wbem;%SYSTEMROOT%\System32\WindowsPowerShell\v1.0\;%SYSTEMROOT%\System32\OpenSSH\;C:\Program Files\PowerShell\7\`.
  This is not a hypothetical: a round trip that expands would rewrite a stock Windows install's PATH.
- Each stored value carries **exactly one** trailing UTF-16 NUL (`cbData` = 2 × (chars + 1); the last four
  bytes are one real character followed by `00 00`).

And the classic .NET/PowerShell divergence, on the same value:

```
HKLM ... Path
RAW    (RegistryValueOptions.DoNotExpandEnvironmentNames) : …;%SystemRoot%\system32;…
EXP    (RegistryKey.GetValue, default)                    : …;C:\WINDOWS\system32;…
NETAPI ([Environment]::GetEnvironmentVariable('Path','Machine')) : …;C:\WINDOWS\system32;…
```

`[Environment]::GetEnvironmentVariable(...,'Machine')` returns the **expanded** string. Round-tripping it
back would bake `C:\WINDOWS\system32` into the registry permanently. See §1 and §8/H1.

---

## 1. Raw (unexpanded) read — the exact API call chain in Rust

### 1.1 The key fact: `RegQueryValueEx` never expands; only `RegGetValue` does

`RegGetValueW` expands `REG_EXPAND_SZ` **by default**, and `RRF_NOEXPAND` (0x10000000) is documented as
"Do not automatically expand environment strings if the value is of type REG_EXPAND_SZ"
([RegGetValueW, dwFlags table](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-reggetvaluew)).

`RegQueryValueExW`'s reference page contains **no expansion behaviour at all** — it returns the stored bytes
and the stored type
([RegQueryValueExW](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regqueryvalueexw)).

Measured on this machine, on the System `Path` (which contains `%SystemRoot%`):

```
RegQueryValueExW                      : rc=0 type=2 cb=1382  contains '%' = True
RegGetValueW (no RRF_NOEXPAND)        : rc=0 type=1 cb=1382  contains '%' = False   <-- expanded
RegGetValueW (RRF_RT_ANY|RRF_NOEXPAND): rc=0 type=2 cb=1384* contains '%' = True
```

Read that middle line twice. Without `RRF_NOEXPAND`, `RegGetValueW` did **two** damaging things at once: it
expanded the string **and** reported `pdwType = 1 (REG_SZ)` instead of the stored `2 (REG_EXPAND_SZ)`. A
naive "read type, edit, write back with the type I was told" implementation therefore loses both the
`%VAR%` references and the value type in one step.

`*` The 1384 is the **size probe** (`pvData = NULL`). The subsequent real read returned `cbReturned = 1382`:

```
probe(pvData=NULL): rc=0 type=2 cbData=1384
actual read       : rc=0 type=2 cbReturned=1382
  tail 6 bytes: 37 00 5C 00 00 00
```

`RegGetValue` guarantees NUL-termination and so its `NULL`-buffer probe pads the reported size by one
`WCHAR` in case the stored data is unterminated. Do not treat a probe size as "the byte length of the stored
value" (§8/H6).

### 1.2 `winreg` **can** do it — and does not need `RRF_NOEXPAND` at all

`winreg` 0.56.0 (crates.io max stable, released 2026-03-14, repo
[gentoo90/winreg-rs](https://github.com/gentoo90/winreg-rs)) implements `RegKey::get_raw_value` on top of
`RegQueryValueExW`
([src/reg_key.rs](https://raw.githubusercontent.com/gentoo90/winreg-rs/master/src/reg_key.rs)):

```rust
pub fn get_raw_value<N: AsRef<OsStr>>(&self, name: N) -> io::Result<RegValue<'static>> {
    // …
    Registry::RegQueryValueExW(self.hkey, c_name.as_ptr(), ptr::null_mut(),
                               &mut buf_type, buf.as_mut_ptr(), &mut buf_len)
    // … returns RegValue { bytes, vtype }
}
```

No `RRF_` flag appears anywhere in the crate, because none is needed: `RegQueryValueExW` has no expansion
behaviour to suppress. **Answer to the ticket's phrasing: `winreg` cannot pass `RRF_NOEXPAND`, and does not
have to — `get_raw_value` is already a raw, unexpanded read that also returns the stored type.**

Note that `RegKey::get_value::<String>()` is also safe from expansion (it just decodes `get_raw_value`), and
its `FromRegValue for String` explicitly accepts `REG_SZ | REG_EXPAND_SZ | REG_MULTI_SZ`
([src/types.rs](https://raw.githubusercontent.com/gentoo90/winreg-rs/master/src/types.rs)). The danger in
`winreg` is entirely on the **write** side — see §2.2.

Recommended chain:

```rust
use winreg::{RegKey, RegValue};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_EXPAND_SZ};

// READ (raw)
let hkcu = RegKey::predef(HKEY_CURRENT_USER);
let env  = hkcu.open_subkey_with_flags("Environment", KEY_READ)?;
let raw: RegValue = env.get_raw_value("Path")?;   // raw.vtype == REG_EXPAND_SZ, raw.bytes == stored bytes

// EDIT: decode raw.bytes as UTF-16LE, strip exactly the trailing NUL(s), split on ';'
// WRITE (raw, type preserved)
let out = RegValue { bytes: encode_utf16_with_one_nul(&new_string).into(), vtype: raw.vtype };
env_writable.set_raw_value("Path", &out)?;         // -> RegSetValueExW with the ORIGINAL type
```

`set_raw_value` is `RegSetValueExW(hkey, name, 0, value.vtype as u32, bytes.as_ptr(), bytes.len())` — it
writes back exactly the type you hand it, so preserving `raw.vtype` is a one-liner.

### 1.3 The `windows` crate route (0.62.2, released 2025-10-06)

If you would rather not depend on `winreg`, the exact calls are (signatures verbatim from Microsoft's own
generated docs, [windows-docs-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Registry/)):

```rust
// feature = "Win32_System_Registry" (not enabled by default; only "std" is)
pub unsafe fn RegGetValueW<P1, P2>(
    hkey: HKEY, lpsubkey: P1, lpvalue: P2,
    dwflags: REG_ROUTINE_FLAGS,
    pdwtype: Option<*mut REG_VALUE_TYPE>,
    pvdata:  Option<*mut c_void>,
    pcbdata: Option<*mut u32>,
) -> WIN32_ERROR where P1: Param<PCWSTR>, P2: Param<PCWSTR>;

pub unsafe fn RegQueryValueExW<P1>(
    hkey: HKEY, lpvaluename: P1, lpreserved: Option<*const u32>,
    lptype:  Option<*mut REG_VALUE_TYPE>,
    lpdata:  Option<*mut u8>,
    lpcbdata: Option<*mut u32>,
) -> WIN32_ERROR where P1: Param<PCWSTR>;

pub unsafe fn RegSetValueExW<P1>(
    hkey: HKEY, lpvaluename: P1, reserved: Option<u32>,
    dwtype: REG_VALUE_TYPE, lpdata: Option<&[u8]>,
) -> WIN32_ERROR where P1: Param<PCWSTR>;
```

If using `RegGetValueW`, the flags must be
`RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | RRF_NOEXPAND` (0x2 | 0x4 | 0x1000_0000). Restricting to the two
string types also gives you a hard failure — rather than garbage — if someone has replaced `Path` with a
`REG_BINARY` or `REG_MULTI_SZ`. Feature names confirmed at
[docs.rs/crate/windows/0.62.2/features](https://docs.rs/crate/windows/0.62.2/features): `Win32_System_Registry`,
`Win32_UI_WindowsAndMessaging`, `Win32_Foundation`, none on by default.

The same functions exist in `windows-sys` 0.61.2 as plain `extern "system"` FFI
([RegGetValueW](https://docs.rs/windows-sys/latest/windows_sys/Win32/System/Registry/fn.RegGetValueW.html)).

### 1.4 Third option: `windows-registry` (Microsoft's own, 0.6.1)

[`windows-registry`](https://docs.rs/windows-registry/latest/windows_registry/) is Microsoft's small safe
wrapper. Its `Key::get_value` goes through `RegQueryValueExW` with **no** `RRF_` flags and does **not**
expand ([crates/libs/registry/src/key.rs](https://raw.githubusercontent.com/microsoft/windows-rs/master/crates/libs/registry/src/key.rs)):

```rust
pub fn get_value<T: AsRef<str>>(&self, name: T) -> Result<Value> {
    let (ty, len) = unsafe { self.raw_get_info(name.as_raw())? };
    let mut data = Data::new(len);
    unsafe { self.raw_get_bytes(name.as_raw(), &mut data)? };
    Ok(Value { data, ty })
}
pub fn set_value<T: AsRef<str>>(&self, name: T, value: &Value) -> Result<()> {
    self.set_bytes(name, value.ty(), value)   // writes back the Value's own type
}
```

`Value` exposes `ty() -> Type`, `set_ty(Type)`, `as_wide() -> &[u16]` and `Deref<Target = [u8]>`, so a
type-preserving `get_value` → edit → `set_value` round trip is expressible without touching raw FFI. Note
`Type::ExpandString` is "A string value that may contain unexpanded environment variables"
([Type](https://docs.rs/windows-registry/latest/windows_registry/enum.Type.html)); there is deliberately no
`get_expand_string`, because reads never expand.

**Recommendation.** Either `winreg::get_raw_value`/`set_raw_value` or `windows-registry`'s
`get_value`/`set_value` is correct and neither needs `RRF_NOEXPAND`. `winreg` is 35M downloads and already
depends on `windows-sys`; `windows-registry` is first-party and pulls the same tree. Pick one; do **not**
use `RegGetValue` without `RRF_NOEXPAND`, and do **not** use anything that goes through .NET semantics.

---

## 2. Value-type policy: preserve `REG_SZ`, or normalise to `REG_EXPAND_SZ`?

### 2.1 What each direction breaks

**Normalising `REG_SZ` → `REG_EXPAND_SZ`** changes the *meaning* of the data already stored. A literal `%` in
an existing `REG_SZ` path (legal in an NTFS directory name — e.g. `C:\builds\100%done\bin`) is inert while
the value is `REG_SZ`, and becomes an expansion attempt the moment the type flips. `ExpandEnvironmentStrings`
leaves unmatched `%…%` pairs alone, so the common case survives — but a directory literally named e.g.
`%TEMP%` or a `…\%USERPROFILE%\…` fragment that was previously a real directory name would start resolving
to something else. The user did not ask for that, and the change is invisible in the UI.

**Preserving an existing `REG_SZ`** costs the user the ability to *introduce* a `%VAR%` entry into that
particular scope — anything they type as `%JAVA_HOME%\bin` will be stored and consumed literally, and will
silently never resolve. That is also invisible, and arguably worse because it is a live editing operation
the user just performed.

**Verdict for v0.1.0:** preserve the existing type by default; do not silently normalise. But detect the
combination "value is `REG_SZ` **and** the text the user is about to store contains a `%…%` pair" and, in
that one case, surface it: either refuse, or offer an explicit, named "convert this value to expandable
(`REG_EXPAND_SZ`)" action. Never flip the type as a side effect of an unrelated edit. When creating a value
that does not exist, create `REG_EXPAND_SZ` (§3).

This also means the type is part of the app's model: read it, carry it through the editing session, show it
in diagnostics, and write it back. `winreg`'s `RegValue.vtype` and `windows-registry`'s `Value::ty()` both
give it to you for free.

### 2.2 The library-level trap on the write side

`winreg`'s `ToRegValue for String` / `for &str` is generated by `to_reg_value_sz!`, which produces
**`REG_SZ`** ([src/types.rs](https://raw.githubusercontent.com/gentoo90/winreg-rs/master/src/types.rs)).
So `key.set_value("Path", &new_path_string)?` **silently downgrades `REG_EXPAND_SZ` → `REG_SZ`**. There is
no warning and no error. Use `set_raw_value` with the preserved `vtype`. (This is the identical bug .NET
ships with — see below.)

### 2.3 .NET / PowerShell get this wrong, which is why real machines have `REG_SZ` `Path`

`RegistryKey.SetValue(String, Object)` documents:

> "This overload of SetValue stores all string values as RegistryValueKind.String, even if they contain
> expandable references to environment variables."
> — [RegistryKey.SetValue](https://learn.microsoft.com/en-us/dotnet/api/microsoft.win32.registrykey.setvalue)

And `Environment.SetEnvironmentVariable(name, value, User|Machine)` calls exactly that overload, with no
`RegistryValueKind`
([Environment.Windows.cs](https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/Environment.Windows.cs)):

```csharp
environmentKey.SetValue(variable, value);          // -> REG_SZ, always
```

while the matching read uses `environmentKey?.GetValue(variable)` with **no**
`RegistryValueOptions.DoNotExpandEnvironmentNames`, so it expands
([RegistryValueOptions](https://learn.microsoft.com/en-us/dotnet/api/microsoft.win32.registryvalueoptions):
`DoNotExpandEnvironmentNames = 1`, "A value of type ExpandString is retrieved without expanding its embedded
environment variables"; `None = 0` is the default).

So one round trip through `[Environment]::GetEnvironmentVariable('Path','Machine')` +
`[Environment]::SetEnvironmentVariable('Path', …, 'Machine')` — the snippet in every StackOverflow answer —
**expands every `%VAR%` and converts `REG_EXPAND_SZ` to `REG_SZ`.** That is the mechanism by which
"real machines sometimes have `Path` as `REG_SZ`" happens. PathMaster must not be the next tool to do it,
and should treat an existing `REG_SZ` `Path` as evidence of prior damage worth reporting, not worth
silently "fixing".

---

## 3. The missing `Path` value on a fresh profile

### 3.1 What a read returns

Measured on both keys, querying a value name that does not exist:

```
RegQueryValueExW on absent value -> rc=2 (2 = ERROR_FILE_NOT_FOUND)
RegGetValueW      on absent value -> rc=2
```

This matches both reference pages verbatim: "If *lpValueName* specifies a value that is not in the registry,
the function returns ERROR_FILE_NOT_FOUND"
([RegQueryValueExW](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regqueryvalueexw))
and "If the lpValue registry value does not exist, the function returns ERROR_FILE_NOT_FOUND"
([RegGetValueW](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-reggetvaluew)).

In `winreg` this surfaces as `Err(io::Error)` with `raw_os_error() == Some(2)` /
`ErrorKind::NotFound`. **It is not a distinguishable "empty PATH" — it is an error.** Map it explicitly to a
domain state (`UserPath::Absent`), never to `Err(unexpected)` and never to `Ok("")`; the difference matters
because "absent" and "present but empty string" require different writes to restore, and the backup/restore
contract (ticket 14) has to be able to express both.

Note also that the two failure modes are distinct and both must be handled:

- **Key absent** — `RegOpenKeyEx` returns `ERROR_FILE_NOT_FOUND`. Handle by `RegCreateKeyEx` /
  `winreg::create_subkey` on first write.
- **Key present, value absent** — the case above.

### 3.2 What a first write must create

`RegSetValueExW` creates the value if it does not exist, so the first write is not special *mechanically*.
What it must get right is the type and the terminator, because there is no existing `vtype` to preserve:

- **Type: `REG_EXPAND_SZ`.** This is the type Windows itself uses. Every `Path` value on this machine —
  `HKCU\Environment`, `HKLM\…\Session Manager\Environment`, `HKU\.DEFAULT\Environment`, and
  `HKU\S-1-5-18\Environment` — is `ExpandString`. Creating `REG_SZ` would permanently deny the user
  `%VAR%` entries in a scope they just created.
- **Bytes: UTF-16LE, exactly one trailing NUL**, i.e. `cbData = 2 * (chars + 1)`. That is the shape Windows
  stores (§0). `RegSetValueExW` does not add a terminator for you — the size you pass is the size stored.
  Omit it and later readers can over-read: "the string may not have been stored with the proper terminating
  null characters … the application should ensure that the string is properly terminated before using it;
  otherwise, it may overwrite a buffer"
  ([RegQueryValueExW remarks](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regqueryvalueexw)).
- **Empty PATH:** prefer deleting the value over writing a zero-length one. If you must write "empty", write
  a single NUL (`cbData = 2`), not `cbData = 0`.

### 3.3 Is `Path` actually absent on a fresh profile?

The read semantics above are proven. Whether a brand-new profile *does* lack `HKCU\Environment\Path` is
**not settled by a first-party source I could find**, and I could not test it here without loading
`C:\Users\Default\NTUSER.DAT` (a write-class operation). On this machine `HKU\.DEFAULT\Environment` does
contain a `Path` — but `.DEFAULT` is the LocalSystem (S-1-5-18) profile, *not* the new-user template, so it
proves nothing about fresh profiles. It is in any case irrelevant to the design: PathMaster must handle the
absent case regardless, because a user can delete the value with regedit at any time.
**UNKNOWN — needs a spike** only if the spec wants to *assert* the fresh-profile shape; skip it and handle
both states.

---

## 4. The 32767 limit — what it actually applies to

### 4.1 The documented number

> "The maximum size of a user-defined environment variable is 32,767 characters. There is no technical
> limitation on the size of the environment block. However, there are practical limits depending on the
> mechanism used to access the block."
>
> "**Windows Server 2003 and Windows XP:** The maximum size of the environment block for the process is
> 32,767 characters. Starting with Windows Vista and Windows Server 2008, there is no technical limitation
> on the size of the environment block."
>
> — [Environment Variables](https://learn.microsoft.com/en-us/windows/win32/procthread/environment-variables)
> (page updated 2025-07-14)

`SetEnvironmentVariable` repeats it on `lpValue`: "The maximum size of a user-defined environment variable
is 32,767 characters", plus the same XP/2003 block caveat
([SetEnvironmentVariable](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setenvironmentvariable)).

So:

| Candidate | Verdict |
| --- | --- |
| A limit on the **registry value** | **No.** Registry value size limit is "Available memory (latest format) / 1 MB (standard format)" ([Registry element size limits](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-element-size-limits)). 32767 appears nowhere on that page. |
| A limit on **one environment variable** | **Yes.** This is the documented meaning: 32,767 characters per variable value. |
| A limit on the **process environment block** | **Only on XP / Server 2003.** Explicitly lifted from Vista onward, by the same two pages. |

### 4.2 Where the folklore comes from, and what to discard

**The `setx` 1024 truncation is real, is `setx`'s own, and is not a Windows limit.**

> "Be aware there's a limit of 1024 characters when assigning contents to a variable using **setx**. This
> means that the content is cropped if you go over 1024 characters, and that the cropped text is what's
> applied to the target variable. If this cropped text is applied to an existing variable, it can result in
> loss of data previously held by the target variable."
> — [setx](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/setx)

The same page also documents `setx`'s *other* PATH-destroying behaviour: "Running this command on an
existing variable removes any variable references and uses expanded values… if the variable %PATH% has a
reference to %JAVADIR%, and %PATH% is manipulated using **setx**, %JAVADIR% is expanded". `setx` is not a
model to copy; it is the cautionary tale.

**The 2047/2048 numbers are two different things, neither a hard registry limit.**

- **2047 is the GUI's limit.** The `sysdm.cpl` single-line editor refuses with "This dialog allows setting
  variables up to 2047 characters"
  ([Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/2784808/cant-edit-environment-variable-over-2047-character));
  the answer there is explicit that "there is no limit to the path length when you modify it directly in the
  registry".
- **2048 is a performance guideline.** "Long values (more than 2,048 bytes) should be stored in a file …
  This helps the registry to perform efficiently"
  ([Registry element size limits](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-element-size-limits)),
  echoed by .NET: "we recommend keeping value under 2,048 bytes (as stored in the Windows registry) for
  registry efficiency. **This isn't a hard limit.**"
  ([Environment.SetEnvironmentVariable](https://learn.microsoft.com/en-us/dotnet/api/system.environment.setenvironmentvariable)).
  This machine's `HKCU\Environment\Path` is 3036 bytes and works fine.

**Conflicting source, named.** Raymond Chen,
[*What is the maximum length of an environment variable?*](https://devblogs.microsoft.com/oldnewthing/20100203-00/?p=15083)
(2010-02-03), states "All environment variables must live together in a single environment block, which
itself has a limit of 32767 characters", and mentions "a 2048-character limit in the code that parses that
registry key and builds an environment block out of it". The first statement **contradicts** the current
Microsoft Learn page, which says the block limit was removed in Vista. The post is from 2010 and describes
a customer report; the Learn pages are versioned, explicit about which OS the 32767 block limit applies to,
and were updated in 2025. **I trust the Learn pages** for the block limit and treat Chen's 32767-block and
2048-registry-parse figures as describing older behaviour. Note that the 2048 registry-parse figure is
directly contradicted by this machine, where a 1517-character user `Path` (3036 bytes) is fully reflected
in the process environment.

### 4.3 FR-diag-length's "combined User+System" claim — **partially verified**

**Verified: a combined check is the right thing to check, and a per-scope check is the wrong thing.**
The variable that lands in a process is the merge of the two, and *that* single variable is what the
documented 32,767-character per-variable limit governs. Each scope individually being under 32,767 tells you
nothing.

**Refined: it must be the length of the *expanded, merged* string, not the sum of the two raw strings.**
Measured here:

```
raw   System=690  User=1517  sum=2207
expanded System=680  User=1517  merged(System + ';' + User)=2198
```

The raw sum over-reports by 9 characters on this machine purely because `%SystemRoot%` (12 chars) contracts
to `C:\WINDOWS` (10). On a machine with heavy `%VAR%` use the sign can flip and the error can be large — a
single `%ProgramFiles(x86)%` (19) expands to `C:\Program Files (x86)` (22). Compute the diagnostic on
`expand(System) + ";" + expand(User)`.

**Merge order** is System first, then User appended — visible in this machine's process `PATH`, whose System
entries precede its User entries, and consistent with every secondary description. I found **no first-party
Microsoft page stating the concatenation rule**; treat the ordering as observed-and-consistent rather than
documented.

**UNKNOWN — needs a spike:** what actually happens when the merged PATH would exceed 32,767. Nothing
first-party documents the overflow behaviour of the logon/`CreateEnvironmentBlock` merge — whether it
truncates, drops the variable, or fails. Do not claim a specific failure mode in the spec. Warn at a
threshold and say *why* ("the merged PATH is approaching the documented 32,767-character per-variable
limit"), rather than asserting what breaks.

**Also worth surfacing in diagnostics, and cheaply provable:** the process environment is a *snapshot*, not a
live computation.

```
this process PATH len = 1796
this process PATH == fresh merge? False
```

PathMaster's own `std::env::var("PATH")` is inherited from whatever launched it and can be arbitrarily
stale. Never diagnose the registry using the process environment.

---

## 5. WOW64 / redirection for the HKLM key on x64

**The key is shared, not redirected.** [Registry Keys Affected by
WOW64](https://learn.microsoft.com/en-us/windows/win32/winprog64/shared-registry-keys) lists
`HKEY_LOCAL_MACHINE` as **Shared**, lists only `HKEY_LOCAL_MACHINE\SOFTWARE` (and its subtree) as
**Redirected**, and states the closure rule: "Subkeys of the keys in this table inherit the parent key's
behavior unless otherwise specified. **If a key has no parent listed in this table, the key is shared.**"
`HKLM\SYSTEM` appears nowhere in the table, so `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`
is shared. Confirmed empirically — all three views return byte-identical data:

```
=== WOW64 view check on HKLM Session Manager\Environment
KEY_WOW64_64KEY: type=2 cb=1382 sha256(first16hex)=BB4B8DD2C64DDAC1
KEY_WOW64_32KEY: type=2 cb=1382 sha256(first16hex)=BB4B8DD2C64DDAC1
(none):          type=2 cb=1382 sha256(first16hex)=BB4B8DD2C64DDAC1
```

So `KEY_WOW64_64KEY` is **not needed** for this key, and there is no `Wow6432Node` copy of PATH to worry
about. Passing it anyway is harmless for the read.

**There is, however, a real WOW64 hazard on the *write* side if PathMaster is ever built 32-bit.** From
[Registry Redirector](https://learn.microsoft.com/en-us/windows/win32/winprog64/registry-redirector):

> "To help 32-bit applications that write **REG_SZ** or **REG_EXPAND_SZ** data containing %ProgramFiles% or
> %commonprogramfiles% to the registry, WOW64 intercepts these write operations and replaces them with
> "%ProgramFiles(x86)%" and "%commonprogramfiles(x86)%"."

Conditions: the string must **begin** with `%ProgramFiles%` or `%commonprogramfiles%` (case-sensitive,
exactly as shown), must not exceed `MAX_PATH*2+15` characters, and — since Windows 7 — the key must not have
been opened with `KEY_WOW64_64KEY`. A PATH whose first entry is `%ProgramFiles%\…` and which is written by a
32-bit process would be silently rewritten. The same page describes a `system32` → `syswow64` substitution,
but scopes it: "This patch is applied only to the keys that were reflected prior to Windows 7", which does
not include this key.

**Mitigation, in order of preference:** (1) ship PathMaster as x64 (and ARM64) only — the map already rules
32-bit Windows out of scope; (2) if a 32-bit build ever exists, open the HKLM key with `KEY_WOW64_64KEY`,
which the doc states defeats the substitution.

**Not a redirection issue but adjacent:** writing to `HKLM\…\Session Manager\Environment` requires
`KEY_SET_VALUE` on a key that is administrator-writable only. That is ticket 12's problem, but the read
path must degrade gracefully: open HKLM with `KEY_READ` (which succeeds unelevated — confirmed, every probe
above ran unelevated) and only request write access at apply time.

---

## 6. The `WM_SETTINGCHANGE` broadcast

### 6.1 Exact `windows`-crate signature

Verbatim from Microsoft's generated docs
([SendMessageTimeoutW](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/fn.SendMessageTimeoutW.html)),
feature `Win32_UI_WindowsAndMessaging`:

```rust
pub unsafe fn SendMessageTimeoutW(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    fuflags: SEND_MESSAGE_TIMEOUT_FLAGS,
    utimeout: u32,
    lpdwresult: Option<*mut usize>,
) -> LRESULT
```

(`windows-sys` 0.61.2 has the same shape with `lpdwresult: *mut usize` and no `Option`.)

The corresponding call:

```rust
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

// MUST be UTF-16 and MUST outlive the call.
let env: Vec<u16> = "Environment".encode_utf16().chain(std::iter::once(0)).collect();

let mut result: usize = 0;
let rc = unsafe {
    SendMessageTimeoutW(
        HWND_BROADCAST,                     // (HWND)0xffff
        WM_SETTINGCHANGE,                   // 0x001A
        WPARAM(0),                          // "When an application sends this message, this parameter must be NULL."
        LPARAM(env.as_ptr() as isize),
        SMTO_ABORTIFHUNG,                   // 0x0002
        5000,
        Some(&mut result),
    )
};
// rc.0 == 0  =>  failed or timed out. See 6.3.
```

`HWND_BROADCAST` is `(HWND)0xffff`, `WM_SETTINGCHANGE` is `0x001A` (aliased to `WM_WININICHANGE`), and
`SMTO_ABORTIFHUNG` is `0x0002`
([SendMessageTimeout](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeouta),
[WM_SETTINGCHANGE](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-settingchange)).

### 6.2 String encoding — the footgun

`lParam` is a **pointer to a string**, and the A/W split is real:

> "To effect a change in the environment variables for the system or the user, broadcast this message with
> *lParam* set to the string "Environment"."
> — [WM_SETTINGCHANGE](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-settingchange)

> "The system only does marshalling for system messages (those in the range 0 to (WM_USER-1))."
> — [SendMessageTimeout remarks](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeouta)

`WM_SETTINGCHANGE` = 0x001A, well below `WM_USER` (0x0400), so USER32 marshals and A/W-translates the
`lParam` string on the caller's behalf — **according to which entry point you called**. Therefore:

- `SendMessageTimeoutW` requires a **UTF-16LE, NUL-terminated** buffer.
- `SendMessageTimeoutA` requires an **ANSI/8-bit, NUL-terminated** buffer.

Two ways to get it wrong from Rust, both of which compile and both of which "succeed":

1. Passing a Rust `&str`/`CStr` (UTF-8, one byte per ASCII char) to **`SendMessageTimeoutW`**. USER32 reads
   it as UTF-16 and sees `"En"` re-encoded from `0x6E45, 0x6976, …` — garbage. Nothing errors.
2. Passing a UTF-16 buffer to **`SendMessageTimeoutA`**. The second byte of `'E'` is `0x00`, so the ANSI
   reader terminates immediately and the receiver sees `"E"`. Nothing errors.

Also: keep the buffer alive across the call. `LPARAM("Environment".encode_utf16()…collect::<Vec<_>>().as_ptr() as isize)`
in a single expression is a dangling pointer — the temporary `Vec` is dropped before `SendMessageTimeoutW`
runs. Bind it to a named local first, as above.

### 6.3 What the timeout means, and who actually honours the broadcast

**Return value.** "If the function succeeds, the return value is nonzero. SendMessageTimeout does not
provide information about individual windows timing out if HWND_BROADCAST is used. If the function fails or
times out, the return value is 0. Note that the function does not always call SetLastError on failure. If
the reason for failure is important to you, call `SetLastError(ERROR_SUCCESS)` before calling
SendMessageTimeout." So a broadcast gives you **one bit**, and it does not tell you which window hung.
Do not surface "some app didn't respond" as an apply failure — the registry write already succeeded and is
authoritative; the broadcast is a courtesy.

**The timeout is per-window, and it multiplies.** "If this parameter is HWND_BROADCAST … The function does
not return until each window has timed out. Therefore, the total wait time can be up to the value of
*uTimeout* multiplied by the number of top-level windows" and "if you specify a five second time-out period
and there are three top-level windows that fail to process the message, you could have up to a 15 second
delay."

Measured on this desktop:

```
top-level windows enumerated by EnumWindows: 226
SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, 'Environment', SMTO_ABORTIFHUNG, 5000)
  return = 1  (nonzero = success)
  elapsed = 37 ms   (worst case would be 5000 x 226)
```

37 ms in the healthy case; the theoretical worst case with `uTimeout = 5000` on this machine is
5000 × 226 ≈ **18.8 minutes** of a frozen UI. The ticket's proposed 5000 ms is far too generous.
`SMTO_ABORTIFHUNG` ("returns without waiting for the time-out period to elapse if the receiving thread
appears to not respond or 'hangs'", where "this function considers that a thread is not responding if it has
not called GetMessage or a similar function within five seconds") caps the damage in the *hung* case, but
does not help against a window that is alive and merely slow. For reference, .NET uses `fuFlags = 0`
(`SMTO_NORMAL`) and `uTimeout = 1000`
([Environment.Windows.cs](https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/Environment.Windows.cs)).
**Recommendation: `SMTO_ABORTIFHUNG | SMTO_NORMAL`, `uTimeout` between 1000 and 2000 ms, and run the
broadcast off the UI thread so the window never freezes regardless.**

**Who honours it.** Microsoft documents the effect precisely, for both scopes:

> "If `target` is EnvironmentVariableTarget.User, the environment variable is stored in the
> HKEY_CURRENT_USER\Environment key … **It is also copied to instances of File Explorer that are running as
> the current user. The environment variable is then inherited by any new processes that the user launches
> from File Explorer.** … If `target` is User or Machine, other applications are notified of the set
> operation by a Windows `WM_SETTINGCHANGE` message."
> — [Environment.SetEnvironmentVariable](https://learn.microsoft.com/en-us/dotnet/api/system.environment.setenvironmentvariable)

and

> "To programmatically add or modify system environment variables, add them to the
> HKEY_LOCAL_MACHINE\System\CurrentControlSet\Control\Session Manager\Environment registry key, then
> broadcast a WM_SETTINGCHANGE message with *lParam* set to the string "Environment". This allows
> applications, such as the shell, to pick up your updates."
> — [Environment Variables](https://learn.microsoft.com/en-us/windows/win32/procthread/environment-variables)

So the honest, documented answer is: **Explorer refreshes its own environment block, and anything launched
from Explorer afterwards inherits the new PATH.** Everything else is opt-in — a process only sees the change
if its own window procedure handles `WM_SETTINGCHANGE` and re-reads. Already-running consoles, editors,
build daemons and language servers overwhelmingly do not, because their environment block was fixed at
`CreateProcess` time and `SetEnvironmentVariable` "has no effect on … the environment variables of other
processes"
([SetEnvironmentVariable](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setenvironmentvariable)).
Independently visible on this machine: this shell's `PATH` (1796 chars) is neither the raw nor the
freshly-merged registry value (2198) — it is a stale inherited snapshot.

**UX consequence for the spec:** after a successful apply, PathMaster must tell the user, in words, that
already-open programs keep the old PATH and that a program must be started fresh (from Explorer or after
sign-out) to see the change. Presenting the broadcast as "applied everywhere" is a lie the user will
discover in their already-open terminal.

---

## 7. External-modification detection for FR-apply

Three candidate mechanisms; the ticket names two.

### 7.1 `RegQueryInfoKey` last-write time — **not sufficient alone**

`RegQueryInfoKeyW`'s `lpftLastWriteTime` is the **key's** timestamp, not the value's. There is one
`Environment` key per scope holding many values:

```
HKCU\Environment  : values=39 lastWrite=2026-08-16T16:45:49.4513564+03:00
HKLM\…\Environment: values=21 lastWrite=2026-07-24T23:02:28.4607438+03:00
```

39 values in the user key. Any installer touching `CARGO_HOME`, `TEMP`, `BUN_INSTALL` — or an app rewriting
an unrelated variable — bumps the same timestamp. So it produces **false positives**: PathMaster would tell
the user "someone changed PATH under you" when nothing about PATH changed, and after two or three such
warnings the user learns to click through the dialog. That is worse than not warning.

It can also miss: a write of byte-identical content still bumps the stamp (false positive again), and there
is no guarantee in the reference page that the granularity distinguishes two writes inside the same clock
tick.

Its one genuine use is as a **cheap pre-filter**: if the stamp is unchanged, nothing in the key changed, so
you can skip the compare. It is a valid negative test, never a positive one.

`winreg` exposes it: `RegKey::query_info() -> RegKeyMetadata` calls `RegQueryInfoKeyW` and the struct's
`last_write_time: FileTime` field is public, with `get_last_write_time_system()` and (feature-gated)
`get_last_write_time_chrono()`
([reg_key.rs](https://raw.githubusercontent.com/gentoo90/winreg-rs/master/src/reg_key.rs),
[reg_key_metadata.rs](https://raw.githubusercontent.com/gentoo90/winreg-rs/master/src/reg_key_metadata.rs)).

### 7.2 Re-read-and-compare — **this is the reliable one**

Immediately before writing, re-do the raw read and compare **`(vtype, bytes)`** against the snapshot taken
when the editing session opened. Byte comparison on the raw buffer, not on a decoded/normalised string —
otherwise you will miss exactly the changes that matter (a `%VAR%` collapsed to a literal path, a type flip
`REG_EXPAND_SZ` → `REG_SZ`, a changed trailing terminator, a case change, an added/removed trailing `;`).

This has no false positives and detects every change to the value it guards. Its limits, honestly stated:

- **It is TOCTOU-racy.** Another process can write between your compare and your `RegSetValueExW`. The window
  is microseconds. A registry transaction
  ([`RegCreateKeyTransactedW`](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regcreatekeytransactedw)
  / `RegOpenKeyTransacted` + `CreateTransaction`) does not close it either — it isolates *your* writes, it
  does not make another process's non-transacted write wait, and the same page warns "If a non-transacted
  operation is performed on the key before the transaction is committed, the transaction is rolled back",
  which on a key 39 other values live in is a rollback waiting to happen. Not worth it. Shrink the window
  instead: compare and write under the same open handle, back to back, with no UI in between.
- **It cannot tell you *who* changed it or *when*.** It only reports that the value differs from the
  snapshot. That is enough for the decision the user must make (reload / overwrite / cancel).
- **It says nothing about the other scope.** Check the scope you are about to write.

### 7.3 `RegNotifyChangeKeyValue` — the missing third option, for *live* detection

If the requirement is "notice while the window is open" rather than only "check at apply time",
[`RegNotifyChangeKeyValue`](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regnotifychangekeyvalue)
with `REG_NOTIFY_CHANGE_LAST_SET` ("Notify the caller of changes to a value of the key. This can include
adding or deleting a value, or changing an existing value.") signals an event. Caveats from the same page,
all load-bearing:

- Requires the key to be opened with `KEY_NOTIFY`.
- **One-shot**: "This function detects a single change. After the caller receives a notification event, it
  should call the function again to receive the next notification."
- Must run on a **persistent** thread, or pass `REG_NOTIFY_THREAD_AGNOSTIC` (0x10000000, Windows 8+),
  otherwise "the event is signaled every time the thread terminates, not just when there is a registry
  change."
- "If the specified key is closed, the event is signaled" — so a signal is not proof of a change.
- Re-registering with the same parameters before the previous wait completes leaks a wait operation.
- It is key-scoped, so like §7.1 it fires for *any* value in `Environment`.

**Recommended design:** `RegNotifyChangeKeyValue` (optional, for a live "changed on disk" indicator) →
always followed by a raw re-read to find out whether `Path` specifically changed → and, independently,
a mandatory re-read-and-compare of `(vtype, bytes)` immediately before every write. Use the
`RegQueryInfoKey` stamp only as a fast negative pre-filter, never as the decision.

---

## 8. Hazards — how a naive implementation silently corrupts a user's PATH

Ordered by how quietly they fail. Every one of these produces a *successful* write with wrong content.

**H1. Reading through anything that expands.** `RegGetValue` without `RRF_NOEXPAND`, or
`[Environment]::GetEnvironmentVariable('Path','Machine')`, or `RegistryKey.GetValue` without
`DoNotExpandEnvironmentNames`. Measured here: the System `Path`'s `%SystemRoot%\system32` comes back as
`C:\WINDOWS\system32`. Write that back and the user's PATH is permanently frozen against a Windows
directory move, an OS upgrade that relocates a folder, or a `%JAVA_HOME%` retarget. The user sees a PATH
that "looks the same" — every entry still resolves today. Nothing surfaces until months later.

**H2. Losing the value type on write.** `winreg`'s `set_value::<String>` writes `REG_SZ` unconditionally;
so does .NET's `SetValue(String, Object)`. Combined with H1 this is the standard PATH-destroying round trip.
After it, every `%VAR%` the user later types into that scope is stored literally and never resolves — again
with no error, and again invisible until something fails to launch.

**H3. Trusting `pdwType` from a non-`RRF_NOEXPAND` `RegGetValue`.** Measured: it reports `REG_SZ` for a
value stored as `REG_EXPAND_SZ`. An implementation that faithfully "preserves the type it read" will still
downgrade the value, because the type it read was a lie.

**H4. Treating "value missing" as an error, or as an empty string.** `ERROR_FILE_NOT_FOUND` (2) is a
legitimate state. Mapping it to `Ok("")` and then writing loses the distinction between "PATH was absent"
and "PATH was empty", which the restore path needs. Mapping it to a hard failure makes the app unusable on
a profile that has no user PATH yet.

**H5. Creating a missing `Path` as `REG_SZ`.** Silently and permanently denies `%VAR%` entries in a scope
the app just created. Every `Path` on this machine — user, machine, `.DEFAULT`, S-1-5-18 — is
`REG_EXPAND_SZ`.

**H6. Getting the NUL terminator wrong.** Stored values here are exactly `2 × (chars + 1)` bytes.
Writing without a terminator leaves a value that later readers may over-read (documented on
`RegQueryValueExW`). Writing the size that a `RegGetValue` **probe** reported stores a *double*-terminated
value (measured: probe says 1384, actual data is 1382) — a 2-byte drift that breaks byte-for-byte comparison
and grows by 2 bytes on every save. Conversely, decoding with `String::from_utf16_lossy` and stripping
`while s.ends_with('\0')` will also silently swallow a *legitimately* double-terminated value, so
round-tripping "raw" through a decoded `String` is not byte-for-byte. Keep and compare the original bytes.

**H7. Normalising while editing.** Trimming whitespace, dropping a trailing `;`, collapsing `\\` to `\`,
case-folding, reordering, or re-joining entries that were never split cleanly. Each is defensible in
isolation; together they mean the value the user "didn't change" comes back different. Split, edit and
re-join must be an exact inverse for any input the app did not deliberately modify. Prove this with a
property test: raw bytes → parse → join → bytes must be the identity when no edit was made.

**H8. Diagnosing against the process environment.** `std::env::var("PATH")` is a snapshot inherited from
whatever launched PathMaster. Measured here: 1796 chars, versus 2198 for the freshly merged registry value —
neither equal to the raw sum (2207) nor to the live merge. Any length check, duplicate check or
"does this directory exist" check run against the process PATH diagnoses the wrong string.

**H9. Length-checking the wrong string.** Checking each scope against 32,767 separately is meaningless (the
per-variable limit applies to the *merged* value); checking the sum of the two *raw* strings is off by the
expansion delta (measured: 2207 vs 2198). And the 32,767 figure is **not** a registry limit and **not** the
environment-block limit on any supported Windows — asserting either in a user-facing message is wrong.
1024 (`setx`), 2047 (`sysdm.cpl`) and 2048 (registry performance guidance) are none of them a limit
PathMaster is subject to.

**H10. Blocking the UI on the broadcast.** `uTimeout` is per top-level window and multiplies: 5000 ms × 226
windows on this machine is a theoretical 18.8-minute freeze. And because `SendMessageTimeout` returns a
single bit for a broadcast and "does not always call SetLastError on failure", a `0` return must not be
reported as "the PATH change failed" — the registry write already succeeded.

**H11. Getting the broadcast string encoding wrong.** UTF-8 into `SendMessageTimeoutW` → receivers read
garbage; UTF-16 into `SendMessageTimeoutA` → receivers see `"E"`. Both "succeed". Related: letting the
UTF-16 `Vec` be a temporary in the call expression leaves `lParam` dangling. The failure mode in all three
cases is identical and invisible — Explorer simply never refreshes, and the user blames the app for
"not applying".

**H12. Promising more than the broadcast delivers.** Documented effect is Explorer's own environment block
plus anything launched from Explorer afterwards. Already-running shells and editors keep the old PATH,
because their block was fixed at `CreateProcess`. Saying "applied" without saying "restart your terminal"
guarantees a bug report.

**H13. Using the key's last-write time to decide whether PATH changed.** It is the *key's* stamp, and
`HKCU\Environment` holds 39 values here. Any installer bumps it. False "someone changed your PATH" warnings
train the user to click through the one that matters. Use a raw `(vtype, bytes)` compare for the decision;
the stamp is only a valid *negative* pre-filter.

**H14. Writing PATH from a 32-bit build.** WOW64 rewrites a leading `%ProgramFiles%` to
`%ProgramFiles(x86)%` on `REG_SZ`/`REG_EXPAND_SZ` writes unless the key was opened `KEY_WOW64_64KEY`. Ship
x64/ARM64. (Reads are safe either way — the `Session Manager\Environment` key is shared, verified
byte-identical across all three views.)

**H15. Not backing up the raw bytes.** Every hazard above is recoverable if, and only if, the pre-write
snapshot preserved `(vtype, exact bytes)` for both scopes. A backup that stores a decoded, normalised
string has already lost the information needed to undo an H1/H2/H6 corruption. Feeds ticket 14.

---

## Appendix — reproduction

The two probe scripts used (read-only P/Invoke, and the idempotent broadcast) are at
`C:\Temp\claude\C--dev-PathMaster2\5cab669f-a49a-4c27-a633-ca7a38dc116d\scratchpad\probe.ps1` and
`…\broadcast.ps1`. They are scratch, not part of the repo; re-create them from the outputs quoted above if
the numbers need re-checking on another machine.
