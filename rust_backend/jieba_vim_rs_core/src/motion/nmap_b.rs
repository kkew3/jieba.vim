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

use super::api::{NmapOutput, WordMotion};
use super::core::buffer::ParsedBuffer;
use super::core::motion::Motion;
use super::core::position::Position;
use super::primitives::text_object::BackwardWord;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `b` (if `word` is `true`) or `B` (if `word` is `false`) in
    /// normal mode. Take in `cursor` (0, lnum, col, off, _), and return the
    /// new cursor position. We denote both `word` and `WORD` with the English
    /// word "word" below.
    ///
    /// # Basics
    ///
    /// `b`/`B` jumps to the first character of previous word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made. And the motion should be
    ///   taken as a failure.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    pub fn nmap_b<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        mut cursor: Position,
        count: u64,
        word: bool,
    ) -> Result<NmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut motion = BackwardWord::new(false);
        let s = motion.map(&mut buffer, count, &mut cursor)?;
        Ok(NmapOutput {
            cursor,
            prevent_change: s.into_prevent_change(),
        })
    }
}
