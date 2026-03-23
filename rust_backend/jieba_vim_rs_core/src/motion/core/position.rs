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

//! Positions in a buffer.

use std::fmt::{self, Display};

use crate::motion::api::MotionType;

/// Position types related to FFI bindings.
pub mod ffi {
    /// The 4-element list of numbers \[0, lnum, col, off] as returned by Vim's
    /// `getpos(...)` where ... equals `.` or `'{local_mark}`. `lnum` and `col`
    /// are indexed from 1. `off` is indexed from 0.
    pub type Position = [usize; 4];

    /// The 5-element list of numbers \[0, lnum, col, off, curswant] as returned by
    /// Vim's `getcurpos()`. `lnum`, `col` and `curswant` are indexed from 1. `off`
    /// is indexed from 0.
    pub type CursorPositionCurswant = [usize; 5];
}

/// A position in current text buffer.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Position {
    /// The line number, indexed from 1.
    pub lnum: usize,
    /// The column number, indexed from 1.
    pub col: usize,
    /// The virtual column offset, indexed from 0.
    pub off: usize,
}

impl Position {
    /// Create a new [`Position`] with `off` equal zero.
    pub fn new(lnum: usize, col: usize) -> Self {
        Self { lnum, col, off: 0 }
    }
}

impl From<ffi::Position> for Position {
    fn from(value: ffi::Position) -> Self {
        let [bufnum, lnum, col, off] = value;
        assert_eq!(bufnum, 0);
        Self { lnum, col, off }
    }
}

impl From<ffi::CursorPositionCurswant> for Position {
    fn from(value: ffi::CursorPositionCurswant) -> Self {
        let [bufnum, lnum, col, off, _] = value;
        assert_eq!(bufnum, 0);
        Self { lnum, col, off }
    }
}

impl From<Position> for ffi::Position {
    fn from(value: Position) -> Self {
        [0, value.lnum, value.col, value.off]
    }
}

#[derive(Debug)]
pub enum PositionError {
    ColTooLarge,
}

impl Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColTooLarge => {
                f.write_str("col is larger than one plus the line length")
            }
        }
    }
}

impl std::error::Error for PositionError {}

/// An operator-pending range.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OperatorRange<'o> {
    pub cursor: Position,
    pub langle: Position,
    pub rangle: Position,
    pub mtype: MotionType,
    pub operator: &'o [u8],
}

impl<'o> OperatorRange<'o> {
    /// Construct a new characterwise exclusive empty range.
    pub fn new_exclusive(cursor: Position, operator: &'o [u8]) -> Self {
        Self {
            cursor,
            langle: cursor,
            rangle: cursor,
            mtype: MotionType::CharExclusive,
            operator,
        }
    }

    /// Construct a new characterwise inclusive empty range.
    pub fn new_inclusive(cursor: Position, operator: &'o [u8]) -> Self {
        Self {
            cursor,
            langle: cursor,
            rangle: cursor,
            mtype: MotionType::CharInclusive,
            operator,
        }
    }

    /// Return (langle, rangle) if langle <= rangle, else (rangle, langle).
    pub fn start_end_ord(&self) -> (&Position, &Position) {
        if self.langle <= self.rangle {
            (&self.langle, &self.rangle)
        } else {
            (&self.rangle, &self.langle)
        }
    }

    /// Return (langle, rangle) if langle <= rangle, else (rangle, langle).
    pub fn start_end_ord_mut(&mut self) -> (&mut Position, &mut Position) {
        if self.langle <= self.rangle {
            (&mut self.langle, &mut self.rangle)
        } else {
            (&mut self.rangle, &mut self.langle)
        }
    }
}
