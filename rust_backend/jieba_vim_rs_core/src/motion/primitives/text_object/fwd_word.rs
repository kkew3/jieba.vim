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

/// Move forward `count` words.
pub struct ForwardWord {
    /// True to stop at Eol in the last motion.
    eol: bool,
}

impl ForwardWord {
    /// Construct a new [`ForwardWord`]. Pass true to `eol` to stop at
    /// Eol in the last motion.
    pub fn new(eol: bool) -> Self {
        Self { eol }
    }
}

impl Motion<Position> for ForwardWord {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        mut count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        let mut state = SemiTolerable::default();
        while count > 0 {
            count -= 1;
            let mut unit_motion = UnitForwardWord {
                eol: self.eol && count == 0,
            };
            if let Some(absorbing_state) =
                state.update(unit_motion.unit_map(buffer, cursor)?)
            {
                return Ok(absorbing_state);
            }
        }
        Ok(state.finalize())
    }
}

/// Note that this is a [`UnitMotion`], but not a
/// [`MarkovianUnit`](crate::motion::core::motion::MarkovianUnit).
struct UnitForwardWord {
    /// True to stop at Eol in *this* motion.
    eol: bool,
}

impl UnitMotion<Position> for UnitForwardWord {
    fn unit_map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        let Position { lnum, col, off } = cursor;
        *off = 0;

        let n_lines = buffer.lines()?;
        let tokens = buffer.getline_parsed(*lnum)?;
        let mut line = ExtendedInlineTokensIter::new(tokens)
            .skip_col(*col)
            .peekable();
        let cursor_token = line.next().unwrap();
        let cursor_token_is_eol = cursor_token.is_empty();

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
                        // If `cursor_token` is at eof and `col` is at end
                        // of `cursor_token`, jump to the Eol(_) and return
                        // Failure.
                        *col = next_t.first_char();
                        return Ok(ExtendedMotionState::Failure);
                    }
                    if next_t.is_empty() {
                        // If `cursor_token` is at eof and `col` is *not*
                        // at end of `cursor_token`, do the same, but
                        // return Pending.
                        *col = next_t.first_char();
                        return Ok(ExtendedMotionState::Pending);
                    }
                    // Till here, there must be at least one non-empty
                    // token after `cursor_token`, if we are in the last
                    // line of the buffer.
                }
            }
        }

        let s = match find_stop_point(line, col, self.eol) {
            Some(GToken::T(t)) => {
                ExtendedMotionState::from_dest_token(GToken::T(t)).unwrap()
            }
            // The stop point must be after `cursor_token`, so it can't be
            // Eol(1).
            Some(GToken::Eol(1)) => unreachable!(),
            // This branch will be reached only if self.eol is true.
            Some(GToken::Eol(_)) => ExtendedMotionState::Success,
            None => loop {
                // `line` can't be empty when `lnum` == `n_lines` at the
                // very beginning, as we have covered above.
                if *lnum >= n_lines {
                    // Calling |w| on a Space at eof ..
                    break ExtendedMotionState::Pending;
                }
                *lnum += 1;

                if cursor_token_is_eol && self.eol {
                    *col = 1;
                    break ExtendedMotionState::Success;
                }

                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(tokens);
                match find_stop_point(line, col, self.eol) {
                    Some(GToken::T(t)) => {
                        break ExtendedMotionState::from_dest_token(GToken::T(
                            t,
                        ))
                        .unwrap();
                    }
                    Some(GToken::Eol(1)) => {
                        break ExtendedMotionState::from_dest_token(
                            GToken::Eol(1),
                        )
                        .unwrap();
                    }
                    // This branch will be reached only if self.eol is
                    // true.
                    Some(GToken::Eol(_)) => {
                        break ExtendedMotionState::Success;
                    }
                    None => (),
                }
            },
        };
        Ok(s)
    }
}

/// If `eol` is true, Eol(_) will also become a stop point.
fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
    eol: bool,
) -> Option<GToken> {
    for token in line {
        *col = token.first_char();
        match token {
            GToken::Eol(1) => {
                return Some(token);
            }
            GToken::Eol(_) => {
                if eol {
                    return Some(token);
                }
            }
            GToken::T(t) => match t.ty {
                TokenType::Word => {
                    return Some(token);
                }
                TokenType::Space => *col = token.last_char(),
            },
        }
    }
    None
}

/// A combination of `Incl + ForwardWord`.
pub struct InclForwardWord {
    incl: Incl,
    fwd: ForwardWord,
}

impl Motion<Position> for InclForwardWord {
    /// Panics if `count` is not 1.
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        assert_eq!(count, 1);

        let mut line =
            ExtendedInlineTokensIter::new(buffer.getline_parsed(cursor.lnum)?)
                .skip_col(cursor.col);
        let cursor_token = line.next().unwrap();
        let need_incl = match cursor_token {
            GToken::Eol(_) => true,
            GToken::T(t) => t.at_end(cursor.col),
        };
        if need_incl {
            if self.incl.map(buffer, 1, cursor)? == MotionState::Failure {
                return Ok(MotionState::Failure);
            }
        }
        self.fwd.map(buffer, 1, cursor)
    }
}

impl Chain<ForwardWord> for Incl {
    type Output = InclForwardWord;

    fn chain(self, rhs: ForwardWord) -> Self::Output {
        InclForwardWord {
            incl: self,
            fwd: rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_word_count1_noeol() -> Result<(), ()> {
        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![], atoken_vec![1..4 as Space]],
        );
        let mut f = ForwardWord::new(false);
        assert_move!(f, b: (1, 1) => (2, 4));

        Ok(())
    }

    #[test]
    fn test_forward_word_count1_eol() -> Result<(), ()> {
        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![], atoken_vec![1..4 as Space]],
        );
        let mut f = ForwardWord::new(true);
        assert_move!(f, b: (1, 1) => (2, 1));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![1..4 as Word, 4..6 as Space, 6..9 as Word],
                atoken_vec![1..2 as Space, 2..5 as Word],
            ],
        );
        assert_move!(f, b: (1, 1) => (1, 6));
        assert_move!(f, b: (1, 3) => (1, 6));
        assert_move!(f, b: (1, 5) => (1, 6));
        assert_move!(f, b: (1, 6) => (1, 9));
        assert_move!(f, b: (1, 8) => (1, 9));
        assert_move!(f, b: (1, 9) => (2, 1));
        assert_move!(f, b: (2, 1) => (2, 2));
        assert_move!(f, b: (2, 2) => (2, 5));
        assert_move!(f, b: (2, 3) => (2, 5));
        assert_move!(f, b: (2, 4) => Failure (2, 5));
        assert_move!(f, b: (2, 5) => Failure);

        Ok(())
    }

    #[test]
    fn test_incl_forward_word() -> Result<(), ()> {
        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![],
                atoken_vec![1..2 as Space],
                atoken_vec![],
                atoken_vec![1..5 as Word, 5..7 as Space],
                atoken_vec![1..2 as Space],
                atoken_vec![],
            ],
        );
        let mut inf = Incl::default().chain(ForwardWord::new(true));
        assert_move!(inf, b: (1, 1) => (2, 2));
        assert_move!(inf, b: (2, 1) => (4, 1));
        assert_move!(inf, b: (2, 2) => (4, 1));
        assert_move!(inf, b: (3, 1) => (4, 7));
        assert_move!(inf, b: (4, 1) => (4, 7));
        assert_move!(inf, b: (4, 2) => (4, 7));
        assert_move!(inf, b: (4, 3) => (4, 7));
        assert_move!(inf, b: (4, 4) => (4, 7));
        assert_move!(inf, b: (4, 5) => (4, 7));
        assert_move!(inf, b: (4, 6) => (5, 2));
        assert_move!(inf, b: (4, 7) => (5, 2));
        assert_move!(inf, b: (5, 1) => Failure (6, 1));
        assert_move!(inf, b: (5, 2) => Failure (6, 1));
        assert_move!(inf, b: (6, 1) => Failure);

        Ok(())
    }
}
