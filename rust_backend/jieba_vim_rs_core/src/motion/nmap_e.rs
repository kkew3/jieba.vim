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
use super::motions::text_object::EndWord;
use super::policy::adjust_cursor::AdjustCursor;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`)
    /// in normal mode. Take in current `cursor` (0, lnum, col, off, _), and
    /// return the new cursor position. We denote both `word` and `WORD` with
    /// the English word "word" below.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If there is no next word to the right of current cursor, jump to the
    ///   last character of the last token in the buffer. And the motion should
    ///   be taken as a failure.
    pub fn nmap_e<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        mut cursor: Position,
        count: u64,
        word: bool,
    ) -> Result<NmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut motion = EndWord::new(false, false);
        let s = motion.map(&mut buffer, count, &mut cursor)?;
        cursor.adjust_cursor(&mut buffer)?;
        Ok(NmapOutput {
            cursor,
            prevent_change: s.into_prevent_change(),
        })
    }
}
