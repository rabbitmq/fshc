# fshc Changelog

## v1.5.0 (in development)

No changes yet


## v1.4.0 (Feb 2, 2026)

### Enhancements

 * Use mimalloc allocator for faster musl builds
 * Disable stack unwinding on termination for faster exit
 * A major release workflow revision using [`michaelklishin/rust-release-action`](https://github.com/michaelklishin/rust-release-action)

### Dependency Updates

 * `sysexits` upgraded to `0.11.0`
 * `thiserror` upgraded to `2.0.18`
 * `clap` upgraded to `4.5.54`
 * `serde_json` upgraded to `1.0.149`
 * `windows-sys` upgraded to `0.61.x`


## v1.3.0 (Jan 20, 2025)

### Enhancements

 * Integration tests for all platforms
 * Windows build fixes and additional tests

### Dependency Updates

 * `procfs` upgraded to `0.18.0`
 * `libproc` upgraded to `0.14.11`
 * `serde` upgraded to `1.0.228`
 * `serde_json` upgraded to `1.0.145`
 * `clap` upgraded to `4.5.53`


## v1.2.0 (Nov 16, 2024)

### Enhancements

 * Platform-specific release scripts (Linux, macOS, Windows)
 * Windows MSI packaging support via `cargo-wix`
 * CLI help now links to GitHub repository

### Bug Fixes

 * Pinned `windows-sys` to `0.48.x` to avoid breaking API changes


## v1.1.0 (Oct 15, 2024)

### Enhancements

 * New `--only-total` flag to report just the total handle count
 * Split `FdList` interface into `list_by_type` and `list_total` for efficiency
 * Windows: use `GetProcessHandleCount` for faster total count queries
 * Release workflow with multi-platform support


## v1.0.2 (Sep 26, 2024)

### Enhancements

 * Initial public release
 * Support for Linux, macOS, and Windows
 * Windows implementation contributed by @the-mikedavis
 * JSON output for machine consumption
 * CI with clippy and formatter checks
