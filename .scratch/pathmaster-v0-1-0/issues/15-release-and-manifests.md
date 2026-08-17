# Release and package manifests

Type: research
Status: open
Blocked by: 04, 07

## Question

What does the release pipeline look like, and what exactly goes in the scoop and winget manifests?

Settled at charting: GitHub Releases built by GitHub Actions on `windows-latest`; **unsigned** in v0.1.0 with
the SmartScreen warning documented in the README; scoop via an **own bucket** with `persist: data`; winget
submitted to `microsoft/winget-pkgs` with `InstallerType: portable`.

Facts to establish and drafts to produce:

- **Name availability.** Is `PathMaster` free in winget and in the main scoop buckets? Which
  `PackageIdentifier` (`<Publisher>.PathMaster`) is available, and what Publisher name should be used?
- **winget portable specifics.** `PortableCommandAlias`, the Links directory it adds **to the user's PATH**
  (an irony worth documenting in a PATH editor), what winget itself writes to the registry — it does, and the
  README must not claim otherwise — and whether `data/` survives `winget upgrade`.
- **scoop specifics.** Manifest fields, `persist: data`, `checkver` / `autoupdate`, and how the shim behaves
  for a GUI executable.
- **Workflow.** Build, artifact naming (`PathMaster-v0.1.0-x64.exe`), SHA256 publication, tag → release, and
  how each manifest gets bumped on release.
- **Archive or bare exe?** Whether the release should also ship a zip, and which installer type each package
  manager is happiest with.

Output: draft manifests and workflow file in `../research/15-packaging/`, ready to lift into the
implementation effort unchanged.
