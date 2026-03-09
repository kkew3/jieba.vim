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

use crate::token::JiebaPlaceholder;
use crate::{BufferLike, CursorPositionCurswant, Position};

use super::token_iter::ParsedBuffer;
use super::word_motion::{
    ExtendedMotionState, Intolerable, Markovian, MarkovianUnit, Motion,
    OneOffMotion, OneOffUnit, SuppressFailure, UnitMotion,
};
use super::xmap_e::UnitXmapE;
use super::{OmapOutput, WordMotion, d_special};

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
        cursor_pos: CursorPositionCurswant,
        count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let [bufnum, lnum, col, off, _] = cursor_pos;
        let mut langle = [bufnum, lnum, col, off];
        let mut rangle = langle;
        let mut motion_langle = OneOffMotion::new(UnitOmapELangle);
        let mut motion_rangle = Markovian::new(UnitOmapERangle);
        let _ = motion_langle.map(&mut buffer, count, &mut langle)?;
        let prevent_change = motion_rangle
            .map(&mut buffer, count, &mut rangle)?
            .into_prevent_change();
        let output = if operator == b"d"
            && d_special::is_d_special(&mut buffer, langle, rangle, true)?
        {
            let mut cursor = langle;
            let n_lines = buffer.lines()?;
            d_special::reset_cursor_when_d_special(
                n_lines,
                &langle,
                &rangle,
                &mut cursor,
            );
            OmapOutput {
                cursor,
                langle,
                rangle,
                visualmode: b"V",
                selection: b"inclusive",
                prevent_change,
            }
        } else {
            OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"v",
                selection: b"inclusive",
                prevent_change,
            }
        };
        Ok(output)
    }
}

pub struct UnitOmapELangle;

impl UnitMotion<Position> for UnitOmapELangle {
    fn unit_map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        _buffer: &mut ParsedBuffer<'b, 'p, B, C>,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        let [_, _, _, off] = cursor;
        *off = 0;
        Ok(ExtendedMotionState::Success)
    }
}

impl OneOffUnit<Position> for UnitOmapELangle {
    type FoldState = Intolerable;
}

pub struct UnitOmapERangle;

impl UnitMotion<Position> for UnitOmapERangle {
    fn unit_map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        buffer: &mut ParsedBuffer<'b, 'p, B, C>,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        UnitXmapE.unit_map(buffer, cursor)
    }
}

impl MarkovianUnit<Position> for UnitOmapERangle {
    // The `omap_e` motion always succeeds.
    type FoldState =
        SuppressFailure<<UnitXmapE as MarkovianUnit<Position>>::FoldState>;
}
