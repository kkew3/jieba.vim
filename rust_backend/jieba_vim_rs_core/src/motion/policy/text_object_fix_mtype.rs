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

use crate::motion::api::MotionType;

use super::core::buffer::ParsedBufferLike;
use super::core::iter::{ExtendedInlineTokensIter, TokenLikeExt};
use super::core::position::OperatorRange;

/// Fix the motion type of word text object.
pub trait TextObjectMotionTypeFix {
    /// Pass true to `include` for |aw| and false for |iw|.
    fn text_object_motion_type_fix<B: ParsedBufferLike + ?Sized>(
        &mut self,
        include: bool,
        buffer: &mut B,
    ) -> Result<(), B::Error>;
}

impl<'o> TextObjectMotionTypeFix for OperatorRange<'o> {
    fn text_object_motion_type_fix<B: ParsedBufferLike + ?Sized>(
        &mut self,
        include: bool,
        buffer: &mut B,
    ) -> Result<(), B::Error> {
        let rangle_token = ExtendedInlineTokensIter::new(
            buffer.getline_parsed(self.rangle.lnum)?,
        )
        .into_col(self.rangle.col);
        if self.mtype == MotionType::CharInclusive
            && !(include
                || self.langle != self.rangle
                || !rangle_token.is_empty())
        {
            self.mtype = MotionType::CharExclusive;
        }
        Ok(())
    }
}
