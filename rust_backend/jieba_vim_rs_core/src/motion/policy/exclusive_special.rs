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

//! Quoted from vimhelp.org:
//!
//! > 1. If the motion is exclusive and the end of the motion is in column 1,
//! >    the end of the motion is moved to the end of the previous line and the
//! >    motion becomes inclusive.  Example: "}" moves to the first line after
//! >    a paragraph, but "d}" will not include that line.
//! >
//! > 2. If the motion is exclusive, the end of the motion is in column 1
//! >    and the start of the motion was at or before the first non-blank
//! >    in the line, the motion becomes linewise.  Example: If a paragraph
//! >    begins with some blanks and you do "d}" while standing on the first
//! >    non-blank, all the lines of the paragraph are deleted, including the
//! >    blanks.  If you do a put now, the deleted lines will be inserted below
//! >    the cursor position.
//!
//! Check <https://vimhelp.org/motion.txt.html#exclusive> for details.

use crate::motion::api::MotionType;
use crate::token::TokenLike;

use super::core::buffer::ParsedBufferLike;
use super::core::iter::{ExtendedInlineTokensIter, GToken};
use super::core::position::OperatorRange;
use super::primitives::predicate::OnOrBeforeFirstNonBlanks;

/// Check if current motion satisfies the two exceptions for
/// exclusive motions, and update current motion accordingly. See
/// <https://vimhelp.org/motion.txt.html#exclusive> for details.
pub trait ExclusiveSpecial {
    fn exclusive_special<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error>;
}

impl<'o> ExclusiveSpecial for OperatorRange<'o> {
    fn exclusive_special<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error> {
        // exclusive-special applies to characterwise exclusive motions.
        if self.mtype != MotionType::CharExclusive {
            return Ok(());
        }

        let (start, end) = self.start_end_ord_mut();
        // exclusive-special appies to motions ending in column 1.
        if end.col > 1 {
            return Ok(());
        }
        // exclusive-special applies to multi-line motions.
        if start.lnum == end.lnum {
            return Ok(());
        }

        end.lnum -= 1;
        if start.on_or_before_first_non_blank(buffer)? {
            self.mtype = MotionType::LineInclusive;
        } else {
            let tokens = buffer.getline_parsed(end.lnum)?;
            let mut line = ExtendedInlineTokensIter::new(tokens).rev();
            let eol = line.next().unwrap();
            match eol {
                GToken::Eol(col) => {
                    if col > 1 {
                        let last_token = line.next().unwrap();
                        end.col = last_token.last_char();
                        self.mtype = MotionType::CharInclusive;
                    }
                }
                GToken::T(_) => unreachable!(),
            }
        }
        Ok(())
    }
}
