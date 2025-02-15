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

use crate::token::token_iter::{BackwardTokenIterator, TokenIteratorItem};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};
use crate::BufferLike;

use super::{MotionOutput, WordMotion};

/// Test if a token is stoppable for `nmap_b`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    match item.token {
        None => true,
        Some(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `b` (if `word` is `true`) or `B` (if `word` is `false`)
    /// in normal mode. Take in `cursor_pos` (lnum, col), and return the new
    /// cursor position. Note that `lnum` is 1-indexed, and `col` is 0-indexed.
    /// We denote both `word` and `WORD` with the English word "word" below.
    ///
    /// # Basics
    ///
    /// `b`/`B` jumps to the first character of previous word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    ///
    /// # Panics
    ///
    /// - If current cursor `col` is to the right of the last token in current
    ///   line of the buffer.
    pub fn nmap_b<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        mut count: u64,
        word: bool,
    ) -> Result<MotionOutput, B::Error> {
        let (mut lnum, mut col) = cursor_pos;
        let mut it = BackwardTokenIterator::new(
            buffer,
            &self.tokenizer,
            lnum,
            col,
            word,
        )?
        .peekable();
        while count > 0 && it.peek().is_some() {
            let item = it.next().unwrap()?;
            if !is_stoppable(&item) {
                lnum = item.lnum;
                col = item.token.first_char();
            } else if !(item.cursor && col == item.token.first_char()) {
                lnum = item.lnum;
                col = item.token.first_char();
                count -= 1;
            }
        }
        Ok(MotionOutput {
            new_cursor_pos: (lnum, col),
            d_special: false,
            prevent_change: false,
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "verifiable_case")]
    use jieba_vim_rs_test_macro::verified_cases;
    #[cfg(not(feature = "verifiable_case"))]
    use jieba_vim_rs_test_macro::verified_cases_dry_run as verified_cases;

    #[verified_cases(
        mode = "n",
        motion = "b",
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["}{"])]
    #[vcase(name = "space", buffer = ["}{ "])]
    #[vcase(name = "space", buffer = ["}   { "])]
    #[vcase(name = "newline_newline", buffer = ["}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "   {  "])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "   {  "])]
    #[vcase(name = "one_word", buffer = ["}{aaaa"])]
    #[vcase(name = "one_word", buffer = ["}aa{aa"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["}aaaa{   "])]
    #[vcase(name = "one_word_space", buffer = ["}aaaa  { "])]
    #[vcase(name = "space_one_word", buffer = ["   }aaa{a"])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["}   {aaaa"])]
    #[vcase(name = "two_words", buffer = ["}aaaa  {aaa"])]
    #[vcase(name = "two_words", buffer = ["aaaa  }aa{a"])]
    #[vcase(name = "two_words", buffer = ["}aaaa  aa{a"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   }aaaa  { "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa  { "], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   }aaaa{   "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa{   "], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["}aaaa", "{"])]
    #[vcase(name = "newline_one_word", buffer = ["", "}aaa{a"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"], count = 2)]
    #[vcase(name = "one_word_space_newline", buffer = ["}aaaa    ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa }aaa    ", "  ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa }aaa    ", "  ", "  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["", "   }aaa{a"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"], count = 2)]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   {aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "  { aaaa"])]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["     ", "}", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["     ", "}", "", "aaa{a"], count = 3)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "", "aaa{a"], count = 4)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", " ", " ", "aaa{a"], count = 2)]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa }aaaa", " ", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaaa", "}", "  ", "{"])]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["", "  ", "}", "aa{a"], count = 2)]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["}", "  ", "", "aa{a"], count = 3)]
    #[vcase(name = "two_words_newline_one_word", buffer = ["aaaa }aaa", "", "  ", "{aaa"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}{"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}aaa  aaa{aa"], count = 10293949403)]
    mod motion_nmap_b {}
}
