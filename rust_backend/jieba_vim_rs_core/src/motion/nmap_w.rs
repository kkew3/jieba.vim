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
use crate::{BufferLike, CursorPositionCurswant, Position};

use super::parsed_buffer::ParsedBuffer;
use super::token_iter::{ExtendedInlineTokensIter, GToken};
use super::word_motion::{
    ExtendedMotionState, Markovian, MarkovianUnit, Motion, SemiTolerable,
    UnitMotion,
};
use super::{NmapOutput, WordMotion};

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
        count: u64,
        word: bool,
    ) -> Result<NmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let [bufnum, lnum, col, off, _] = cursor_pos;
        let mut cursor = [bufnum, lnum, col, off];
        let mut motion = Markovian::new(UnitNmapW);
        let s = motion.map(&mut buffer, count, &mut cursor)?;
        Ok(NmapOutput {
            cursor,
            prevent_change: s.into_prevent_change(),
        })
    }
}

pub struct UnitNmapW;

impl UnitMotion<Position> for UnitNmapW {
    fn unit_map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        buffer: &mut ParsedBuffer<'b, 'p, B, C>,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        let [_, lnum, col, off] = cursor;
        *off = 0;

        let n_lines = buffer.lines()?;
        let tokens = buffer.getline_parsed(*lnum)?;
        let mut line = ExtendedInlineTokensIter::new(&tokens)
            .skip_col(*col)
            .expect("col too large")
            .peekable();
        let cursor_token = line.next().unwrap();

        if *lnum == n_lines {
            match line.peek() {
                None => {
                    assert!(cursor_token.is_empty());
                    return Ok(ExtendedMotionState::Failure);
                }
                Some(next_t) => {
                    // If `next_t` exists, `cursor_token` can't be empty.
                    if cursor_token.at_end(*col) && next_t.is_empty() {
                        // If `cursor_token` is at eof and `col` is at end of
                        // `cursor_token` ..
                        //
                        // TODO we should be able to jump to the first_char of
                        // `next_t` in 'virtualedit' mode, and then return
                        // Failure.
                        return Ok(ExtendedMotionState::Failure);
                    }
                    if next_t.is_empty() {
                        // If `cursor_token` is at eof and `col` is *not* at
                        // end of `cursor_token` ..
                        let s = match cursor_token {
                            // As above, if `next_t` exists, `cursor_token`
                            // can't be empty.
                            GToken::Eol(_) => unreachable!(),
                            GToken::T(t) => {
                                *col = t.last_char();
                                ExtendedMotionState::Pending
                            }
                        };
                        return Ok(s);
                    }
                    // Till here, there must be at least one non-empty token
                    // after `cursor_token`, if we are in the last line of the
                    // buffer.
                }
            }
        }

        let s = match find_stop_point(line, col) {
            // `unwrap` is safe because `find_stop_point` return only empty
            // line or words.
            Some(t) => ExtendedMotionState::from_dest_token(t).unwrap(),
            None => loop {
                // `line` can't be empty when `lnum` == `n_lines`, as we have
                // covered above.
                if *lnum >= n_lines {
                    // Calling |w| on a Space at bof ..
                    break ExtendedMotionState::Pending;
                }
                *lnum += 1;
                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(&tokens);
                if let Some(t) = find_stop_point(line, col) {
                    break ExtendedMotionState::from_dest_token(t).unwrap();
                }
            },
        };
        Ok(s)
    }
}

impl MarkovianUnit<Position> for UnitNmapW {
    type FoldState = SemiTolerable;
}

fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
) -> Option<GToken> {
    for token in line {
        *col = token.first_char();
        match token {
            GToken::Eol(1) => {
                *col = token.first_char();
                return Some(token);
            }
            GToken::Eol(_) => *col = token.first_char(),
            GToken::T(t) => match t.ty {
                TokenType::Word => {
                    *col = token.first_char();
                    return Some(token);
                }
                TokenType::Space => *col = token.last_char(),
            },
        }
    }
    None
}
