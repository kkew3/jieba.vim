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

//! Several predicates on positions that are useful in motions.

use crate::token::{TokenLike, TokenType};

use super::core::buffer::ParsedBufferLike;
use super::core::iter::{ExtendedInlineTokensIter, GToken};
use super::core::position::Position;

pub trait OnOrBeforeFirstNonBlanks {
    /// Return true if `self` is either at the start of the first non-blank, or
    /// before the first non-blank.
    fn on_or_before_first_non_blank<B: ParsedBufferLike + ?Sized>(
        &self,
        buffer: &mut B,
    ) -> Result<bool, B::Error>;
}

impl OnOrBeforeFirstNonBlanks for Position {
    fn on_or_before_first_non_blank<B: ParsedBufferLike + ?Sized>(
        &self,
        buffer: &mut B,
    ) -> Result<bool, B::Error> {
        let tokens = buffer.getline_parsed(self.lnum)?;
        let line = ExtendedInlineTokensIter::new(tokens);
        for token in line {
            let is_whitespace = match token {
                GToken::Eol(_) => false,
                GToken::T(t) => match t.ty {
                    TokenType::Word => false,
                    TokenType::Space => true,
                },
            };
            if !is_whitespace {
                return Ok(self.col <= token.first_char());
            }
        }
        // Unreachable because `line` always ends with an Eol(_).
        unreachable!();
    }
}
