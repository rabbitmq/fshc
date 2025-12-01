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

//! A test helper binary that opens a file and two sockets, then waits.
//! Used by integration tests to verify fshc can query process descriptors.
//!
//! Outputs its PID on stdout and waits for any input on stdin before exiting.
//! This allows tests to:
//! 1. Start this process
//! 2. Read its PID from stdout
//! 3. Run fshc against that PID
//! 4. Send input to stdin to terminate

use std::fs::File;
use std::io::{self, BufRead};
use std::net::TcpListener;

fn main() {
    // Open one file (will be cleaned up when process exits)
    let _file = File::create(temp_file_path()).expect("Failed to create temp file");

    // Open two TCP listeners (sockets) on ephemeral ports
    let _socket1 = TcpListener::bind("127.0.0.1:0").expect("Failed to bind socket 1");
    let _socket2 = TcpListener::bind("127.0.0.1:0").expect("Failed to bind socket 2");

    // Output our PID so the test can query us
    println!("{}", std::process::id());

    // Wait for input on stdin (blocks until parent sends something or closes stdin)
    let stdin = io::stdin();
    let _ = stdin.lock().lines().next();

    // Clean up the temp file
    let _ = std::fs::remove_file(temp_file_path());
}

fn temp_file_path() -> String {
    format!(
        "{}/fshc_test_{}.tmp",
        std::env::temp_dir().display(),
        std::process::id()
    )
}
