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

use super::core::buffer::ParsedBufferLike;
use super::core::iter::{ExtendedInlineTokensIter, TokenLikeExt};
use super::core::motion::Motion;
use super::core::position::OperatorRange;
use super::primitives::misc::Dec;

/// Some adjustment to rangle in operator-pending mode of word text object for
/// |c| and |y| operators.
pub trait TextObjectYankChangeFixRangle {
    fn text_object_yank_change_fix<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error>;
}

impl<'o> TextObjectYankChangeFixRangle for OperatorRange<'o> {
    fn text_object_yank_change_fix<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error> {
        if self.operator != b"y" && self.operator != b"c" {
            return Ok(());
        }

        let cursor_token = ExtendedInlineTokensIter::new(
            buffer.getline_parsed(self.rangle.lnum)?,
        )
        .into_col(self.rangle.col);
        if cursor_token.is_empty() {
            Dec::new(true, true).map(buffer, 1, &mut self.rangle)?;
        }
        Ok(())
    }
}
