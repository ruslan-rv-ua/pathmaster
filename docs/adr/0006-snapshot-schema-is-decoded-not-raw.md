# A Snapshot stores decoded Entries and Value Type, never raw registry bytes

The entire point of taking a Snapshot before Apply is that it can undo a bad write. Ticket 05 named the
failure mode directly (hazard H15): a backup that cannot reproduce what was actually in the registry makes
every other corruption mode unrecoverable, which defeats the reason a Snapshot exists at all. So the shape of
the file was not a formatting detail — it decided whether Restore can ever be trusted.

Two shapes were open. The first mirrors the registry exactly: the Value Type tag plus the raw byte buffer
`RegQueryValueExW` returns. The second records what an Editing Session already models: the Value Type plus the
list of Entries, exactly as read — a raw substring between `;` separators, with nothing parsed out of it
(`CONTEXT.md`, **Entry**). The raw-bytes shape is the more literal restore source, but it also stops being a
JSON file a person can open and read, which is the reason JSON was chosen for Snapshots at all (ticket 07's
rationale for the whole `data\` layout is an inspectable, diffable file structure). It was rejected because it
buys nothing: a registry `PATH` value has no structure beyond its Value Type and its `;`-separated Entries, and
an Entry is already defined to survive a round trip losslessly. `valueType` + `entries` and "the raw byte
buffer" encode the identical information — one of them is also readable.

## Decision

A Snapshot is a JSON file recording:

- `timestamp`, `scope` — as already specified by the PRD.
- `valueType` — `REG_SZ` or `REG_EXPAND_SZ`, carried alongside the entries rather than assumed.
- either `entries: [...]` (the Scope's Entries, decoded strings, in order) or `absent: true` — a Scope that did
  not exist is a distinct, representable state, not inferred from a missing file.

Restoring loads a Snapshot into the Working Copy (see ticket 14) rather than writing the registry directly, so
"is this Snapshot faithful" only has to mean "does it reproduce the Working Copy Apply would have written" —
not "does it reproduce a byte buffer no other part of the application ever touches."

## Consequences

- **Hazard H15 is closed.** A Snapshot now reproduces the exact Value Type and Entry content it captured;
  nothing about the restored value is guessed.
- **Snapshot files stay human-readable JSON** — openable, diffable, greppable — which is what the PRD wanted
  a backup format to be in the first place.
- **Absent and zero-Entries are distinct, representable states**, matching the Working Copy's own model
  (`CONTEXT.md`, **Absent**) rather than collapsing them into "no file" or "empty array" by accident.
- **This schema cannot represent a value that fails to round-trip through decode/split/join.** Not observed for
  `PATH` — a `REG_SZ`/`REG_EXPAND_SZ` payload is well-formed UTF-16 text by definition of the type — and
  accepted rather than defended against, because nothing else in the application handles raw bytes either.
