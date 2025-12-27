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
use crate::token::token_iter::{ForwardTokenIterator, TokenIteratorItem};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};

use super::{MotionOutput, WordMotion};

/// Test if a token is stoppable for `omap_w`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    match item.token {
        None => true,
        Some(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

// Checkout https://vimhelp.org/intro.txt.html#%7Bmotion%7D, on the subsection
// "Ex commands". We may opt to set 'virtualedit' before `omap`, and do not use
// `o_v` to invert the exclusiveness. Example omap:
//
//     function! MoveToEOL()
//         set ve=all  " Can't be placed outside the cursor position function
//         call cursor(line('.'), col('$'))
//     endfunction
//
//     onoremap $ :<c-u>call MoveToEOL()<cr>:set ve=none<cr>

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`)
    /// in operator-pending mode. Since Vim's help states in section
    /// "exclusive-linewise" that:
    ///
    /// > When using ":" any motion becomes characterwise exclusive.
    ///
    /// with plain onoremap we won't be able to operate on the last character
    /// in a line. Therefore, we assume that `+virtualedit` feature is enabled
    /// and `set virtualedit=onemore` temporarily to circumvent this issue.
    /// See also about this trick at https://vimhelp.org/intro.txt.html#%7Bmotion%7D
    /// and https://github.com/svermeulen/vim-NotableFt/blob/master/plugin/NotableFt.vim.
    ///
    /// Take in current `cursor_pos` (lnum, col), and return the new cursor
    /// position. Note that `lnum` is 1-indexed, and `col` is 0-indexed. We
    /// denote both `word` and `WORD` with the English word "word" below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character after the last token in the buffer (`virtualedit`).
    /// - Quoted from Vim's help section "WORD": "When using the `w` motion in
    ///   combination with an operator and the last word moved over is at the
    ///   end of a line, the end of that word becomes the end of the operated
    ///   text, not the first word in the next line."
    ///
    /// # Panics
    ///
    /// - If current cursor `col` is to the right of the last token in current
    ///   line of the buffer.
    pub fn omap_w<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        mut count: u64,
        word: bool,
    ) -> Result<MotionOutput, B::Error> {
        let (mut lnum, mut col) = cursor_pos;
        let mut it = ForwardTokenIterator::new(
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
                if it.peek().is_none() || (count == 1 && item.eol) {
                    col = item.token.last_char1();
                    count -= 1;
                } else {
                    col = item.token.last_char();
                }
            } else {
                if !item.cursor {
                    lnum = item.lnum;
                    col = item.token.first_char();
                    count -= 1;
                }
                if count > 0 && it.peek().is_none() {
                    col = item.token.last_char1();
                    count -= 1;
                } else if count == 1 && item.eol && it.peek().is_some() {
                    if item.token.is_none() {
                        let next_item = it.next().unwrap()?;
                        lnum = next_item.lnum;
                        col = next_item.token.first_char();
                    } else {
                        col = item.token.last_char1();
                    }
                    count -= 1;
                }
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
        mode = "o",
        operator = "d",
        motion = "w",
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "empty_empty", buffer = ["{", "}"])]
    #[vcase(name = "space_newline", buffer = ["   { }", ""])]
    #[vcase(name = "space_newline", buffer = ["   { }", "  "])]
    #[vcase(name = "space_newline", buffer = ["{   }", ""])]
    #[vcase(name = "space_newline", buffer = ["{   }", "  "])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "}       ", ""])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "}       abcd", ""])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "}abcd", ""])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "   abcd}", "       ", "  ef"], count = 2)]
    #[vcase(name = "empty_space_empty", buffer = ["{", "   abcd}", "         efg", "  hi"], count = 2)]
    #[vcase(name = "empty_word", buffer = ["{", "}abc  def"])]
    #[vcase(name = "empty_word", buffer = ["{", "abc  }def"], count = 2)]
    #[vcase(name = "one_word", buffer = ["{abcd}"])]
    #[vcase(name = "one_word", buffer = ["a{bcd}"])]
    #[vcase(name = "one_word", buffer = ["abc{d}"])]
    #[vcase(name = "one_word_space", buffer = ["{abcd   }"])]
    #[vcase(name = "one_word_space", buffer = ["ab{cd   }"])]
    #[vcase(name = "space_word", buffer = ["{    }abc"])]
    #[vcase(name = "space_word", buffer = [" {   }abc"])]
    #[vcase(name = "space_word", buffer = ["{    abc  }def"], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    abc  def}"], count = 3)]
    #[vcase(name = "two_words", buffer = ["{abcd    }efg"])]
    #[vcase(name = "two_words", buffer = ["ab{cd    }efg"])]
    #[vcase(name = "two_words", buffer = ["abc{d    }efg"])]
    #[vcase(name = "two_words", buffer = ["abcd{    }efg"])]
    #[vcase(name = "two_words", buffer = ["abcd {   }efg"])]
    #[vcase(name = "two_words", buffer = ["abcd   { }efg"])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", ""])]
    #[vcase(name = "word_newline", buffer = ["abcd   e{fgh}", ""])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", "  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   efg{h}", "  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", "  ijkl"])]
    #[vcase(name = "word_newline", buffer = ["abcd   efg{h}", "  ijkl"])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", "ijkl  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   efg{h}", "ijkl  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh", "   ijkl}"], count = 2)]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh", "ijkl   }"], count = 2)]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh", "   ijkl   }"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["{abc}", "def"])]
    #[vcase(name = "word_newline_word", buffer = ["{abc", "def}"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["abc", "{def}"])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   }", "   "])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "   }"], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["    {  }", "       "])]
    #[vcase(name = "space_newline_space", buffer = ["    {  ", "       }"], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["  {    ", "   ", "    }"], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["  {    ", "   ", "", "}    "], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["  {    ", "   ", "", "    }"], count = 3)]
    #[vcase(name = "word_space_newline_space", buffer = ["a{bcd     }", "    "])]
    #[vcase(name = "word_space_newline_space", buffer = ["a{bcd     ", "     }"], count = 2)]
    #[vcase(name = "word_space_newline_space", buffer = ["a{bcd     ", "      ", "  }"], count = 2)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  hij}", ""], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", "", "}  hij"], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg}", ""], count = 2)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg}", " ", "  ", "  ", "  hij"], count = 2)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  ", "  ", "  hij}", "  ", ""], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", "", "} ", "  hij"], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  hij   }", ""], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  ", "  ", "  hij  }", "  ", ""], count = 3)]
    mod motion_omap_d_w {}

    // Copied from omap_d_w above.
    #[verified_cases(
        mode = "o",
        operator = "y",
        motion = "w",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "empty_empty", buffer = ["{", "}"])]
    #[vcase(name = "space_newline", buffer = ["   { }", ""])]
    #[vcase(name = "space_newline", buffer = ["   { }", "  "])]
    #[vcase(name = "space_newline", buffer = ["{   }", ""])]
    #[vcase(name = "space_newline", buffer = ["{   }", "  "])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "}       ", ""])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "}       abcd", ""])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "}abcd", ""])]
    #[vcase(name = "empty_space_empty", buffer = ["{", "   abcd}", "       ", "  ef"], count = 2)]
    #[vcase(name = "empty_space_empty", buffer = ["{", "   abcd}", "         efg", "  hi"], count = 2)]
    #[vcase(name = "empty_word", buffer = ["{", "}abc  def"])]
    #[vcase(name = "empty_word", buffer = ["{", "abc  }def"], count = 2)]
    #[vcase(name = "one_word", buffer = ["{abcd}"])]
    #[vcase(name = "one_word", buffer = ["a{bcd}"])]
    #[vcase(name = "one_word", buffer = ["abc{d}"])]
    #[vcase(name = "one_word_space", buffer = ["{abcd   }"])]
    #[vcase(name = "one_word_space", buffer = ["ab{cd   }"])]
    #[vcase(name = "space_word", buffer = ["{    }abc"])]
    #[vcase(name = "space_word", buffer = [" {   }abc"])]
    #[vcase(name = "space_word", buffer = ["{    abc  }def"], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    abc  def}"], count = 3)]
    #[vcase(name = "two_words", buffer = ["{abcd    }efg"])]
    #[vcase(name = "two_words", buffer = ["ab{cd    }efg"])]
    #[vcase(name = "two_words", buffer = ["abc{d    }efg"])]
    #[vcase(name = "two_words", buffer = ["abcd{    }efg"])]
    #[vcase(name = "two_words", buffer = ["abcd {   }efg"])]
    #[vcase(name = "two_words", buffer = ["abcd   { }efg"])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", ""])]
    #[vcase(name = "word_newline", buffer = ["abcd   e{fgh}", ""])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", "  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   efg{h}", "  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", "  ijkl"])]
    #[vcase(name = "word_newline", buffer = ["abcd   efg{h}", "  ijkl"])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh}", "ijkl  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   efg{h}", "ijkl  "])]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh", "   ijkl}"], count = 2)]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh", "ijkl   }"], count = 2)]
    #[vcase(name = "word_newline", buffer = ["abcd   {efgh", "   ijkl   }"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["{abc}", "def"])]
    #[vcase(name = "word_newline_word", buffer = ["{abc", "def}"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["abc", "{def}"])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   }", "   "])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "   }"], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["    {  }", "       "])]
    #[vcase(name = "space_newline_space", buffer = ["    {  ", "       }"], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["  {    ", "   ", "    }"], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["  {    ", "   ", "", "}    "], count = 2)]
    #[vcase(name = "space_newline_space", buffer = ["  {    ", "   ", "", "    }"], count = 3)]
    #[vcase(name = "word_space_newline_space", buffer = ["a{bcd     }", "    "])]
    #[vcase(name = "word_space_newline_space", buffer = ["a{bcd     ", "     }"], count = 2)]
    #[vcase(name = "word_space_newline_space", buffer = ["a{bcd     ", "      ", "  }"], count = 2)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  hij}", ""], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", "", "}  hij"], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg}", ""], count = 2)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg}", " ", "  ", "  ", "  hij"], count = 2)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  ", "  ", "  hij}", "  ", ""], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", "", "} ", "  hij"], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  hij   }", ""], count = 3)]
    #[vcase(name = "word_newline_counts", buffer = ["ab{cd  efg", " ", "  ", "  ", "  hij  }", "  ", ""], count = 3)]
    mod motion_omap_y_w {}
}
