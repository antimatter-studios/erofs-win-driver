# Changelog

Notable changes to `erofs-win-driver`, newest first. Sections for releases before this
file existed were drawn from the commits between their tags; from here on each
release's section is written before it is tagged, and the GitHub release's
notes are that section (rust-fs-core's `release-notes`).

## [Unreleased]

### Added

- Erofs probe, mount, and overlay backend.
- WiX MSI bundle and Mount-Erofs helper.
- Integrate fs-test-harness v3.5.0.

### Fixed

- **A directory the reader cannot list is an I/O error, never an empty folder.** A directory whose contents, or one of whose children, could not be read was shown to Windows as empty or short; the listing now fails with `STATUS_IO_DEVICE_ERROR`, or `STATUS_FILE_CORRUPT_ERROR` for corrupt metadata (including a bad directory block, which was reported as "not found").
- **A rebuild that cannot read the underlay fails.** `--scratch-rebuild` left out any subtree it could not read and reported success; it now fails and names the path.
- Use winfsp_sys FILE_FLAGS_AND_ATTRIBUTES; patch system fsctl.h.
- Cover Hit(OverlayEntry::Deleted) match arm.
- Rename Ext4 -> Erofs throughout WiX sources.

### Changed

- Initialize Cargo project.
- Overlay integration coverage.
- Winget package manifests.
- Release workflow.
- Project README.
- Vendor build deps under vendor/, add service feature and [[bin]] erofs.
- Silent-install verification + project hygiene parity with ext4-win-driver.
- Bump rust-fs-erofs to b2527f6 (fsck.erofs-clean mkfs).
- Siblings for the family, submodule for the fork, and CI to prove it ([#1](https://github.com/antimatter-studios/erofs-win-driver/pull/1)).
- Use the shared time and path conversions ([#2](https://github.com/antimatter-studios/erofs-win-driver/pull/2)).
- No submodules left; chore orchestrates every dependency ([#3](https://github.com/antimatter-studios/erofs-win-driver/pull/3)).
- Move to winfsp-rs 0.13.0, with the portability fix ([#4](https://github.com/antimatter-studios/erofs-win-driver/pull/4)).
- A release is tested installed before it is published, and winget follows ([#7](https://github.com/antimatter-studios/erofs-win-driver/pull/7)).
- The driver builds on rust-fs-erofs 0.4 and rust-fs-core 0.3 ([#8](https://github.com/antimatter-studios/erofs-win-driver/pull/8)).
- Wip.


