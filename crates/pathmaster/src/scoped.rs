//! Scoped access (spec §11, ADR-0011): the wrapper for every state cell that
//! more than one kind of call reaches — a command, the diagnostic Timer's
//! tick, or a synchronous handler the toolkit calls back.
//!
//! The guard is created and dropped inside [`with`](Scoped::with) /
//! [`with_mut`](Scoped::with_mut), and the closure's argument cannot outlive
//! the call — so no `Ref` can escape into a scope that opens a dialog, and
//! holding a borrow across a nested event loop is unrepresentable. What
//! remains is one reviewable rule about a short closure body: **it must not
//! dispatch** — no dialog, no `set_selection`, no page rebuild inside it.
//! Data that a rebuild or a dialog needs is copied out of the closure as owned
//! values and used after it has returned.
//!
//! Cells only one kind of call touches (`last_read`, `settings`, the `Pump`'s
//! worker behind its interface) stay plain `RefCell`s with local borrows; a
//! future cell classifies itself by the same rule, not by a roster.

use std::cell::RefCell;

/// One state cell behind scoped access. A `RefCell` whose guards cannot leave
/// this module.
pub struct Scoped<T> {
    cell: RefCell<T>,
}

impl<T> Scoped<T> {
    pub fn new(value: T) -> Scoped<T> {
        Scoped {
            cell: RefCell::new(value),
        }
    }

    /// Reads the cell through a closure. The answer must be owned — the
    /// closure's argument cannot escape it, which is the whole mechanism.
    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        read(&self.cell.borrow())
    }

    /// Writes the cell through a closure, under the same containment.
    pub fn with_mut<R>(&self, write: impl FnOnce(&mut T) -> R) -> R {
        write(&mut self.cell.borrow_mut())
    }
}
