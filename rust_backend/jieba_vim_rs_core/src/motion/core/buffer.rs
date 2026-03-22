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

use std::collections::HashMap;

use crate::BufferLike;
use crate::token::{JiebaPlaceholder, Token, Tokenizer};

/// Any type that resembles a Vim buffer but returns tokenized lines, often
/// implemented with an internal cache.
pub trait ParsedBufferLike: BufferLike {
    /// Either return the cached tokenized line, or tokenize the requested
    /// line, update the cache (which requires mut self), and return the
    /// tokenization result.
    fn getline_parsed(&mut self, lnum: usize) -> Result<&[Token], Self::Error>;
}

/// A buffer that caches parsed tokens.
pub struct ParsedBuffer<'b, 'p, B: ?Sized, C> {
    buffer: &'b B,
    tokenizer: &'p Tokenizer<C>,
    into_word: bool,
    parsed_lines: HashMap<usize, Vec<Token>>,
}

impl<'b, 'p, B: ?Sized, C> ParsedBuffer<'b, 'p, B, C> {
    pub fn new(
        buffer: &'b B,
        tokenizer: &'p Tokenizer<C>,
        into_word: bool,
    ) -> Self {
        Self {
            buffer,
            tokenizer,
            into_word,
            parsed_lines: HashMap::new(),
        }
    }
}

impl<'b, 'p, B: BufferLike + ?Sized, C> BufferLike
    for ParsedBuffer<'b, 'p, B, C>
{
    type Error = B::Error;

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        self.buffer.getline(lnum)
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        self.buffer.lines()
    }
}

impl<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder> ParsedBufferLike
    for ParsedBuffer<'b, 'p, B, C>
{
    fn getline_parsed(&mut self, lnum: usize) -> Result<&[Token], B::Error> {
        if !self.parsed_lines.contains_key(&lnum) {
            let line = self.buffer.getline(lnum)?;
            let parsed_line = self.tokenizer.parse_str1(&line, self.into_word);
            self.parsed_lines.insert(lnum, parsed_line);
        }
        Ok(self.parsed_lines.get(&lnum).unwrap())
    }
}

#[cfg(test)]
mod pre_tokenized_buffer {
    use crate::BufferLike;
    use crate::token::{Token, TokenLike, TokenType};

    use super::ParsedBufferLike;

    /// Pre-tokenized [`ParsedBufferLike`] used in tests.
    pub struct PreTokenizedBuffer {
        /// The lnum of `parsed_lines[0]`.
        base_lnum: usize,
        parsed_lines: Vec<Vec<Token>>,
    }

    impl PreTokenizedBuffer {
        pub fn new<I: IntoIterator<Item = J>, J: IntoIterator<Item = Token>>(
            base_lnum: usize,
            tokens: I,
        ) -> Self {
            let mut parsed_lines = Vec::new();
            for line in tokens {
                parsed_lines.push(line.into_iter().collect());
            }
            Self {
                base_lnum,
                parsed_lines,
            }
        }

        fn getline_parsed_helper(
            &self,
            lnum: usize,
        ) -> Result<&[Token], <Self as BufferLike>::Error> {
            Ok(self
                .parsed_lines
                .get(lnum.checked_sub(self.base_lnum).ok_or(())?)
                .ok_or(())?)
        }
    }

    fn repeat_push(c: char, count: usize, to_string: &mut String) {
        for _ in 0..count {
            to_string.push(c);
        }
    }

    impl BufferLike for PreTokenizedBuffer {
        type Error = ();

        fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
            let tokens = self.getline_parsed_helper(lnum)?;
            let mut line = String::new();
            for t in tokens {
                let c = match t.ty {
                    TokenType::Space => ' ',
                    TokenType::Word => 'a',
                };
                repeat_push(c, t.last_char1() - t.first_char(), &mut line);
            }
            Ok(line)
        }

        fn lines(&self) -> Result<usize, Self::Error> {
            Ok(self.parsed_lines.len())
        }
    }

    impl ParsedBufferLike for PreTokenizedBuffer {
        fn getline_parsed(
            &mut self,
            lnum: usize,
        ) -> Result<&[Token], Self::Error> {
            self.getline_parsed_helper(lnum)
        }
    }
}

#[cfg(test)]
pub use pre_tokenized_buffer::PreTokenizedBuffer;
