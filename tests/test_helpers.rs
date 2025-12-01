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

#![allow(dead_code)]

use assert_cmd::assert::Assert;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::predicate;
use predicates::str::ContainsPredicate;
use std::ffi::OsStr;

pub fn run_succeeds<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cargo_bin_cmd!("fshc").args(args).assert().success()
}

pub fn run_fails<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cargo_bin_cmd!("fshc").args(args).assert().failure()
}

pub fn output_includes(content: &str) -> ContainsPredicate {
    predicate::str::contains(content)
}
