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

use crate::motion::token_iter::{
    BackwardTokenIterator, ForwardTokenIterator, GToken, TokenLikeExt,
};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType, Tokenizer};
use crate::{BufferLike, Position};

/// Check if current motion satisfies d-special case. See
/// https://vimhelp.org/change.txt.html#d-special.
pub fn is_d_special<B: BufferLike + ?Sized, C: JiebaPlaceholder>(
    buffer: &B,
    tokenizer: &Tokenizer<C>,
    langle: Position,
    rangle: Position,
    inclusive: bool,
    word: bool,
) -> Result<bool, B::Error> {
    let (langle, rangle) = if langle <= rangle {
        (langle, rangle)
    } else {
        (rangle, langle)
    };
    let [_, llnum, lcol, _] = langle;
    let [_, rlnum, rcol, _] = rangle;
    if llnum == rlnum {
        return Ok(false);
    }

    let mut it = BackwardTokenIterator::new(buffer, tokenizer, &langle, word)?;
    let cursor_item = it.first();
    if !cursor_item.token.at_start(lcol) {
        if let GToken::T(t) = cursor_item.token {
            if t.ty == TokenType::Word {
                return Ok(false);
            }
        }
    }
    for item in
        it.take_while(|res| !res.as_ref().is_ok_and(|item| item.lnum != llnum))
    {
        if let GToken::T(t) = item?.token {
            if t.ty == TokenType::Word {
                return Ok(false);
            }
        }
    }

    let mut it = ForwardTokenIterator::new(buffer, tokenizer, &rangle, word)?;
    let cursor_item = it.first();
    if !((rcol == cursor_item.token.last_char() && inclusive)
        || (rcol == cursor_item.token.last_char1() && !inclusive))
    {
        if let GToken::T(t) = cursor_item.token {
            if t.ty == TokenType::Word {
                return Ok(false);
            }
        }
    }
    for item in
        it.take_while(|res| !res.as_ref().is_ok_and(|item| item.lnum != rlnum))
    {
        if let GToken::T(t) = item?.token {
            if t.ty == TokenType::Word {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
