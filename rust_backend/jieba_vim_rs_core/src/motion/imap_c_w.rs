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
use crate::motion::core::motion::Motion;
use crate::token::JiebaPlaceholder;

use super::api::{ImapCtrlWOutput, WordMotion};
use super::core::buffer::ParsedBuffer;
use super::core::position::Position;
use super::primitives::text_object::PreviousWord;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Delete the word before the cursor.
    pub(crate) fn imap_ctrl_w_helper<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        mut cursor: Position,
    ) -> Result<ImapCtrlWOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, true);
        let mut motion = PreviousWord::default();
        let _ = motion.map(&mut buffer, 1, &mut cursor)?;
        Ok(ImapCtrlWOutput { cursor })
    }
}
