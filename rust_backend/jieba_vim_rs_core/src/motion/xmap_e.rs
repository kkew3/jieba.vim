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

use super::nmap_e::UnitNmapE;
use super::token_iter::{ExtendedInlineTokensIter, GToken, ParsedBuffer};
use super::word_motion::{
    ExtendedMotionState, Intolerable, Markovian, MarkovianUnit, Motion,
    UnitMotion,
};
use super::{WordMotion, XmapOutput};

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`)
    /// in visual mode. Take in current `visual_end` (0, lnum, col, off), and
    /// return the new `visual_end`. Note that `visual_begin` will be left
    /// intact. We denote both `word` and `WORD` with the English word "word"
    /// below.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character to the right of the last character of the last token in the
    ///   buffer. And the motion should be taken as a failure.
    pub fn xmap_e<'a, B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        visualmode: &'a [u8],
        visual_begin: Position,
        mut visual_end: Position,
        count: u64,
        word: bool,
    ) -> Result<XmapOutput<'a>, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut motion = Markovian::new(UnitXmapE);
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

pub struct UnitXmapE;

impl UnitMotion<Position> for UnitXmapE {
    fn unit_map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        buffer: &mut ParsedBuffer<'b, 'p, B, C>,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        use ExtendedMotionState::*;

        let s = UnitNmapE.unit_map(buffer, cursor)?;
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

impl MarkovianUnit<Position> for UnitXmapE {
    type FoldState = Intolerable;
}
