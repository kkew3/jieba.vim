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

/// Move backward `count` words.
pub struct BackwardWord {
    /// True to move one less if we are already on the start of a word.
    stop: bool,
}

impl BackwardWord {
    /// Construct a new [`BackwardWord`]. Pass true to `stop` to move one
    /// less if we are already on the start of a word.
    pub fn new(stop: bool) -> Self {
        Self { stop }
    }
}

impl Motion<Position> for BackwardWord {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        let mut motion = Markovian::new(UnitBackwardWord { stop: self.stop });
        motion.map(buffer, count, cursor)
    }
}

struct UnitBackwardWord {
    /// True to not move at all if we are already on the start of a word.
    stop: bool,
}

impl UnitMotion<Position> for UnitBackwardWord {
    fn unit_map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        let stop = self.stop;
        self.stop = false;

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
        // `unwrap` is safe because `take_col_rev` yields at least one
        // item.
        let cursor_token = line.next().unwrap();

        if *lnum == 1 && line.peek().is_none() {
            let s = match &cursor_token {
                // cursor_token can't be Eol, since it would result in col
                // == 1, but we have tested for col == 1 above.
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

        if let GToken::T(t) = &cursor_token
            && t.ty == TokenType::Word
        {
            if !t.at_start(*col) {
                *col = t.first_char();
                return Ok(ExtendedMotionState::Success);
            }
            if stop {
                return Ok(ExtendedMotionState::Success);
            }
        }

        let s = match find_stop_point(line, col) {
            // `unwrap` is safe because `find_stop_point` return only empty
            // line or words.
            Some(t) => ExtendedMotionState::from_dest_token(t).unwrap(),
            None => loop {
                // `line` can't be empty when `lnum` == 1, as we have
                // covered above.
                if *lnum <= 1 {
                    // Calling |b| on a Space at bof ..
                    break ExtendedMotionState::Pending;
                }
                *lnum -= 1;
                let tokens = buffer.getline_parsed(*lnum)?;
                let line = ExtendedInlineTokensIter::new(tokens).rev();
                if let Some(t) = find_stop_point(line, col) {
                    // `unwrap` is safe because `find_stop_point` return
                    // only empty line or words.
                    break ExtendedMotionState::from_dest_token(t).unwrap();
                }
            },
        };
        Ok(s)
    }
}

impl MarkovianUnit<Position> for UnitBackwardWord {
    type FoldState = Tolerable;
}

/// Test if a token is stoppable.
fn is_stoppable(token: &GToken) -> bool {
    match token {
        GToken::Eol(1) => true,
        GToken::Eol(_) => false,
        GToken::T(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
) -> Option<GToken> {
    for token in line {
        *col = token.first_char();
        if is_stoppable(&token) {
            return Some(token);
        }
    }
    None
}

/// A combination of `Decl + BackwardWord`.
pub struct DeclBackwardWord {
    decl: Decl,
    bck: BackwardWord,
}

impl Motion<Position> for DeclBackwardWord {
    /// Panics if `count` is not 1.
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        assert_eq!(count, 1);
        if !self.bck.stop {
            unimplemented!();
        }

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
        self.bck.map(buffer, 1, cursor)
    }
}

impl Chain<BackwardWord> for Decl {
    type Output = DeclBackwardWord;

    fn chain(self, rhs: BackwardWord) -> Self::Output {
        DeclBackwardWord {
            decl: self,
            bck: rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backward_word_count1_stop() -> Result<(), ()> {
        let mut bck = BackwardWord::new(true);

        let mut b = PreTokenizedBuffer::new(1, vec![atoken_vec![]]);
        assert_move!(bck, b: (1, 1) => Failure);

        let mut b =
            PreTokenizedBuffer::new(1, vec![atoken_vec![1..4 as Space]]);
        assert_move!(bck, b: (1, 1) => Failure);
        assert_move!(bck, b: (1, 2) => (1, 1));
        assert_move!(bck, b: (1, 3) => (1, 1));
        assert_move!(bck, b: (1, 4) => (1, 1));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![1..4 as Word, 4..6 as Space, 6..9 as Word],
                atoken_vec![1..2 as Space, 2..5 as Word],
            ],
        );
        assert_move!(bck, b: (1, 1) => Failure);
        assert_move!(bck, b: (1, 4) => (1, 1));
        assert_move!(bck, b: (1, 6) => (1, 6));
        assert_move!(bck, b: (1, 7) => (1, 6));
        assert_move!(bck, b: (1, 9) => (1, 6));
        assert_move!(bck, b: (2, 1) => (1, 6));
        assert_move!(bck, b: (2, 2) => (2, 2));
        assert_move!(bck, b: (2, 3) => (2, 2));
        assert_move!(bck, b: (2, 5) => (2, 2));

        Ok(())
    }

    #[test]
    fn test_decl_backward_word() -> Result<(), ()> {
        let mut db = Decl::default().chain(BackwardWord::new(true));

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
        assert_move!(db, b: (1, 5) => (1, 1));
        assert_move!(db, b: (1, 6) => (1, 1));
        assert_move!(db, b: (1, 7) => (1, 6));
        assert_move!(db, b: (1, 8) => (1, 6));
        assert_move!(db, b: (1, 9) => (1, 6));
        assert_move!(db, b: (2, 1) => (1, 6));
        assert_move!(db, b: (2, 2) => (1, 6));
        assert_move!(db, b: (2, 3) => (2, 2));
        assert_move!(db, b: (2, 4) => (2, 2));
        assert_move!(db, b: (2, 5) => (2, 2));

        Ok(())
    }
}
