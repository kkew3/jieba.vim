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

//! The main interface of module [`jieba_vim_rs_core::motion`](crate::motion).

use crate::BufferLike;
use crate::token::{JiebaPlaceholder, Tokenizer};

pub struct WordMotion<C> {
    pub(super) tokenizer: Tokenizer<C>,
}

/// Output types related to FFI bindings.
pub mod ffi {
    pub use crate::motion::core::position::ffi::{
        CursorPositionCurswant, Position,
    };

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
}

/// Output types for inner-crate use.
mod inner {
    use crate::motion::core::position::Position;

    use super::ffi;

    /// Visualmode used in xmap.
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum VisualMode {
        Char,
        Line,
        Block,
    }

    impl From<&[u8]> for VisualMode {
        fn from(value: &[u8]) -> Self {
            match value {
                b"v" => Self::Char,
                b"V" => Self::Line,
                b"\x16" | br"\<C-v>" | br"\u0016" => Self::Block,
                bs => panic!("cannot convert bytes `{:?}` to VisualMode", bs),
            }
        }
    }

    /// Output selection used to select operation range in omap.
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum Selection {
        /// Inclusive selection by characters.
        CharInclusive,
        /// Exclusive selection by characters.
        CharExclusive,
        /// Selection by line (always inclusive).
        LineInclusive,
        /// Implicit selection by operator-colon trick, e.g.
        /// `d:call cursor(lnum, col)<CR>`.
        OperatorColon,
    }

    pub struct NmapOutput {
        pub cursor: Position,
        pub prevent_change: bool,
    }

    pub struct XmapOutput {
        pub langle: Position,
        pub rangle: Position,
        pub visualmode: VisualMode,
        pub prevent_change: bool,
    }

    pub struct OmapOutput {
        pub cursor: Position,
        pub langle: Position,
        pub rangle: Position,
        pub selection: Selection,
        pub prevent_change: bool,
    }

    fn to_prevent_change(prevent_change: bool) -> &'static [u8] {
        if prevent_change { b"1" } else { b"0" }
    }

    impl From<NmapOutput> for ffi::NmapOutput {
        fn from(value: NmapOutput) -> Self {
            Self {
                cursor: value.cursor.into(),
                prevent_change: to_prevent_change(value.prevent_change),
            }
        }
    }

    impl<'a> From<XmapOutput> for ffi::XmapOutput<'a> {
        fn from(value: XmapOutput) -> Self {
            Self {
                langle: value.langle.into(),
                rangle: value.rangle.into(),
                visualmode: match value.visualmode {
                    VisualMode::Char => b"v",
                    VisualMode::Line => b"V",
                    VisualMode::Block => b"\x16",
                },
                prevent_change: to_prevent_change(value.prevent_change),
            }
        }
    }

    impl From<OmapOutput> for ffi::OmapOutput {
        fn from(value: OmapOutput) -> Self {
            let (visualmode, selection) = match value.selection {
                Selection::CharInclusive => (b"v", b"inclusive".as_ref()),
                Selection::CharExclusive => (b"v", b"exclusive".as_ref()),
                Selection::LineInclusive => (b"V", b"inclusive".as_ref()),
                Selection::OperatorColon => (b"v", b"colon".as_ref()),
            };
            Self {
                cursor: value.cursor.into(),
                langle: value.langle.into(),
                rangle: value.rangle.into(),
                visualmode,
                selection,
                prevent_change: to_prevent_change(value.prevent_change),
            }
        }
    }
}

pub(crate) use inner::{
    NmapOutput, OmapOutput, Selection, VisualMode, XmapOutput,
};

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
        cursor: ffi::CursorPositionCurswant,
        mut count: u64,
    ) -> Result<ffi::NmapOutput, B::Error> {
        if count == 0 {
            count = 1;
        }
        let cursor = cursor.into();
        let output = match motion {
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
        }?;
        Ok(output.into())
    }

    pub fn xmap<'a, B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        visualmode: &'a [u8],
        motion: &[u8],
        visual_begin: ffi::Position,
        visual_end: ffi::Position,
        mut count: u64,
    ) -> Result<ffi::XmapOutput<'a>, B::Error> {
        if count == 0 {
            count = 1;
        }
        let visualmode = visualmode.into();
        let visual_begin = visual_begin.into();
        let visual_end = visual_end.into();
        let output = match motion {
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
        }?;
        Ok(output.into())
    }

    pub fn omap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &[u8],
        cursor: ffi::CursorPositionCurswant,
        mut count: u64,
        operator: &[u8],
    ) -> Result<ffi::OmapOutput, B::Error> {
        if count == 0 {
            count = 1;
        }
        let cursor = cursor.into();
        let output = match motion {
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
        }?;
        Ok(output.into())
    }
}
