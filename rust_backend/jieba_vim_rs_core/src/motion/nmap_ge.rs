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

use crate::token::{JiebaPlaceholder, TokenLike, TokenType};
use crate::{BufferLike, CursorPositionCurswant, Position};

use super::token_iter::{ExtendedInlineTokensIter, GToken, ParsedBuffer};
use super::word_motion::{
    ExtendedMotionState, Markovian, MarkovianUnit, Motion, SemiTolerable,
    UnitMotion,
};
use super::{NmapOutput, WordMotion};

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `ge` (if `word` is `true`) or `gE` (if `word` is `false`)
    /// in normal mode. Take in `cursor_pos` (0, lnum, col, off, _), and return
    /// the new cursor position. We denote both `word` and `WORD` with the
    /// English word "word" below.
    ///
    /// # Basics
    ///
    /// `ge`/`gE` jumps to the last character of previous word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made.  And the motion should be
    ///   taken as a failure.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    pub fn nmap_ge<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: CursorPositionCurswant,
        count: u64,
        word: bool,
    ) -> Result<NmapOutput, B::Error> {
        let buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let [bufnum, lnum, col, off, _] = cursor_pos;
        let mut cursor = [bufnum, lnum, col, off];
        let mut motion = Markovian::new(UnitNmapGe);
        let s = motion.map(&buffer, count, &mut cursor)?;
        Ok(NmapOutput {
            cursor,
            prevent_change: s.into_prevent_change(),
        })
    }
}

pub struct UnitNmapGe;

impl UnitMotion<Position> for UnitNmapGe {
    fn unit_map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        buffer: &ParsedBuffer<'b, 'p, B, C>,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        let [_, lnum, col, off] = cursor;
        *off = 0;

        // Quick path.
        if *lnum == 1 && *col == 1 {
            return Ok(ExtendedMotionState::Failure);
        }

        let tokens = buffer.getline_parsed(*lnum)?;
        let mut line = ExtendedInlineTokensIter::new(&tokens)
            .take_col_rev(*col)
            .expect("col too large")
            .peekable();
        let cursor_token = line.next().unwrap();

        if *lnum == 1 && line.peek().is_none() {
            let s = match &cursor_token {
                // cursor_token can't be Eol, since it would result in col ==
                // 1, but we have tested for col == 1 above.
                GToken::Eol(_) => unreachable!(),
                GToken::T(t) => {
                    // If we are at a regular token at bof ..

                    // We have tested that col > 1 above.
                    *col = 1;
                    match t.ty {
                        TokenType::Space => ExtendedMotionState::Pending,
                        TokenType::Word => ExtendedMotionState::Success,
                    }
                }
            };
            return Ok(s);
        }

        let s = match find_stop_point(line, col) {
            // `unwrap` is safe because `find_stop_point` return only empty
            // line or words.
            Some(t) => ExtendedMotionState::from_dest_token(t).unwrap(),
            None => loop {
                // `line` can't be empty when `lnum` == 1, as we have covered
                // above.
                if *lnum <= 1 {
                    // Calling |ge| on a Space at bof ..
                    break ExtendedMotionState::Pending;
                }
                *lnum -= 1;
                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(&tokens).rev();
                if let Some(t) = find_stop_point(line, col) {
                    break ExtendedMotionState::from_dest_token(t).unwrap();
                }
            },
        };
        Ok(s)
    }
}

impl MarkovianUnit<Position> for UnitNmapGe {
    type FoldState = SemiTolerable;
}

fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
) -> Option<GToken> {
    for token in line {
        match token {
            GToken::Eol(1) => {
                *col = token.first_char();
                return Some(token);
            }
            GToken::Eol(_) => (),
            GToken::T(t) => match t.ty {
                TokenType::Space => *col = t.first_char(),
                TokenType::Word => {
                    *col = t.last_char();
                    return Some(token);
                }
            },
        }
    }
    None
}
