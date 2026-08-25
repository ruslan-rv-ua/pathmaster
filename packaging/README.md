# packaging

The package-manager manifests for PathMaster. Nothing here is built or installed by this
repository — these are the files that are **submitted elsewhere**, kept here so that the
release they describe and the manifests that describe it live in one history.

Both are finalised against the release shape the workflow produces
([.github/workflows/release.yml](../.github/workflows/release.yml)): a bare
`PathMaster-v<version>-x64.exe` and a `<hex64> *<filename>` sidecar beside it, no archive.

## `winget/`

**Deferred indefinitely.** Scoop and direct download are the release channels for now; nothing
below is submitted until that decision is revisited, and the README no longer offers the winget
command. The manifests stay here finished — the identity test named at the bottom still guards
them — so resuming costs only the version and the F5 hash. The checklist's F7-F8 sit in their own
deferred block for the same reason.

Three files, winget manifest schema 1.12.0, submitted as one directory to
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) under
`manifests/r/RuslanIskov/PathMaster/<version>/`.

`InstallerType: portable`, so winget copies the exe rather than running an installer, writes an
Add/Remove Programs key under `HKCU`, and puts a symlink named by `Commands` in its Links
directory (which is on the user `PATH`). The user-facing half of that sentence was a bullet in the
README's *What gets written where* section, removed with the deferral; F8's row in the checklist
says to restore it when the submission is taken up.

`InstallerSha256` is a placeholder until a release exists to hash: the hash comes from step **F5**
of the [release checklist](../docs/release-checklist.md) and the submission is **F7**.

## `scoop/`

One manifest for the own bucket at
[ruslan-rv-ua/scoop-bucket](https://github.com/ruslan-rv-ua/scoop-bucket). The copy here is the
source of truth for what is **first placed** in the bucket, not a second manifest scoop reads.

**How it gets bumped.** The bucket runs no excavator. It takes a `repository_dispatch` named
`update-<app>` from the application's own repository, verifies the URL and hash it is given by
re-downloading, rewrites `bucket/<app>.json` and pushes; its CI then validates the manifest and
reverts the push if the structure is wrong. Our side of that is
[.github/workflows/update-scoop.yml](../.github/workflows/update-scoop.yml), started by hand at
release checklist step **F9** — the same shape the bucket's other five applications use.

So the sequence is: **F9 places this file in the bucket once, by hand**, with the hash from F5;
every release after that, the workflow bumps it. The bucket's dispatch handler fails on a missing
`bucket/<app>.json` rather than creating one, which is why the first placement cannot be
automated away.

`checkver` and `autoupdate` in the manifest are kept and correct, but nothing in this setup runs
them on a schedule — they serve `scoop update` and would be what an excavator read, if one were
ever added.

## Keeping the two honest

`Publisher` and `PackageName` in the winget locale manifest must equal the exe's `CompanyName`
and `ProductName`: for a portable install winget writes the ARP entry from the manifest, and for
an unsigned binary that agreement is the whole of its identity. The Rust half is gated by
`the_versioninfo_carries_the_identity_the_package_managers_were_built_from` in
[crates/pathmaster-core/tests/versioninfo.rs](../crates/pathmaster-core/tests/versioninfo.rs); the
manifest half is a
review step, because these files leave the repository.
