// Copyright 2026 Kaiwen Wu. All Rights Reserved.
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

/// Find previous word under cursor.
#[derive(Default)]
pub struct PreviousWord;

impl Motion<Position> for PreviousWord {
    /// Panics if `count` is not 1.
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        assert_eq!(count, 1);

        if cursor.off > 0 {
            cursor.off = 0;
        } else if cursor.col == 1 {
            Dec::new(true, true).map(buffer, 1, cursor).ok();
        } else {
            let Position { lnum, col, .. } = cursor;
            let tokens = buffer.getline_parsed(*lnum)?;
            let mut line =
                ExtendedInlineTokensIter::new(tokens).take_col_rev(*col);
            // `unwrap` is safe because `take_col_rev` yields at least one
            // item.
            let cursor_token = line.next().unwrap();
            match cursor_token {
                GToken::Eol(_) => {
                    let success = find_stop_point(line, col);
                    assert!(success);
                }
                GToken::T(t) => {
                    let need_find_prev_word = match t.ty {
                        TokenType::Word => t.at_start(*col),
                        TokenType::Space => true,
                    };
                    if need_find_prev_word {
                        if !find_stop_point(line, col) {
                            *col = t.first_char();
                        }
                    } else {
                        *col = t.first_char();
                    }
                }
            }
        }

        Ok(MotionState::Success)
    }
}

/// Find the first Word token, or the last Space token if there is no Word. We
/// assume `line` yields non-Eol items if it's not empty. Return false if it's
/// empty.
fn find_stop_point<L: IntoIterator<Item = GToken>>(
    line: L,
    col: &mut usize,
) -> bool {
    let mut line = line.into_iter();
    match line.next() {
        None => false,
        Some(first) => {
            *col = first.first_char();
            match first {
                GToken::Eol(_) => unreachable!(),
                GToken::T(first_t) => {
                    if first_t.ty == TokenType::Space {
                        if let Some(second) = line.next() {
                            match second {
                                GToken::Eol(_) => unreachable!(),
                                GToken::T(second_t) => match second_t.ty {
                                    // By tokenization, there can't be two
                                    // adjacent Space tokens.
                                    TokenType::Space => unreachable!(),
                                    TokenType::Word => {
                                        *col = second.first_char();
                                    }
                                },
                            }
                        }
                    }
                }
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_previous_word() -> Result<(), ()> {
        let mut p = PreviousWord;

        let mut b = PreTokenizedBuffer::new(1, vec![atoken_vec![]]);
        assert_move!(p, b: (1, 1) => (1, 1));

        let mut b =
            PreTokenizedBuffer::new(1, vec![atoken_vec![1..4 as Space]]);
        assert_move!(p, b: (1, 1) => (1, 1));
        assert_move!(p, b: (1, 3) => (1, 1));
        assert_move!(p, b: (1, 4) => (1, 1));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![1..4 as Space, 4..8 as Word]],
        );
        assert_move!(p, b: (1, 3) => (1, 1));
        assert_move!(p, b: (1, 4) => (1, 1));
        assert_move!(p, b: (1, 5) => (1, 4));
        assert_move!(p, b: (1, 7) => (1, 4));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![1..4 as Word, 4..8 as Space]],
        );
        assert_move!(p, b: (1, 3) => (1, 1));
        assert_move!(p, b: (1, 4) => (1, 1));
        assert_move!(p, b: (1, 5) => (1, 1));
        assert_move!(p, b: (1, 7) => (1, 1));
        assert_move!(p, b: (1, 8) => (1, 1));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![1..4 as Space, 4..8 as Word, 8..9 as Space]],
        );
        assert_move!(p, b: (1, 7) => (1, 4));
        assert_move!(p, b: (1, 8) => (1, 4));
        assert_move!(p, b: (1, 9) => (1, 4));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![1..4 as Word, 4..8 as Space, 8..9 as Word]],
        );
        assert_move!(p, b: (1, 7) => (1, 1));
        assert_move!(p, b: (1, 8) => (1, 1));
        assert_move!(p, b: (1, 9) => (1, 8));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![], atoken_vec![1..4 as Space, 4..8 as Word]],
        );
        assert_move!(p, b: (1, 1) => (1, 1));
        assert_move!(p, b: (2, 1) => (1, 1));
        assert_move!(p, b: (2, 2) => (2, 1));
        assert_move!(p, b: (2, 4) => (2, 1));
        assert_move!(p, b: (2, 5) => (2, 4));
        assert_move!(p, b: (2, 7) => (2, 4));
        assert_move!(p, b: (2, 8) => (2, 4));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![1..5 as Space],
                atoken_vec![1..4 as Space, 4..8 as Word],
            ],
        );
        assert_move!(p, b: (1, 1) => (1, 1));
        assert_move!(p, b: (1, 2) => (1, 1));
        assert_move!(p, b: (1, 5) => (1, 1));
        assert_move!(p, b: (2, 1) => (1, 5));
        assert_move!(p, b: (2, 2) => (2, 1));
        assert_move!(p, b: (2, 4) => (2, 1));
        assert_move!(p, b: (2, 5) => (2, 4));
        assert_move!(p, b: (2, 8) => (2, 4));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![
                atoken_vec![],
                atoken_vec![1..5 as Space],
                atoken_vec![],
                atoken_vec![1..2 as Space, 2..4 as Word, 4..8 as Space],
            ],
        );
        assert_move!(p, b: (1, 1) => (1, 1));
        assert_move!(p, b: (2, 1) => (1, 1));
        assert_move!(p, b: (2, 4) => (2, 1));
        assert_move!(p, b: (3, 1) => (2, 5));
        assert_move!(p, b: (4, 1) => (3, 1));
        assert_move!(p, b: (4, 2) => (4, 1));
        assert_move!(p, b: (4, 7) => (4, 2));
        assert_move!(p, b: (4, 8) => (4, 2));

        let mut b = PreTokenizedBuffer::new(
            1,
            vec![atoken_vec![], atoken_vec![1..3 as Word, 3..6 as Word]],
        );
        assert_move!(p, b: (1, 1) => (1, 1));
        assert_move!(p, b: (2, 1) => (1, 1));
        assert_move!(p, b: (2, 3) => (2, 1));
        assert_move!(p, b: (2, 4) => (2, 3));
        assert_move!(p, b: (2, 6) => (2, 3));

        Ok(())
    }
}
