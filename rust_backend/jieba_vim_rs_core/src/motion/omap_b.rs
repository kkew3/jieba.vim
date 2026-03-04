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
use crate::{BufferLike, CursorPositionCurswant, pos};

use super::{OmapOutput, WordMotion, d_special};

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
    ///
    /// # Panics
    ///
    /// - If current cursor `col` is to the right of the last token in current
    ///   line of the buffer.
    pub fn omap_b<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor: CursorPositionCurswant,
        count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let [_, lnum0, col0, _, _] = cursor;
        let output = self.nmap_b(buffer, cursor, count, word)?;
        let [_, lnum1, col1, _] = output.cursor;
        let langle = pos![lnum0, col0];
        let rangle = pos![lnum1, col1];
        if operator == b"d"
            && d_special::is_d_special(
                buffer,
                &self.tokenizer,
                langle,
                rangle,
                false,
                word,
            )?
        {
            Ok(OmapOutput {
                cursor: rangle,
                langle,
                rangle,
                visualmode: b"V",
                selection: b"exclusive",
                prevent_change: output.prevent_change,
            })
        } else {
            Ok(OmapOutput {
                cursor: rangle,
                langle,
                rangle,
                visualmode: b"v",
                selection: b"exclusive",
                prevent_change: output.prevent_change,
            })
        }
    }
}
