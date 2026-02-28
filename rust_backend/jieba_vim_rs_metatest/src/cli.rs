// Copyright 2026 Kaiwen Wu. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

use crate::parsing;
use crate::rust_transpiler::rust_test;
use crate::vimscript_transpiler::bootstrap;
use crate::vimscript_transpiler::unit_verification;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run parsing in check mode.
    Check(parsing::Cli),
    /// Run unit test verification.
    VerifyUnit(unit_verification::Cli),
    /// Generate rust tests from unit test verification outputs.
    GenUnit(rust_test::Cli),
    /// Run bootstrap test verification.
    Bootstrap(bootstrap::Cli),
}
