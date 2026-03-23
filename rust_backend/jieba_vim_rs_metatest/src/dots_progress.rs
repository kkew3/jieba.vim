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

//! A simple dots progress bar.

use std::io::{self, Write};

/// Show progress by printing dots to stdout.
pub struct DotsProgress {
    dots: u32,
    n_dots_in_a_row: u32,
}

impl Default for DotsProgress {
    fn default() -> Self {
        Self {
            dots: 0,
            n_dots_in_a_row: 80,
        }
    }
}

impl DotsProgress {
    pub fn step(&mut self) {
        print!(".");
        io::stdout().flush().ok();
        self.dots += 1;
        if self.dots.is_multiple_of(self.n_dots_in_a_row) {
            println!(" {}", self.dots);
        }
    }

    pub fn reset(&mut self) {
        if !self.dots.is_multiple_of(self.n_dots_in_a_row) {
            println!(" {}", self.dots);
        }
        self.dots = 0;
    }
}

impl Drop for DotsProgress {
    fn drop(&mut self) {
        self.reset();
    }
}
