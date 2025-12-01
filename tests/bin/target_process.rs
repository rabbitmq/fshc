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

//! Test helper binary that opens 1 file and 2 sockets.
//! Outputs PID on stdout, waits for stdin input, then exits.

use std::fs::File;
use std::io::{self, BufRead};
use std::net::TcpListener;

fn main() {
    let _file = File::create(temp_file_path()).expect("Failed to create temp file");
    let _socket1 = TcpListener::bind("127.0.0.1:0").expect("Failed to bind socket 1");
    let _socket2 = TcpListener::bind("127.0.0.1:0").expect("Failed to bind socket 2");

    println!("{}", std::process::id());

    let stdin = io::stdin();
    let _ = stdin.lock().lines().next();

    let _ = std::fs::remove_file(temp_file_path());
}

fn temp_file_path() -> String {
    format!(
        "{}/fshc_test_{}.tmp",
        std::env::temp_dir().display(),
        std::process::id()
    )
}
