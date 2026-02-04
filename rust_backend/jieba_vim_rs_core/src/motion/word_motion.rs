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

pub type Position = [i32; 4];
pub type CursorPosition = [i32; 5];

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

impl<C: JiebaPlaceholder> WordMotion<C> {
    pub fn nmap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &str,
        cursor: CursorPosition,
        count: u64,
    ) -> NmapOutput {
        todo!()
    }

    pub fn xmap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        visualmode: &str,
        motion: &str,
        visual_begin: Position,
        visual_end: Position,
        count: u64,
    ) -> XmapOutput {
        todo!()
    }

    pub fn omap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &str,
        cursor: CursorPosition,
        count: u64,
        operator: &str,
    ) -> OmapOutput {
        todo!()
    }
}
