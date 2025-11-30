# AI Agent Instructions for fshc

This document provides guidance for AI coding assistants working on this codebase.

## Project Overview

**fshc** (File and Socket Handle Counter) is a small, cross-platform CLI tool that counts file and socket descriptors for a process.
It outputs JSON for machine consumption.


## Repository Structure

```
fshc/
├── src/
│   ├── main.rs           # Entry point, CLI parsing with clap
│   ├── outcome.rs        # Error types and result handling
│   ├── fds.rs            # Platform-agnostic FD enumeration (Linux, macOS)
│   └── fds/
│       └── windows.rs    # Windows-specific implementation
├── scripts/              # Nu shell release scripts
├── wix/                  # Windows installer configuration
├── .github/
│   ├── workflows/
│   │   ├── ci.yml        # Build and test workflow
│   │   └── release.yml   # Release build workflow
│   └── dependabot.yml    # Dependency updates
└── .config/
    └── nextest.toml      # Test runner configuration
```

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point using clap derive macros |
| `src/outcome.rs` | Custom error type `FshcError` with exit code mapping |
| `src/fds.rs` | `FdList` struct with platform-specific implementations |
| `src/fds/windows.rs` | Windows FFI using `windows-sys` crate |

## Build and Test Commands

```shell
# Build
cargo build

# Build release
cargo build --release

# Run tests
cargo nextest run

# Lint
cargo clippy --all-targets

# Format
cargo fmt --all
```

## Architecture Notes

### Platform Abstraction

The codebase uses conditional compilation (`#[cfg]`) to provide platform-specific implementations:

 * Linux: `procfs` to read `/proc/<pid>/fd`
 * macOS: `libproc` and the BSD API
 * Windows: `windows-sys` and `NtQuerySystemInformation`

All platforms expose the same public interface via `FdList`:

 * `FdList::list_by_type(pid)`: returns counts by type (file vs socket)
 * `FdList::list_total(pid)`: returns only total count

## Dependencies

Avoid adding heavy dependencies. This tool should remain lightweight.

## Code Style

 * Use `cargo fmt` before committing
 * Ensure `cargo clippy` passes with no warnings
 * Keep the codebase minimal: this is intentionally a small, focused tool
 * Preserve platform abstraction patterns when adding features
 * Use conditional compilation for platform-specific code

## Comments

 * Only add very important comments, both in tests and in the implementation

## Git Instructions

 * Never add yourself to the list of commit co-authors

## Style Guide

 * Never add full stops to Markdown list items
