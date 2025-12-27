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

/// Test if a token is stoppable for `xmap_e`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    match item.token {
        None => false,
        Some(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`)
    /// in visual mode. Take in current `cursor_pos` (lnum, col), and return
    /// the new cursor position. Note that `lnum` is 1-indexed, and `col`
    /// is 0-indexed. We denote both `word` and `WORD` with the English word
    /// "word" below.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the one character to the right of the last
    ///   character of the last token in the buffer, no further jump should
    ///   be made.
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character to the right of the last character of the last token in the
    ///   buffer.
    pub fn xmap_e<B: BufferLike + ?Sized>(
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
                if it.peek().is_some() {
                    col = item.token.last_char();
                } else {
                    col = item.token.last_char1();
                }
            } else if !(item.cursor && col == item.token.last_char()) {
                lnum = item.lnum;
                col = item.token.last_char();
                count -= 1;
                if count > 0 && it.peek().is_none() {
                    col = item.token.last_char1();
                }
            } else if it.peek().is_none() {
                col = item.token.last_char1();
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
        mode = "xc",
        motion = "e",
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "one_word", buffer = ["aaa{a}"])]
    #[vcase(name = "one_word", buffer = ["a{aa}a"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{aa}a    "])]
    #[vcase(name = "one_word_space", buffer = ["aaa{a    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaaa {   }"])]
    #[vcase(name = "space_one_word", buffer = ["{    aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["   { aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["    {aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["    aaa{a}"])]
    #[vcase(name = "two_words", buffer = ["a{aa}a  aaa"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  aa}a"], count = 2)]
    #[vcase(name = "two_words", buffer = ["a{aaa  aaa}"], count = 3)]
    #[vcase(name = "two_words", buffer = ["aaa{a aa}a"])]
    #[vcase(name = "two_words", buffer = ["aaa{a aaa}"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["    {aaa}a   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaa}a   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaaa   }"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["a{aa}a", ""])]
    #[vcase(name = "one_word_newline", buffer = ["a{aaa", "}"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["aaa{a", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["a{aa}a    ", ""])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaa{a     ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaaa", "{  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "", "    }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", "", "  }"])]
    #[vcase(name = "word_newline_word", buffer = ["a{aa}a", "", " ", "", "aaa"])]
    #[vcase(name = "word_newline_word", buffer = ["a{aaa", "", " ", "", "aa}a"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["a{aaa", "", " ", "", "aaa   }"], count = 3)]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", " ", "", "aa}a"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "  ", "", " ", "aaa}a"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", "aa}a", "", "aaaa"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", "aaa", "", "aaa}a"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{aa aaaa}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["aaa {aaaa}"], count = 10293949403)]
    mod motion_xcmap_e {}

    // Copied from xcmap_e above.
    #[verified_cases(
        mode = "xl",
        motion = "e",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "one_word", buffer = ["aaa{a}"])]
    #[vcase(name = "one_word", buffer = ["a{aa}a"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{aa}a    "])]
    #[vcase(name = "one_word_space", buffer = ["aaa{a    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaaa {   }"])]
    #[vcase(name = "space_one_word", buffer = ["{    aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["   { aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["    {aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["    aaa{a}"])]
    #[vcase(name = "two_words", buffer = ["a{aa}a  aaa"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  aa}a"], count = 2)]
    #[vcase(name = "two_words", buffer = ["a{aaa  aaa}"], count = 3)]
    #[vcase(name = "two_words", buffer = ["aaa{a aa}a"])]
    #[vcase(name = "two_words", buffer = ["aaa{a aaa}"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["    {aaa}a   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaa}a   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaaa   }"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["a{aa}a", ""])]
    #[vcase(name = "one_word_newline", buffer = ["a{aaa", "}"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["aaa{a", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["a{aa}a    ", ""])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaa{a     ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaaa", "{  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "", "    }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", "", "  }"])]
    #[vcase(name = "word_newline_word", buffer = ["a{aa}a", "", " ", "", "aaa"])]
    #[vcase(name = "word_newline_word", buffer = ["a{aaa", "", " ", "", "aa}a"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["a{aaa", "", " ", "", "aaa   }"], count = 3)]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", " ", "", "aa}a"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "  ", "", " ", "aaa}a"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", "aa}a", "", "aaaa"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", "aaa", "", "aaa}a"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{aa aaaa}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["aaa {aaaa}"], count = 10293949403)]
    mod motion_xlmap_e {}

    // Copied from xcmap_e above.
    #[verified_cases(
        mode = "xb",
        motion = "e",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "one_word", buffer = ["aaa{a}"])]
    #[vcase(name = "one_word", buffer = ["a{aa}a"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{aa}a    "])]
    #[vcase(name = "one_word_space", buffer = ["aaa{a    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaaa {   }"])]
    #[vcase(name = "space_one_word", buffer = ["{    aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["   { aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["    {aaa}a"])]
    #[vcase(name = "space_one_word", buffer = ["    aaa{a}"])]
    #[vcase(name = "two_words", buffer = ["a{aa}a  aaa"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  aa}a"], count = 2)]
    #[vcase(name = "two_words", buffer = ["a{aaa  aaa}"], count = 3)]
    #[vcase(name = "two_words", buffer = ["aaa{a aa}a"])]
    #[vcase(name = "two_words", buffer = ["aaa{a aaa}"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["    {aaa}a   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaa}a   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaaa   }"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["a{aa}a", ""])]
    #[vcase(name = "one_word_newline", buffer = ["a{aaa", "}"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["aaa{a", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["a{aa}a    ", ""])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaa{a     ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaaa", "{  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaa{a", "", "    }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", " ", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", " ", "", "  }"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["aaa{a", "", "", "  }"])]
    #[vcase(name = "word_newline_word", buffer = ["a{aa}a", "", " ", "", "aaa"])]
    #[vcase(name = "word_newline_word", buffer = ["a{aaa", "", " ", "", "aa}a"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["a{aaa", "", " ", "", "aaa   }"], count = 3)]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", " ", "", "aa}a"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "  ", "", " ", "aaa}a"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", "aa}a", "", "aaaa"])]
    #[vcase(name = "word_newline_word", buffer = ["aaa{a", "", "aaa", "", "aaa}a"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{aa aaaa}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["aaa {aaaa}"], count = 10293949403)]
    mod motion_xbmap_e {}
}
