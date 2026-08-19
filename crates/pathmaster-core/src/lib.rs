//! PathMaster's pure core: no I/O, no OS calls, any-OS (spec §17, ADR-0007).
//!
//! Modules land with their tickets: `path` (split/join), `normalize`, `diagnostics`,
//! `session`, `snapshot`, `rotation`, `thresholds`, `settings`, `logfmt`, `msgids`.

#![forbid(unsafe_code)]

pub mod path;
pub mod session;
