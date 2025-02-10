// Copyright 2024-2025 Kaiwen Wu. All Rights Reserved.
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

#[cfg(test)]
use crate::token::jieba::KeywordCutter;
#[cfg(test)]
use crate::token::Tokenizer;
#[cfg(test)]
use jieba_vim_rs_test::verified_case::cases::MotionOutput as TestMotionOutput;
#[cfg(test)]
use once_cell::sync::Lazy;

mod d_special;
mod nmap_b;
mod nmap_e;
mod nmap_ge;
mod nmap_w;
mod omap_b;
mod omap_c_w;
mod omap_d_e;
mod omap_d_ge;
mod omap_e;
mod omap_ge;
mod omap_w;
mod word_motion;
mod xmap_b;
mod xmap_e;
mod xmap_ge;
mod xmap_w;

pub use word_motion::WordMotion;

/// The motion return type.
#[derive(Debug)]
pub struct MotionOutput {
    /// The new cursor position after the motion.
    pub new_cursor_pos: (usize, usize),
    /// Whether the motion induces d-special. Should be false when not in
    /// operator-pending mode
    pub d_special: bool,
    /// Whether the motion should prevent changes, where the operation is
    /// silently aborted. Should be false when not in operator-pending mode
    pub prevent_change: bool,
}

#[cfg(test)]
impl PartialEq<TestMotionOutput> for MotionOutput {
    fn eq(&self, other: &TestMotionOutput) -> bool {
        self.new_cursor_pos == other.new_cursor_pos
            && self.d_special == other.d_special
            && self.prevent_change == other.prevent_change
    }
}

#[cfg(test)]
static WORD_MOTION: Lazy<WordMotion<KeywordCutter>> = Lazy::new(|| {
    WordMotion::new(Tokenizer::new(KeywordCutter::new([]), "@,48-57,_,192-255"))
});

#[cfg(test)]
impl<C> WordMotion<C> {
    fn _noop(&self) {}
}

#[cfg(test)]
#[ctor::ctor]
fn init_word_motion() {
    WORD_MOTION._noop(); // force initialization
}
