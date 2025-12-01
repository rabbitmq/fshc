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

mod test_helpers;

#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use test_helpers::{output_includes, run_fails, run_succeeds};

#[allow(deprecated)]
fn target_process_bin() -> std::path::PathBuf {
    cargo_bin("target_process")
}

#[test]
fn show_help_with_help_flag() -> Result<(), Box<dyn Error>> {
    run_succeeds(["--help"]).stdout(output_includes("Usage:"));
    Ok(())
}

#[test]
fn show_version_with_version_flag() -> Result<(), Box<dyn Error>> {
    run_succeeds(["--version"]).stdout(output_includes("fshc"));
    Ok(())
}

#[test]
fn fail_without_pid_argument() -> Result<(), Box<dyn Error>> {
    let args: [&str; 0] = [];
    run_fails(args).stderr(output_includes("--pid"));
    Ok(())
}

#[test]
fn fail_with_invalid_pid_zero() -> Result<(), Box<dyn Error>> {
    run_fails(["--pid", "0"]).stderr(output_includes("only pid numbers between 1 and 99999"));
    Ok(())
}

#[test]
fn fail_with_invalid_pid_negative() -> Result<(), Box<dyn Error>> {
    run_fails(["--pid", "-1"]);
    Ok(())
}

#[test]
fn fail_with_invalid_pid_too_large() -> Result<(), Box<dyn Error>> {
    run_fails(["--pid", "100000"]).stderr(output_includes("only pid numbers between 1 and 99999"));
    Ok(())
}

#[test]
fn fail_with_nonexistent_pid() -> Result<(), Box<dyn Error>> {
    // PID 99999 is very unlikely to exist
    run_fails(["--pid", "99999"]);
    Ok(())
}

#[test]
fn query_target_process_returns_json_with_descriptors() -> Result<(), Box<dyn Error>> {
    // Start the target process that opens 1 file + 2 sockets
    let mut child = Command::new(target_process_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Read the PID from the target process's stdout
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut pid_line = String::new();
    reader.read_line(&mut pid_line)?;
    let target_pid = pid_line.trim();

    // Query the target process with fshc
    run_succeeds(["--pid", target_pid])
        .stdout(output_includes(&format!("\"pid\":{}", target_pid)))
        .stdout(output_includes("\"total_descriptors\":"))
        .stdout(output_includes("\"socket_descriptors\":"))
        .stdout(output_includes("\"file_descriptors\":"));

    // Clean up: send input to terminate the target process
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"quit\n");
    }
    let _ = child.wait();

    Ok(())
}

#[test]
fn query_target_process_with_only_total_flag() -> Result<(), Box<dyn Error>> {
    // Start the target process
    let mut child = Command::new(target_process_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Read the PID
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut pid_line = String::new();
    reader.read_line(&mut pid_line)?;
    let target_pid = pid_line.trim();

    // Query with --only-total flag
    let assert = run_succeeds(["--pid", target_pid, "--only-total"]);
    let output = String::from_utf8_lossy(&assert.get_output().stdout);

    // Should have pid and total_descriptors
    assert!(output.contains(&format!("\"pid\":{}", target_pid)));
    assert!(output.contains("\"total_descriptors\":"));

    // Should NOT have socket_descriptors or file_descriptors breakdown
    assert!(!output.contains("\"socket_descriptors\":"));
    assert!(!output.contains("\"file_descriptors\":"));

    // Clean up
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"quit\n");
    }
    let _ = child.wait();

    Ok(())
}

#[test]
fn query_target_process_has_expected_minimum_descriptors() -> Result<(), Box<dyn Error>> {
    // Start the target process that opens 1 file + 2 sockets
    let mut child = Command::new(target_process_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Read the PID
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut pid_line = String::new();
    reader.read_line(&mut pid_line)?;
    let target_pid = pid_line.trim();

    // Query and parse the JSON output
    let assert = run_succeeds(["--pid", target_pid]);
    let output = String::from_utf8_lossy(&assert.get_output().stdout);

    // Parse the JSON to verify minimum descriptor counts
    let json: serde_json::Value = serde_json::from_str(&output)?;

    let total = json["total_descriptors"].as_u64().unwrap_or(0);
    let sockets = json["socket_descriptors"].as_u64().unwrap_or(0);
    let files = json["file_descriptors"].as_u64().unwrap_or(0);

    // The target process opens:
    // - 1 file explicitly
    // - 2 sockets explicitly
    // - Plus stdin, stdout, stderr (3 file descriptors)
    // So we expect at least 2 sockets and at least 1 file (temp file)
    assert!(
        sockets >= 2,
        "Expected at least 2 sockets, got {}",
        sockets
    );
    // Files include stdin/stdout/stderr plus our temp file
    // On some platforms, the count might differ, so we just check > 0
    assert!(files >= 1, "Expected at least 1 file descriptor, got {}", files);
    assert!(
        total >= 3,
        "Expected total descriptors >= 3, got {}",
        total
    );

    // Clean up
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"quit\n");
    }
    let _ = child.wait();

    Ok(())
}
