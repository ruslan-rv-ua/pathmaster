# packaging

The package-manager manifests for PathMaster. Nothing here is built or installed by this
repository — these are the files that are **submitted elsewhere**, kept here so that the
release they describe and the manifests that describe it live in one history.

Both are finalised against the release shape the workflow produces
([.github/workflows/release.yml](../.github/workflows/release.yml)): a bare
`PathMaster-v<version>-x64.exe` and a `<hex64> *<filename>` sidecar beside it, no archive.

## `winget/`

Three files, winget manifest schema 1.12.0, submitted as one directory to
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) under
`manifests/r/RuslanIskov/PathMaster/<version>/`.

`InstallerType: portable`, so winget copies the exe rather than running an installer, writes an
Add/Remove Programs key under `HKCU`, and puts a symlink named by `Commands` in its Links
directory (which is on the user `PATH`). The README's *What gets written where* section is the
user-facing half of that sentence.

`InstallerSha256` is a placeholder until a release exists to hash. Filling it in and submitting
is step F3 of the [release checklist](../docs/release-checklist.md).

## `scoop/`

One manifest for the own bucket at
[ruslan-rv-ua/scoop-bucket](https://github.com/ruslan-rv-ua/scoop-bucket), which is generated
from `ScoopInstaller/BucketTemplate` — its `excavator.yml` runs `checkver`/`autoupdate` on a
schedule, so after the first submission this file is bumped by the bucket rather than by hand.
The copy here is the source of truth for what is put in the bucket, not a second manifest scoop
reads.

## Keeping the two honest

`Publisher` and `PackageName` in the winget locale manifest must equal the exe's `CompanyName`
and `ProductName`: for a portable install winget writes the ARP entry from the manifest, and for
an unsigned binary that agreement is the whole of its identity. The Rust half is gated by
`the_versioninfo_carries_the_identity_the_package_managers_were_built_from` in
[crates/pathmaster/src/version.rs](../crates/pathmaster/src/version.rs); the manifest half is a
review step, because these files leave the repository.
