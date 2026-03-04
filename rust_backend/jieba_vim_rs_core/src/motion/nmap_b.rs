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
use crate::{BufferLike, CursorPositionCurswant, pos};

use super::token_iter::{BackwardTokenIterator, GToken, TokenIteratorItem};
use super::{NmapOutput, WordMotion};

/// Test if a token is stoppable for `nmap_b`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    match item.token {
        GToken::Eol(1) => true,
        GToken::Eol(_) => false,
        GToken::T(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `b` (if `word` is `true`) or `B` (if `word` is `false`) in
    /// normal mode. Take in `cursor_pos` (0, lnum, col, off, _), and return
    /// the new cursor position. We denote both `word` and `WORD` with the
    /// English word "word" below.
    ///
    /// # Basics
    ///
    /// `b`/`B` jumps to the first character of previous word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made. And the motion should be
    ///   taken as a failure.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    pub fn nmap_b<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: CursorPositionCurswant,
        mut count: u64,
        word: bool,
    ) -> Result<NmapOutput, B::Error> {
        let [_, mut lnum, mut col, _, _] = cursor_pos;
        let mut it = BackwardTokenIterator::new(
            buffer,
            &self.tokenizer,
            &cursor_pos,
            word,
        )?;
        let cursor_item = it.first();
        let mut it = it.peekable();
        if count > 0 {
            if cursor_item.token.at_start(col) {
                if it.peek().is_none() {
                    return Ok(NmapOutput {
                        cursor: pos![1, 1],
                        prevent_change: b"1",
                    });
                }
            } else if is_stoppable(&cursor_item) {
                col = cursor_item.token.first_char();
                count -= 1;
            } else {
                col = cursor_item.token.first_char();
            }
        }

        while count > 0
            && let Some(item) = it.next().transpose()?
        {
            lnum = item.lnum;
            col = item.token.first_char();
            if is_stoppable(&item) {
                count -= 1;
            }
        }
        Ok(NmapOutput {
            cursor: pos![lnum, col],
            prevent_change: b"0",
        })
    }
}
