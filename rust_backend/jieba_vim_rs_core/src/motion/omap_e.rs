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
        cursor: CursorPositionCurswant,
        count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let [_, lnum0, col0, _, _] = cursor;
        let output = self.nmap_e(buffer, cursor, count, word)?;
        let [_, lnum1, col1, _] = output.cursor;
        let langle = pos![lnum0, col0];
        let rangle = pos![lnum1, col1];
        if operator == b"d"
            && d_special::is_d_special(
                buffer,
                &self.tokenizer,
                langle,
                rangle,
                true,
                word,
            )?
        {
            Ok(OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"V",
                selection: b"inclusive",
                prevent_change: b"0",
            })
        } else {
            Ok(OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"v",
                selection: b"inclusive",
                prevent_change: b"0",
            })
        }
    }
}
