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
use crate::token::JiebaPlaceholder;

use super::{MotionOutput, WordMotion};

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`) in
    /// operator-pending mode while used with operator `d`. Since Vim's help
    /// states in section "exclusive-linewise" that:
    ///
    /// > When using ":" any motion becomes characterwise exclusive,
    ///
    /// But since `e`/`E` is itself inclusive, and `o_v`
    /// (https://vimhelp.org/motion.txt.html#o_v) can be used to invert
    /// exclusiveness to inclusiveness, we may use prefix the colon command
    /// with it and reuse most code from `nmap e`.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the last character of the last token in the
    ///   buffer, no further jump should be made.
    /// - If there is no next word to the right of current cursor, jump to the
    ///   last character of the last token in the buffer.
    ///
    /// # Panics
    ///
    /// - If current cursor `col` is to the right of the last token in current
    ///   line of the buffer.
    pub fn omap_e<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        count: u64,
        word: bool,
    ) -> Result<MotionOutput, B::Error> {
        let mo = self.nmap_e(buffer, cursor_pos, count, word)?;
        Ok(MotionOutput {
            new_cursor_pos: mo.new_cursor_pos,
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
        operator = "c",
        motion = "e",
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "one_word", buffer = ["abc{}d"])]
    #[vcase(name = "one_word", buffer = ["abc{}d"], count = 2)]
    #[vcase(name = "one_word", buffer = ["a{bc}d"])]
    #[vcase(name = "one_word", buffer = ["a{bc}d"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{bc}d    "])]
    #[vcase(name = "one_word_space", buffer = ["a{bcd   } "], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["abc{d   } "])]
    #[vcase(name = "one_word_space", buffer = ["abc{d   } "], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["abcd {  } "])]
    #[vcase(name = "one_word_space", buffer = ["abcd {  } "], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    ab}c"])]
    #[vcase(name = "space_word", buffer = [" {   ab}c"])]
    #[vcase(name = "space_word", buffer = ["{    ab}c  def"])]
    #[vcase(name = "space_word", buffer = ["{    abc  de}f"], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    abc  de}f"], count = 3)]
    #[vcase(name = "two_words", buffer = ["a{bc}d  efg"])]
    #[vcase(name = "two_words", buffer = ["a{bcd  ef}g"], count = 2)]
    #[vcase(name = "two_words", buffer = ["a{bcd  ef}g"], count = 3)]
    #[vcase(name = "two_words", buffer = ["abc{d ef}g"])]
    #[vcase(name = "two_words", buffer = ["abc{d ef}g"], count = 2)]
    #[vcase(name = "two_words", buffer = ["abc{d efg  } "], count = 3)]
    #[vcase(name = "one_word_newline", buffer = ["a{bc}d", ""])]
    #[vcase(name = "one_word_newline", buffer = ["a{bcd", "}"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["abc{d", "}"])]
    #[vcase(name = "newline_one_word", buffer = ["{", "", "abc}d"])]
    #[vcase(name = "newline_one_word", buffer = ["{", "  ", "abc}d"])]
    #[vcase(name = "newline_two_words", buffer = ["{", "", "abc}d", "efg"])]
    #[vcase(name = "newline_one_word_space", buffer = ["{", "", "abc}d    "])]
    #[vcase(name = "newline_one_word_space_word", buffer = ["{", "", "abc}d    e"])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "  } "])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "  } "], count = 2)]
    #[vcase(name = "one_word_space_newline", buffer = ["a{bc}d    ", ""])]
    #[vcase(name = "one_word_space_newline", buffer = ["abc{d     ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["abcd{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["abcd {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "  ", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abcd", "{  ", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "", "   } "])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", "", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", "", "", "}"])]
    #[vcase(name = "word_newline_word", buffer = ["a{bc}d", "", " ", "", "efg"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", " ", "", "ef}g  "])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "  ", "", " ", "efg}h"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "ef}g", "", "efgh"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "efg", "", "efg}h"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "efg", "", "efg}h  "], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{bc def}g"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["abc {def}g"], count = 10293949403)]
    mod motion_omap_c_e {}

    // Copied from omap_c_e above.
    #[verified_cases(
        mode = "o",
        operator = "y",
        motion = "e",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "one_word", buffer = ["abc{}d"])]
    #[vcase(name = "one_word", buffer = ["abc{}d"], count = 2)]
    #[vcase(name = "one_word", buffer = ["a{bc}d"])]
    #[vcase(name = "one_word", buffer = ["a{bc}d"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{bc}d    "])]
    #[vcase(name = "one_word_space", buffer = ["a{bcd   } "], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["abc{d   } "])]
    #[vcase(name = "one_word_space", buffer = ["abc{d   } "], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["abcd {  } "])]
    #[vcase(name = "one_word_space", buffer = ["abcd {  } "], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    ab}c"])]
    #[vcase(name = "space_word", buffer = [" {   ab}c"])]
    #[vcase(name = "space_word", buffer = ["{    ab}c  def"])]
    #[vcase(name = "space_word", buffer = ["{    abc  de}f"], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    abc  de}f"], count = 3)]
    #[vcase(name = "two_words", buffer = ["a{bc}d  efg"])]
    #[vcase(name = "two_words", buffer = ["a{bcd  ef}g"], count = 2)]
    #[vcase(name = "two_words", buffer = ["a{bcd  ef}g"], count = 3)]
    #[vcase(name = "two_words", buffer = ["abc{d ef}g"])]
    #[vcase(name = "two_words", buffer = ["abc{d ef}g"], count = 2)]
    #[vcase(name = "two_words", buffer = ["abc{d efg  } "], count = 3)]
    #[vcase(name = "one_word_newline", buffer = ["a{bc}d", ""])]
    #[vcase(name = "one_word_newline", buffer = ["a{bcd", "}"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["abc{d", "}"])]
    #[vcase(name = "newline_one_word", buffer = ["{", "", "abc}d"])]
    #[vcase(name = "newline_one_word", buffer = ["{", "  ", "abc}d"])]
    #[vcase(name = "newline_two_words", buffer = ["{", "", "abc}d", "efg"])]
    #[vcase(name = "newline_one_word_space", buffer = ["{", "", "abc}d    "])]
    #[vcase(name = "newline_one_word_space_word", buffer = ["{", "", "abc}d    e"])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "  } "])]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "  } "], count = 2)]
    #[vcase(name = "one_word_space_newline", buffer = ["a{bc}d    ", ""])]
    #[vcase(name = "one_word_space_newline", buffer = ["abc{d     ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["abcd{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["abcd {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "  ", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abcd", "{  ", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "", "   } "])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", "", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", "", "", "}"])]
    #[vcase(name = "word_newline_word", buffer = ["a{bc}d", "", " ", "", "efg"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", " ", "", "ef}g  "])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "  ", "", " ", "efg}h"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "ef}g", "", "efgh"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "efg", "", "efg}h"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "efg", "", "efg}h  "], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{bc def}g"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["abc {def}g"], count = 10293949403)]
    mod motion_omap_y_e {}
}
