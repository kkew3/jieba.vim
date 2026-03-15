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

use super::api::{OmapOutput, Selection, WordMotion};
use super::core::buffer::ParsedBuffer;
use super::core::motion::{Markovian, Motion, MotionState};
use super::core::position::Position;
use super::nmap_b::UnitNmapB;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `b` (if `word` is `true`) or `B` (if `word` is `false`) in
    /// operator-pending mode. Take in current `cursor_pos` (0, lnum, col, off,
    /// _), and return the operation range and the new cursor position.
    ///
    /// # Basics
    ///
    /// `b`/`B` jumps to the first character of previous word. Empty line is
    /// considered as a word. If there's no previous word except for the empty
    /// line, issue `prevent_change` flag.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    pub fn omap_b<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: Position,
        count: u64,
        word: bool,
        _operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let langle = cursor_pos;
        let mut rangle = langle;
        let mut motion = Markovian::new(UnitNmapB);
        // Motion state is transitive from nmap_b to omap_b.
        let output = match motion.map(&mut buffer, count, &mut rangle)? {
            MotionState::Failure => OmapOutput {
                cursor: rangle,
                langle,
                rangle,
                selection: Selection::CharExclusive, // is arbitrary due to the failure
                prevent_change: true,
            },
            MotionState::Success => {
                // Apply operator-colon trick whatsoever.
                // A bit weird, but seems to work.
                OmapOutput {
                    // When using operator-colon trick, `cursor` value is
                    // arbitrary. We pick this value to ensure that the
                    // verifier is satisfactory, as it's always a valid
                    // position even if the buffer becomes empty due to
                    // d-special.
                    cursor: Position::new(1, 1),
                    langle,
                    rangle,
                    selection: Selection::OperatorColon,
                    prevent_change: false,
                }
            }
        };
        Ok(output)
    }
}
