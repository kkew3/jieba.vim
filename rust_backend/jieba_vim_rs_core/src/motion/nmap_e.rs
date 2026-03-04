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

/// Test if a token is stoppable for `nmap_e`.
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
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`) in
    /// normal mode. Take in current `cursor_pos` (0, lnum, col, off, _), and
    /// return the new cursor position. We denote both `word` and `WORD` with
    /// the English word "word" below.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If there is no next word to the right of current cursor, jump to the
    ///   last character of the last token in the buffer. And the motion should
    ///   be taken as a failure.
    pub fn nmap_e<B: BufferLike + ?Sized>(
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
        let mut stopped = false;
        if count > 0 {
            let at_end = cursor_item.token.at_end(col);
            if is_stoppable(&cursor_item) && !at_end {
                // Call to last_char is valid because only word is stoppable.
                col = cursor_item.token.last_char();
                count -= 1;
                stopped = true;
            } else if !cursor_item.token.is_empty() && !at_end {
                // Call to last_char is valid because we have ensured
                // non-emptiness.
                col = cursor_item.token.last_char();
            }
        }

        while count > 0
            && let Some(item) = it.next().transpose()?
        {
            stopped = is_stoppable(&item);
            if stopped {
                // If item.token is a word ..
                lnum = item.lnum;
                col = item.token.last_char();
                count -= 1;
            } else if !item.token.is_empty() {
                // If item.token is a space ..
                lnum = item.lnum;
                col = item.token.last_char();
            } else {
                // If item.token is an Eol ..
                lnum = item.lnum;
                col = item.token.first_char();
            }
        }

        Ok(NmapOutput {
            cursor: pos![lnum, col],
            prevent_change: if stopped { b"0" } else { b"1" },
        })
    }
}
