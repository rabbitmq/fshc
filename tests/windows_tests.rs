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

//! Windows-specific CLI tests.
//!
//! Windows handle enumeration differs from Unix file descriptor enumeration:
//! - Non-existent PIDs return success with 0 handles (no error)
//! - Socket descriptors are not reported separately from file descriptors

#![cfg(target_os = "windows")]

mod test_helpers;

use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use test_helpers::{output_includes, run_succeeds, target_process_bin};

#[test]
fn nonexistent_pid_returns_success_with_zero_handles() -> Result<(), Box<dyn Error>> {
    // Windows returns success with 0 handles for non-existent PIDs
    // because NtQuerySystemInformation returns all handles system-wide,
    // and filtering by a non-existent PID simply yields an empty set.
    run_succeeds(["--pid", "99999"])
        .stdout(output_includes("\"pid\":99999"))
        .stdout(output_includes("\"total_descriptors\":0"))
        .stdout(output_includes("\"file_descriptors\":0"));
    Ok(())
}

#[test]
fn query_target_process_returns_expected_json_fields() -> Result<(), Box<dyn Error>> {
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

    assert!(json.get("pid").is_some(), "Output should include pid");
    assert!(
        json.get("total_descriptors").is_some(),
        "Output should include total_descriptors"
    );
    assert!(
        json.get("file_descriptors").is_some(),
        "Output should include file_descriptors"
    );
    assert!(
        json.get("socket_descriptors").is_none(),
        "Output should NOT include socket_descriptors on Windows"
    );

    let total = json["total_descriptors"].as_u64().unwrap_or(0);
    let files = json["file_descriptors"].as_u64().unwrap_or(0);

    assert!(
        files >= 1,
        "Expected at least 1 file descriptor, got {}",
        files
    );
    assert!(total >= 3, "Expected total descriptors >= 3, got {}", total);

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"quit\n");
    }
    let _ = child.wait();

    Ok(())
}
