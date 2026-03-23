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

//! Token iterators.
//!
//! The cursor may be "off" by some nonzero virtual column with respect to
//! a column (see https://vimhelp.org/builtin.txt.html#getpos%28%29). We
//! define that it's off w.r.t. a general token, if it's off w.r.t. the last
//! column ([`last_char`](crate::token::TokenLike::last_char)) of the token;
//! otherwise, it will be taken as lying on that token (i.e. not "off" w.r.t.
//! the token).

use std::cmp::Ordering;
use std::iter::{Rev, Skip, Take};

use crate::token::{Token, TokenLike};

pub trait TokenLikeExt: TokenLike {
    // This is how we define the cursor being "on" a token:
    //
    // /// `true` if `(col, off)` is on self token.
    // fn is_on(&self, [col, off]: [usize; 2]) -> bool {
    //     let fc = self.first_char();
    //     let lc = self.last_char();
    //     (col >= fc && col < lc) || (col == lc && off == 0)
    // }

    // This is how we define the cursor being "off" a token:
    //
    // /// `true` if `(col, off)` is off self token.
    // fn is_off(&self, [col, off]: [usize; 2]) -> bool {
    //     let lc = self.last_char();
    //     let lc1 = self.last_char1();
    //     (col == lc && off > 0) || (col > lc && col < lc1)
    // }

    // This is how we define the cursor being "over" a token:
    //
    // /// `true` if the columnn `col` is on or off (in one word, over) self
    // /// token. In other words, return `true` if the column position is
    // /// contained in self token.
    // fn is_over(&self, col: usize) -> bool {
    //     let fc = self.first_char();
    //     let lc1 = self.last_char1();
    //     (fc..lc1).contains(&col)
    // }

    /// Return a total order between a `col` and self token.
    fn cmp(&self, col: usize) -> Ordering {
        if col < self.first_char() {
            // col occurs to the left of self token.
            Ordering::Greater
        } else if col >= self.last_char1() {
            // col occurs to the right of self token.
            Ordering::Less
        } else {
            // Occurs when `self.is_over(cpos)`.
            Ordering::Equal
        }
    }

    /// Return true if `col` equals [`first_char`](TokenLike::first_char). Note
    /// that this does not mean the cursor appears in the first virtual column
    /// of the token, since it may be off w.r.t. the `first_char` column.
    fn at_start(&self, col: usize) -> bool {
        col == self.first_char()
    }

    /// Return true if `col` equals [`last_char`](TokenLike::last_char).
    /// Note that this function does not make sense if
    /// [`is_empty`](TokenLikeExt::is_empty) is true.
    fn at_end(&self, col: usize) -> bool {
        col == self.last_char()
    }

    /// A token is called "empty" if it takes up no bytes in a string, in which
    /// case evaluating [`TokenLike::last_char`] does not make sense.
    fn is_empty(&self) -> bool {
        self.first_char() == self.last_char1()
    }
}

impl<T: TokenLike> TokenLikeExt for T {}

/// An enum of [`Token`](crate::token::Token) and Eol (end-of-line). Although
/// we may well represent the Eol as a zero-width Token, it's concrete
/// [`TokenType`](crate::token::TokenType) is subject to the motion of
/// interest. Thus, we'd better consider it a type of its own.
///
/// Here, "G" is a shorthand notation for "General".
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GToken {
    /// A regular token.
    T(Token),
    /// The Eol. The enclosed `usize` is the length of current line in bytes
    /// plus 1. Hence, it's never 0.
    Eol(usize),
}

impl TokenLike for GToken {
    fn first_char(&self) -> usize {
        match self {
            Self::T(t) => t.first_char(),
            Self::Eol(len) => *len,
        }
    }

    fn last_char(&self) -> usize {
        match self {
            Self::T(t) => t.last_char(),
            Self::Eol(len) => *len,
        }
    }

    fn last_char1(&self) -> usize {
        match self {
            Self::T(t) => t.last_char1(),
            Self::Eol(len) => *len,
        }
    }
}

impl From<Token> for GToken {
    fn from(value: Token) -> Self {
        Self::T(value)
    }
}

/// Get the index of the token in `tokens` where `col` is contained. Return
/// None if `col` is at the Eol of `tokens`.
fn index_tokens(tokens: &[Token], col: usize) -> Option<usize> {
    tokens.binary_search_by(|t| t.cmp(col)).ok()
}

/// Get the index of the token in `tokens` where `col` is contained. Return
/// `tokens.len()` if `col` is at the Eol of `tokens`. Panics otherwise.
fn index_tokens_extended(tokens: &[Token], col: usize) -> usize {
    match index_tokens(tokens, col) {
        Some(i) => i,
        None => {
            // If `col_at_eol` is true, it means `col` lies on/off the Eol of
            // current lnum.
            let col_at_eol =
                col == tokens.last().map_or(1, TokenLike::last_char1);
            if !col_at_eol {
                panic!(
                    "col ({}) too large when index_tokens_extended for tokens: {:?}",
                    col, tokens
                );
            }
            tokens.len()
        }
    }
}

pub struct ExtendedInlineTokensIter<'p> {
    line: &'p [Token],
    left_index: usize,
    right_index_compl: usize,
    eol: GToken,
    n: usize,
}

impl<'p> ExtendedInlineTokensIter<'p> {
    pub fn new(line: &'p [Token]) -> Self {
        let n = line.len();
        let eol = GToken::Eol(line.last().map_or(1, TokenLike::last_char1));
        Self {
            line,
            left_index: 0,
            right_index_compl: 0,
            eol,
            n,
        }
    }

    /// Take up to `col`'s token (inclusive) and reverse. The resulting
    /// iterator is guaranteed to contain at least one item.
    pub fn take_col_rev(self, col: usize) -> Rev<Take<Self>> {
        let i = index_tokens_extended(self.line, col);
        self.take(i + 1).rev()
    }

    /// Skip up to `col`'s token (exclusive). The resulting iterator is
    /// guaranteed to contain at least one item.
    pub fn skip_col(self, col: usize) -> Skip<Self> {
        let i = index_tokens_extended(self.line, col);
        self.skip(i)
    }
}

impl<'p> Iterator for ExtendedInlineTokensIter<'p> {
    type Item = GToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left_index + self.right_index_compl > self.n {
            return None;
        }

        let item = if self.left_index < self.line.len() {
            Some(GToken::T(self.line[self.left_index]))
        } else if self.left_index == self.line.len() {
            Some(self.eol)
        } else {
            None
        };
        self.left_index += 1;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = self.len();
        (s, Some(s))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.left_index += n;
        self.next()
    }
}

impl<'p> ExactSizeIterator for ExtendedInlineTokensIter<'p> {
    fn len(&self) -> usize {
        let t = self.left_index + self.right_index_compl;
        if self.n < t { 0 } else { self.n + 1 - t }
    }
}

impl<'p> DoubleEndedIterator for ExtendedInlineTokensIter<'p> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.left_index + self.right_index_compl > self.n {
            return None;
        }

        let right_index = self.n - self.right_index_compl;
        let item = if right_index < self.line.len() {
            Some(GToken::T(self.line[right_index]))
        } else if right_index == self.line.len() {
            Some(self.eol)
        } else {
            None
        };
        self.right_index_compl += 1;
        item
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.right_index_compl += n;
        self.next_back()
    }
}
