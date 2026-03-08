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

use crate::token::{JiebaPlaceholder, TokenLike};
use crate::{BufferLike, Position};

use super::nmap_w::UnitNmapW;
use super::token_iter::{ExtendedInlineTokensIter, GToken, ParsedBuffer};
use super::word_motion::{
    ExtendedMotionState, Markovian, MarkovianUnit, Motion, SemiTolerable,
    UnitMotion,
};
use super::{WordMotion, XmapOutput};

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`)
    /// in visual mode. Take in current `visual_end` (0, lnum, col, off),
    /// and return the new visual_end. Note that `visual_begin` will be left
    /// intact. We denote both `word` and `WORD` with the English word "word"
    /// below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current `visual_end` is on the last character of the last token
    ///   in the buffer, jump to the right of of that token. And the motion
    ///   should be taken as a failure.
    /// - If current cursor is on the one character to the right of the last
    ///   character of the last token in the buffer, no further jump should be
    ///   made. And the motion should be taken as a failure.
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character to the right of the last character of the last token in the
    ///   buffer.
    pub fn xmap_w<'a, B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        visualmode: &'a [u8],
        visual_begin: Position,
        mut visual_end: Position,
        count: u64,
        word: bool,
    ) -> Result<XmapOutput<'a>, B::Error> {
        let buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut motion = Markovian::new(UnitXmapW);
        let s = motion.map(&buffer, count, &mut visual_end)?;
        let prevent_change = s.into_prevent_change();
        Ok(XmapOutput {
            langle: visual_begin,
            rangle: visual_end,
            visualmode,
            prevent_change,
        })
    }
}

pub struct UnitXmapW;

impl UnitMotion<Position> for UnitXmapW {
    fn unit_map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        buffer: &ParsedBuffer<'b, 'p, B, C>,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        use ExtendedMotionState::*;

        let s = UnitNmapW.unit_map(buffer, cursor)?;
        if s == Failure || s == Pending {
            let [_, lnum, col, _] = cursor;
            let tokens = buffer.getline_parsed(*lnum)?;
            let cursor_token = ExtendedInlineTokensIter::new(&tokens)
                .skip_col(*col)
                .expect("col too large")
                .next()
                .unwrap();
            if let GToken::T(t) = cursor_token {
                *col = t.last_char1();
            }
        }
        Ok(s)
    }
}

impl MarkovianUnit<Position> for UnitXmapW {
    type FoldState = SemiTolerable;
}
