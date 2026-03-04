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

use super::token_iter::{ForwardTokenIterator, GToken, TokenIteratorItem};
use super::{NmapOutput, WordMotion};

/// Test if a token is stoppable for `nmap_w`.
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
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`) in
    /// normal mode. Take in current `cursor_pos` (0, lnum, col, off, _), and
    /// return the new cursor position. We denote both `word` and `WORD` with
    /// the English word "word" below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the last character of the last token in the
    ///   buffer, no further jump should be made. And the motion should be
    ///   taken as a failure.
    /// - If there is no next word to the right of current cursor, jump to the
    ///   last character of the last token in the buffer.
    pub fn nmap_w<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: CursorPositionCurswant,
        mut count: u64,
        word: bool,
    ) -> Result<NmapOutput, B::Error> {
        let [_, mut lnum, mut col, _, _] = cursor_pos;
        let mut it = ForwardTokenIterator::new(
            buffer,
            &self.tokenizer,
            &cursor_pos,
            word,
        )?;
        let cursor_item = it.first();
        // Skip all non-stoppable Eol tokens.
        let mut it = it
            .filter(|res| {
                !res.as_ref().is_ok_and(|item| {
                    item.token.is_empty() && !is_stoppable(item)
                })
            })
            .peekable();
        let mut moved = false;
        if !cursor_item.token.is_empty() && !cursor_item.token.at_end(col) {
            col = cursor_item.token.last_char();
            moved = true;
        }

        while count > 0 && it.peek().is_some() {
            let item = it.next().unwrap()?;
            if !is_stoppable(&item) {
                lnum = item.lnum;
                if item.token.is_empty() {
                    col = item.token.first_char();
                } else {
                    col = item.token.last_char();
                    moved = true;
                }
            } else {
                moved = true;
                lnum = item.lnum;
                col = item.token.first_char();
                count -= 1;
                if count > 0 && it.peek().is_none() && !item.token.is_empty() {
                    col = item.token.last_char();
                }
            }
        }
        Ok(NmapOutput {
            cursor: pos![lnum, col],
            prevent_change: if moved { b"0" } else { b"1" },
        })
    }
}
