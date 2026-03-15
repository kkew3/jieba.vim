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

use super::api::{VisualMode, WordMotion, XmapOutput};
use super::core::buffer::ParsedBuffer;
use super::core::motion::{Markovian, Motion};
use super::core::position::Position;
use super::nmap_b::UnitNmapB;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `b` (if `word` is `true`) or `B` (if `word` is `false`)
    /// in visual mode. Take in current `visual_end` (0, lnum, col, off), and
    /// return the new `visual_end`. Note that `visual_begin` will be left
    /// intact. We denote both `word` and `WORD` with the English word "word"
    /// below.
    ///
    /// # Basics
    ///
    /// `b`/`B` jumps to the first character of previous word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    pub fn xmap_b<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        visualmode: VisualMode,
        visual_begin: Position,
        mut visual_end: Position,
        count: u64,
        word: bool,
    ) -> Result<XmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut motion = Markovian::new(UnitNmapB);
        let s = motion.map(&mut buffer, count, &mut visual_end)?;
        let prevent_change = s.into_prevent_change();
        Ok(XmapOutput {
            langle: visual_begin,
            rangle: visual_end,
            visualmode,
            prevent_change,
        })
    }
}
