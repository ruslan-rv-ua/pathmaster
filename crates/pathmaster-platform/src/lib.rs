//! PathMaster's imperative shell — Windows API adapters, no wx (spec §17, ADR-0007).
//!
//! Modules land with their tickets: `registry` (key path as a constructor parameter),
//! `datadir`, `elevation`, `logwriter`, `panic_hook`, `broadcast`.

#[cfg(windows)]
pub mod registry;
