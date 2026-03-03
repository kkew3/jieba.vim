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

/// The 4-element list of numbers \[0, lnum, col, off] as returned by Vim's
/// `getpos(...)` where ... equals `.` or `'{local_mark}`. `lnum` and `col`
/// are indexed from 1. `off` is indexed from 0.
pub type Position = [usize; 4];

/// The 5-element list of numbers \[0, lnum, col, off, curswant] as returned by
/// Vim's `getcurpos()`. `lnum`, `col` and `curswant` are indexed from 1. `off`
/// is indexed from 0.
pub type CursorPositionCurswant = [usize; 5];

/// The 3-element list of numbers \[lnum, col, off] obtained by stripping off
/// the leading zero in [`Position`].
pub type CurrentBufferPosition = [usize; 3];

/// A position in a buffer with at least (bufnum, lnum, col, off) info.
pub trait BasicPosition: Sized {
    fn bufnum(&self) -> usize;
    fn lnum(&self) -> usize;
    fn col(&self) -> usize;
    fn off(&self) -> usize;

    /// Convert to [`CurrentBufferPosition`] provided that `bufnum` is zero.
    fn try_to_cb_position(
        &self,
    ) -> Result<CurrentBufferPosition, PositionError> {
        if self.bufnum() == 0 {
            Ok([self.lnum(), self.col(), self.off()])
        } else {
            Err(PositionError::NonzeroBufnum)
        }
    }
}

impl BasicPosition for Position {
    fn bufnum(&self) -> usize {
        self[0]
    }

    fn lnum(&self) -> usize {
        self[1]
    }

    fn col(&self) -> usize {
        self[2]
    }

    fn off(&self) -> usize {
        self[3]
    }
}

impl BasicPosition for CursorPositionCurswant {
    fn bufnum(&self) -> usize {
        self[0]
    }

    fn lnum(&self) -> usize {
        self[1]
    }

    fn col(&self) -> usize {
        self[2]
    }

    fn off(&self) -> usize {
        self[3]
    }
}

impl BasicPosition for CurrentBufferPosition {
    fn bufnum(&self) -> usize {
        0
    }

    fn lnum(&self) -> usize {
        self[0]
    }

    fn col(&self) -> usize {
        self[1]
    }

    fn off(&self) -> usize {
        self[2]
    }

    fn try_to_cb_position(
        &self,
    ) -> Result<CurrentBufferPosition, PositionError> {
        Ok(*self)
    }
}

#[derive(Debug)]
pub enum PositionError {
    ZeroLnum,
    ZeroCol,
    NonzeroBufnum,
    ColTooLarge,
}

impl Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroLnum => {
                f.write_str("lnum =0 but should be indexed from 1")
            }
            Self::ZeroCol => f.write_str("col =0 but should be indexed from 1"),
            Self::NonzeroBufnum => f.write_str("bufnum >0 but should be 0"),
            Self::ColTooLarge => {
                f.write_str("col is larger than one plus the line length")
            }
        }
    }
}

impl std::error::Error for PositionError {}

pub trait PositionSanityCheck: Sized {
    /// Check for violation of indexing basis.
    fn check_indexing_basis(self) -> Result<Self, PositionError>;
}

impl<T: BasicPosition> PositionSanityCheck for T {
    fn check_indexing_basis(self) -> Result<Self, PositionError> {
        if self.lnum() == 0 {
            Err(PositionError::ZeroCol)
        } else if self.col() == 0 {
            Err(PositionError::ZeroCol)
        } else {
            Ok(self)
        }
    }
}
