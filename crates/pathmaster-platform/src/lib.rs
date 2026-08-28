//! PathMaster's imperative shell — Windows API adapters, no wx (spec §17, ADR-0007).
//!
//! Modules land with their tickets: `registry` (key path as a constructor parameter),
//! `datadir`, `diagnostics`, `elevation`, `geometry` (where the window opens), `locale`,
//! `logwriter`, `panic_hook`, `settings`, `broadcast`, `snapshots`, `apply`, `startup`,
//! `args` (what the command line says, and how a self-relaunch writes one),
//! `shell` and `help` (the User Guide's file, and handing it to the browser).

#[cfg(windows)]
pub mod apply;
#[cfg(windows)]
pub mod args;
#[cfg(windows)]
pub mod broadcast;
#[cfg(windows)]
pub mod datadir;
#[cfg(windows)]
pub mod diagnostics;
#[cfg(windows)]
pub mod elevation;
#[cfg(windows)]
pub mod geometry;
#[cfg(windows)]
pub mod help;
#[cfg(windows)]
pub mod locale;
#[cfg(windows)]
pub mod logwriter;
#[cfg(windows)]
pub mod panic_hook;
#[cfg(windows)]
pub mod registry;
#[cfg(windows)]
pub mod settings;
#[cfg(windows)]
pub mod shell;
#[cfg(windows)]
pub mod snapshots;
#[cfg(windows)]
pub mod startup;
