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

use super::token_iter::{
    ExtendedInlineTokensIter, GToken, ParsedBuffer, TokenLikeExt,
};
use super::word_motion::{
    AbsolutelyIntolerable, ExtendedMotionState, Markovian, MarkovianUnit,
    Motion, UnitMotion,
};
use super::{NmapOutput, WordMotion};

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
        count: u64,
        word: bool,
    ) -> Result<NmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let [bufnum, lnum, col, off, _] = cursor_pos;
        let mut cursor = [bufnum, lnum, col, off];
        let mut motion = Markovian::new(UnitNmapE);
        let s = motion.map(&mut buffer, count, &mut cursor)?;
        Ok(NmapOutput {
            cursor,
            prevent_change: s.into_prevent_change(),
        })
    }
}

pub struct UnitNmapE;

impl UnitMotion<Position> for UnitNmapE {
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
                    assert!(!cursor_token.is_empty());
                    if cursor_token.at_end(*col) && next_t.is_empty() {
                        // If `cursor_token` is at eof and `col` is at end of
                        // `cursor_token` ..
                        //
                        // TODO we should be able to jump to the first_char of
                        // `next_t` in 'virtualedit' mode, and then return
                        // Failure.
                        return Ok(ExtendedMotionState::Failure);
                    }
                    if next_t.is_empty()
                        && let GToken::T(t) = &cursor_token
                    {
                        *col = t.last_char();
                        let s = match t.ty {
                            TokenType::Space => ExtendedMotionState::Pending,
                            TokenType::Word => ExtendedMotionState::Success,
                        };
                        return Ok(s);
                    }
                }
            }
        }

        if let GToken::T(t) = &cursor_token
            && t.ty == TokenType::Word
            && !t.at_end(*col)
        {
            *col = t.last_char();
            return Ok(ExtendedMotionState::Success);
        }

        fn is_not_non_empty_line_eol(token: &GToken) -> bool {
            match token {
                GToken::T(_) => true,
                GToken::Eol(1) => true,
                GToken::Eol(_) => false,
            }
        }

        // We need to filter out non-empty line Eol because they occur after
        // all other tokens in a line, and thus prevents us from collecting the
        // last regular token (or empty line Eol) of each line.
        let line = line.filter(is_not_non_empty_line_eol);
        let s = match find_stop_point(line, col) {
            // `find_stop_point` only return Ok(Word).
            Ok(_) => ExtendedMotionState::Success,
            Err(mut last_t) => loop {
                if *lnum >= n_lines {
                    // If `lnum` == n_lines, `last_t` can't be None; and if
                    // `last_t` is None, `lnum` can't be `n_lines`.
                    let s = match last_t.unwrap() {
                        GToken::T(t) => match t.ty {
                            // If `last_t` were a word, it should already
                            // be captured by `find_stop_point`.
                            TokenType::Word => unreachable!(),
                            TokenType::Space => ExtendedMotionState::Pending,
                        },
                        GToken::Eol(1) => ExtendedMotionState::SemiFailure,
                        // Due to the filtering.
                        GToken::Eol(_) => unreachable!(),
                    };
                    break s;
                }
                *lnum += 1;
                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(&tokens)
                    .filter(is_not_non_empty_line_eol);
                match find_stop_point(line, col) {
                    // `find_stop_point` only return Ok(Word).
                    Ok(_) => break ExtendedMotionState::Success,
                    Err(err) => last_t = err,
                }
            },
        };
        Ok(s)
    }
}

impl MarkovianUnit<Position> for UnitNmapE {
    type FoldState = AbsolutelyIntolerable;
}

/// Return either the stop point (a Word), or the last token yielded by `line`.
fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
) -> Result<GToken, Option<GToken>> {
    let mut last_token = None;
    for token in line {
        last_token = Some(token);
        match token {
            GToken::T(t) => {
                *col = t.last_char();
                if t.ty == TokenType::Word {
                    return Ok(token);
                }
            }
            GToken::Eol(1) => *col = 1,
            GToken::Eol(_) => (),
        }
    }
    Err(last_token)
}
