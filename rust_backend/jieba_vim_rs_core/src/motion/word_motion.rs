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

use crate::token::{JiebaPlaceholder, Tokenizer};
use crate::{BufferLike, CursorPositionCurswant, Position};

pub struct WordMotion<C> {
    pub(super) tokenizer: Tokenizer<C>,
}

pub struct NmapOutput {
    pub cursor: Position,
    pub prevent_change: &'static [u8],
}

pub struct XmapOutput<'a> {
    pub langle: Position,
    pub rangle: Position,
    pub visualmode: &'a [u8],
    pub prevent_change: &'static [u8],
}

pub struct OmapOutput {
    pub cursor: Position,
    pub langle: Position,
    pub rangle: Position,
    pub visualmode: &'static [u8],
    pub selection: &'static [u8],
    pub prevent_change: &'static [u8],
}

impl<C> WordMotion<C> {
    pub fn new(tokenizer: Tokenizer<C>) -> Self {
        Self { tokenizer }
    }

    pub fn get_tokenizer_mut(&mut self) -> &mut Tokenizer<C> {
        &mut self.tokenizer
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    pub fn nmap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &[u8],
        cursor: CursorPositionCurswant,
        mut count: u64,
    ) -> Result<NmapOutput, B::Error> {
        if count == 0 {
            count = 1;
        }
        match motion {
            b"w" | b"W" => {
                self.nmap_w(buffer, cursor, count, motion[0] == b'w')
            }
            b"b" | b"B" => {
                self.nmap_b(buffer, cursor, count, motion[0] == b'b')
            }
            b"e" | b"E" => {
                self.nmap_e(buffer, cursor, count, motion[0] == b'e')
            }
            b"ge" | b"gE" => {
                self.nmap_ge(buffer, cursor, count, motion[1] == b'e')
            }
            _ => unreachable!("invalid motion key sequence: {:?}", motion),
        }
    }

    pub fn xmap<'a, B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        visualmode: &'a [u8],
        motion: &[u8],
        visual_begin: Position,
        visual_end: Position,
        mut count: u64,
    ) -> Result<XmapOutput<'a>, B::Error> {
        if count == 0 {
            count = 1;
        }
        match motion {
            b"w" | b"W" => self.xmap_w(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[0] == b'w',
            ),
            b"b" | b"B" => self.xmap_b(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[0] == b'b',
            ),
            b"e" | b"E" => self.xmap_e(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[0] == b'e',
            ),
            b"ge" | b"gE" => self.xmap_ge(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[1] == b'e',
            ),
            _ => unreachable!("invalid motion key sequence: {:?}", motion),
        }
    }

    pub fn omap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &[u8],
        cursor: CursorPositionCurswant,
        mut count: u64,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        if count == 0 {
            count = 1;
        }
        match motion {
            b"w" | b"W" => {
                self.omap_w(buffer, cursor, count, motion[0] == b'w', operator)
            }
            b"b" | b"B" => {
                self.omap_b(buffer, cursor, count, motion[0] == b'b', operator)
            }
            b"e" | b"E" => {
                self.omap_e(buffer, cursor, count, motion[0] == b'e', operator)
            }
            b"ge" | b"gE" => {
                self.omap_ge(buffer, cursor, count, motion[1] == b'e', operator)
            }
            _ => unreachable!("invalid motion key sequence: {:?}", motion),
        }
    }
}
