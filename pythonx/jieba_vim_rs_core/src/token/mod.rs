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

mod char;
mod isk;
mod jieba;
pub mod token_iter;
mod tokenize;

pub use jieba::JiebaPlaceholder;
pub use tokenize::{Token, TokenLike, TokenType, Tokenizer};

/// Get the index of the token in `tokens` that covers `col`. Return `None` if
/// `col` is to the right of the last token.
pub(crate) fn index_tokens(tokens: &[Token], col: usize) -> Option<usize> {
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

#[cfg(test)]
mod tests {
    use super::index_tokens;

    #[test]
    fn test_index_tokens() {
        assert_eq!(index_tokens(&[], 0), None);
    }
}
