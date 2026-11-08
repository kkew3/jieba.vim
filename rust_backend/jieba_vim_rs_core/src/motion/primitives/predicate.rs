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

//! Several predicates on positions that are useful in motions.

use crate::BufferLike;

use super::core::position::Position;

pub trait OnOrBeforeFirstNonBlanks {
    /// Return true if `self` is either at the start of the first non-blank, or
    /// before the first non-blank.
    fn on_or_before_first_non_blank<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
    ) -> Result<bool, B::Error>;
}

impl OnOrBeforeFirstNonBlanks for Position {
    fn on_or_before_first_non_blank<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
    ) -> Result<bool, B::Error> {
        let line = buffer.getline(self.lnum)?;
        for c in line[..self.col - 1].chars().rev() {
            if !c.is_whitespace() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
