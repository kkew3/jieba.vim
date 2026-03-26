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

use crate::BufferLike;
use crate::token::JiebaPlaceholder;

use super::api::{OmapOutput, WordMotion};
use super::core::buffer::ParsedBuffer;
use super::core::motion::{Motion, MotionState};
use super::core::position::{OperatorRange, Position};
use super::policy::d_special::DSpecial;
use super::policy::exclusive_special::ExclusiveSpecial;
use super::policy::position_cursor::PositionCursor;
use super::policy::text_object_change_yank_fix_rangle::TextObjectYankChangeFixRangle;
use super::policy::text_object_fix_mtype::TextObjectMotionTypeFix;
use super::policy::yank_linewise::YankLinewise;
use super::primitives::text_object::CurrentWord;

impl<C: JiebaPlaceholder> WordMotion<C> {
    pub fn omap_aw<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor: Position,
        count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        // The exclusiveness is subject to the buffer context. Setting it
        // to an arbitrary value (exclusive here) won't affect its final
        // exclusiveness.
        let mut orng = OperatorRange::new_exclusive(cursor, operator);
        orng.langle.off = 0;
        let mut motion_rangle = CurrentWord::new(true);
        let s = motion_rangle.map(&mut buffer, count, &mut orng)?;
        orng.cursor = orng.langle;
        if s == MotionState::Success {
            if motion_rangle.need_fix_change_yank {
                orng.text_object_yank_change_fix(&mut buffer)?;
            }
            orng.text_object_motion_type_fix(true, &mut buffer)?;
            orng.exclusive_special(&mut buffer)?;
            orng.d_special(&mut buffer)?;
            orng.yank_linewise();
        } else {
            orng.cursor = orng.rangle;
        }
        orng.position_cursor(&mut buffer)?;
        let OperatorRange {
            cursor,
            langle,
            rangle,
            mtype,
            ..
        } = orng;
        Ok(OmapOutput {
            cursor,
            langle,
            rangle,
            mtype,
            prevent_change: s.into_prevent_change(),
        })
    }
}
