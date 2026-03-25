// Copyright 2024-2026 Kaiwen Wu. All Rights Reserved.
// Portions Copyright (c) by Bram Moolenaar and others.
//
// This file contains code adapted from Vim's textobject.c. The Vim License
// applies to the adapted portions. See the vim-LICENSE.txt file in the project
// root for the full license text.
//
// In accordance with the Vim License (Section II):
// - Contact: Kaiwen Wu <kps6326@hotmail.com>
// - Changes are available to the Vim maintainer upon request.
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

use super::*;

/// Move to the end of `count` words.
pub struct EndWord {
    /// True to move one less if we are already on the end of a word.
    stop: bool,
    /// True to stop on empty lines.
    empty: bool,
}

impl EndWord {
    /// Construct a new [`EndWord`]. Pass true to `stop` to move one less
    /// if we are already on the end of a word. Pass true to `empty` to
    /// stop on empty lines.
    pub fn new(stop: bool, empty: bool) -> Self {
        Self { stop, empty }
    }
}

impl Motion<Position> for EndWord {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        let mut motion = Markovian::new(UnitEndWord {
            stop: self.stop,
            empty: self.empty,
        });
        motion.map(buffer, count, cursor)
    }
}

/// Note that this is a [`UnitMotion`], but not a
/// [`MarkovianUnit`](crate::motion::core::motion::MarkovianUnit).
struct UnitEndWord {
    /// True to not move at all if we are already on the end of a word.
    stop: bool,
    /// True to stop on empty lines.
    empty: bool,
}

impl UnitMotion<Position> for UnitEndWord {
    fn unit_map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        let Position { lnum, col, off } = cursor;
        *off = 0;

        let stop = self.stop;
        self.stop = false;

        let n_lines = buffer.lines()?;
        let tokens = buffer.getline_parsed(*lnum)?;
        let mut line = ExtendedInlineTokensIter::new(tokens)
            .skip_col(*col)
            .peekable();
        let cursor_token = line.next().unwrap();

        /// Handle the case where `cursor_token` is the last regular token
        /// before Eol(_) at eof.
        fn last_line_eof_case(
            cursor_token: &Token,
            col: &mut usize,
            stop: bool,
        ) -> ExtendedMotionState {
            match cursor_token.ty {
                TokenType::Word => {
                    if cursor_token.at_end(*col) {
                        if stop {
                            ExtendedMotionState::Success
                        } else {
                            *col = cursor_token.last_char1();
                            ExtendedMotionState::Failure
                        }
                    } else {
                        *col = cursor_token.last_char();
                        ExtendedMotionState::Success
                    }
                }
                TokenType::Space => {
                    *col = cursor_token.last_char1();
                    ExtendedMotionState::Pending
                }
            }
        }

        if *lnum == n_lines {
            match line.peek() {
                None => {
                    assert!(cursor_token.is_empty());
                    return Ok(ExtendedMotionState::Failure);
                }
                Some(next_t) => {
                    if next_t.is_empty() {
                        let s = match &cursor_token {
                            GToken::T(cursor_token) => {
                                last_line_eof_case(cursor_token, col, stop)
                            }
                            // If `next_t` exists, `cursor_token` can't be
                            // empty.
                            GToken::Eol(_) => unreachable!(),
                        };
                        return Ok(s);
                    }
                }
            }
        }

        if let GToken::T(t) = &cursor_token
            && t.ty == TokenType::Word
        {
            if !t.at_end(*col) {
                *col = t.last_char();
                return Ok(ExtendedMotionState::Success);
            }
            if stop {
                return Ok(ExtendedMotionState::Success);
            }
        }

        fn is_not_non_empty_line_eol(token: &GToken) -> bool {
            match token {
                GToken::T(_) => true,
                GToken::Eol(1) => true,
                GToken::Eol(_) => false,
            }
        }

        // We need to filter out non-empty line Eol because they occur
        // after all other tokens in a line, and thus prevents us from
        // collecting the last regular token (or empty line Eol(1)) of
        // each line.
        let line = line.filter(is_not_non_empty_line_eol);
        let s = match find_stop_point(line, col, self.empty) {
            Ok(GToken::T(_)) => ExtendedMotionState::Success,
            // This branch will be reached only if `self.empty` is true.
            Ok(GToken::Eol(_)) => ExtendedMotionState::Success,
            Err(mut last_t) => loop {
                if *lnum >= n_lines {
                    // If `lnum` == `n_lines`, `last_t` can't be None; and
                    // if `last_t` is None, `lnum` can't be `n_lines`.
                    let s = match last_t.unwrap() {
                        GToken::T(t) => match t.ty {
                            // If `last_t` were a word, it should already
                            // be captured by `find_stop_point`.
                            TokenType::Word => unreachable!(),
                            TokenType::Space => {
                                *col = t.last_char1();
                                ExtendedMotionState::Pending
                            }
                        },
                        // This branch will be reached only if `self.empty` is
                        // false.
                        GToken::Eol(1) => ExtendedMotionState::SemiFailure,
                        // Due to the filtering.
                        GToken::Eol(_) => unreachable!(),
                    };
                    break s;
                }
                *lnum += 1;
                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(tokens)
                    .filter(is_not_non_empty_line_eol);
                match find_stop_point(line, col, self.empty) {
                    Ok(GToken::T(_)) => break ExtendedMotionState::Success,
                    // This branch will be reached only if `self.empty`
                    // is true.
                    Ok(GToken::Eol(_)) => {
                        break ExtendedMotionState::Success;
                    }
                    Err(curr_last_t) => last_t = curr_last_t,
                }
            },
        };
        Ok(s)
    }
}

impl MarkovianUnit<Position> for UnitEndWord {
    type FoldState = Intolerable;
}

/// Return either the stop point (a Word), or the last token yielded by
/// `line`. When `empty` is true, the stop point may also be an Eol(1).
fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
    empty: bool,
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
            GToken::Eol(1) => {
                *col = 1;
                if empty {
                    return Ok(token);
                }
            }
            GToken::Eol(_) => (),
        }
    }
    Err(last_token)
}

/// A combination of `Incl + EndWord`.
pub struct InclEndWord {
    incl: Incl,
    end: EndWord,
}

impl Motion<Position> for InclEndWord {
    /// Panics if `count` is not 1.
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        assert_eq!(count, 1);
        if !self.end.stop {
            unimplemented!();
        }

        let mut line =
            ExtendedInlineTokensIter::new(buffer.getline_parsed(cursor.lnum)?)
                .skip_col(cursor.col);
        let cursor_token = line.next().unwrap();
        let need_incl = match cursor_token {
            GToken::T(t) => match t.ty {
                TokenType::Word => {
                    if !t.at_end(cursor.col) {
                        cursor.col = t.last_char();
                        return Ok(MotionState::Success);
                    }
                    true
                }
                TokenType::Space => t.at_end(cursor.col),
            },
            GToken::Eol(_) => true,
        };
        if need_incl {
            if self.incl.map(buffer, 1, cursor)? == MotionState::Failure {
                return Ok(MotionState::Failure);
            }
        }
        self.end.map(buffer, 1, cursor)
    }
}

impl Chain<EndWord> for Incl {
    type Output = InclEndWord;

    fn chain(self, rhs: EndWord) -> Self::Output {
        InclEndWord {
            incl: self,
            end: rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_word_count1_stop_empty() -> Result<(), ()> {
        let mut e = EndWord::new(true, true);
        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![1..4 as Word, 4..6 as Space, 6..9 as Word],
                atoken_vec![1..3 as Space],
                atoken_vec![],
                atoken_vec![1..2 as Space, 2..5 as Word],
            ],
        );
        assert_move!(e, b: (1, 1) => (1, 3));
        assert_move!(e, b: (1, 2) => (1, 3));
        assert_move!(e, b: (1, 3) => (1, 3));
        assert_move!(e, b: (1, 4) => (1, 8));
        assert_move!(e, b: (1, 6) => (1, 8));
        assert_move!(e, b: (1, 7) => (1, 8));
        assert_move!(e, b: (1, 8) => (1, 8));
        assert_move!(e, b: (1, 9) => (3, 1));
        assert_move!(e, b: (2, 1) => (3, 1));
        assert_move!(e, b: (2, 3) => (3, 1));
        assert_move!(e, b: (3, 1) => (4, 4));
        assert_move!(e, b: (4, 1) => (4, 4));
        assert_move!(e, b: (4, 3) => (4, 4));
        assert_move!(e, b: (4, 4) => (4, 4));
        assert_move!(e, b: (4, 5) => Failure);

        Ok(())
    }

    #[test]
    fn test_incl_end_word() -> Result<(), ()> {
        let mut ie = Incl::default().chain(EndWord::new(true, true));
        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![1..4 as Word, 4..6 as Space, 6..9 as Word],
                atoken_vec![1..3 as Space],
                atoken_vec![],
                atoken_vec![1..2 as Space, 2..5 as Word],
            ],
        );
        assert_move!(ie, b: (1, 1) => (1, 3));
        assert_move!(ie, b: (1, 2) => (1, 3));
        assert_move!(ie, b: (1, 3) => (1, 8));
        assert_move!(ie, b: (1, 4) => (1, 8));
        assert_move!(ie, b: (1, 5) => (1, 8));
        assert_move!(ie, b: (1, 6) => (1, 8));
        assert_move!(ie, b: (1, 7) => (1, 8));
        assert_move!(ie, b: (1, 8) => (3, 1));
        assert_move!(ie, b: (1, 9) => (3, 1));
        assert_move!(ie, b: (2, 1) => (3, 1));
        assert_move!(ie, b: (2, 2) => (4, 4));
        assert_move!(ie, b: (2, 3) => (4, 4));
        assert_move!(ie, b: (3, 1) => (4, 4));
        assert_move!(ie, b: (4, 1) => (4, 4));
        assert_move!(ie, b: (4, 2) => (4, 4));
        assert_move!(ie, b: (4, 3) => (4, 4));
        assert_move!(ie, b: (4, 4) => Failure (4, 5));
        assert_move!(ie, b: (4, 5) => Failure);

        Ok(())
    }
}
