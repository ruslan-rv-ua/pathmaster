//! `copy()`: the application's one clipboard write (v0.2.0 §8).
//!
//! **Deliberately not `wxdragon::Clipboard`**, which the delta-spec named
//! before this was measured. `wxClipboard::AddData` reports a failed
//! `OleSetClipboard` through `wxLogSysError`, and a GUI app's default log
//! target turns that into a modal `MessageBox` — measured live: holding the
//! clipboard open from another process makes a Ctrl+C speak Announcement 14
//! *and then* raise an untranslated «Pathmaster Error» box, outside the closed
//! Catalogue, stealing focus, and misspelling the product name (wx capitalises
//! it). `wxClipboard::Flush` reports the same way, so the spec's best-effort
//! flush could raise one too.
//!
//! The wxWidgets answer is `wxLogNull` around the call — what KiCad did for
//! this exact message. wxdragon binds **no** `wxLog` at any level, 0.9.20
//! included, so it is unreachable from here; the reachable answer is to not
//! ask wx. This is the plain Win32 path `clipboard-win` takes, and it says
//! nothing to anyone: the `bool` is the whole report, which is exactly what
//! §8 asks for — success speaks Announcement 13, failure speaks 14.
//!
//! **And it needs no flush.** `SetClipboardData` with a real `HGLOBAL` hands
//! the memory to the system, which owns it from then on; only *delayed
//! rendering* — a null handle — dies with the process, and only OLE's live
//! data object needs `OleFlushClipboard`. So "the copy outlives the Run" is
//! the mechanism's own property here rather than a second call whose result
//! §8 then has to say is never announced.

use windows_sys::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::System::Ole::CF_UNICODETEXT;

/// Puts `text` on the clipboard as Unicode text, owned by `owner`. `false` is
/// a failed write and nothing else — the caller speaks Announcement 14 and
/// does not retry (v0.2.0 §8).
///
/// `owner` must be a real window: `OpenClipboard(nullptr)` leaves
/// `EmptyClipboard` setting the owner to null, which makes the very next
/// `SetClipboardData` fail. The window is the frame, so a copy is owned by the
/// application that made it for as long as that application is up.
///
/// The clipboard is held open across the whole write and closed on every road
/// out, including the failing ones: an application that leaves it open is the
/// application that makes *everybody else's* copy fail, which is precisely the
/// failure this module reports.
pub fn copy(owner: HWND, text: &str) -> bool {
    if owner.is_null() {
        return false;
    }
    // NUL-terminated UTF-16, and the terminator is not optional: the clipboard
    // reads the block until one, not for the length that was allocated. An
    // interior NUL cannot arrive — an Entry is registry text, which the reader
    // already stops at the first NUL of.
    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = std::mem::size_of_val(&utf16[..]);

    // SAFETY: `owner` is a live window handle from wx. A failed open means
    // another process holds the clipboard — the transient case §8 exists to
    // report — and there is then nothing to close.
    if unsafe { OpenClipboard(owner) } == 0 {
        return false;
    }
    // SAFETY: the clipboard is open and this thread holds it.
    unsafe { EmptyClipboard() };

    // SAFETY: a plain allocation; a null answer is out of memory and is
    // handled rather than dereferenced.
    let block: HGLOBAL = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) };
    if block.is_null() {
        // SAFETY: the clipboard is open and this thread holds it.
        unsafe { CloseClipboard() };
        return false;
    }
    // SAFETY: `block` is a live moveable handle, so locking it answers a
    // pointer to at least `bytes` writable bytes — exactly what `utf16`
    // occupies. The two regions belong to different allocations and cannot
    // overlap. The lock is released before the handle is given away.
    unsafe {
        let target = GlobalLock(block);
        if target.is_null() {
            GlobalFree(block);
            CloseClipboard();
            return false;
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), target.cast::<u16>(), utf16.len());
        GlobalUnlock(block);
    }

    // SAFETY: the clipboard is open, this thread owns it, and `block` is a
    // handle it may take. On success the system owns the memory and this code
    // must not free it; on failure it is still ours, and leaking it would be
    // the one way a rare, recoverable failure could cost anything lasting.
    let taken = unsafe { SetClipboardData(CF_UNICODETEXT as u32, block as HANDLE) };
    let copied = !taken.is_null();
    if !copied {
        // SAFETY: the handle was refused, so it is still this code's to free.
        unsafe { GlobalFree(block) };
    }
    // SAFETY: the clipboard is open and this thread holds it. Closed on the
    // failing road as well as the successful one.
    unsafe { CloseClipboard() };
    copied
}
