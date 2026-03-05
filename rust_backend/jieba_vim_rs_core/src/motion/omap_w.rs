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
use super::{OmapOutput, WordMotion, d_special};

/// Test if a token is stoppable for `omap_w`.
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
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`)
    /// in operator-pending mode.
    ///
    /// Take in current `cursor_pos` (0, lnum, col, off, _), and return the
    /// operation range and the new cursor position. We denote both `word` and
    /// `WORD` with the English word "word" below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - Quoted from Vim's help section "WORD": "When using the `w` motion in
    ///   combination with an operator and the last word moved over is at the
    ///   end of a line, the end of that word becomes the end of the operated
    ///   text, not the first word in the next line."
    /// - Quoted from Vim's help section "WORD": "cw" and "cW" are treated like
    ///   "ce" and "cE" if the cursor is on a non-blank. This is because "cw"
    ///   is interpreted as change-word, and a word does not include the
    ///   following white space (see also cw).
    pub fn omap_w<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor: CursorPositionCurswant,
        mut count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let mut it =
            ForwardTokenIterator::new(buffer, &self.tokenizer, &cursor, word)?;
        let cursor_item = it.first();
        let [_, mut lnum, mut col, _, _] = cursor;
        if let GToken::T(t) = cursor_item.token
            && t.ty == TokenType::Word
            && operator == b"c"
        {
            return self.omap_e(buffer, cursor, count, word, operator);
        }

        let langle = pos![lnum, col];
        if !cursor_item.token.is_empty() {
            col = cursor_item.token.last_char();
        }

        // `last_stoppable_moved_over`: (last stoppable item, its rangle).
        let mut last_stoppable_moved_over = None;
        if is_stoppable(&cursor_item) {
            last_stoppable_moved_over = Some(if cursor_item.token.is_empty() {
                (
                    cursor_item,
                    pos![cursor_item.lnum, cursor_item.token.first_char()],
                )
            } else {
                (
                    cursor_item,
                    pos![cursor_item.lnum, cursor_item.token.last_char()],
                )
            });
        }

        while count > 0
            && let Some(item) = it.next().transpose()?
        {
            if !is_stoppable(&item) && item.token.is_empty() {
                lnum = item.lnum;
                col = item.token.first_char();
                if let Some((s_item, rangle)) =
                    last_stoppable_moved_over.as_mut()
                {
                    if s_item.lnum == item.lnum {
                        *rangle = pos![item.lnum, item.token.first_char()];
                    } else if s_item.lnum + 1 == item.lnum {
                        *rangle = pos![item.lnum, 1];
                    }
                }
            } else if !is_stoppable(&item) {
                lnum = item.lnum;
                col = item.token.last_char();
            } else {
                lnum = item.lnum;
                col = item.token.first_char();
                count -= 1;
                if item.token.is_empty() {
                    if let Some((s_item, rangle)) =
                        last_stoppable_moved_over.as_mut()
                    {
                        if s_item.lnum == item.lnum {
                            *rangle = pos![item.lnum, item.token.first_char()];
                        } else if s_item.lnum + 1 == item.lnum {
                            *rangle = pos![item.lnum, 1];
                        }
                    }
                }
                if count > 0 {
                    last_stoppable_moved_over =
                        Some(if item.token.is_empty() {
                            (item, pos![item.lnum, item.token.first_char()])
                        } else {
                            (item, pos![item.lnum, item.token.last_char()])
                        });
                } else if item.token.first_char() > 1 {
                    // If we will stop at item.token which is a word ..
                    last_stoppable_moved_over = None;
                }
            }
        }
        let rangle = match last_stoppable_moved_over {
            None => pos![lnum, col],
            Some((_, rangle)) => rangle,
        };
        if operator == b"d"
            && last_stoppable_moved_over.is_none()
            && d_special::is_d_special(
                buffer,
                &self.tokenizer,
                langle,
                rangle,
                false,
                word,
            )?
        {
            Ok(OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"V",
                selection: b"inclusive",
                prevent_change: b"0",
            })
        } else {
            Ok(OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"v",
                selection: if last_stoppable_moved_over
                    .is_some_and(|(item, _)| item.token.is_empty())
                {
                    &b"colon"[..]
                } else {
                    &b"exclusive"[..]
                },
                prevent_change: b"0",
            })
        }
    }
}
