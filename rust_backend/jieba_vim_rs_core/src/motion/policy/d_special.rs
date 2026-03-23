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

use crate::motion::api::MotionType;
use crate::token::TokenType;

use super::core::buffer::ParsedBufferLike;
use super::core::iter::{ExtendedInlineTokensIter, GToken, TokenLikeExt};
use super::core::position::OperatorRange;
use super::primitives::predicate::OnOrBeforeFirstNonBlanks;

/// Check if current motion satisfies d-special case, and make the motion
/// linewise if true. See <https://vimhelp.org/change.txt.html#d-special> for
/// details.
pub trait DSpecial {
    fn d_special<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error>;
}

impl<'o> DSpecial for OperatorRange<'o> {
    fn d_special<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error> {
        // d-special applies only to operator "d".
        if self.operator != b"d" {
            return Ok(());
        }

        // d-special applies to characterwise motions.
        if self.mtype != MotionType::CharExclusive
            && self.mtype != MotionType::CharInclusive
        {
            return Ok(());
        }

        // d-special applies to multi-line motions.
        let (start, end) = self.start_end_ord();
        if start.lnum == end.lnum {
            return Ok(());
        }

        if !start.on_or_before_first_non_blank(buffer)? {
            return Ok(());
        }

        let rlnum = end.lnum;
        let rcol = end.col;

        let rtokens = buffer.getline_parsed(rlnum)?;
        let mut rline = ExtendedInlineTokensIter::new(&rtokens).skip_col(rcol);
        if let GToken::T(t) = rline.next().unwrap()
            && t.ty == TokenType::Word
        {
            if self.mtype == MotionType::CharExclusive {
                return Ok(());
            }
            if !t.at_end(rcol) {
                return Ok(());
            }
        }
        for token in rline {
            if let GToken::T(t) = token
                && t.ty == TokenType::Word
            {
                return Ok(());
            }
        }

        // Make the motion linewise if it's indeed d-special.
        self.mtype = MotionType::LineInclusive;
        Ok(())
    }
}
