// Copyright 2024-2026 Kaiwen Wu. All Rights Reserved.
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

use crate::token::token_iter::{ForwardTokenIterator, TokenIteratorItem};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};
use crate::{BufferLike, Position};

use super::{WordMotion, XmapOutput};

/// Test if a token is stoppable for `xmap_w`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    if item.cursor {
        false
    } else {
        match item.token {
            None => true,
            Some(token) => match token.ty {
                TokenType::Word => true,
                TokenType::Space => false,
            },
        }
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`)
    /// in visual mode. Take in current `cursor_pos` (lnum, col), and return
    /// the new cursor position. Note that `lnum` is 1-indexed, and `col`
    /// is 0-indexed. We denote both `word` and `WORD` with the English word
    /// "word" below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the one character to the right of the last
    ///   character of the last token in the buffer, no further jump should be
    ///   made.
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character to the right of the last character of the last token in the
    ///   buffer.
    pub fn xmap_w<'a, B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        visualmode: &'a [u8],
        visual_begin: Position,
        visual_end: Position,
        mut count: u64,
        word: bool,
    ) -> Result<XmapOutput<'a>, B::Error> {
        let [bufnum, lnum_orig, col_p1_orig, off] = visual_end;
        let mut lnum = lnum_orig;
        let mut col = col_p1_orig - 1;
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
            } else {
                lnum = item.lnum;
                col = item.token.first_char();
                count -= 1;
                if count > 0 && it.peek().is_none() {
                    col = item.token.last_char1();
                }
            }
        }
        let col_p1 = col + 1;
        let prevent_change = if (lnum, col_p1) != (lnum_orig, col_p1_orig) {
            b"0"
        } else {
            b"1"
        };
        Ok(XmapOutput {
            langle: visual_begin,
            rangle: [bufnum, lnum, col_p1, off],
            visualmode,
            prevent_change,
        })
    }
}
