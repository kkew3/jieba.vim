// Copyright 2025-2026 Kaiwen Wu. All Rights Reserved.
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

//! This module defines abstraction over `Jieba`, and re-exports an
//! implementation of it.

/// Jieba-like types, defined so that this crate won't need to actually depend
/// on `jieba-rs`.
pub trait JiebaPlaceholder {
    /// Cut sentence with `hmm` enabled.
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str>;
}

#[cfg(test)]
pub use jieba_vim_rs_test::keyword_cutter::KeywordCutter;

#[cfg(test)]
impl JiebaPlaceholder for KeywordCutter {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.cut(sentence)
    }
}
