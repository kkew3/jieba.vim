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

use super::api::{ImapOutput, WordMotion};
use super::core::buffer::ParsedBuffer;
use super::core::motion::Motion;
use super::core::position::Position;
use super::primitives::text_object::BackwardWord;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `C-Left` in insert mode. Take in `cursor` (0, lnum, col,
    /// off, _), and return the new cursor position.
    ///
    /// # Basics
    ///
    /// Equivalent to `B` in normal mode, except that the motion never fails.
    pub fn imap_ctrl_left<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        mut cursor: Position,
    ) -> Result<ImapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, false);
        let mut motion = BackwardWord::new(false);
        let _ = motion.map(&mut buffer, 1, &mut cursor)?;
        Ok(ImapOutput { cursor })
    }
}
