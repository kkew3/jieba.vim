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
//! This module contains forward and backward iterators of general tokens
//! (either a concrete token or an Eol (end-of-line)) in a buffer. In either
//! implementation, the iterator starts with the token where the cursor lies
//! on or off (see below), and ends with the first token in the buffer (for
//! backward iterator), or the last Eol in the buffer (for forward iterator).
//! At least one token will be yielded regardless the content of the buffer and
//! position of the cursor.
//!
//! The cursor may be "off" by some nonzero virtual column with respect to
//! a column (see https://vimhelp.org/builtin.txt.html#getpos%28%29). We
//! define that it's off w.r.t. a general token, if it's off w.r.t. the last
//! column ([`last_char`](crate::token::TokenLike::last_char)) of the token;
//! otherwise, it will be taken as lying on that token (i.e. not "off" w.r.t.
//! the token).

use std::cmp::Ordering;

use crate::BufferLike;
use crate::position::{
    BasicPosition, ColumnPosition, PositionError, PositionSanityCheck,
};
use crate::token::{JiebaPlaceholder, Token, TokenLike, Tokenizer};

pub trait TokenLikeExt: TokenLike {
    /// `true` if `(col, off)` is on self token.
    #[allow(unused)]
    fn is_on(&self, [col, off]: ColumnPosition) -> bool {
        let fc = self.first_char();
        let lc = self.last_char();
        (col >= fc && col < lc) || (col == lc && off == 0)
    }

    /// `true` if `(col, off)` is off self token.
    #[allow(unused)]
    fn is_off(&self, [col, off]: ColumnPosition) -> bool {
        let lc = self.last_char();
        let lc1 = self.last_char1();
        (col == lc && off > 0) || (col > lc && col < lc1)
    }

    /// `true` if the columnn `col` of a [`crate::position::ColumnPosition`] is
    /// on or off (in one word, over) self token. In other words, return `true`
    /// if the column position is contained in self token.
    #[allow(unused)]
    fn is_over(&self, col: usize) -> bool {
        let fc = self.first_char();
        let lc1 = self.last_char1();
        (fc..lc1).contains(&col)
    }

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
pub fn index_tokens(tokens: &[Token], col: usize) -> Option<usize> {
    tokens.binary_search_by(|t| t.cmp(col)).ok()
}

/// Get the index of the token in `tokens` where `col` is contained. Return
/// `tokens.len()` if `col` is at the Eol of `tokens`.
fn index_tokens_extended(
    tokens: &[Token],
    col: usize,
) -> Result<usize, PositionError> {
    match index_tokens(&tokens, col) {
        Some(i) => Ok(i),
        None => {
            // If `col_at_eol` is true, it means `col` lies on/off the Eol of
            // current lnum.
            let col_at_eol =
                col == tokens.last().map_or(1, TokenLike::last_char1);
            if !col_at_eol {
                return Err(PositionError::ColTooLarge);
            }
            Ok(tokens.len())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TokenIteratorItem {
    /// The `lnum` of current token.
    pub lnum: usize,
    /// Current token.
    pub token: GToken,
    /// `true` if the general token immediately next (for
    /// [`ForwardTokenIterator`]) or previous (for [`BackwardTokenIterator`])
    /// to the current token is an [`Eol`](GToken::Eol). Will always be `false`
    /// if `token` is an [`Eol`](GToken::Eol).
    pub eol: bool,
}

impl TokenIteratorItem {
    /// Construct a new iterator item. See the documentation of
    /// [`TokenIteratorItem`] for the meaning of each argument.
    fn new(lnum: usize, token: GToken, eol: bool) -> Self {
        Self { lnum, token, eol }
    }
}

/// Get either the i-th token from `tokens`, or the `Eol` GToken in the end.
fn get_gtoken(tokens: &[Token], i: usize) -> GToken {
    tokens.get(i).copied().map(Into::into).unwrap_or_else(|| {
        GToken::Eol(tokens.last().map_or(1, TokenLike::last_char1))
    })
}

/// Forward iterator of [`TokenIteratorItem`]s in a `buffer`. See module
/// documentation for details.
pub struct ForwardTokenIterator<'b, 'p, B: ?Sized, C> {
    /// A reference to the underlying buffer.
    buffer: &'b B,
    tokenizer: &'p Tokenizer<C>,
    /// Tokens of current lnum.
    tokens: Vec<Token>,
    /// The token index of current token in `tokens`. When `token_index` equals
    /// the length of `tokens`, it denotes the [`Eol`](GToken::Eol) of current
    /// lnum.
    token_index: usize,
    /// Current line number, 1-based.
    lnum: usize,
    /// Number of lines in `buffer`.
    lines: usize,
    /// Whether to cut into word (true) or WORD (false).
    word: bool,
}

impl<'b, 'p, B, C> ForwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    /// Construct a [`ForwardTokenIterator`], starting from the token where the
    /// cursor position `(lnum, col, off)` lies on or off.
    pub fn new<P: BasicPosition>(
        buffer: &'b B,
        tokenizer: &'p Tokenizer<C>,
        cursor_position: &P,
        word: bool,
    ) -> Result<Self, B::Error> {
        let [lnum, col, _] = cursor_position
            .try_to_cb_position()
            .and_then(PositionSanityCheck::check_indexing_basis)
            .unwrap_or_else(|err| {
                // TODO Return an error rather than panic.
                panic!("{}", err)
            });
        let tokens = tokenizer.parse_str1(&buffer.getline(lnum)?, word);
        // The resulting `token_index` must be no larger than `tokens.len()`.
        let token_index =
            index_tokens_extended(&tokens, col).unwrap_or_else(|err| {
                // TODO Return an error rather than panic.
                panic!("{}", err);
            });
        let lines = buffer.lines()?;
        Ok(Self {
            buffer,
            tokenizer,
            tokens,
            token_index,
            lnum,
            lines,
            word,
        })
    }

    /// Yield the first item, which is guaranteed to exist, and is the item
    /// where cursor lies on or off.
    pub fn first(&mut self) -> TokenIteratorItem {
        let token = get_gtoken(&self.tokens, self.token_index);
        self.token_index += 1;
        TokenIteratorItem::new(
            self.lnum,
            token,
            self.token_index == self.tokens.len(),
        )
    }

    /// Fetch the line at `lnum + 1` from the buffer, and populate
    /// `self.tokens`. Return `lnum + 1` if successful.
    fn fetch_next_line(&mut self, lnum: usize) -> Result<usize, B::Error> {
        let next_lnum = lnum + 1;
        self.tokens = self
            .tokenizer
            .parse_str1(&self.buffer.getline(next_lnum)?, self.word);
        Ok(next_lnum)
    }

    /// The helper function to implement [`Iterator::next`], containing the
    /// main procedure.
    fn next_helper(&mut self) -> Result<Option<TokenIteratorItem>, B::Error> {
        let next_item = if self.token_index <= self.tokens.len() {
            let token = get_gtoken(&self.tokens, self.token_index);
            self.token_index += 1;
            Some(TokenIteratorItem::new(
                self.lnum,
                token,
                self.token_index == self.tokens.len(),
            ))
        } else if self.lnum < self.lines {
            self.lnum = self.fetch_next_line(self.lnum)?;
            let token = get_gtoken(&self.tokens, 0);
            self.token_index = 1;
            Some(TokenIteratorItem::new(
                self.lnum,
                token,
                self.token_index == self.tokens.len(),
            ))
        } else {
            None
        };
        Ok(next_item)
    }
}

impl<'b, 'p, B, C> Iterator for ForwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    type Item = Result<TokenIteratorItem, B::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_helper().transpose()
    }
}

/// Backward iterator of [`TokenIteratorItem`]s in a `buffer`. See module
/// documentation for details.
pub struct BackwardTokenIterator<'b, 'p, B: ?Sized, C> {
    /// A reference to the underlying buffer.
    buffer: &'b B,
    tokenizer: &'p Tokenizer<C>,
    /// Tokens of current lnum.
    tokens: Vec<Token>,
    /// The token index of current token in `tokens` plus 1. When
    /// `token_index_p1` equals the length of `tokens` plus 1, it denotes the
    /// [`Eol`](GToken::Eol) of current lnum.
    token_index_p1: usize,
    /// Current line number, 1-based.
    lnum: usize,
    /// Whether to cut into word (true) or WORD (false).
    word: bool,
}

impl<'b, 'p, B, C> BackwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    /// Construct a [`BackwardTokenIterator`], starting from the token where
    /// the cursor position `(lnum, col, off)` lies on or off.
    pub fn new<P: BasicPosition>(
        buffer: &'b B,
        tokenizer: &'p Tokenizer<C>,
        cursor_position: &P,
        word: bool,
    ) -> Result<Self, B::Error> {
        let [lnum, col, _] = cursor_position
            .try_to_cb_position()
            .and_then(PositionSanityCheck::check_indexing_basis)
            .unwrap_or_else(|err| {
                // TODO Return an error rather than panic.
                panic!("{}", err)
            });
        let tokens = tokenizer.parse_str1(&buffer.getline(lnum)?, word);
        // 1 plus the extended token index.
        let token_index_p1 = index_tokens_extended(&tokens, col).map_or_else(
            |err| {
                // TODO Return an error rather than panic.
                panic!("{}", err)
            },
            |i| i + 1,
        );
        Ok(Self {
            buffer,
            tokenizer,
            tokens,
            token_index_p1,
            lnum,
            word,
        })
    }

    /// Yield the first item, which is guaranteed to exist, and is the item
    /// where cursor lies on or off.
    pub fn first(&mut self) -> TokenIteratorItem {
        // `self.token_index_p1 - 1` is the extended token index in range
        // [0, self.tokens.len()].
        let eol = self.token_index_p1 == self.tokens.len();
        self.token_index_p1 -= 1;
        let token = get_gtoken(&self.tokens, self.token_index_p1);
        TokenIteratorItem::new(self.lnum, token, eol)
    }

    /// Fetch the line at `lnum - 1` from the buffer, and populate
    /// `self.tokens`. Return `lnum - 1` if successful.
    fn fetch_prev_line(&mut self, lnum: usize) -> Result<usize, B::Error> {
        let next_lnum = lnum - 1;
        self.tokens = self
            .tokenizer
            .parse_str1(&self.buffer.getline(next_lnum)?, self.word);
        Ok(next_lnum)
    }

    /// The helper function to implement [`Iterator::next`], containing the
    /// main procedure.
    fn next_helper(&mut self) -> Result<Option<TokenIteratorItem>, B::Error> {
        let next_item = if self.token_index_p1 > 0 {
            // `self.token_index_p1 - 1` is the extended token index in range
            // [0, self.tokens.len()].
            let eol = self.token_index_p1 == self.tokens.len();
            self.token_index_p1 -= 1;
            let token = get_gtoken(&self.tokens, self.token_index_p1);
            Some(TokenIteratorItem::new(self.lnum, token, eol))
        } else if self.lnum > 1 {
            self.lnum = self.fetch_prev_line(self.lnum)?;
            self.token_index_p1 = self.tokens.len();
            let token = get_gtoken(&self.tokens, self.token_index_p1);
            Some(TokenIteratorItem::new(self.lnum, token, false))
        } else {
            None
        };
        Ok(next_item)
    }
}

impl<'b, 'p, B, C> Iterator for BackwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    type Item = Result<TokenIteratorItem, B::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_helper().transpose()
    }
}

#[cfg(test)]
mod tests {
    use crate::token::jieba::KeywordCutter;
    use crate::token::{Token, TokenType, Tokenizer};

    use super::{
        BackwardTokenIterator, ForwardTokenIterator, GToken, TokenIteratorItem,
        index_tokens,
    };

    macro_rules! pos {
        ($($i:expr),*) => {
            &crate::pos![$($i),*]
        };
    }

    #[test]
    fn test_index_tokens() {
        use TokenType::*;

        assert_eq!(index_tokens(&[], 1), None);
        assert_eq!(
            index_tokens(
                &[Token::new(1, 2, 3, Word), Token::new(3, 3, 4, Space)],
                1
            ),
            Some(0)
        );
        assert_eq!(
            index_tokens(
                &[Token::new(1, 2, 3, Word), Token::new(3, 3, 4, Space)],
                2
            ),
            Some(0)
        );
        assert_eq!(
            index_tokens(
                &[Token::new(1, 2, 3, Word), Token::new(3, 3, 4, Space)],
                3
            ),
            Some(1)
        );
        assert_eq!(
            index_tokens(
                &[Token::new(1, 2, 3, Word), Token::new(3, 3, 4, Space)],
                4
            ),
            None
        );
    }

    trait CollectIntoVec<T> {
        fn collect_into_vec(self) -> Vec<T>;
    }

    impl<T, E, I> CollectIntoVec<T> for I
    where
        I: Iterator<Item = Result<T, E>>,
    {
        fn collect_into_vec(self) -> Vec<T> {
            self.collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<Vec<T>, _>>()
                .unwrap_or_else(|_| panic!())
        }
    }

    mod test_forward_token_iterator {
        use super::*;

        #[test]
        fn empty() {
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec![""];
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
        }

        #[test]
        fn three_empty() {
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["", "", ""];
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                ]
            );
        }

        #[test]
        fn empty_space() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["", "   "];
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 3, 4, Space)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(4), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 3, 4, Space)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(4), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 3, 4, Space)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(4), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1, 4],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 3, 4, Space)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(4), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 2, 4],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 3, 4, Space)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(4), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 3, 4],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 3, 4, Space)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(4), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 4],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(2, GToken::Eol(4), false)]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 4, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(2, GToken::Eol(4), false)]
            );
        }

        #[test]
        fn word_space() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa  "];
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 2, 3],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 3],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 3, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 4],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 5],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 5, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 6],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(6), false)]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 6, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(6), false)]
            );
        }

        #[test]
        fn word_space_word() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa aaa"];
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 4, 5, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(5, 7, 8, Word)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(8), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 4, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 4, 5, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(5, 7, 8, Word)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(8), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 8, 10],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(8), false)]
            );
        }

        #[test]
        fn complex_case_1() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa", "aa aa", "", "  aaa"];
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(4, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 4],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(4, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 4, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(4, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![3, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(4, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![3, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(4, GToken::Eol(6), false)
                ]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![4, 6],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(4, GToken::Eol(6), false)]
            );
            let it = ForwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![4, 6, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(4, GToken::Eol(6), false)]
            );
        }
    }

    mod test_backward_token_iterator {
        use super::*;

        #[test]
        fn empty() {
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec![""];
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1, 10],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
        }

        #[test]
        fn three_empty() {
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["", "", ""];
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(1, GToken::Eol(1), false)]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![3, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![3, 1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(1, GToken::Eol(1), false),
                ]
            );
        }

        #[test]
        fn space_empty() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec![" ", ""];
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(1, 1, 2, Space)),
                    true
                )]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(2), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 1, 2, Space)),
                        true
                    )
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 2, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(2), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 1, 2, Space)),
                        true
                    )
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(1, GToken::Eol(2), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 1, 2, Space)),
                        true
                    )
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![2, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(2, GToken::Eol(1), false),
                    TokenIteratorItem::new(1, GToken::Eol(2), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 1, 2, Space)),
                        true
                    )
                ]
            );
        }

        #[test]
        fn word_space() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa  "];
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(1, 3, 4, Word)),
                    false
                )]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 5],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 6],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 6, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 5, 6, Space)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
        }

        #[test]
        fn word_space_word() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa aaa"];
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(1, 3, 4, Word)),
                    false
                )]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 6],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(5, 7, 8, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 4, 5, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 7],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(5, 7, 8, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 4, 5, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 7, 10],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(5, 7, 8, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 4, 5, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 8],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(8), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(5, 7, 8, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 4, 5, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 8, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(8), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(5, 7, 8, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 4, 5, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        false
                    ),
                ]
            );
        }

        #[test]
        fn complex_case_1() {
            use TokenType::*;
            let kc = KeywordCutter::new([]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa", "aa aa", "", "  aaa"];
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(1, 3, 4, Word)),
                    true
                )]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 3],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(1, 3, 4, Word)),
                    true
                )]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 3, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(1, 3, 4, Word)),
                    true
                )]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 4],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    )
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![1, 4, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    )
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![3, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![3, 1, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![4, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![4, 5],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![4, 5, 2],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![4, 6],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(4, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                ]
            );
            let it = BackwardTokenIterator::new(
                &buffer,
                &tokenizer,
                pos![4, 6, 1],
                true,
            )
            .unwrap();
            assert_eq!(
                it.collect_into_vec(),
                vec![
                    TokenIteratorItem::new(4, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(3, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(1, 2, 3, Space)),
                        false
                    ),
                    TokenIteratorItem::new(3, GToken::Eol(1), false),
                    TokenIteratorItem::new(2, GToken::Eol(6), false),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(4, 5, 6, Word)),
                        true
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false
                    ),
                    TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(1, 2, 3, Word)),
                        false
                    ),
                    TokenIteratorItem::new(1, GToken::Eol(4), false),
                    TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(1, 3, 4, Word)),
                        true
                    ),
                ]
            );
        }
    }
}
