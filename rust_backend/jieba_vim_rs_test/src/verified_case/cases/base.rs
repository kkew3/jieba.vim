// Copyright 2024 Kaiwen Wu. All Rights Reserved.
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

use std::fmt;
use std::path::Path;

/// A test case that can be verified through Vim vader test (see
/// https://github.com/junegunn/vader.vim). The test should implement Display
/// so that it can be pretty-printed on test error.
pub trait VerifiableCase: fmt::Display + Clone + Into<MotionOutput> {
    /// Write the test case to a file that can be used by vader.vim. Panics if
    /// the file cannot be written.
    fn to_vader(&self, path: &Path);
}

/// A mirror definition of `MotionOutput` defined in `jieba_vim_rs_core` crate.
#[derive(Debug)]
pub struct MotionOutput {
    pub new_cursor_pos: (usize, usize),
    pub d_special: bool,
    pub prevent_change: bool,
}
