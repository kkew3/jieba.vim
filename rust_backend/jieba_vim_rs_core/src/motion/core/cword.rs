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

//! Identify <cword> and <cWORD>.

use crate::token::{Token, TokenType};

use super::buffer::ParsedBufferLike;
use super::iter::{ExtendedInlineTokensIter, GToken};
use super::position::Position;

/// Find <cword> or <cWORD>, depending on the context. Return None if there is
/// no word under cursor.
pub fn find_cword<B: ParsedBufferLike + ?Sized>(
    buffer: &mut B,
    cursor: &Position,
) -> Result<Option<Token>, B::Error> {
    let tokens = buffer.getline_parsed(cursor.lnum)?;
    let line = ExtendedInlineTokensIter::new(tokens).skip_col(cursor.col);
    let mut first_word = None;
    for token in line {
        if let GToken::T(t) = token {
            if t.cword {
                assert!(matches!(t.ty, TokenType::Word));
                return Ok(Some(t));
            } else if t.ty == TokenType::Word {
                first_word.get_or_insert(t);
            }
        }
    }
    Ok(first_word)
}

/// Find the token under cursor. Return None if the cursor is on or off the
/// eol.
pub fn find_ctoken<B: ParsedBufferLike + ?Sized>(
    buffer: &mut B,
    cursor: &Position,
) -> Result<Option<Token>, B::Error> {
    let tokens = buffer.getline_parsed(cursor.lnum)?;
    let cursor_token =
        ExtendedInlineTokensIter::new(tokens).into_col(cursor.col);
    match cursor_token {
        GToken::T(t) => Ok(Some(t)),
        GToken::Eol(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::motion::core::buffer::{ParsedBuffer, PreTokenizedBuffer};
    use crate::token::Tokenizer;
    use crate::token::jieba::KeywordCutter;

    #[test]
    fn test_find_cword() {
        use crate::token::TokenType::*;

        let mut b = PreTokenizedBuffer::new(
            1,
            [[
                Token::new(1, 2, 3, 4, Word, true),
                Token::new(4, 5, 4, 5, Space, false),
                Token::new(5, 6, 8, 9, Word, true),
                Token::new(9, 10, 11, 12, Space, false),
            ]],
        );
        let cword = find_cword(&mut b, &Position::new(1, 1)).unwrap();
        assert_eq!(cword, Some(Token::new(1, 2, 3, 4, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 3)).unwrap();
        assert_eq!(cword, Some(Token::new(1, 2, 3, 4, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 4)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 5)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 9)).unwrap();
        assert_eq!(cword, None);
        let cword = find_cword(&mut b, &Position::new(1, 11)).unwrap();
        assert_eq!(cword, None);

        let mut b = PreTokenizedBuffer::new(
            1,
            [[
                Token::new(1, 2, 3, 4, Word, false),
                Token::new(4, 5, 4, 5, Space, false),
                Token::new(5, 6, 8, 9, Word, true),
                Token::new(9, 10, 11, 12, Space, false),
            ]],
        );
        let cword = find_cword(&mut b, &Position::new(1, 1)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 3)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 4)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 5)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 7)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 9)).unwrap();
        assert_eq!(cword, None);
        let cword = find_cword(&mut b, &Position::new(1, 11)).unwrap();
        assert_eq!(cword, None);

        let mut b = PreTokenizedBuffer::new(
            1,
            [[
                Token::new(1, 2, 3, 4, Word, true),
                Token::new(4, 5, 4, 5, Space, false),
                Token::new(5, 6, 7, 8, Word, false),
                Token::new(8, 9, 9, 10, Space, false),
            ]],
        );
        let cword = find_cword(&mut b, &Position::new(1, 1)).unwrap();
        assert_eq!(cword, Some(Token::new(1, 2, 3, 4, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 4)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 7, 8, Word, false)));
        let cword = find_cword(&mut b, &Position::new(1, 6)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 7, 8, Word, false)));
        let cword = find_cword(&mut b, &Position::new(1, 8)).unwrap();
        assert_eq!(cword, None);
        let cword = find_cword(&mut b, &Position::new(1, 9)).unwrap();
        assert_eq!(cword, None);

        let mut b = PreTokenizedBuffer::new(
            1,
            [[
                Token::new(1, 2, 3, 4, Word, false),
                Token::new(4, 5, 4, 5, Space, false),
                Token::new(5, 6, 8, 9, Word, false),
                Token::new(9, 10, 11, 12, Space, false),
            ]],
        );
        let cword = find_cword(&mut b, &Position::new(1, 1)).unwrap();
        assert_eq!(cword, Some(Token::new(1, 2, 3, 4, Word, false)));
        let cword = find_cword(&mut b, &Position::new(1, 3)).unwrap();
        assert_eq!(cword, Some(Token::new(1, 2, 3, 4, Word, false)));
        let cword = find_cword(&mut b, &Position::new(1, 4)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, false)));
        let cword = find_cword(&mut b, &Position::new(1, 5)).unwrap();
        assert_eq!(cword, Some(Token::new(5, 6, 8, 9, Word, false)));
        let cword = find_cword(&mut b, &Position::new(1, 9)).unwrap();
        assert_eq!(cword, None);
        let cword = find_cword(&mut b, &Position::new(1, 11)).unwrap();
        assert_eq!(cword, None);
    }

    #[test]
    fn test_find_cword_str() {
        use crate::token::TokenType::*;

        let tokenizer = Tokenizer::new(KeywordCutter::new([]), "");
        let b = vec!["abd abcde你好   abc,def".to_string()];
        let mut b = ParsedBuffer::new(&b, &tokenizer, true);

        let cword = find_cword(&mut b, &Position::new(1, 1)).unwrap();
        assert_eq!(cword, Some(Token::new(10, 13, 13, 16, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 13)).unwrap();
        assert_eq!(cword, Some(Token::new(10, 13, 13, 16, Word, true)));
        let cword = find_cword(&mut b, &Position::new(1, 16)).unwrap();
        assert_eq!(cword, Some(Token::new(19, 20, 25, 26, Word, false)));
        let cword = find_cword(&mut b, &Position::new(1, 20)).unwrap();
        assert_eq!(cword, Some(Token::new(19, 20, 25, 26, Word, false)));
    }
}
