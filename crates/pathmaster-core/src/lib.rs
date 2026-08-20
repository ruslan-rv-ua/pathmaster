//! PathMaster's pure core: no I/O, no OS calls, any-OS (spec §17, ADR-0007).
//!
//! Modules land with their tickets: `path` (split/join), `normalize`, `diagnostics`,
//! `session`, `snapshot`, `rotation`, `thresholds`, `settings`, `logfmt`, `language`,
//! `msgids`.

#![forbid(unsafe_code)]

pub mod diagnostics;
pub mod language;
pub mod logfmt;
pub mod msgids;
pub mod normalize;
pub mod path;
pub mod rotation;
pub mod session;
pub mod settings;
pub mod snapshot;
pub mod thresholds;
