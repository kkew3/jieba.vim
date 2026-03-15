// Copyright 2026 Kaiwen Wu. All Rights Reserved.
// Portions Copyright (c) by Bram Moolenaar and others.
//
// This file contains code adapted from Vim's normal.c. The Vim License applies
// to the adapted portions. See the vim-LICENSE.txt file in the project root
// for the full license text.
//
// In accordance with the Vim License (Section II):
// - Contact: Kaiwen Wu <kps6326@hotmail.com>
// - Changes are available to the Vim maintainer upon request.
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

use crate::motion::core::buffer::ParsedBufferLike;
use crate::motion::core::iter::{ExtendedInlineTokensIter, GToken};
use crate::motion::core::position::Position;
use crate::token::TokenLike;

/// Used after a movement command: If the cursor ends up on the Eol(_),
/// may move it back to the last character in the line and make the motion
/// inclusive.
pub trait AdjustCursor {
    fn adjust_cursor<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error>;
}

impl AdjustCursor for Position {
    /// Adjust cursor after an nmap motion.
    fn adjust_cursor<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
    ) -> Result<(), B::Error> {
        let tokens = buffer.getline_parsed(self.lnum)?;
        let mut line = ExtendedInlineTokensIter::new(tokens)
            .take_col_rev(self.col)
            .expect("cursor col too large");
        let cursor_token = line.next().unwrap();
        if let GToken::Eol(col) = cursor_token
            && col > 1
        {
            // There must be at least one token before an Eol(_).
            let prev_token = line.next().unwrap();
            self.col = prev_token.last_char();
        }
        Ok(())
    }
}
