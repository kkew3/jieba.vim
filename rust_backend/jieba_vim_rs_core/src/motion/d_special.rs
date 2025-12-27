// Copyright 2024-2025 Kaiwen Wu. All Rights Reserved.
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

use crate::BufferLike;
use crate::token::{self, JiebaPlaceholder, TokenLike, TokenType, Tokenizer};

/// Check if current motion satisfies d-special case. See
/// https://vimhelp.org/change.txt.html#d-special.
pub fn is_d_special<B: BufferLike + ?Sized, C: JiebaPlaceholder>(
    buffer: &B,
    tokenizer: &Tokenizer<C>,
    start_cursor_pos: (usize, usize),
    end_cursor_pos: (usize, usize),
    word: bool,
) -> Result<bool, B::Error> {
    let (start_lnum, start_col) = start_cursor_pos;
    let (end_lnum, end_col) = end_cursor_pos;

    if start_lnum == end_lnum {
        return Ok(false);
    }

    let tokens_cursor_line =
        tokenizer.parse_str(&buffer.getline(start_lnum)?, word);
    if !tokens_cursor_line.is_empty() {
        let i = token::index_tokens(&tokens_cursor_line, start_col).unwrap();
        if tokens_cursor_line[..i].iter().any(|tok| match tok.ty {
            TokenType::Space => false,
            TokenType::Word => true,
        }) {
            return Ok(false);
        }
        let cursor_token = &tokens_cursor_line[i];
        if let TokenType::Word = cursor_token.ty {
            if start_col > cursor_token.first_char() {
                return Ok(false);
            }
        }
    }

    let tokens_new_cursor_line =
        tokenizer.parse_str(&buffer.getline(end_lnum)?, word);
    if !tokens_new_cursor_line.is_empty() {
        let j = token::index_tokens(&tokens_new_cursor_line, end_col).unwrap();
        if tokens_new_cursor_line[j + 1..]
            .iter()
            .any(|tok| match tok.ty {
                TokenType::Space => false,
                TokenType::Word => true,
            })
        {
            return Ok(false);
        }
        let new_cursor_token = &tokens_new_cursor_line[j];
        if let TokenType::Word = new_cursor_token.ty {
            if end_col < new_cursor_token.last_char() {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
