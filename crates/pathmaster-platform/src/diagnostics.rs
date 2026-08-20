//! The diagnostic pass's shell (spec §7, FR-diag-async): the two adapters that
//! answer the rulebook's questions about *this* machine, and the one worker
//! thread that runs it.
//!
//! The rules themselves are in `pathmaster-core` and take no OS call. What they
//! cannot answer from text alone arrives here — the process environment, for
//! `%VAR%`, and the filesystem, for "does this name a directory?" — so this
//! module is the whole of the machine the diagnosis depends on.
//!
//! **A pass never runs on the UI thread.** A `PATH` of a few hundred Entries is
//! a few hundred `GetFileAttributesW` calls, and a stalled message pump is a
//! window NVDA cannot read. It never runs on the UI thread and it never touches
//! a widget from this one: wx's event tables are thread-local, so a widget call
//! from here would not crash — it would silently do nothing, which is worse.
//! Results cross back over an `mpsc` channel that the caller drains from a wx
//! Timer.
//!
//! **The subject of a pass is the two Working Copies** — never the process's
//! own `PATH`, never a fresh registry read. What the user is editing is what
//! gets diagnosed, including the changes they have not applied.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use pathmaster_core::diagnostics::{diagnose, Diagnosis, Existence, Filesystem, RootKind};
use pathmaster_core::normalize::Environment;

use windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows_sys::Win32::Storage::FileSystem::{
    GetDriveTypeW, GetFileAttributesW, FILE_ATTRIBUTE_DIRECTORY, INVALID_FILE_ATTRIBUTES,
};
use windows_sys::Win32::System::Diagnostics::Debug::{SetThreadErrorMode, SEM_FAILCRITICALERRORS};
use windows_sys::Win32::System::Environment::GetEnvironmentVariableW;
use windows_sys::Win32::System::WindowsProgramming::DRIVE_REMOTE;

/// One pass to run: both Working Copies as text, in runtime order, and the
/// generation that says which edit they were taken from.
struct Request {
    generation: u64,
    system: Vec<String>,
    user: Vec<String>,
}

/// One pass's result, still carrying the generation it was asked for.
struct Reply {
    generation: u64,
    diagnosis: Diagnosis,
}

/// The diagnostic pass, seen from the UI thread: ask for one with
/// [`request`](Worker::request), drain it with [`take`](Worker::take) from a
/// Timer, and stop the Timer when nothing is [`outstanding`](Worker::outstanding).
///
/// The generation counter is the whole of the staleness rule, and it lives
/// here so the caller cannot forget it: an edit lands, a pass is already in
/// flight, and the answer it brings back describes Entries that have since
/// moved. Findings are read against the Working Copy a pass ran over, so
/// showing an overtaken one would put the wrong words on the wrong rows. Such
/// a reply is dropped, never shown; the pass the edit asked for is already on
/// its way.
pub struct Worker {
    requests: Sender<Request>,
    replies: Receiver<Reply>,
    /// The generation of the last pass asked for.
    sent: u64,
    /// The generation of the last pass whose result reached the UI. Equal to
    /// `sent` exactly when nothing is outstanding.
    settled: u64,
}

impl Worker {
    /// Spawns the worker over this machine's environment and filesystem.
    pub fn spawn() -> Worker {
        Worker::spawn_over(Box::new(ProcessEnvironment), Box::new(LocalFilesystem))
    }

    /// Spawns the worker over injected adapters — the seam the tests drive it
    /// through, and the reason the two questions are traits at all.
    pub fn spawn_over(
        env: Box<dyn Environment + Send>,
        filesystem: Box<dyn Filesystem + Send>,
    ) -> Worker {
        let (requests, incoming) = mpsc::channel::<Request>();
        let (outgoing, replies) = mpsc::channel::<Reply>();
        thread::spawn(move || run(&incoming, &outgoing, env.as_ref(), filesystem.as_ref()));
        Worker {
            requests,
            replies,
            sent: 0,
            settled: 0,
        }
    }

    /// Asks for a pass over both Working Copies, System first — the runtime
    /// order, which is what decides who carries a cross-scope duplicate.
    ///
    /// One is asked for after **every** change to either Copy: a System edit
    /// changes what User's Entries are duplicates of, so there is one pass over
    /// both rather than one per Scope.
    pub fn request(&mut self, system: Vec<String>, user: Vec<String>) {
        self.sent += 1;
        let request = Request {
            generation: self.sent,
            system,
            user,
        };
        if self.requests.send(request).is_err() {
            // The worker thread is gone. Nothing is outstanding any more, so
            // the Timer stops rather than polling forever for a reply that is
            // never coming — the Status column simply stops being updated,
            // which is the degraded answer, not a hung window.
            self.settled = self.sent;
        }
    }

    /// Whether a pass is still on its way. The wx Timer runs exactly while
    /// this is true.
    pub fn outstanding(&self) -> bool {
        self.sent != self.settled
    }

    /// The newest completed pass, if one has landed — and nothing at all for a
    /// pass the Working Copies have already outrun.
    pub fn take(&mut self) -> Option<Diagnosis> {
        let mut current = None;
        loop {
            match self.replies.try_recv() {
                Ok(reply) if reply.generation == self.sent => {
                    self.settled = reply.generation;
                    current = Some(reply.diagnosis);
                }
                // Overtaken: the Working Copies changed while it was running.
                Ok(_) => {}
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.settled = self.sent;
                    break;
                }
            }
        }
        current
    }
}

/// The worker thread: one pass at a time, always over the newest request.
///
/// A burst of edits — a held arrow key on Move Down, an undo run — queues one
/// request each, and running them in turn would spend the whole budget on
/// states the user has already left. So the queue is drained to its last
/// element before a pass starts, and the generations in between are simply
/// never answered. [`Worker::take`] is what makes that safe: it waits for the
/// generation it asked for.
fn run(
    requests: &Receiver<Request>,
    replies: &Sender<Reply>,
    env: &dyn Environment,
    filesystem: &dyn Filesystem,
) {
    // A probe of `A:\…` on a machine with no floppy — or of any removable
    // drive with no medium — raises the OS's own "There is no disk in the
    // drive" dialog, from a thread that has no window to own it. Thread-scoped
    // and set once: this thread does nothing but probe paths it did not choose.
    silence_device_errors();
    while let Ok(mut request) = requests.recv() {
        while let Ok(newer) = requests.try_recv() {
            request = newer;
        }
        let reply = Reply {
            generation: request.generation,
            diagnosis: diagnose(&request.system, &request.user, env, filesystem),
        };
        if replies.send(reply).is_err() {
            break; // The window is gone.
        }
    }
}

/// `SEM_FAILCRITICALERRORS` alone: it is the flag that suppresses the "There is
/// no disk in the drive" box a removable root raises. Its usual travelling
/// companion `SEM_NOOPENFILEERRORBOX` governs the legacy `OpenFile` API, which
/// nothing here calls, so setting it would be decoration.
fn silence_device_errors() {
    let mut previous = 0;
    // SAFETY: a thread-local mode flag; the out parameter is a live local that
    // outlives the call.
    unsafe {
        SetThreadErrorMode(SEM_FAILCRITICALERRORS, &mut previous);
    }
}

/// The process environment, which is what `%VAR%` expands against (spec §7,
/// FR-diag-normalise).
///
/// It is read, never written, and never merged with the Working Copies: the
/// values being edited are not this process's own `PATH`, and diagnosing the
/// latter would answer a question nobody asked.
pub struct ProcessEnvironment;

impl Environment for ProcessEnvironment {
    /// `GetEnvironmentVariableW`, whose lookup ignores case — `%systemroot%`
    /// and `%SystemRoot%` are one variable.
    ///
    /// An undefined name answers `None` rather than an empty string: the rules
    /// leave such a reference literal and report it, and an empty value would
    /// expand it silently away.
    fn lookup(&self, name: &str) -> Option<String> {
        let name = wide(name);
        // SAFETY: both calls read `name` (NUL-terminated) and write only into
        // `buffer`, never past the length handed to them. The two-call shape is
        // the documented one: the first answers the size it needs.
        unsafe {
            let needed = GetEnvironmentVariableW(name.as_ptr(), std::ptr::null_mut(), 0);
            if needed == 0 {
                return None;
            }
            let mut buffer = vec![0u16; needed as usize];
            let written = GetEnvironmentVariableW(name.as_ptr(), buffer.as_mut_ptr(), needed);
            // A value that changed size between the two calls, or vanished.
            if written == 0 || written >= needed {
                return None;
            }
            Some(String::from_utf16_lossy(&buffer[..written as usize]))
        }
    }
}

/// The filesystem, asked the two questions the rules keep apart: where a path's
/// root lives, and — for a local root only — what the path names.
pub struct LocalFilesystem;

impl Filesystem for LocalFilesystem {
    /// Classifies the root without a network round trip: the UNC prefix by
    /// text, a drive letter by `GetDriveTypeW`, which answers from the mount
    /// table.
    ///
    /// **Everything beginning with two separators is Network**, the device
    /// namespace included. `\\?\UNC\server\share` is a UNC path wearing a
    /// prefix and `\\?\C:\tools` is not, and the only cheap way to be sure is
    /// not to ask — a `\\?\C:` Entry therefore goes unprobed and never flags
    /// Missing. That is a false negative on a spelling all but unheard of in a
    /// `PATH`, traded against the 20-60 second uncancellable block a dead UNC
    /// costs, which is the whole reason this question exists.
    ///
    /// Text with no root at all — an Entry whose leading `%VAR%` this run does
    /// not define — is Local, because it must reach the probe: failing there
    /// is how an undefined reference flags Missing (spec §7, D10).
    fn root_kind(&self, path: &str) -> RootKind {
        let mut chars = path.chars();
        match (chars.next(), chars.next()) {
            (Some(first), Some(second)) if is_separator(first) && is_separator(second) => {
                RootKind::Network
            }
            (Some(drive), Some(':')) if drive.is_ascii_alphabetic() => {
                let root = wide(&format!("{drive}:\\"));
                // SAFETY: `root` is a NUL-terminated UTF-16 buffer bound above,
                // so it outlives the call; the call reads it and returns a
                // scalar.
                let kind = unsafe { GetDriveTypeW(root.as_ptr()) };
                if kind == DRIVE_REMOTE {
                    RootKind::Network
                } else {
                    RootKind::Local
                }
            }
            _ => RootKind::Local,
        }
    }

    /// `GetFileAttributesW` on the expanded text **verbatim** — slashes,
    /// trailing separator and all. Measured (ticket impl-09): Win32 resolves
    /// `C:/Windows` exactly as `C:\Windows`, and rewriting the text would
    /// change what a `\\?\` path means. Slash direction belongs to the
    /// comparison key, which is a different question.
    ///
    /// A directory is the only healthy answer: a `PATH` search appends
    /// `\name.exe` to the Entry, so an Entry naming a file finds nothing.
    /// Access denied is emphatically **not** not-found — the path is there,
    /// and the rules do not tell the user to delete it. That branch is a good
    /// deal harder to reach than it looks: `GetFileAttributesW` needs only
    /// `FILE_READ_ATTRIBUTES`, which Windows grants implicitly to anyone who
    /// may traverse the parent, so a deny-ACL'd directory still reads its
    /// attributes back and so does `C:\System Volume Information` on an
    /// ordinary account (both measured). It survives for the hardened tokens
    /// without that implicit grant.
    fn probe(&self, path: &str) -> Existence {
        let path = wide(path);
        // SAFETY: `path` is a NUL-terminated UTF-16 buffer bound above, so it
        // outlives the call; the call reads it and returns a scalar.
        let attributes = unsafe { GetFileAttributesW(path.as_ptr()) };
        if attributes != INVALID_FILE_ATTRIBUTES {
            return if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                Existence::Directory
            } else {
                Existence::File
            };
        }
        // Every other failure — not found, a bad name, a missing drive — means
        // the Entry names nothing usable, which is what Missing says.
        match std::io::Error::last_os_error().raw_os_error() {
            Some(code) if code as u32 == ERROR_ACCESS_DENIED => Existence::AccessDenied,
            _ => Existence::NotFound,
        }
    }
}

fn is_separator(c: char) -> bool {
    c == '\\' || c == '/'
}

/// A NUL-terminated UTF-16 copy, the only string shape these APIs take.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
