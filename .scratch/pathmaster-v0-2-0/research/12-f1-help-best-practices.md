# Research: F1 and Help → Documentation — the offline question

Supporting ticket [12-f1-help-documentation](../issues/12-f1-help-documentation.md).
Researched 2026-08-26, per the map's standing directive 7 (research before grilling).

## 1. What F1 is actually reserved for

F1 is the one shortcut Windows itself defines for the *application*, not for the system: "the key
opens the help for the operating system or the active running program"
([Computer Hope](https://www.computerhope.com/jargon/f/f1.htm)), and Microsoft's own keyboard design
guidance treats the function keys as first-class shortcut real estate
([Guidelines for Keyboard User Interface Design](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/dnacc/guidelines-for-keyboard-user-interface-design)).
Where an application defines nothing, the shell answers instead: on the desktop and in Explorer,
Windows 10/11 send F1 to a **Bing search for "how to get help in Windows"**
([Winhelponline](https://www.winhelponline.com/blog/disable-f1-key-help-windows-10/),
[Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/1659967/pressing-f-number-keys-opens-tabs-such-as-pressing)).

Two consequences for this ticket:

- **F1 → About is off-convention.** Nothing in the literature binds F1 to identity; it binds it to
  *documentation*. An About dialog answers "what am I running", which is a different question and
  already has its own item. The ticket's fourth option is the cheapest, and it is the one a user
  arriving from every other Windows application will read as broken.
- **Doing nothing is not neutral.** F1 unhandled in a focused window is simply swallowed — this
  application is not the desktop, so the Bing fallback does not fire — but the user learns nothing
  either way. "F1 does nothing" and "F1 is not implemented" are indistinguishable to a screen-reader
  user, which is the exact failure the Announcement mechanism exists to prevent elsewhere.

**Context-sensitive help** is the older half of the convention: F1 on a focused dialog or control was
meant to open a topic *for that thing* (`WM_HELP`, the title-bar `?` button,
[MFC TN028](https://learn.microsoft.com/en-us/cpp/mfc/tn028-context-sensitive-help-support?view=msvc-170),
[Guidelines for Creating a Context-Sensitive Help File](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/htmlhelp/guidelines-for-creating-a-context-sensitive-help-file)).
That machinery is tied to the CHM stack below and is effectively dead in new applications; the
surviving practice is F1 → one document, from anywhere.

## 2. The offline-help format landscape, 2026

| Format | Status | Fatal to this project? |
|---|---|---|
| **CHM** (`.chm`) | Viewer still ships in Windows 11; **HTML Help Workshop is discontinued and unmaintained** ([Wikipedia](https://en.wikipedia.org/wiki/Microsoft_Compiled_HTML_Help), [Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/265752/htmlhelp-workshop-download-for-chm-compiler-instal)) | **Yes, twice over** — see below |
| **eWriter** (`.ewriter`) | Modern CHM replacement, but the reader is a **Microsoft Store app the user is prompted to install** ([Help+Manual](https://www.helpandmanual.com/help/hm_ewriter_compared.html)) | Yes — a help file that first requires a download is not offline help |
| **MSHC** | Visual Studio's format; needs a dedicated viewer ([10Tec](https://10tec.com/help-viewer/alternative-way-view-chm.aspx)) | Yes — same reason |
| **HTML** (`.html`, local file) | Universal handler on every Windows install; opens in the default browser | No |
| **Markdown** (`.md`) | **Windows does not register `.md` out of the box** — a clean install has no handler and shows the "How do you want to open this file?" picker ([Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/5788351/md-file-type), [MDHero](https://mdhero.app/blogs/open-md-windows/)); newer Notepad builds render Markdown, but only where the association was set | Effectively yes for shell-open |
| **Plain text in-app** | No format risk at all | No, but see §4 |

**CHM's two killers.** First, the compiler is gone: adding a `.chm` means adding an unmaintained,
undownloadable build dependency to a pipeline that currently gates its own artifact. Second, and
worse for a *portable* application, CHM is the one format Windows security actively breaks:

> a CHM file consists of HTML code and may contain scripts, [so] it can be considered by Windows
> security policy as a potentially harmful object

A `.chm` carrying the Mark of the Web — downloaded from GitHub, or opened **from a network share or
a UNC path** — renders as "Navigation to the webpage was canceled" until the user finds the
**Unblock** checkbox in file Properties
([HelpSmith](https://blog.helpsmith.com/2015/08/14/why-my-chm-help-file-is-not-displayed-correctly/),
[Microsoft Learn](https://learn.microsoft.com/en-us/troubleshoot/windows-client/shell-experience/dot-chm-file-not-render-properly),
[HelpSmith network note](https://www.helpsmith.com/webhelp/topics/warning-chm-shared-location.htm)).
"Carried on a stick, run from a share" is this application's stated home ground, so the format whose
documented failure mode is exactly that is the wrong one.

**Local HTML has no equivalent block.** MOTW on an `.html` file makes the browser treat it as
internet-zone content; it still renders. There is no unblock prompt and no viewer to install.

## 3. What the target audience's own tool does — the strongest precedent

NVDA is itself a screen-reader-first Windows application, and its Help submenu is:

> From this submenu you can access the User Guide, a quick reference of commands, history of new
> features and more. **These first three options open in the default web browser.**
> — [NVDA User Guide](https://download.nvaccess.org/documentation/userGuide.html)

Reached by `NVDA+N`, then `H`, then Enter ([NV Access](https://www.nvaccess.org/get-help/)). The
documents are **local HTML files shipped with the program**, translated as whole documents per
language (NVDA ships in 56 languages, Ukrainian among them); the same files are also published
online at `download.nvaccess.org`. So the pattern that the intended user already has in their hands
is: *help is a local HTML document, opened in the browser, translated as a document.*

Three things follow that are worth taking as given rather than re-deriving:

- **The browser is the help viewer.** No screen-reader-first application on Windows builds its own.
- **Documents are translated as documents**, not through the message catalogue. NVDA does not put its
  user guide in a `.po` file, and neither should this — the Catalogue holds interface strings
  (ADR-0004), and a long document in it would be one msgid per paragraph with a completeness gate
  that cannot meaningfully pass.
- **Online and offline are the same document**, published in two places, not two different artifacts.

## 4. Why the browser, specifically — the accessibility half

A browser gives a screen-reader user **browse mode**: heading navigation (`H`), a headings list,
find, links, and say-all over a semantically structured document. The generic advice is the same —
headings exist so screen-reader users "skip between headings to locate specific information" rather
than reading a wall of text line by line
([Google developer documentation style guide](https://developers.google.com/style/accessibility),
[WebAIM](https://webaim.org/techniques/screenreader/)).

Against that, the two in-app options:

- **A read-only multiline `TextCtrl`.** Native `EDIT`, so NVDA reads it perfectly *as text* — arrow
  keys, say-all — but it is flat: no headings, no links, no find beyond the app's own. For a document
  the size of the README this is the difference between navigating and scrolling.
- **`wxHtmlWindow`.** Would render the document, but it is a **generic wx-drawn control**, not a
  native one — the same class of surface ticket 01 flagged for `SearchCtrl`. It exposes no native
  text to the screen reader. This is unusable here and should be ruled out by name, not left as an
  option someone re-proposes later.

## 5. Project-internal constraints that pre-decide most of this

These are not preferences; they are already-settled rules that eliminate options before the grilling
starts.

1. **The label *is* the binding.** wxdragon binds no `wxAcceleratorTable` at any level, so `F1`
   exists only as a `"\tF1"` suffix appended in code to a menu item's translated label
   (ADR-0004, spec §11, §15). F1 therefore *must* have a menu item — the ticket's premise is forced
   by the architecture, not a style choice.
2. **The shell hatch already exists.** `pathmaster-platform` already calls `ShellExecuteW` with the
   `open` verb for Tools → Open Backups Folder
   ([snapshots.rs](../../../crates/pathmaster-platform/src/snapshots.rs)), and its documented failure
   posture is *silence*: "A shell that will not open it is silence. There is no Announcement for it".
   The same call opens an `https://` URL or a local `.html` path with no new dependency and no new
   wxdragon binding — which matters, because **wxdragon exposes no `wxLaunchDefaultBrowser` and no
   help controller at all** (checked against the crate's public API).
3. **The Data Directory is the only place this application writes.** `CONTEXT.md`: "Nothing the
   application writes lives anywhere else", and the README makes it a public promise ("The
   application writes to two places and nowhere else"). **A generated help file in `%TEMP%` breaks a
   documented invariant** — that option is not merely inelegant, it contradicts a shipped claim.
4. **"One executable" is a shipped promise too.** README Features: "**Portable.** One executable."
   A second file beside the exe contradicts it as literally as the temp file contradicts (3); a file
   *generated into `data\`* does not, since `data\` is already the documented write target.
5. **`data\` is persisted across upgrades by scoop** (`persist: "data"`, a junction — spec §16). A
   help document written there **survives `scoop update` and goes stale**, unless it is version-stamped
   and rewritten when the version changes.
6. **Read-only Data runs cannot write at all** (spec §3, §13) — so any generate-then-open scheme needs
   a named answer for the run where generation fails, which is also the run where the user is most
   likely to be looking for help.
7. **Both languages already exist as documents.** [README.md](../../../README.md) and
   [README.uk.md](../../../README.uk.md) are complete, maintained, and mirror each other — including
   a Keyboard table that spec §15 says "the README table mirrors". Whatever ships or is linked, the
   Ukrainian half is not new work.
8. **`…` has a fixed meaning here**: it marks the items that open a dialog *asking* something
   (msgids.rs on `MENU_ABOUT`). A Documentation item hands off to the browser and asks nothing, so by
   the house rule it takes **no** ellipsis — the same reasoning that left About and Restart as
   Administrator bare.
9. **Help's mnemonic space is nearly empty**: the menu holds `&About` alone, so a second item collides
   only on `A`. The bar's own mnemonics are F, E, T, H and ticket 02 adds View (V).

## 6. The four shapes, with their failure stories

| Shape | What F1 does | Failure story | Costs |
|---|---|---|---|
| **A. Online URL** | `ShellExecuteW` on the README's GitHub URL | Browser **opens** and shows its own offline page — visible, not silent, but useless. On a machine with no browser at all: silence. | None beyond the URL. Doc can be fixed after release without a build. |
| **B. Ship a document beside the exe** | Shell-open `PathMaster.html` (or `.md`) next to the exe | File deleted or left behind when the exe was copied alone → shell-open of a missing path is **silent**. `.md` additionally hits the no-handler picker. | Ends "one executable". Two files to publish, hash and keep in step. |
| **C. Embed in the exe, write to `data\`, open** | `include_bytes!` the HTML (the `.mo` pattern already in the build), write `data\help-<lang>.html` if absent/stale, shell-open it | Read-only Data run → cannot write → needs a fallback (realistically A). Stale copy across scoop upgrades unless version-stamped. | New write path, a staleness rule, one more thing the release checklist must verify. Keeps "one executable" **and** works offline. |
| **D. In-app dialog** | Modal with a read-only native multiline text | No failure mode at all — it cannot fail. | Flat text, no headings/links/find; the document must then live in the Catalogue or as an embedded blob; a long dialog is the worst reading surface of the four. |

**Not on the list, deliberately**: CHM (§2), eWriter/MSHC (§2), `wxHtmlWindow` (§4), and `%TEMP%`
(§5.3).

A fifth shape exists that the ticket did not name: **A + B together** — ship nothing, but have the
menu item point at a *version-pinned* URL (`/blob/v0.2.0/README.md`), so an old binary opens the
documentation it was built against rather than whatever `main` says today. The About dialog already
carries the version that would fill it, and §16's three-way version gate already keeps that number
honest.

## 7. F1 inside dialogs

The old convention says F1 in a dialog opens a topic *about that dialog* (§1). Modern practice in
small applications is either nothing at all or the same single document.

For this application the answer is close to forced: a modal dialog **has no menu bar**, so an F1 there
could not have a menu home — and "every shortcut has a menu home" is the rule ADR-0004 exists to
enforce. Wiring F1 in dialogs would either create the one shortcut in the application that no menu
names, or require a Help button in every dialog (which §15's "our own buttons" discipline would then
have to carry through nine dialogs and their Catalogue strings). The cheap, consistent answer is that
F1 belongs to the frame; the expensive one buys context-sensitivity this application's help does not
have topics for.

Whichever is chosen, the ticket is right that it must be **stated**, because the Release Checklist can
only test a written expectation.

## 8. Sources

- [Computer Hope — F1](https://www.computerhope.com/jargon/f/f1.htm) · [Winhelponline — F1 opens Bing help](https://www.winhelponline.com/blog/disable-f1-key-help-windows-10/) · [Microsoft Q&A — F1 opens "how to get help in Windows 11"](https://learn.microsoft.com/en-us/answers/questions/1659967/pressing-f-number-keys-opens-tabs-such-as-pressing)
- [Microsoft — Guidelines for Keyboard User Interface Design](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/dnacc/guidelines-for-keyboard-user-interface-design) · [MFC TN028 — Context-Sensitive Help Support](https://learn.microsoft.com/en-us/cpp/mfc/tn028-context-sensitive-help-support?view=msvc-170) · [Guidelines for Creating a Context-Sensitive Help File](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/htmlhelp/guidelines-for-creating-a-context-sensitive-help-file)
- [Wikipedia — Microsoft Compiled HTML Help](https://en.wikipedia.org/wiki/Microsoft_Compiled_HTML_Help) · [Microsoft Q&A — HTML Help Workshop download failed](https://learn.microsoft.com/en-us/answers/questions/265752/htmlhelp-workshop-download-for-chm-compiler-instal) · [HelpSmith — why a CHM is not displayed](https://blog.helpsmith.com/2015/08/14/why-my-chm-help-file-is-not-displayed-correctly/) · [Microsoft Learn — .chm not rendering](https://learn.microsoft.com/en-us/troubleshoot/windows-client/shell-experience/dot-chm-file-not-render-properly) · [HelpSmith — CHM on a network share](https://www.helpsmith.com/webhelp/topics/warning-chm-shared-location.htm) · [Help+Manual — eWriter vs CHM](https://www.helpandmanual.com/help/hm_ewriter_compared.html) · [10Tec — MSHC viewer](https://10tec.com/help-viewer/alternative-way-view-chm.aspx)
- [Microsoft Q&A — .md file type has no default handler](https://learn.microsoft.com/en-us/answers/questions/5788351/md-file-type) · [MDHero — opening .md on Windows](https://mdhero.app/blogs/open-md-windows/)
- [NVDA User Guide](https://download.nvaccess.org/documentation/userGuide.html) · [NV Access — Get Help](https://www.nvaccess.org/get-help/)
- [Google developer documentation style guide — accessibility](https://developers.google.com/style/accessibility) · [WebAIM — Designing for Screen Reader Compatibility](https://webaim.org/techniques/screenreader/)
