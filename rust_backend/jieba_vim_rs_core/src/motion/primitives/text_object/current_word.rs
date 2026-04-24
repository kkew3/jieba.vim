// Copyright 2026 Kaiwen Wu. All Rights Reserved.
// Portions Copyright (c) by Bram Moolenaar and others.
//
// This module contains code adapted from Vim's textobject.c. The Vim License
// applies to the adapted portions. See the vim-LICENSE.txt file in the project
// root for the full license text.
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

use super::*;

/// Find word under cursor.
pub struct CurrentWord {
    /// True to include word and whitespace.
    include: bool,
}

impl CurrentWord {
    /// Pass true to `include` to include word and whitespace.
    pub fn new(include: bool) -> Self {
        Self { include }
    }
}

impl Motion<VisualRange> for CurrentWord {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        mut count: u64,
        cursor: &mut VisualRange,
    ) -> Result<MotionState, B::Error> {
        let mut include_white = false;
        let mut start_pos = None;
        if count > 0 && cursor.langle == cursor.rangle {
            let cursor_token = get_cursor_token(&mut cursor.rangle, buffer)?;
            let start_col = cursor_token.first_char();
            cursor.rangle.col = start_col;
            start_pos = Some(cursor.rangle);
            if is_empty_or_whitespace(&cursor_token) == self.include {
                if EndWord::new(true, true).map(
                    buffer,
                    1,
                    &mut cursor.rangle,
                )? == MotionState::Failure
                {
                    return Ok(MotionState::Failure);
                }
            } else {
                let _ = ForwardWord::new(true).map(
                    buffer,
                    1,
                    &mut cursor.rangle,
                )?;
                // Won't panic because we are either at the start of a word or
                // at an Eol.
                Decl::default().map(buffer, 1, &mut cursor.rangle)?;

                include_white = self.include;
            }

            cursor.langle.col = start_col;

            count -= 1;
        }

        let mut inclusive = true;
        while count > 0 {
            inclusive = true;

            if cursor.rangle < cursor.langle {
                match decl_is_empty_or_whitespace(&cursor.rangle, buffer)? {
                    None => return Ok(MotionState::Failure),
                    Some(cls_eq_0) => {
                        let decl = Decl::default();
                        // In Bram's code this was written as:
                        // > if (include != (cls() != 0))
                        if self.include == cls_eq_0 {
                            if decl.chain(BackwardWord::new(true)).map(
                                buffer,
                                1,
                                &mut cursor.rangle,
                            )? == MotionState::Failure
                            {
                                return Ok(MotionState::Failure);
                            }
                        } else {
                            if decl.chain(BackwardEndWord::new(true)).map(
                                buffer,
                                1,
                                &mut cursor.rangle,
                            )? == MotionState::Failure
                            {
                                return Ok(MotionState::Failure);
                            }
                            Incl::default().map(
                                buffer,
                                1,
                                &mut cursor.rangle,
                            )?;
                        }
                    }
                }
            } else {
                match incl_is_empty_or_whitespace(&cursor.rangle, buffer)? {
                    None => {
                        // Inc to one pass last char in the buffer and fail.
                        Incl::default().map(buffer, 1, &mut cursor.rangle)?;
                        return Ok(MotionState::Failure);
                    }
                    Some(cls_eq_0) => {
                        // In Bram's code this was written as:
                        // > if (include != (cls() == 0))
                        if self.include != cls_eq_0 {
                            if Incl::default()
                                .chain(ForwardWord::new(true))
                                .map(buffer, 1, &mut cursor.rangle)?
                                == MotionState::Failure
                                && count > 1
                            {
                                return Ok(MotionState::Failure);
                            }
                            if Dec::new(false, false).map(
                                buffer,
                                1,
                                &mut cursor.rangle,
                            )? == MotionState::Failure
                            {
                                inclusive = false;
                            }
                        } else {
                            if Incl::default()
                                .chain(EndWord::new(true, true))
                                .map(buffer, 1, &mut cursor.rangle)?
                                == MotionState::Failure
                            {
                                return Ok(MotionState::Failure);
                            }
                        }
                    }
                }
            }

            count -= 1;
        }

        let cursor_token = get_cursor_token(&cursor.rangle, buffer)?;
        if include_white
            && (!is_empty_or_whitespace(&cursor_token)
                || (cursor.rangle.col == 1 && !inclusive))
            && let Some(start_pos) = start_pos.as_mut()
            && Dec::new(false, false).map(buffer, 1, start_pos)?
                == MotionState::Success
        {
            let cursor_token = get_cursor_token(start_pos, buffer)?;
            start_pos.col = cursor_token.first_char();
            if is_empty_or_whitespace(&cursor_token) && start_pos.col > 1 {
                cursor.langle = *start_pos;
            }
        }

        if cursor.visualmode == VisualMode::Line {
            cursor.visualmode = VisualMode::Char;
        }
        Ok(MotionState::Success)
    }
}

impl<'o> Motion<OperatorRange<'o>> for CurrentWord {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        mut count: u64,
        cursor: &mut OperatorRange,
    ) -> Result<MotionState, B::Error> {
        let mut include_white = false;
        let mut start_pos = None;
        if count > 0 {
            let cursor_token = get_cursor_token(&cursor.rangle, buffer)?;
            let start_col = cursor_token.first_char();
            cursor.rangle.col = start_col;
            start_pos = Some(cursor.rangle);
            if is_empty_or_whitespace(&cursor_token) == self.include {
                if EndWord::new(true, true).map(
                    buffer,
                    1,
                    &mut cursor.rangle,
                )? == MotionState::Failure
                {
                    return Ok(MotionState::Failure);
                }
            } else {
                let _ = ForwardWord::new(true).map(
                    buffer,
                    1,
                    &mut cursor.rangle,
                )?;
                // Won't panic because we are either at the start of a word or
                // at an Eol.
                Decl::default().map(buffer, 1, &mut cursor.rangle)?;

                include_white = self.include;
            }

            cursor.langle.col = start_col;

            count -= 1;
        }

        let mut inclusive = true;
        while count > 0 {
            inclusive = true;

            match incl_is_empty_or_whitespace(&cursor.rangle, buffer)? {
                None => {
                    // Inc to one pass last char in the buffer and fail.
                    Incl::default().map(buffer, 1, &mut cursor.rangle)?;
                    return Ok(MotionState::Failure);
                }
                Some(cls_eq_0) => {
                    // In Bram's code this was written as:
                    // > if (include != (cls() == 0))
                    if self.include != cls_eq_0 {
                        if Incl::default().chain(ForwardWord::new(true)).map(
                            buffer,
                            1,
                            &mut cursor.rangle,
                        )? == MotionState::Failure
                            && count > 1
                        {
                            return Ok(MotionState::Failure);
                        }
                        if Dec::new(false, false).map(
                            buffer,
                            1,
                            &mut cursor.rangle,
                        )? == MotionState::Failure
                        {
                            inclusive = false;
                        }
                    } else {
                        if Incl::default().chain(EndWord::new(true, true)).map(
                            buffer,
                            1,
                            &mut cursor.rangle,
                        )? == MotionState::Failure
                        {
                            return Ok(MotionState::Failure);
                        }
                    }
                }
            }

            count -= 1;
        }

        let cursor_token = get_cursor_token(&cursor.rangle, buffer)?;
        if include_white
            && (!is_empty_or_whitespace(&cursor_token)
                || (cursor.rangle.col == 1 && !inclusive))
            && let Some(start_pos) = start_pos.as_mut()
            && Dec::new(false, false).map(buffer, 1, start_pos)?
                == MotionState::Success
        {
            let cursor_token = get_cursor_token(start_pos, buffer)?;
            start_pos.col = cursor_token.first_char();
            if is_empty_or_whitespace(&cursor_token) && start_pos.col > 1 {
                cursor.langle = *start_pos;
            }
        }

        cursor.mtype = if inclusive {
            MotionType::CharInclusive
        } else {
            MotionType::CharExclusive
        };
        Ok(MotionState::Success)
    }
}

fn get_cursor_token<B: ParsedBufferLike + ?Sized>(
    cursor: &Position,
    buffer: &mut B,
) -> Result<GToken, B::Error> {
    Ok(
        ExtendedInlineTokensIter::new(buffer.getline_parsed(cursor.lnum)?)
            .into_col(cursor.col),
    )
}

fn is_empty_or_whitespace(token: &GToken) -> bool {
    match token {
        GToken::T(t) => match t.ty {
            TokenType::Space => true,
            TokenType::Word => false,
        },
        GToken::Eol(_) => true,
    }
}

/// Return true if the cursor position after [`Decl1`] would be on an empty or
/// space token. If `Decl1` would fail, return None.
fn decl_is_empty_or_whitespace<B: ParsedBufferLike + ?Sized>(
    cursor: &Position,
    buffer: &mut B,
) -> Result<Option<bool>, B::Error> {
    let cursor_token = get_cursor_token(cursor, buffer)?;
    let r = if !cursor_token.at_start(cursor.col) {
        // After `Decl1`, cursor would still stay on `cursor_token`.
        Some(is_empty_or_whitespace(&cursor_token))
    } else {
        // Else, we run `Decl1` on a copy of `cursor` and check its type.
        let mut cursor_copy = *cursor;
        match Decl::default().map(buffer, 1, &mut cursor_copy)? {
            MotionState::Failure => None,
            MotionState::Success => {
                let cursor_token_after_decl1 =
                    get_cursor_token(&cursor_copy, buffer)?;
                Some(is_empty_or_whitespace(&cursor_token_after_decl1))
            }
        }
    };
    Ok(r)
}

/// Return true if the cursor position after [`Incl1`] would be on an empty or
/// space token. If `Incl1` would fail, return None.
fn incl_is_empty_or_whitespace<B: ParsedBufferLike + ?Sized>(
    cursor: &Position,
    buffer: &mut B,
) -> Result<Option<bool>, B::Error> {
    // The procedure is basically the same, except that we will test for
    // `at_end()` in this case.
    let cursor_token = get_cursor_token(cursor, buffer)?;
    let r = if !cursor_token.is_empty() && !cursor_token.at_end(cursor.col) {
        // After `Incl1`, cursor would still stay on `cursor_token`.
        Some(is_empty_or_whitespace(&cursor_token))
    } else {
        // Else, we run `Incl1` on a copy of `cursor` and check its type.
        let mut cursor_copy = *cursor;
        match Incl::default().map(buffer, 1, &mut cursor_copy)? {
            MotionState::Failure => None,
            MotionState::Success => {
                let cursor_token_after_incl1 =
                    get_cursor_token(&cursor_copy, buffer)?;
                Some(is_empty_or_whitespace(&cursor_token_after_incl1))
            }
        }
    };
    Ok(r)
}
