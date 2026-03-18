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
use super::core::position::{OperatorRange, Position};

/// Position the cursor after an operator-pending motion.
pub trait PositionCursor {
    fn position_cursor<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error>;
}

impl<'o> PositionCursor for OperatorRange<'o> {
    fn position_cursor<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error> {
        if self.operator == b"d" || self.operator == b"c" {
            let (start, end) = self.start_end_ord();
            if start.lnum == 1
                && end.lnum == buffer.lines()?
                && self.mtype == MotionType::LineInclusive
            {
                self.cursor = Position::new(1, 1);
            }
        }
        Ok(())
    }
}
