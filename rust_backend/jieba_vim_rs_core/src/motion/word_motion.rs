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
use crate::token::{JiebaPlaceholder, Tokenizer};

pub struct WordMotion<C> {
    pub(super) tokenizer: Tokenizer<C>,
}

pub type Position = [usize; 4];
pub type CursorPosition = [usize; 5];

pub struct NmapOutput {
    pub cursor: CursorPosition,
}

pub struct XmapOutput {
    pub langle: Position,
    pub rangle: Position,
    pub visualmode: String,
}

pub struct OmapOutput {
    pub cursor: CursorPosition,
    pub langle: Position,
    pub rangle: Position,
    pub prevent_change: String,
}

impl<C> WordMotion<C> {
    pub fn new(tokenizer: Tokenizer<C>) -> Self {
        Self { tokenizer }
    }

    pub fn get_tokenizer_mut(&mut self) -> &mut Tokenizer<C> {
        &mut self.tokenizer
    }
}

fn get_motion_bytes(motion: &str) -> &[u8] {
    &motion.as_bytes()[..2.min(motion.len())]
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    pub fn nmap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &str,
        cursor: CursorPosition,
        count: u64,
    ) -> Result<NmapOutput, B::Error> {
        let [_, lnum, col, _, _] = cursor;
        let col_m1 = col - 1;
        let motion_bytes = get_motion_bytes(motion);
        let (lnum, col_m1) = match motion_bytes {
            b"w" | b"W" => {
                self.nmap_w(
                    buffer,
                    (lnum, col_m1),
                    count,
                    motion_bytes[0] == b'w',
                )?
                .new_cursor_pos
            }
            b"b" | b"B" => {
                self.nmap_b(
                    buffer,
                    (lnum, col_m1),
                    count,
                    motion_bytes[0] == b'b',
                )?
                .new_cursor_pos
            }
            b"e" | b"E" => {
                self.nmap_e(
                    buffer,
                    (lnum, col_m1),
                    count,
                    motion_bytes[0] == b'e',
                )?
                .new_cursor_pos
            }
            b"ge" | b"gE" => {
                self.nmap_ge(
                    buffer,
                    (lnum, col_m1),
                    count,
                    motion_bytes[1] == b'e',
                )?
                .new_cursor_pos
            }
            _ => unreachable!("invalid motion key sequence: {}", motion),
        };
        Ok(NmapOutput {
            cursor: [0, lnum, col_m1 + 1, 0, col_m1 + 1],
        })
    }

    pub fn xmap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        visualmode: &str,
        motion: &str,
        visual_begin: Position,
        visual_end: Position,
        count: u64,
    ) -> Result<XmapOutput, B::Error> {
        let [_, ve_lnum, ve_col, _] = visual_end;
        let ve_col_m1 = ve_col - 1;
        let motion_bytes = get_motion_bytes(motion);
        let (ve_lnum, ve_col_m1) = match motion_bytes {
            b"w" | b"W" => {
                self.xmap_w(
                    buffer,
                    (ve_lnum, ve_col_m1),
                    count,
                    motion_bytes[0] == b'w',
                )?
                .new_cursor_pos
            }
            b"b" | b"B" => {
                self.xmap_b(
                    buffer,
                    (ve_lnum, ve_col_m1),
                    count,
                    motion_bytes[0] == b'b',
                )?
                .new_cursor_pos
            }
            b"e" | b"E" => {
                self.xmap_e(
                    buffer,
                    (ve_lnum, ve_col_m1),
                    count,
                    motion_bytes[0] == b'e',
                )?
                .new_cursor_pos
            }
            b"ge" | b"gE" => {
                self.xmap_ge(
                    buffer,
                    (ve_lnum, ve_col_m1),
                    count,
                    motion_bytes[1] == b'e',
                )?
                .new_cursor_pos
            }
            _ => unreachable!("invalid motion key sequence: {}", motion),
        };
        Ok(XmapOutput {
            langle: visual_begin,
            rangle: [0, ve_lnum, ve_col_m1 + 1, 0],
            visualmode: visualmode.into(),
        })
    }

    pub fn omap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &str,
        cursor: CursorPosition,
        count: u64,
        operator: &str,
    ) -> Result<OmapOutput, B::Error> {
        let [_, lnum, col, _, _] = cursor;
        let col_m1 = col - 1;
        let motion_bytes = get_motion_bytes(motion);
        let operator_bytes = &operator.as_bytes()[..1];
        let output = match (motion_bytes, operator_bytes) {
            (b"w", b"c") | (b"W", b"c") => self.omap_c_w(
                buffer,
                (lnum, col_m1),
                count,
                motion_bytes[0] == b'w',
            )?,
            (b"w", _) | (b"W", _) => self.omap_w(
                buffer,
                (lnum, col_m1),
                count,
                motion_bytes[0] == b'w',
            )?,
            (b"b", _) | (b"B", _) => self.omap_b(
                buffer,
                (lnum, col_m1),
                count,
                motion_bytes[0] == b'b',
            )?,
            (b"e", b"d") | (b"E", b"d") => self.omap_d_e(
                buffer,
                (lnum, col_m1),
                count,
                motion_bytes[0] == b'e',
            )?,
            (b"e", _) | (b"E", _) => self.omap_e(
                buffer,
                (lnum, col_m1),
                count,
                motion_bytes[0] == b'e',
            )?,
            (b"ge", b"d") | (b"gE", b"d") => self.omap_d_ge(
                buffer,
                (lnum, col_m1),
                count,
                motion_bytes[1] == b'e',
            )?,
            (b"ge", _) | (b"gE", _) => self.omap_ge(
                buffer,
                (lnum, col_m1),
                count,
                motion_bytes[1] == b'e',
            )?,
            _ => unreachable!("invalid motion key sequence: {}", motion),
        };
        let (
            next_lnum,
            next_col_m1,
            langle_lnum,
            langle_col_m1,
            rangle_lnum,
            rangle_col_m1,
        ) = match motion_bytes {
            b"w" | b"W" | b"e" | b"E" => (
                lnum,
                col_m1,
                lnum,
                col_m1,
                output.new_cursor_pos.0,
                output.new_cursor_pos.1,
            ),
            b"b" | b"B" | b"ge" | b"gE" => (
                output.new_cursor_pos.0,
                output.new_cursor_pos.1,
                lnum,
                col_m1,
                output.new_cursor_pos.0,
                output.new_cursor_pos.1,
            ),
            _ => unreachable!("invalid motion key sequence: {}", motion),
        };
        let prevent_change =
            if output.prevent_change { "1" } else { "0" }.into();
        Ok(OmapOutput {
            cursor: [0, next_lnum, next_col_m1 + 1, 0, next_col_m1 + 1],
            langle: [0, langle_lnum, langle_col_m1 + 1, 0],
            rangle: [0, rangle_lnum, rangle_col_m1 + 1, 0],
            prevent_change,
        })
    }
}
