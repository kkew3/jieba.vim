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

use std::collections::BTreeMap;

use crate::BufferLike;
use crate::token::{JiebaPlaceholder, Token, Tokenizer};

/// A buffer that caches parsed tokens.
pub struct ParsedBuffer<'b, 'p, B: ?Sized, C> {
    buffer: &'b B,
    tokenizer: &'p Tokenizer<C>,
    into_word: bool,
    parsed_lines: BTreeMap<usize, Vec<Token>>,
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
            parsed_lines: BTreeMap::new(),
        }
    }
}

impl<'b, 'p, B: BufferLike + ?Sized, C> ParsedBuffer<'b, 'p, B, C> {
    pub fn lines(&self) -> Result<usize, B::Error> {
        self.buffer.lines()
    }
}

impl<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>
    ParsedBuffer<'b, 'p, B, C>
{
    pub fn getline_parsed(
        &mut self,
        lnum: usize,
    ) -> Result<&[Token], B::Error> {
        if !self.parsed_lines.contains_key(&lnum) {
            let line = self.buffer.getline(lnum)?;
            let parsed_line = self.tokenizer.parse_str1(&line, self.into_word);
            self.parsed_lines.insert(lnum, parsed_line);
        }
        Ok(self.parsed_lines.get(&lnum).unwrap())
    }
}
