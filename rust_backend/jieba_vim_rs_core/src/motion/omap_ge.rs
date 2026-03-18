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

use crate::BufferLike;
use crate::token::JiebaPlaceholder;

use super::api::{OmapOutput, WordMotion};
use super::core::buffer::ParsedBuffer;
use super::core::motion::{Motion, MotionState};
use super::core::position::{OperatorRange, Position};
use super::policy::d_special::DSpecial;
use super::policy::exclusive_special::ExclusiveSpecial;
use super::policy::position_cursor::PositionCursor;
use super::policy::zero_off::ZeroOff;
use super::primitives::text_object::BackwardEndWord;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `ge` (if `word` is `true`) or `gE` (if `word` is `false`) in
    /// operator-pending mode. Take in current `cursor_pos` (0, lnum, col, off,
    /// _), and return the operation range and the new cursor position.
    ///
    /// # Basics
    ///
    /// `ge`/`gE` jumps to the last character of previous word. Empty line is
    /// considered as a word. If there's no previous word except for the empty
    /// line, issue `prevent_change` flag.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    pub fn omap_ge<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: Position,
        count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut orng = OperatorRange::new_inclusive(cursor_pos, operator);
        orng.langle.zero_off();
        let mut motion_rangle = BackwardEndWord::new(false);
        let s = motion_rangle.map(&mut buffer, count, &mut orng.rangle)?;
        if s == MotionState::Success {
            orng.exclusive_special(&mut buffer)?;
            orng.d_special(&mut buffer)?;
        }
        orng.cursor = orng.rangle;
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
