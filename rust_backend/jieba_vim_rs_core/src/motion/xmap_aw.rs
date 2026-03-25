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

use super::api::{VisualMode, WordMotion, XmapOutput};
use super::core::buffer::ParsedBuffer;
use super::core::motion::Motion;
use super::core::position::{Position, VisualRange};
use super::primitives::text_object::CurrentWordVisual;

impl<C: JiebaPlaceholder> WordMotion<C> {
    pub fn xmap_aw<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        visualmode: VisualMode,
        visual_begin: Position,
        visual_end: Position,
        count: u64,
        word: bool,
    ) -> Result<XmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut motion = CurrentWordVisual::new(true);
        let mut vrng = VisualRange {
            langle: visual_begin,
            rangle: visual_end,
            visualmode,
        };
        let s = motion.map(&mut buffer, count, &mut vrng)?;
        let prevent_change = s.into_prevent_change();
        Ok(XmapOutput {
            langle: vrng.langle,
            rangle: vrng.rangle,
            visualmode: vrng.visualmode,
            prevent_change,
        })
    }
}
