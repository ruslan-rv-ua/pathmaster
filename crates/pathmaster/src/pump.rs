//! The Timer drain (spec §17): the worker thread that runs a diagnostic pass
//! and the wx Timer that collects it, held together because they are one
//! mechanism (spec §7, FR-diag-async).
//!
//! The window asks for a pass and, later, takes the result. It never sees the
//! Timer, which is the point: **"the Timer runs only while a pass is
//! outstanding"** is one rule, and splitting it between the caller's request
//! path and its tick handler is how it would come apart. Here, asking starts
//! it and taking the last outstanding result stops it.
//!
//! Nothing crosses to the worker but text, and nothing comes back but a
//! [`Diagnosis`]. Widgets are never touched off the UI thread — wx's event
//! tables are thread-local, so a widget call from the worker would not crash,
//! it would silently do nothing.

use std::cell::RefCell;

use pathmaster_core::diagnostics::Diagnosis;
use pathmaster_platform::diagnostics::Worker;
use wxdragon::prelude::*;
use wxdragon::timer::Timer;

/// How often the UI thread looks for a finished pass (spec §7, FR-diag-async).
const POLL_MS: i32 = 100;

/// The diagnostic pass, as the window holds it.
pub struct Pump {
    worker: RefCell<Worker>,
    timer: Timer<Frame>,
}

impl Pump {
    /// Spawns the worker and creates the Timer over `frame`, which owns it.
    pub fn new(frame: &Frame) -> Pump {
        Pump {
            worker: RefCell::new(Worker::spawn()),
            timer: Timer::new(frame),
        }
    }

    /// Binds what the window does on each tick — which must be to call
    /// [`take`](Self::take), because that is also what stops the Timer.
    ///
    /// There is exactly one Timer in the application, and that matters:
    /// wxdragon binds a tick on the timer's *owner*, so a second Timer on the
    /// same frame would fire this handler too.
    pub fn on_tick(&self, handler: impl FnMut(Event) + 'static) {
        self.timer.on_tick(handler);
    }

    /// Asks for a pass over both Working Copies, System first — the runtime
    /// order, which is what decides who carries a cross-scope duplicate.
    ///
    /// The Timer is started unconditionally rather than only when it is not
    /// already running. Guarding on `is_running()` would let a `Start` that
    /// answered false — a Timer that could not be created, a system out of
    /// timer handles — strand the pass it was asked for until the next edit
    /// happened to retry; asking every time is what makes it self-healing.
    /// Restarting a running Timer only resets its countdown, and the pass just
    /// asked for is precisely the one worth waiting the interval for.
    pub fn request(&self, system: Vec<String>, user: Vec<String>) {
        self.worker.borrow_mut().request(system, user);
        self.timer.start(POLL_MS, false);
    }

    /// The newest completed pass, if one has landed — and nothing at all for a
    /// pass the Working Copies have already outrun, which the worker drops.
    ///
    /// Stops the Timer once nothing is outstanding: an application at rest
    /// does not wake its UI thread ten times a second.
    pub fn take(&self) -> Option<Diagnosis> {
        // Two statements, not one: the mutable borrow must be released before
        // the shared one below, and an `if let` would hold it to the end of
        // the block.
        let landed = self.worker.borrow_mut().take();
        if !self.worker.borrow().outstanding() {
            self.timer.stop();
        }
        landed
    }
}
