// Copyright (C) 2024-2025 Broadcom. All Rights Reserved.
// The term "Broadcom" refers to Broadcom Inc. and/or its subsidiaries.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unix-specific CLI tests (macOS, Linux).
//!
//! Unix file descriptor enumeration differs from Windows:
//! - Non-existent PIDs return an error
//! - Socket descriptors are reported separately from file descriptors

#![cfg(unix)]

mod test_helpers;

use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use test_helpers::{output_includes, run_fails, run_succeeds, target_process_bin};

#[test]
fn fail_with_nonexistent_pid() -> Result<(), Box<dyn Error>> {
    run_fails(["--pid", "99999"]);
    Ok(())
}

#[test]
fn query_target_process_includes_socket_descriptors() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new(target_process_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut pid_line = String::new();
    reader.read_line(&mut pid_line)?;
    let target_pid = pid_line.trim();

    let _ = run_succeeds(["--pid", target_pid])
        .stdout(output_includes(&format!("\"pid\":{}", target_pid)))
        .stdout(output_includes("\"socket_descriptors\":"));

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"quit\n");
    }
    let _ = child.wait();

    Ok(())
}

#[test]
fn query_target_process_has_expected_minimum_sockets() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new(target_process_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut pid_line = String::new();
    reader.read_line(&mut pid_line)?;
    let target_pid = pid_line.trim();

    let assert = run_succeeds(["--pid", target_pid]);
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&output)?;

    let sockets = json["socket_descriptors"].as_u64().unwrap_or(0);
    assert!(sockets >= 2, "Expected at least 2 sockets, got {}", sockets);

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"quit\n");
    }
    let _ = child.wait();

    Ok(())
}
