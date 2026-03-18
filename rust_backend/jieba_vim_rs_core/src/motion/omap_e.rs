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
use super::core::motion::Motion;
use super::core::position::{OperatorRange, Position};
use super::motions::text_object::EndWord;
use super::policy::adjust_cursor::AdjustCursor;
use super::policy::d_special::DSpecial;
use super::policy::position_cursor::PositionCursor;
use super::policy::zero_off::ZeroOff;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`) in
    /// operator-pending mode. Take in current `cursor_pos` (0, lnum, col, off,
    /// _), and return the operation range and the new cursor position.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the last character of the last token in the
    ///   buffer, no further jump should be made. But the motion should *not*
    ///   be taken as a failure.
    /// - If there is no next word to the right of current cursor, jump to the
    ///   last character of the last token in the buffer.
    pub fn omap_e<B: BufferLike + ?Sized>(
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
        orng.cursor = orng.langle;
        let mut motion_rangle = EndWord::new(false, false);
        let _ = motion_rangle.map(&mut buffer, count, &mut orng.rangle)?;
        orng.adjust_cursor(&mut buffer)?;
        orng.d_special(&mut buffer)?;
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
            prevent_change: false,
        })
    }
}
