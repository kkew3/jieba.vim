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
use crate::token::{JiebaPlaceholder, TokenType};

use super::api::{MotionType, OmapOutput, WordMotion};
use super::core::buffer::{ParsedBuffer, ParsedBufferLike};
use super::core::iter::{ExtendedInlineTokensIter, GToken};
use super::core::motion::Motion;
use super::core::position::{OperatorRange, Position};
use super::motions::text_object::{EndWord, ForwardWord};
use super::policy::adjust_cursor::AdjustCursor;
use super::policy::d_special::DSpecial;
use super::policy::exclusive_special::ExclusiveSpecial;
use super::policy::position_cursor::PositionCursor;
use super::policy::yank_linewise::YankLinewise;
use super::policy::zero_off::ZeroOff;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`)
    /// in operator-pending mode.
    ///
    /// Take in current `cursor_pos` (0, lnum, col, off, _), and return the
    /// operation range and the new cursor position. We denote both `word` and
    /// `WORD` with the English word "word" below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - Quoted from Vim's help section "WORD": "When using the `w` motion in
    ///   combination with an operator and the last word moved over is at the
    ///   end of a line, the end of that word becomes the end of the operated
    ///   text, not the first word in the next line." (*)
    /// - Quoted from Vim's help section "WORD": "cw" and "cW" are treated like
    ///   "ce" and "cE" if the cursor is on a non-blank. This is because "cw"
    ///   is interpreted as change-word, and a word does not include the
    ///   following white space (see also cw). (**)
    pub fn omap_w<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor: Position,
        count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let mut orng = OperatorRange::new_exclusive(cursor, operator);
        orng.langle.zero_off();
        orng.cursor = orng.langle;
        if operator == b"c" && on_word(&orng.cursor, &mut buffer)? {
            orng.mtype = MotionType::CharInclusive;
            let mut motion_rangle = EndWord::new(true, false);
            let _ = motion_rangle.map(&mut buffer, count, &mut orng.rangle)?;
        } else {
            let mut motion_rangle = ForwardWord::new(true);
            let _ = motion_rangle.map(&mut buffer, count, &mut orng.rangle)?;
        }
        orng.adjust_cursor(&mut buffer)?;
        orng.exclusive_special(&mut buffer)?;
        orng.d_special(&mut buffer)?;
        orng.yank_linewise();
        orng.position_cursor(&mut buffer)?;
        let OperatorRange {
            cursor,
            langle,
            rangle,
            mtype,
            ..
        } = orng;
        Ok(OmapOutput {
            cursor,
            langle,
            rangle,
            mtype,
            prevent_change: false,
        })
    }
}

/// Return true if `cursor` is on a word.
fn on_word<B: ParsedBufferLike + ?Sized>(
    cursor: &Position,
    buffer: &mut B,
) -> Result<bool, B::Error> {
    let tokens = buffer.getline_parsed(cursor.lnum)?;
    let cursor_token = ExtendedInlineTokensIter::new(tokens)
        .skip_col(cursor.col)
        .expect("cursor col too large")
        .next()
        .unwrap();
    let on_word = match cursor_token {
        GToken::T(t) => match t.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
        GToken::Eol(_) => false,
    };
    Ok(on_word)
}
