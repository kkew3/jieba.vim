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

use crate::BufferLike;
use crate::token::token_iter::{BackwardTokenIterator, TokenIteratorItem};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};

use super::{MotionOutput, WordMotion, d_special};

/// Test if a token is stoppable for `omap_d_ge`.
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
    /// Vim motion `ge` (if `word` is `true`) or `gE` (if `word` is `false`) in
    /// operator-pending mode while used with operator `d`. Since Vim's help
    /// states in section "exclusive-linewise" that:
    ///
    /// > When using ":" any motion becomes characterwise exclusive,
    ///
    /// But since `ge`/`gE` is itself inclusive, and `o_v`
    /// (https://vimhelp.org/motion.txt.html#o_v) can be used to invert
    /// exclusiveness to inclusiveness, we may prefix the colon command with
    /// it and reuse most code from `nmap ge`. Note also the special case
    /// `d-special` (https://vimhelp.org/change.txt.html#d-special), where we
    /// have to postprocess the buffer.
    ///
    /// Take in current `cursor_pos` (lnum, col), and return the new cursor
    /// position. Also return a bool indicating if `d-special` takes effect.
    /// Note that `lnum` is 1-indexed, and `col` is 0-indexed. We denote both
    /// `word` and `WORD` with the English word "word" below.
    ///
    /// # Basics
    ///
    /// `ge`/`gE` jumps to the last character of previous word. Empty line is
    /// considered as a word. If there's no previous word except for the empty
    /// line, issue `prevent_change` flag.
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
    pub fn omap_d_ge<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        mut count: u64,
        word: bool,
    ) -> Result<MotionOutput, B::Error> {
        let (mut lnum, mut col) = cursor_pos;
        let mut prevent_change = lnum == 1 && col == 0 && count > 0;
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
            if !is_stoppable(&item) || item.cursor {
                lnum = item.lnum;
                col = item.token.first_char();
            } else {
                lnum = item.lnum;
                col = item.token.last_char();
                count -= 1;
                if it.peek().is_none() && count > 0 {
                    col = item.token.first_char();
                    count -= 1;
                    if let None = item.token {
                        prevent_change = true;
                    }
                }
            }
        }
        let d_special = d_special::is_d_special(
            buffer,
            &self.tokenizer,
            (lnum, col),
            cursor_pos,
            word,
        )?;
        Ok(MotionOutput {
            new_cursor_pos: (lnum, col),
            d_special,
            prevent_change,
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
        mode = "o",
        operator = "d",
        motion = "ge",
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["}{"], prevent_change)]
    #[vcase(name = "space", buffer = ["}{ "], prevent_change)]
    #[vcase(name = "space", buffer = ["}   { "])]
    #[vcase(name = "newline_newline", buffer = ["}", "{"], d_special)]
    #[vcase(name = "newline_newline", buffer = ["}", "{"], count = 2, d_special, prevent_change)]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "{"], d_special)]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "{"], d_special)]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "  ", "{"], d_special)]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "   {  "], d_special)]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "   {  "], d_special)]
    #[vcase(name = "one_word", buffer = ["}{aaaa"], prevent_change)]
    #[vcase(name = "one_word", buffer = ["}aa{aa"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["aaa}a{   "])]
    #[vcase(name = "one_word_space", buffer = ["aaa}a  { "])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["}   {aaaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  {aaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  aa{a"])]
    #[vcase(name = "two_words", buffer = ["}aaaa  aa{a"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a  { "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa  { "], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a{   "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa{   "], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["aaa}a", "{"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"], d_special)]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"], count = 2, d_special, prevent_change)]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"], count = 3, d_special, prevent_change)]
    #[vcase(name = "one_word_space_newline", buffer = ["aaa}a    ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"], d_special)]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"], count = 2, d_special, prevent_change)]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   {aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "  { aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["", "   aaa}a  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaaa  { "], count = 2, d_special)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "aaa{a"], d_special)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "aaa{a"], count = 2, d_special)]
    #[vcase(name = "space_newline_one_word", buffer = ["     ", "}", "", "aaa{a"], count = 2, d_special)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "", "aaa{a"], count = 3, d_special)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", " ", " ", "aaa{a"], d_special)]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", " ", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aa}a aaaa", " ", "  ", "{"], count = 2)]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaaa", "}", "  ", "{"], d_special)]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", "", "  ", "{"], count = 2)]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["", "  ", "}", "aa{a"], d_special)]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["}", "  ", "", "aa{a"], count = 2, d_special)]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["}", "  ", "", "aa{a"], count = 3, d_special, prevent_change)]
    #[vcase(name = "two_words_newline_one_word", buffer = ["aaaa aa}a", "", "  ", "{aaa"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}{"], count = 10293949403, prevent_change)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}aaa  aaa{aa"], count = 10293949403)]
    mod motion_omap_d_ge {}
}
