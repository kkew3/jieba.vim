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
            Some(GToken::Eol(1)) => {
                ExtendedMotionState::from_dest_token(GToken::Eol(1)).unwrap()
            }
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
                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(&tokens);
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
