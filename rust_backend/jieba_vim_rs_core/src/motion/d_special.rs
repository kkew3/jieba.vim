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
    ExtendedInlineTokensIter, GToken, ParsedBuffer, TokenLikeExt,
};
use crate::token::{JiebaPlaceholder, TokenType};
use crate::{BufferLike, Position};

/// Check if current motion satisfies d-special case. See
/// https://vimhelp.org/change.txt.html#d-special.
pub fn is_d_special<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
    buffer: &ParsedBuffer<'b, 'p, B, C>,
    langle: Position,
    rangle: Position,
    inclusive: bool,
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

    let lline = buffer.getline_parsed(llnum)?;
    let mut ltokens = ExtendedInlineTokensIter::new(&lline)
        .take_col_rev(lcol)
        .expect("col of langle too large");
    if let GToken::T(t) = ltokens.next().unwrap()
        && t.ty == TokenType::Word
        && !t.at_start(lcol)
    {
        return Ok(false);
    }
    for token in ltokens {
        if let GToken::T(t) = token
            && t.ty == TokenType::Word
        {
            return Ok(false);
        }
    }

    let rline = buffer.getline_parsed(rlnum)?;
    let mut rtokens = ExtendedInlineTokensIter::new(&rline)
        .skip_col(rcol)
        .expect("col of rangle too large");
    if let GToken::T(t) = rtokens.next().unwrap()
        && t.ty == TokenType::Word
    {
        if !inclusive {
            return Ok(false);
        }
        if !t.at_end(rcol) {
            return Ok(false);
        }
    }
    for token in rtokens {
        if let GToken::T(t) = token
            && t.ty == TokenType::Word
        {
            return Ok(false);
        }
    }

    Ok(true)
}
