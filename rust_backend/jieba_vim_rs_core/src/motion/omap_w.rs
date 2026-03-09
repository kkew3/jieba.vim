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

use std::marker::PhantomData;

use crate::token::{JiebaPlaceholder, TokenLike, TokenType};
use crate::{BufferLike, CursorPositionCurswant, Position};

use super::omap_e::UnitOmapERangle;
use super::token_iter::{
    ExtendedInlineTokensIter, GToken, ParsedBuffer, TokenLikeExt,
};
use super::word_motion::{
    ExtendedMotionState, FoldState, Intolerable, Markovian, MarkovianUnit,
    Motion, MotionState, UnitMotion,
};
use super::xmap_w::UnitXmapW;
use super::{OmapOutput, WordMotion, d_special};

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
        cursor: CursorPositionCurswant,
        count: u64,
        word: bool,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        assert!(count >= 1);
        let mut buffer = ParsedBuffer::new(buffer, &self.tokenizer, word);
        let [bufnum, lnum, col, off, _] = cursor;
        let langle = [bufnum, lnum, col, off];
        let mut rangle = langle;

        let tokens = buffer.getline_parsed(lnum)?;
        let mut line = ExtendedInlineTokensIter::new(&tokens)
            .skip_col(col)
            .expect("col too large")
            .peekable();
        let cursor_token = line.peek().unwrap();

        // |cw| special case (**).
        if let GToken::T(t) = cursor_token
            && t.ty == TokenType::Word
            && operator == b"c"
        {
            let mut motion = Markovian::new(UnitOmapERangle);
            let prevent_change = motion
                .map(&mut buffer, count, &mut rangle)?
                .into_prevent_change();
            return Ok(OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"v",
                selection: b"inclusive",
                prevent_change,
            });
        }

        // First stage.
        let mut motion_rangle_first_stage =
            MarkovianOmapW::new(UnitOmapWRangleFirstStage);
        let _ =
            motion_rangle_first_stage.map(&mut buffer, count, &mut rangle)?;

        // Operator-pending w special case (*).
        let output = match motion_rangle_first_stage.cursor_before_last_motion {
            None => OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"v",
                selection: b"exclusive",
                prevent_change: b"0",
            },
            Some(prev_cursor) => {
                let n_lines = buffer.lines()?;
                let selection = operator_w_special_case(
                    &mut buffer,
                    &langle,
                    &mut rangle,
                    prev_cursor,
                    count == 1,
                    motion_rangle_first_stage.last_motion_ends_with_failure,
                )?;
                match selection {
                    Selection::Colon => OmapOutput {
                        cursor: langle,
                        langle,
                        rangle,
                        visualmode: b"v",
                        selection: b"colon",
                        prevent_change: b"0",
                    },
                    Selection::Exclusive => {
                        if operator == b"d"
                            && d_special::is_d_special(
                                &mut buffer,
                                langle,
                                rangle,
                                false,
                            )?
                        {
                            // `cursor` is essentially arbitrary for d-special;
                            // setting to this value to please the verifier, in
                            // case d-special deletes the entire buffer.
                            let [_, llnum, _, _] = langle;
                            let [_, rlnum, _, _] = rangle;
                            let cursor = if llnum == 1 && rlnum == n_lines {
                                [bufnum, 1, 1, 0]
                            } else {
                                langle
                            };
                            OmapOutput {
                                cursor,
                                langle,
                                rangle,
                                visualmode: b"V",
                                selection: b"inclusive",
                                prevent_change: b"0",
                            }
                        } else {
                            OmapOutput {
                                cursor: langle,
                                langle,
                                rangle,
                                visualmode: b"v",
                                selection: b"exclusive",
                                prevent_change: b"0",
                            }
                        }
                    }
                    Selection::Inclusive => {
                        if operator == b"d"
                            && d_special::is_d_special(
                                &mut buffer,
                                langle,
                                rangle,
                                true,
                            )?
                        {
                            // `cursor` is essentially arbitrary for d-special;
                            // setting to this value to please the verifier, in
                            // case d-special deletes the entire buffer.
                            let [_, llnum, _, _] = langle;
                            let [_, rlnum, _, _] = rangle;
                            let cursor = if llnum == 1 && rlnum == n_lines {
                                [bufnum, 1, 1, 0]
                            } else {
                                langle
                            };
                            OmapOutput {
                                cursor,
                                langle,
                                rangle,
                                visualmode: b"V",
                                selection: b"inclusive",
                                prevent_change: b"0",
                            }
                        } else {
                            OmapOutput {
                                cursor: langle,
                                langle,
                                rangle,
                                visualmode: b"v",
                                selection: b"inclusive",
                                prevent_change: b"0",
                            }
                        }
                    }
                }
            }
        };

        Ok(output)
    }
}

/// The first stage of omap w for rangle.
pub struct UnitOmapWRangleFirstStage;

impl UnitMotion<Position> for UnitOmapWRangleFirstStage {
    fn unit_map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        buffer: &mut ParsedBuffer<'b, 'p, B, C>,
        cursor: &mut Position,
    ) -> Result<ExtendedMotionState, B::Error> {
        UnitXmapW.unit_map(buffer, cursor)
    }
}

impl MarkovianUnit<Position> for UnitOmapWRangleFirstStage {
    // The `omap_w` motion always succeeds.
    type FoldState = Intolerable;
}

pub struct MarkovianOmapW<M, S, P> {
    unit_motion: M,
    phantom_data: PhantomData<S>,
    cursor_before_last_motion: Option<P>,
    last_motion_ends_with_failure: bool,
}

impl<M, S, P> MarkovianOmapW<M, S, P> {
    pub fn new(unit_motion: M) -> Self {
        Self {
            unit_motion,
            phantom_data: PhantomData,
            cursor_before_last_motion: None,
            last_motion_ends_with_failure: false,
        }
    }
}

impl<P, M> Motion<P> for MarkovianOmapW<M, M::FoldState, P>
where
    M: MarkovianUnit<P>,
    P: Clone,
{
    fn map<'b, 'p, B: BufferLike + ?Sized, C: JiebaPlaceholder>(
        &mut self,
        buffer: &mut ParsedBuffer<'b, 'p, B, C>,
        mut count: u64,
        cursor: &mut P,
    ) -> Result<MotionState, B::Error> {
        let mut state = M::FoldState::default();
        self.cursor_before_last_motion = None;
        while count > 0 {
            let current_cursor_before_motion = cursor.clone();
            let s = self.unit_motion.unit_map(buffer, cursor)?;
            if s != ExtendedMotionState::Failure {
                self.cursor_before_last_motion =
                    Some(current_cursor_before_motion);
            }
            self.last_motion_ends_with_failure =
                s == ExtendedMotionState::Failure;
            if let Some(absorbing_state) = state.update(s) {
                return Ok(absorbing_state);
            }
            count -= 1;
        }
        Ok(state.finalize())
    }
}

enum Selection {
    Exclusive,
    Inclusive,
    /// The operator-colon trick. Works when the last token moved over is an
    /// Eol.
    Colon,
}

/// Return the selection type of the resulting operation range.
fn operator_w_special_case<'b, 'p, B, C>(
    buffer: &mut ParsedBuffer<'b, 'p, B, C>,
    langle: &Position,
    rangle: &mut Position,
    prev_cursor: Position,
    count_eq_1: bool,
    last_motion_ends_with_failure: bool,
) -> Result<Selection, B::Error>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    let [_, lnum0, col0, _] = langle;
    let [_, lnum1, col1, _] = rangle;

    let tokens = buffer.getline_parsed(*lnum0)?;
    // This is how the `langle_token` (commented below) would be defined:
    //
    // ```
    // let langle_token = ExtendedInlineTokensIter::new(&tokens)
    //     .skip_col(*col0)
    //     .expect("col0 too large")
    //     .next()
    //     .unwrap();
    // ```
    //
    // But we will postpone actually defining it until it's needed.
    if *lnum0 == *lnum1 {
        let rangle_token = ExtendedInlineTokensIter::new(&tokens)
            .take_col_rev(*col1)
            .expect("col1 too large")
            .next()
            .unwrap();

        // `rangle_token` can't be a Space, since xmap |w| never stops on a
        // Space, even if it's at eof.
        //
        // If `rangle_token` is a Word, then the last word moved over by |w|
        // can't be at the end of a line. First, `col0` < `col1`, since if
        // they are equal, then `col1` would have jumped over `rangle_token`.
        // Second, `col0` and `col1` are on two different tokens, since
        // `col1` must be at the start of `rangle_token`. Third, the last word
        // moved over by |w|, if exists, can't be at the end of a line, since
        // `rangle_token` is a Word token after `langle_token`. Don't need to
        // move langle/rangle in this case.
        //
        // If `rangle_token` is an Eol(_), then it must be at eof, since it's
        // the only circumstance where |w| would stop at an Eol(_). In this
        // case, we simply set the range up to `rangle_token`; though both
        // inclusive and exclusive leads to the same operation range, we will
        // pick exclusive. Don't need to move langle/rangle in this case.
        let s = match rangle_token {
            GToken::Eol(_) => Selection::Exclusive,
            GToken::T(t) => match t.ty {
                TokenType::Space => unreachable!(),
                TokenType::Word => Selection::Exclusive,
            },
        };
        return Ok(s);
    }

    // Else, `lnum0` < `lnum1`. Hence, `lnum0` can't be the last line.

    if count_eq_1 {
        // First, if `count` == 1 and `langle_token` is an Eol(_), then the
        // last "word" moved over is an Eol, and we simply apply operator-colon
        // trick and let Vim to handle the complexity.
        //
        // Second, if `count` == 1 and `langle_token` is a Space, then the last
        // "word" moved over must be `langle_token`, the Space. The next token
        // after `langle_token` can't be a Word, since otherwise |w| will land
        // the cursor on the start of that Word, but we have been asserted that
        // `langle_token` and `rangle_token` are on different lines. The next
        // token can't be a Space either, since two adjacent Spaces would have
        // been merged during tokenization. Thus, the next token must be an
        // Eol(_), and thus the conclusion.
        //
        // Third, if `count` == 1 and `langle_token` is a Word, then the last
        // "word" moved over must be `langle_token`, plus some trailing Spaces,
        // if any. The following tokens in the same line as `langle_token`
        // can't be Words, since otherwise, `rangle_token` would be on the same
        // line as `langle_token`, which we have asserted not.
        let mut line = ExtendedInlineTokensIter::new(&tokens)
            .skip_col(*col0)
            .expect("col0 too large")
            .peekable();
        let langle_token = line.next().unwrap();
        let s = match langle_token {
            GToken::Eol(_) => {
                *lnum1 = *lnum0 + 1;
                *col1 = 1;
                Selection::Colon
            }
            GToken::T(t) => match t.ty {
                TokenType::Space => {
                    *lnum1 = *lnum0;
                    *col1 = t.last_char();
                    Selection::Inclusive
                }
                TokenType::Word => {
                    *lnum1 = *lnum0;
                    *col1 = t.last_char();
                    if line.peek().is_some_and(|token| !token.is_empty()) {
                        let next_token = line.next().unwrap();
                        match next_token {
                            GToken::Eol(_) => unreachable!(),
                            GToken::T(next_t) => match next_t.ty {
                                TokenType::Space => *col1 = next_t.last_char(),
                                TokenType::Word => {
                                    unreachable!("can't be a Word")
                                }
                            },
                        }
                    }
                    Selection::Inclusive
                }
            },
        };
        return Ok(s);
    }

    if last_motion_ends_with_failure {
        // If last motion ends with Failure, then `rangle` must be an Eol at
        // eof. This means that the operation intends to span till eof. Thus,
        // if `rangle` is an empty line, we simply apply operator-colon trick;
        // else, span the motion range exclusively up to `rangle`.

        let [_, rangle_lnum, rangle_col, _] = *rangle;
        let rangle_token =
            ExtendedInlineTokensIter::new(&buffer.getline_parsed(rangle_lnum)?)
                .take_col_rev(rangle_col)
                .expect("rangle_col too large")
                .next()
                .unwrap();
        let s = match rangle_token {
            GToken::T(_) => unreachable!(),
            GToken::Eol(1) => {
                // Apply operator-colon trick. Since `rangle` must be
                // at eof, don't need to set `lnum1` here. And since at
                // empty line `col1` is already 1, don't need to set it
                // either.
                Selection::Colon
            }
            GToken::Eol(_) => Selection::Exclusive,
        };
        return Ok(s);
    }

    // Else, since `count` > 1, we focus on the last jump. The starting
    // point of the last jump can't be a Space. If it's a Space, it must be
    // `langle`, since |w| never stops in a Space. But this will contradict our
    // assumption that `count` > 1 and there is no failure in the motion. The
    // starting point can't be `langle` due to the same argument.
    //
    // If the starting point of the last jump is an Eol(1), then we simply
    // apply operator-colon trick.
    //
    // If the starting point of the last jump is a Word, then we need to see if
    // we should use exclusive or inclusive. Denote word by 'w', space by 's',
    // eol by 'l'. We will brute force all possible jumps up to `rangle_token`
    // (excluding `rangle_token`) below:
    //
    //  1: w        -- inclusive up to 'w'
    //  2: w s      -- exclusive up to `rangle_token` if `rangle_token` is a
    //                 Word, else inclusive up to 's'
    //  3: w l      -- inclusive up to 'w'
    //  4: w w      -- IMPOSSIBLE, because 'w' -> 'w' is a second jump
    //  5: w s s    -- IMPOSSIBLE, because two 's' would have been merged into
    //                 one 's' during tokenization
    //  6: w s l    -- inclusive up to 's'
    //  7: w s w    -- IMPOSSIBLE, because 'w' -> 'w' is a second jump
    //  8: w l s    -- inclusive up to 'w'
    //  9: w l l    -- IMPOSSIBLE, because 'w' -> 'l' is a second jump
    // 10: w l w    -- IMPOSSIBLE, because 'w' -> 'w' is a second jump
    // 11: w w _    -- IMPOSSIBLE, because 'w' -> 'w' is a second jump
    // 12: w s s _  -- IMPOSSIBLE, because two 's' would have been merged into
    //                 one 's' during tokenization
    // 13: w s l s  -- inclusive up to first 's'
    // 14: w s l l  -- IMPOSSIBLE, because 'l' -> 'l' is a second jump
    // 15: w s l w  -- IMPOSSIBLE, because 'w' -> 'w' is a second jump
    //  ...
    //
    // Basically, valid last jumps are:
    //
    //     w [alternating occurrences of {s, l}]..
    //
    // From above, we know that:
    //
    // - The selection is inclusive up to 'l' (excluding 'l') if:
    //   * 'w' -> 'l' where 'l' is `rangle_token`
    //   * 'w' -> 's' -> 'l' where 'l' is `rangle_token`
    //   * 'w' -> 'l' -> ...
    //   * 'w' -> 's' -> 'l' -> ...
    // - The selection is exclusive up to 'w' (including 'w') if:
    //   * 'w' -> 'w' where the 2nd 'w' is `rangle_token`
    //   * 'w' -> 's' -> 'w' where the 2nd 'w' is `rangle_token`
    // - The selection is colon (operator-colon trick) if:
    //   * 'l' -> ...

    // Revert one jump to `prev_cursor`.
    let [_, prev_lnum, prev_col, _] = prev_cursor;
    let tokens = buffer.getline_parsed(prev_lnum)?;
    let mut line = ExtendedInlineTokensIter::new(&tokens)
        .skip_col(prev_col)
        .expect("prev_cursor col too large")
        .peekable();
    let mut prev_token = line.next().unwrap();
    let s = match prev_token {
        GToken::Eol(1) => {
            // Apply operator-colon trick.
            assert!(*lnum1 >= prev_lnum + 1);
            *lnum1 = prev_lnum + 1;
            *col1 = 1;
            Selection::Colon
        }
        GToken::Eol(_) => unreachable!(),
        GToken::T(t) => match t.ty {
            TokenType::Space => unreachable!(),
            TokenType::Word => {
                let mut exclusive = false;
                for token in line {
                    match token {
                        GToken::Eol(_) => break,
                        GToken::T(t) => match t.ty {
                            TokenType::Space => prev_token = token,
                            TokenType::Word => {
                                exclusive = true;
                                break;
                            }
                        },
                    }
                }
                if exclusive {
                    assert_eq!(*lnum1, prev_lnum);
                    *col1 = prev_token.last_char1();
                    Selection::Exclusive
                } else {
                    assert!(*lnum1 >= prev_lnum);
                    *lnum1 = prev_lnum;
                    *col1 = prev_token.last_char();
                    Selection::Inclusive
                }
            }
        },
    };
    Ok(s)
}
