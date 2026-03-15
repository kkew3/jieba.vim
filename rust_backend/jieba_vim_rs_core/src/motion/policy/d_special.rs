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

//! Quoted from vimhelp.org:
//!
//! > An exception for the d{motion} command: If the motion is not linewise,
//! > the start and end of the motion are not in the same line, and there are
//! > only blanks before the start and there are no non-blanks after the end of
//! > the motion, the delete becomes linewise.  This means that the delete also
//! > removes the line of blanks that you might expect to remain.
//!
//! Check <https://vimhelp.org/change.txt.html#d-special> for details.

use crate::motion::core::buffer::ParsedBufferLike;
use crate::motion::core::iter::{
    ExtendedInlineTokensIter, GToken, TokenLikeExt,
};
use crate::motion::core::position::Position;
use crate::token::TokenType;

/// Check if current motion satisfies d-special case. See
/// https://vimhelp.org/change.txt.html#d-special.
pub fn is_d_special<B: ParsedBufferLike + ?Sized>(
    buffer: &mut B,
    langle: Position,
    rangle: Position,
    inclusive: bool,
) -> Result<bool, B::Error> {
    let (langle, rangle) = if langle <= rangle {
        (langle, rangle)
    } else {
        (rangle, langle)
    };
    let Position {
        lnum: llnum,
        col: lcol,
        ..
    } = langle;
    let Position {
        lnum: rlnum,
        col: rcol,
        ..
    } = rangle;
    if rcol == 1 && !inclusive {
        panic!(
            "`exclusive + rcol=1` case must be handled first by \
            `exclusive_special` mod"
        );
    }
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

/// `cursor` is essentially arbitrary for d-special; setting `cursor` to
/// (lnum=1, col=1, off=0) to please the verifier, in case d-special deletes
/// the entire buffer.
pub fn reset_cursor_when_d_special(
    n_lines: usize,
    langle: &Position,
    rangle: &Position,
    cursor: &mut Position,
) {
    let llnum = langle.lnum;
    let rlnum = rangle.lnum;
    let Position { lnum, col, off } = cursor;
    if llnum == 1 && rlnum == n_lines {
        *lnum = 1;
        *col = 1;
        *off = 0;
    }
}
