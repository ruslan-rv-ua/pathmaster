# 09 — Diagnostics rules (core)

**Spec:** [spec §7](../../pathmaster-v0-1-0/spec.md) · research/13

**What to build:** The complete diagnostic rulebook in `pathmaster-core`, verified by `cargo test`: Normalisation, the six Issue types, coexistence and severity ordering, and the merged-length threshold logic. Issues are a derived view of the Working Copies — never part of them, excluded from Checkpoints. (The async pass, Status column, and StatusBar wiring are ticket 12.)

**Blocked by:** 02 — operates on the core Entry/Working Copy types.

**Status:** ready-for-agent

- [ ] Split rule: the raw value splits on every `;`; quotes never protect a separator
- [ ] Normalisation is comparison-only, never stored, never written, never touches the filesystem: strip one pair of surrounding `"` → expand `%VAR%` (unknown names stay literal) → `/`→`\` → trim trailing `\` unless that leaves a bare root (`C:\` stays) → compare ordinal case-insensitively; property test: Normalisation idempotence
- [ ] `Duplicate`: equal Normalisations; evaluation order is runtime order — System Working Copy first, then User, left to right; first occurrence canonical and clean, every later copy flags, cross-scope included (the User copy carries it)
- [ ] `Missing`: local-rooted Entries only (root classified via drive type / UNC prefix, no network round trip); flags when the quote-stripped expanded path does not name an existing directory (not-found and is-a-file both flag; access-denied does not); network-rooted Entries are never probed and never flag; an undefined `%VAR%` flags naturally (the filesystem probe itself is injected/adapted so the rules stay unit-testable)
- [ ] `Relative`: any Entry not fully qualified (qualified: `X:\…`, `\\server\share…`, `\\?\…`; flagged: `.`, `..`, bare names, rooted `\foo`, drive-relative `C:foo`); Relative Entries skip the existence check
- [ ] `Empty`: zero-length or whitespace-only Entry; an Absent or empty Scope reports nothing; a trailing `;` produces a genuine empty Entry and does flag
- [ ] `Quoted`: any Entry containing `"`
- [ ] Coexistence: Empty is exclusive; Relative and Missing never co-occur; Quoted co-occurs freely; severity order Missing > Relative > Quoted > Duplicate > Empty
- [ ] Over-length is scope-level, never per-entry: merged length = `len(expand(System WC) + ";" + expand(User WC))` in UTF-16 code units; threshold logic for 8,191 (warn) and 32,767 (hard cap) unit-tested
