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

use crate::motion::token_iter::TokenLikeExt;
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};
use crate::{BufferLike, Position, pos};

use super::token_iter::{ForwardTokenIterator, GToken, TokenIteratorItem};
use super::{WordMotion, XmapOutput};

/// Test if a token is stoppable for `xmap_e`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    match item.token {
        GToken::Eol(_) => false,
        GToken::T(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`)
    /// in visual mode. Take in current `visual_end` (0, lnum, col, off), and
    /// return the new `visual_end`. Note that `visual_begin` will be left
    /// intact. We denote both `word` and `WORD` with the English word "word"
    /// below.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character to the right of the last character of the last token in the
    ///   buffer. And the motion should be taken as a failure.
    pub fn xmap_e<'a, B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        visualmode: &'a [u8],
        visual_begin: Position,
        visual_end: Position,
        mut count: u64,
        word: bool,
    ) -> Result<XmapOutput<'a>, B::Error> {
        let [_, mut lnum, mut col, _] = visual_end;
        let mut it = ForwardTokenIterator::new(
            buffer,
            &self.tokenizer,
            &visual_end,
            word,
        )?;
        let cursor_item = it.first();
        let mut it = it.peekable();
        let mut stopped = false;
        if count > 0 {
            if is_stoppable(&cursor_item) {
                if !cursor_item.token.at_end(col) {
                    // Call to last_char is valid because only word is stoppable.
                    col = cursor_item.token.last_char();
                    count -= 1;
                    stopped = true;
                }
            } else {
                col = cursor_item.token.last_char1();
            }
        }

        while count > 0 && it.peek().is_some() {
            let item = it.next().unwrap()?;
            if !is_stoppable(&item) {
                lnum = item.lnum;
                col = item.token.last_char1();
            } else {
                stopped = true;
                lnum = item.lnum;
                col = item.token.last_char();
                count -= 1;
            }
        }

        Ok(XmapOutput {
            langle: visual_begin,
            rangle: pos![lnum, col],
            visualmode,
            prevent_change: if stopped { b"0" } else { b"1" },
        })
    }
}
