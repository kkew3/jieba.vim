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

/// Move back to the end of `count` words.
pub struct BackwardEndWord(Markovian<UnitBackwardEndWord>);

impl BackwardEndWord {
    /// Construct a new [`BackwardEndWord`]. Pass true to `eol` to stop at
    /// Eol(_).
    pub fn new(eol: bool) -> Self {
        Self(Markovian::new(UnitBackwardEndWord { eol }))
    }
}

impl Motion<Position> for BackwardEndWord {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        self.0.map(buffer, count, cursor)
    }
}

struct UnitBackwardEndWord {
    /// True to stop at Eol.
    eol: bool,
}

impl UnitMotion<Position> for UnitBackwardEndWord {
    fn unit_map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        let Position { lnum, col, off } = cursor;
        *off = 0;

        // Quick path.
        if *lnum == 1 && *col == 1 {
            return Ok(ExtendedMotionState::Failure);
        }

        let tokens = buffer.getline_parsed(*lnum)?;
        let mut line = ExtendedInlineTokensIter::new(tokens)
            .take_col_rev(*col)
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

        let s = match find_stop_point(line, col, self.eol) {
            // `unwrap` is safe because `t` can only be words.
            Some(GToken::T(t)) => {
                ExtendedMotionState::from_dest_token(GToken::T(t)).unwrap()
            }
            Some(GToken::Eol(1)) => {
                ExtendedMotionState::from_dest_token(GToken::Eol(1)).unwrap()
            }
            Some(GToken::Eol(_)) => ExtendedMotionState::Success,
            None => loop {
                // `line` can't be empty when `lnum` == 1, as we have covered
                // above.
                if *lnum <= 1 {
                    // Calling |ge| on a Space at bof ..
                    break ExtendedMotionState::Pending;
                }
                *lnum -= 1;
                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(tokens).rev();
                if let Some(t) = find_stop_point(line, col, self.eol) {
                    let s = match t {
                        // `unwrap` is safe because `t` can only be words
                        // when it's non-empty.
                        GToken::T(_) | GToken::Eol(1) => {
                            ExtendedMotionState::from_dest_token(t).unwrap()
                        }
                        GToken::Eol(_) => ExtendedMotionState::Success,
                    };
                    break s;
                }
            },
        };
        Ok(s)
    }
}

impl MarkovianUnit<Position> for UnitBackwardEndWord {
    type FoldState = Tolerable;
}

/// If `eol` is true, Eol(_) will also become a stop point.
fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
    eol: bool,
) -> Option<GToken> {
    for token in line {
        match token {
            GToken::Eol(1) => {
                *col = token.first_char();
                return Some(token);
            }
            GToken::Eol(_) => {
                if eol {
                    *col = token.first_char();
                    return Some(token);
                }
            }
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

/// A combination of `Decl + BackwardEndWord` in one step.
pub struct DeclBackwardEndWord {
    decl: Decl,
    bckend: BackwardEndWord,
}

impl Motion<Position> for DeclBackwardEndWord {
    /// Panics if `count` is not 1.
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        assert_eq!(count, 1);
        if cursor.lnum == 1 && cursor.col == 1 {
            return Ok(MotionState::Failure);
        }

        let cursor_token =
            ExtendedInlineTokensIter::new(buffer.getline_parsed(cursor.lnum)?)
                .into_col(cursor.col);
        let need_decl = match cursor_token {
            GToken::T(t) => cursor.col <= t.first_char1(),
            GToken::Eol(_) => true,
        };
        if need_decl {
            // Must return success since we have tested above that we are not
            // at bof.
            self.decl.map(buffer, 1, cursor)?;
        }
        self.bckend.map(buffer, 1, cursor)
    }
}

impl Chain<BackwardEndWord> for Decl {
    type Output = DeclBackwardEndWord;

    fn chain(self, rhs: BackwardEndWord) -> Self::Output {
        DeclBackwardEndWord {
            decl: self,
            bckend: rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backward_end_word_count1_eol() -> Result<(), ()> {
        let mut bckend = BackwardEndWord::new(true);
        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![1..4 as Word, 4..6 as Space, 6..9 as Word],
                atoken_vec![1..2 as Space, 2..5 as Word],
            ],
        );
        assert_move!(bckend, b: (1, 1) => Failure);
        assert_move!(bckend, b: (1, 2) => (1, 1));
        assert_move!(bckend, b: (1, 4) => (1, 3));
        assert_move!(bckend, b: (1, 5) => (1, 3));
        assert_move!(bckend, b: (1, 6) => (1, 3));
        assert_move!(bckend, b: (1, 7) => (1, 3));
        assert_move!(bckend, b: (1, 8) => (1, 3));
        assert_move!(bckend, b: (1, 9) => (1, 8));
        assert_move!(bckend, b: (2, 1) => (1, 9));
        assert_move!(bckend, b: (2, 2) => (1, 9));
        assert_move!(bckend, b: (2, 3) => (1, 9));
        assert_move!(bckend, b: (2, 5) => (2, 4));

        Ok(())
    }

    #[test]
    fn test_decl_backward_end_word() -> Result<(), ()> {
        let mut db = Decl::default().chain(BackwardEndWord::new(true));
        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![1..4 as Word, 4..6 as Space, 6..9 as Word],
                atoken_vec![1..2 as Space, 2..5 as Word],
            ],
        );
        assert_move!(db, b: (1, 1) => Failure);
        assert_move!(db, b: (1, 2) => Failure (1, 1));
        assert_move!(db, b: (1, 3) => (1, 1));
        assert_move!(db, b: (1, 4) => (1, 1));
        assert_move!(db, b: (1, 5) => (1, 3));
        assert_move!(db, b: (1, 6) => (1, 3));
        assert_move!(db, b: (1, 7) => (1, 3));
        assert_move!(db, b: (1, 8) => (1, 3));
        assert_move!(db, b: (1, 9) => (1, 3));
        assert_move!(db, b: (2, 1) => (1, 3));
        assert_move!(db, b: (2, 2) => (1, 9));
        assert_move!(db, b: (2, 3) => (1, 9));
        assert_move!(db, b: (2, 4) => (1, 9));
        assert_move!(db, b: (2, 5) => (1, 9));

        Ok(())
    }
}
