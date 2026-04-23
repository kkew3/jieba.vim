// Copyright 2026 Kaiwen Wu. All Rights Reserved.
// Portions Copyright (c) by Bram Moolenaar and others.
//
// This module contains code adapted from Vim's textobject.c. The Vim License
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

use crate::token::TokenLike;
#[cfg(test)]
use crate::token::{Token, TokenType};

use super::core::buffer::ParsedBufferLike;
#[cfg(test)]
use super::core::buffer::PreTokenizedBuffer;
use super::core::iter::{ExtendedInlineTokensIter, GToken, TokenLikeExt};
use super::core::motion::{Motion, MotionState};
use super::core::position::Position;

/// Move cursor 1 char back. Panics if cursor is not already on the start or
/// the 2nd char of a token.
pub struct Dec {
    /// True to move into Eol(col) for col > 1, else skip those Eol.
    eol: bool,
    /// True to move across line boundaries.
    line: bool,
}

impl Dec {
    /// Construct a new [`Dec`]. Pass true to `eol` to move into Eol(col) for
    /// col > 1. Pass true to `line` to move across line boundaries.
    pub fn new(eol: bool, line: bool) -> Self {
        Self { eol, line }
    }
}

impl Motion<Position> for Dec {
    /// Panics if `count` is not 1.
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        assert_eq!(count, 1);

        let tokens = buffer.getline_parsed(cursor.lnum)?;
        let mut line =
            ExtendedInlineTokensIter::new(tokens).take_col_rev(cursor.col);
        let cursor_token = line.next().unwrap();
        if let GToken::T(cursor_token) = &cursor_token
            && cursor_token.first_char1() == cursor.col
        {
            cursor.col = cursor_token.first_char();
            return Ok(MotionState::Success);
        }

        if !cursor_token.at_start(cursor.col) {
            panic!(
                "cursor ({:?}) not at start/2nd-char of any tokens in: {:?}",
                cursor, tokens
            );
        }
        let s = match line.next() {
            Some(prev_token) => {
                // If prev_token exists, it can't be empty.
                cursor.col = prev_token.last_char();
                MotionState::Success
            }
            None => {
                if !self.line || cursor.lnum <= 1 {
                    MotionState::Failure
                } else {
                    cursor.lnum -= 1;
                    let mut prev_line = ExtendedInlineTokensIter::new(
                        buffer.getline_parsed(cursor.lnum)?,
                    )
                    .rev();
                    let eol = prev_line.next().unwrap();
                    if self.eol || eol.first_char() <= 1 {
                        cursor.col = eol.first_char();
                        MotionState::Success
                    } else {
                        let prev_token_not_eol = prev_line.next().unwrap();
                        cursor.col = prev_token_not_eol.last_char();
                        MotionState::Success
                    }
                }
            }
        };
        Ok(s)
    }
}

/// A wrapper of `Dec(eol=false, line=true)`.
pub struct Decl(Dec);

impl Default for Decl {
    fn default() -> Self {
        Self(Dec::new(false, true))
    }
}

impl Motion<Position> for Decl {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        self.0.map(buffer, count, cursor)
    }
}

/// Move cursor 1 char forward. Panics if cursor is not already on the
/// start/end of a token.
pub struct Inc {
    /// True to move into Eol(col) for col > 1, else skip those Eol.
    eol: bool,
    /// True to move across line boundaries.
    line: bool,
}

impl Inc {
    /// Construct a new [`Inc`]. Pass true to `eol` to move into Eol(col) for
    /// col > 1. Pass true to `line` to move across line boundaries.
    pub fn new(eol: bool, line: bool) -> Self {
        Self { eol, line }
    }
}

impl Motion<Position> for Inc {
    /// Panics if `count` is not 1.
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        assert_eq!(count, 1);

        let tokens = buffer.getline_parsed(cursor.lnum)?;
        let mut line =
            ExtendedInlineTokensIter::new(tokens).skip_col(cursor.col);
        let cursor_token = line.next().unwrap();
        if let GToken::T(cursor_token) = &cursor_token
            && cursor_token.at_start(cursor.col)
            && !cursor_token.at_end(cursor.col)
        {
            cursor.col = cursor_token.first_char1();
            return Ok(MotionState::Success);
        }

        let s = if cursor_token.is_empty() {
            if !self.line || cursor.lnum >= buffer.lines()? {
                MotionState::Failure
            } else {
                // The first token of a line is never an Eol(col) for col > 1.
                cursor.lnum += 1;
                cursor.col = 1;
                MotionState::Success
            }
        } else if cursor_token.at_end(cursor.col) {
            match line.next() {
                Some(next_token_maybe_eol) => {
                    cursor.col = next_token_maybe_eol.first_char();
                    if self.eol || !next_token_maybe_eol.is_empty() {
                        MotionState::Success
                    } else if !self.line || cursor.lnum >= buffer.lines()? {
                        MotionState::Failure
                    } else {
                        // The first token of a line is never an Eol(col) for col > 1.
                        cursor.lnum += 1;
                        cursor.col = 1;
                        MotionState::Success
                    }
                }
                None => unreachable!(),
            }
        } else {
            panic!(
                "cursor ({:?}) not on Eol(_) and not at start/end of any tokens in: {:?}",
                cursor, tokens
            );
        };
        Ok(s)
    }
}

/// A wrapper of `Inc(eol=false, line=true)`.
pub struct Incl(Inc);

impl Default for Incl {
    fn default() -> Self {
        Self(Inc::new(false, true))
    }
}

impl Motion<Position> for Incl {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut Position,
    ) -> Result<MotionState, B::Error> {
        self.0.map(buffer, count, cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple buffer filled with word tokens.
    fn build_simple_buffer(
        base_lnum: usize,
        token_nchars_buffer: Vec<Vec<usize>>,
        char_len: usize,
    ) -> PreTokenizedBuffer {
        fn get_token_from_token_nchars(
            start: usize,
            token_nchars: usize,
            char_len: usize,
        ) -> Token {
            assert!(token_nchars > 0);
            Token::new(
                start,
                start + char_len,
                start + (token_nchars - 1) * char_len,
                start + token_nchars * char_len,
                TokenType::Word,
            )
        }
        let mut tokens_buffer = Vec::new();
        for token_nchars_line in token_nchars_buffer {
            let mut start = 1;
            let mut tokens_line = Vec::new();
            for token_nchars in token_nchars_line {
                let tok =
                    get_token_from_token_nchars(start, token_nchars, char_len);
                start = tok.last_char1();
                tokens_line.push(tok);
            }
            tokens_buffer.push(tokens_line);
        }
        PreTokenizedBuffer::new(base_lnum, tokens_buffer)
    }

    #[test]
    fn test_dec() -> Result<(), ()> {
        let mut dec = Dec::new(true, false);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(dec, b: (1, 1) => Failure);
        assert_move!(dec, b: (1, 2) => (1, 1));
        assert_move!(dec, b: (1, 5) => (1, 4));

        let mut dec = Dec::new(true, false);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 3);
        assert_move!(dec, b: (1, 1) => Failure);
        assert_move!(dec, b: (1, 4) => (1, 1));
        assert_move!(dec, b: (1, 13) => (1, 10));

        let mut dec = Dec::new(true, false);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(dec, b: (1, 1) => Failure);
        assert_move!(dec, b: (1, 2) => (1, 1));
        assert_move!(dec, b: (2, 1) => Failure);
        assert_move!(dec, b: (2, 2) => (2, 1));
        assert_move!(dec, b: (2, 5) => (2, 4));
        assert_move!(dec, b: (2, 6) => (2, 5));
        assert_move!(dec, b: (2, 9) => (2, 8));

        let mut dec = Dec::new(false, true);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(dec, b: (1, 1) => Failure);
        assert_move!(dec, b: (1, 2) => (1, 1));
        assert_move!(dec, b: (1, 5) => (1, 4));

        let mut dec = Dec::new(false, true);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(dec, b: (1, 1) => Failure);
        assert_move!(dec, b: (1, 2) => (1, 1));
        assert_move!(dec, b: (1, 4) => (1, 3));
        assert_move!(dec, b: (2, 1) => (1, 3));
        assert_move!(dec, b: (2, 2) => (2, 1));
        assert_move!(dec, b: (2, 5) => (2, 4));
        assert_move!(dec, b: (2, 6) => (2, 5));
        assert_move!(dec, b: (2, 9) => (2, 8));

        let mut dec = Dec::new(true, true);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(dec, b: (1, 1) => Failure);
        assert_move!(dec, b: (1, 2) => (1, 1));
        assert_move!(dec, b: (1, 5) => (1, 4));

        let mut dec = Dec::new(true, true);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(dec, b: (1, 1) => Failure);
        assert_move!(dec, b: (1, 2) => (1, 1));
        assert_move!(dec, b: (1, 4) => (1, 3));
        assert_move!(dec, b: (2, 1) => (1, 4));
        assert_move!(dec, b: (2, 2) => (2, 1));
        assert_move!(dec, b: (2, 5) => (2, 4));
        assert_move!(dec, b: (2, 6) => (2, 5));
        assert_move!(dec, b: (2, 9) => (2, 8));

        Ok(())
    }

    #[test]
    fn test_inc() -> Result<(), ()> {
        let mut inc = Inc::new(false, false);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 4) => Failure (1, 5));
        assert_move!(inc, b: (1, 5) => Failure);

        let mut inc = Inc::new(false, false);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 3);
        assert_move!(inc, b: (1, 1) => (1, 4));
        assert_move!(inc, b: (1, 10) => Failure (1, 13));
        assert_move!(inc, b: (1, 13) => Failure);

        let mut inc = Inc::new(false, false);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 3) => Failure (1, 4));
        assert_move!(inc, b: (1, 4) => Failure);
        assert_move!(inc, b: (2, 1) => (2, 2));
        assert_move!(inc, b: (2, 4) => (2, 5));
        assert_move!(inc, b: (2, 5) => (2, 6));
        assert_move!(inc, b: (2, 8) => Failure (2, 9));
        assert_move!(inc, b: (2, 9) => Failure);

        let mut inc = Inc::new(true, false);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 4) => (1, 5));
        assert_move!(inc, b: (1, 5) => Failure);

        let mut inc = Inc::new(true, false);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 3) => (1, 4));
        assert_move!(inc, b: (1, 4) => Failure);
        assert_move!(inc, b: (2, 1) => (2, 2));
        assert_move!(inc, b: (2, 4) => (2, 5));
        assert_move!(inc, b: (2, 5) => (2, 6));
        assert_move!(inc, b: (2, 8) => (2, 9));
        assert_move!(inc, b: (2, 9) => Failure);

        let mut inc = Inc::new(false, true);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 4) => Failure (1, 5));
        assert_move!(inc, b: (1, 5) => Failure);

        let mut inc = Inc::new(false, true);
        let token_nchars_buffer = vec![vec![1], vec![2]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (2, 1));
        assert_move!(inc, b: (1, 2) => (2, 1));
        assert_move!(inc, b: (2, 1) => (2, 2));
        assert_move!(inc, b: (2, 2) => Failure (2, 3));
        assert_move!(inc, b: (2, 3) => Failure);

        let mut inc = Inc::new(false, true);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 3) => (2, 1));
        assert_move!(inc, b: (1, 4) => (2, 1));
        assert_move!(inc, b: (2, 1) => (2, 2));
        assert_move!(inc, b: (2, 4) => (2, 5));
        assert_move!(inc, b: (2, 5) => (2, 6));
        assert_move!(inc, b: (2, 8) => Failure (2, 9));
        assert_move!(inc, b: (2, 9) => Failure);

        let mut inc = Inc::new(false, true);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 3);
        assert_move!(inc, b: (1, 1) => (1, 4));
        assert_move!(inc, b: (1, 7) => (2, 1));
        assert_move!(inc, b: (1, 10) => (2, 1));
        assert_move!(inc, b: (2, 1) => (2, 4));
        assert_move!(inc, b: (2, 10) => (2, 13));
        assert_move!(inc, b: (2, 13) => (2, 16));
        assert_move!(inc, b: (2, 22) => Failure (2, 25));
        assert_move!(inc, b: (2, 25) => Failure);

        let mut inc = Inc::new(true, true);
        let token_nchars_buffer = vec![vec![4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 4) => (1, 5));
        assert_move!(inc, b: (1, 5) => Failure);

        let mut inc = Inc::new(true, true);
        let token_nchars_buffer = vec![vec![3], vec![4, 4]];
        let mut b = build_simple_buffer(1, token_nchars_buffer, 1);
        assert_move!(inc, b: (1, 1) => (1, 2));
        assert_move!(inc, b: (1, 3) => (1, 4));
        assert_move!(inc, b: (1, 4) => (2, 1));
        assert_move!(inc, b: (2, 1) => (2, 2));
        assert_move!(inc, b: (2, 4) => (2, 5));
        assert_move!(inc, b: (2, 5) => (2, 6));
        assert_move!(inc, b: (2, 8) => (2, 9));
        assert_move!(inc, b: (2, 9) => Failure);

        Ok(())
    }
}
