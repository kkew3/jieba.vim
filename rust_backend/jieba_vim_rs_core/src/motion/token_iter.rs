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

use crate::BufferLike;
use crate::token::{JiebaPlaceholder, Token, TokenLike, Tokenizer};

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
    /// The Eol. The enclosed `usize` is the length of current line in bytes.
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

/// Get the index of the token in `tokens` that covers `col`. Return `None` if
/// `col` is to the right of the last token.
pub fn index_tokens(tokens: &[Token], col: usize) -> Option<usize> {
    use std::cmp::Ordering;
    tokens
        .binary_search_by(|tok| {
            if col < tok.first_char() {
                Ordering::Greater
            } else if col >= tok.last_char1() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
}

/// Item type yieled by token iterators.
#[derive(Debug, PartialEq, Eq)]
pub struct TokenIteratorItem {
    /// The `lnum` of current token.
    pub lnum: usize,
    /// Current token.
    pub token: GToken,
    /// `true` if the cursor lies in current token.
    pub cursor: bool,
    /// `true` if the cursor lies in a token at end-of-line.
    pub eol: bool,
}

impl TokenIteratorItem {
    #[cfg(test)]
    fn new(lnum: usize, token: GToken, cursor: bool, eol: bool) -> Self {
        Self {
            lnum,
            token,
            cursor,
            eol,
        }
    }
}

/// Forward iterator of [`TokenIteratorItem`]s in a `buffer`. If the cursor
/// `col` is in a token, starts from that token; if `col` is to the right of
/// the last token in current line, starts from the next token in the buffer.
/// An empty line is regarded as a `None` token. If the cursor is at an empty
/// line, also starts from that empty line.
pub struct ForwardTokenIterator<'b, 'p, B: ?Sized, C> {
    buffer: &'b B,
    tokenizer: &'p Tokenizer<C>,
    tokens: Vec<Token>,
    token_index: usize,
    lnum: usize,
    /// Number of lines in `buffer`.
    lines: usize,
    /// Whether to cut into word (true) or WORD (false).
    word: bool,
    /// Whether current item is the cursor item or not.
    cursor: bool,
}

impl<'b, 'p, B, C> ForwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    /// Construct a [`ForwardTokenIterator`], starting from the token where the
    /// cursor position `(lnum, col)` lies in.
    pub fn new(
        buffer: &'b B,
        tokenizer: &'p Tokenizer<C>,
        lnum: usize,
        col: usize,
        word: bool,
    ) -> Result<Self, B::Error> {
        let tokens = tokenizer.parse_str(&buffer.getline(lnum)?, word);
        let token_index = index_tokens(&tokens, col).unwrap_or(tokens.len());
        let cursor =
            (col == 0 && tokens.is_empty()) || token_index < tokens.len();
        let lines = buffer.lines()?;
        Ok(Self {
            buffer,
            tokenizer,
            tokens,
            token_index,
            lnum,
            lines,
            word,
            cursor,
        })
    }

    fn fetch_next_line(&mut self, lnum: usize) -> Result<(), B::Error> {
        self.tokens = self
            .tokenizer
            .parse_str(&self.buffer.getline(lnum + 1)?, self.word);
        Ok(())
    }
}

impl<'b, 'p, B, C> Iterator for ForwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    type Item = Result<TokenIteratorItem, B::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = {
            if self.token_index < self.tokens.len() {
                let to_yield =
                    self.tokens.get(self.token_index).copied().unwrap();
                let eol = self.token_index == self.tokens.len() - 1;
                self.token_index += 1;
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: GToken::T(to_yield),
                    cursor: self.cursor,
                    eol,
                }))
            } else if self.cursor
                && self.tokens.is_empty()
                && self.token_index == 0
            {
                // The cursor line is empty.
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: GToken::Eol(0),
                    cursor: self.cursor,
                    eol: true,
                }))
            } else if self.lnum < self.lines {
                match self.fetch_next_line(self.lnum) {
                    Err(err) => Some(Err(err)),
                    Ok(()) => {
                        self.lnum += 1;
                        self.token_index = 0;
                        if self.tokens.is_empty() {
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: GToken::Eol(0),
                                cursor: self.cursor,
                                eol: true,
                            }))
                        } else {
                            let to_yield = self
                                .tokens
                                .get(self.token_index)
                                .copied()
                                .unwrap();
                            let eol = self.token_index == self.tokens.len() - 1;
                            self.token_index += 1;
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: GToken::T(to_yield),
                                cursor: self.cursor,
                                eol,
                            }))
                        }
                    }
                }
            } else {
                None
            }
        };
        if self.cursor {
            self.cursor = false;
        }
        next_item
    }
}

/// Backward iterator of [`TokenIteratorItem`]s in a `buffer`. If the cursor
/// `col` is in a token, starts from that token; if `col` is to the right of
/// the last token in current line, starts from that last token. An empty line
/// is regarded as a `None` token. If the cursor is at an empty line, also
/// starts from that empty line.
pub struct BackwardTokenIterator<'b, 'p, B: ?Sized, C> {
    buffer: &'b B,
    tokenizer: &'p Tokenizer<C>,
    tokens: Vec<Token>,
    token_index: usize,
    lnum: usize,
    /// Whether to cut into word (true) or WORD (false).
    word: bool,
    /// Whether current item is the cursor item or not.
    cursor: bool,
    /// Whether current item is the first item or not.
    first: bool,
}

impl<'b, 'p, B, C> BackwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    /// Construct a [`BackwardTokenIterator`], starting from the token where
    /// the cursor position `(lnum, col)` lies in.
    pub fn new(
        buffer: &'b B,
        tokenizer: &'p Tokenizer<C>,
        lnum: usize,
        col: usize,
        word: bool,
    ) -> Result<Self, B::Error> {
        let tokens = tokenizer.parse_str(&buffer.getline(lnum)?, word);
        let token_index = index_tokens(&tokens, col);
        let cursor = (col == 0 && tokens.is_empty()) || token_index.is_some();
        // One past the cursor token index.
        let token_index = token_index.map(|i| i + 1).unwrap_or(tokens.len());
        Ok(Self {
            buffer,
            tokenizer,
            tokens,
            token_index,
            lnum,
            word,
            cursor,
            first: true,
        })
    }

    fn fetch_prev_line(&mut self, lnum: usize) -> Result<(), B::Error> {
        self.tokens = self
            .tokenizer
            .parse_str(&self.buffer.getline(lnum - 1)?, self.word);
        Ok(())
    }
}

impl<'b, 'p, B, C> Iterator for BackwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    type Item = Result<TokenIteratorItem, B::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = {
            if self.token_index > 0 {
                self.token_index -= 1;
                let eol = self.token_index == self.tokens.len() - 1;
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: GToken::T(
                        self.tokens.get(self.token_index).copied().unwrap(),
                    ),
                    cursor: self.cursor,
                    eol,
                }))
            } else if self.first && self.tokens.is_empty() {
                // The cursor line is empty.
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: GToken::Eol(0),
                    cursor: self.cursor,
                    eol: true,
                }))
            } else if self.lnum > 1 {
                match self.fetch_prev_line(self.lnum) {
                    Err(err) => Some(Err(err)),
                    Ok(()) => {
                        self.lnum -= 1;
                        self.token_index = self.tokens.len();
                        if self.tokens.is_empty() {
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: GToken::Eol(0),
                                cursor: self.cursor,
                                eol: true,
                            }))
                        } else {
                            self.token_index -= 1;
                            let eol = self.token_index == self.tokens.len() - 1;
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: GToken::T(
                                    self.tokens
                                        .get(self.token_index)
                                        .copied()
                                        .unwrap(),
                                ),
                                cursor: self.cursor,
                                eol,
                            }))
                        }
                    }
                }
            } else {
                None
            }
        };
        if self.cursor {
            self.cursor = false;
        }
        if self.first {
            self.first = false;
        }
        next_item
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackwardTokenIterator, ForwardTokenIterator, GToken, TokenIteratorItem,
        index_tokens,
    };
    use crate::token::jieba::KeywordCutter;
    use crate::token::{Token, TokenType, Tokenizer};

    #[test]
    fn test_index_tokens() {
        assert_eq!(index_tokens(&[], 0), None);
    }

    mod test_forward_token_iterator {
        use super::*;

        #[test]
        fn empty() {
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec![""];
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(1, GToken::Eol(0), true, true))]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 1, true)
                .unwrap();
            assert!(it.collect::<Vec<_>>().is_empty());
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 2, true)
                .unwrap();
            assert!(it.collect::<Vec<_>>().is_empty());
        }

        #[test]
        fn three_empty() {
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["", "", ""];
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(1, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 2, 0, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 1, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                ]
            );
        }

        #[test]
        fn empty_space() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["", " "];
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(1, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 0, 1, Space)),
                        false,
                        true
                    )),
                ]
            );
        }

        #[test]
        fn word_space() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa  "];
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        true,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(3, 4, 5, Space)),
                        false,
                        true
                    )),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 3, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(3, 4, 5, Space)),
                    true,
                    true
                ))]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 4, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(3, 4, 5, Space)),
                    true,
                    true
                ))]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 5, true)
                .unwrap();
            assert!(it.collect::<Vec<_>>().is_empty());
        }

        #[test]
        fn word_space_word() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa aaa"];
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        true,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 6, 7, Word)),
                        false,
                        true
                    )),
                ]
            );
        }

        #[test]
        fn complex_case_1() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa", "aa aa", "", "  aaa"];
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 1, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        true,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 1, 2, Word)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(2, 2, 3, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 4, 5, Word)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(0, 1, 2, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(2, 4, 5, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 3, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 1, 2, Word)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(2, 2, 3, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 4, 5, Word)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(0, 1, 2, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(2, 4, 5, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 1, 4, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 1, 2, Word)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(2, 2, 3, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 4, 5, Word)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(0, 1, 2, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(2, 4, 5, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 3, 0, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(0, 1, 2, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(2, 4, 5, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 3, 1, true)
                .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(0, 1, 2, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(2, 4, 5, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 4, 5, true)
                .unwrap();
            assert!(it.collect::<Vec<_>>().is_empty());
            let it = ForwardTokenIterator::new(&buffer, &tokenizer, 4, 6, true)
                .unwrap();
            assert!(it.collect::<Vec<_>>().is_empty());
        }
    }

    mod test_backward_token_iterator {
        use super::*;

        #[test]
        fn empty() {
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec![""];
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(1, GToken::Eol(0), true, true))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 1, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::Eol(0),
                    false,
                    true
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 2, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::Eol(0),
                    false,
                    true
                ))]
            );
        }

        #[test]
        fn three_empty() {
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["", "", ""];
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(1, GToken::Eol(0), true, true))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 1, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::Eol(0),
                    false,
                    true
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 2, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(1, GToken::Eol(0), false, true)),
                ]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 2, 2, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(1, GToken::Eol(0), false, true)),
                ]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 3, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(1, GToken::Eol(0), false, true)),
                ]
            );
        }

        #[test]
        fn space_empty() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec![" ", ""];
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(0, 0, 1, Space)),
                    true,
                    true
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 1, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(0, 0, 1, Space)),
                    false,
                    true
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 2, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 0, 1, Space)),
                        false,
                        true
                    ))
                ]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 2, 2, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(2, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 0, 1, Space)),
                        false,
                        true
                    ))
                ]
            );
        }

        #[test]
        fn word_space() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa  "];
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(0, 2, 3, Word)),
                    true,
                    false
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 4, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(3, 4, 5, Space)),
                        true,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        false,
                        false
                    )),
                ]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 5, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(3, 4, 5, Space)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        false,
                        false
                    )),
                ]
            );
        }

        #[test]
        fn word_space_word() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa aaa"];
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(0, 2, 3, Word)),
                    true,
                    false
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 5, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(4, 6, 7, Word)),
                        true,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(3, 3, 4, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        false,
                        false
                    )),
                ]
            );
        }

        #[test]
        fn complex_case_1() {
            use TokenType::*;
            let kc = KeywordCutter::new(["你好".into(), "世界".into()]);
            let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");

            let buffer = vec!["aaa", "aa aa", "", "  aaa"];
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 1, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(0, 2, 3, Word)),
                    true,
                    true
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 1, 3, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![Ok(TokenIteratorItem::new(
                    1,
                    GToken::T(Token::new(0, 2, 3, Word)),
                    false,
                    true
                ))]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 3, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), true, true)),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 4, 5, Word)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(2, 2, 3, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 1, 2, Word)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 3, 1, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 4, 5, Word)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(2, 2, 3, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 1, 2, Word)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 4, 0, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(0, 1, 2, Space)),
                        true,
                        false
                    )),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 4, 5, Word)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(2, 2, 3, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 1, 2, Word)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        false,
                        true
                    )),
                ]
            );
            let it =
                BackwardTokenIterator::new(&buffer, &tokenizer, 4, 4, true)
                    .unwrap();
            assert_eq!(
                it.collect::<Vec<_>>(),
                vec![
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(2, 4, 5, Word)),
                        true,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        4,
                        GToken::T(Token::new(0, 1, 2, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(3, GToken::Eol(0), false, true)),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(3, 4, 5, Word)),
                        false,
                        true
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(2, 2, 3, Space)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        2,
                        GToken::T(Token::new(0, 1, 2, Word)),
                        false,
                        false
                    )),
                    Ok(TokenIteratorItem::new(
                        1,
                        GToken::T(Token::new(0, 2, 3, Word)),
                        false,
                        true
                    )),
                ]
            );
        }
    }
}
