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

use crate::token::{JiebaPlaceholder, TokenLike, TokenType};
use crate::{BufferLike, CursorPositionCurswant, Position};

use super::nmap_b::UnitNmapB;
use super::omap_e::UnitOmapERangle;
use super::token_iter::{ExtendedInlineTokensIter, GToken, ParsedBuffer};
use super::word_motion::{
    ExtendedMotionState, Intolerable, Markovian, MarkovianUnit, Motion,
    MotionState, UnitMotion,
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
            Markovian::new(UnitOmapWRangleFirstStage);
        let s =
            motion_rangle_first_stage.map(&mut buffer, count, &mut rangle)?;

        // Operator-pending w special case (*).
        let output = match s {
            MotionState::Failure => OmapOutput {
                cursor: langle,
                langle,
                rangle,
                visualmode: b"v",
                selection: b"exclusive",
                prevent_change: b"0",
            },
            MotionState::Success => {
                let selection = operator_w_special_case(
                    &mut buffer,
                    &langle,
                    &mut rangle,
                    count == 1,
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
                            OmapOutput {
                                cursor: langle,
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
                                false,
                            )?
                        {
                            OmapOutput {
                                cursor: langle,
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
    count_eq_1: bool,
) -> Result<Selection, B::Error>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    let rangle_copy = *rangle;
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
    // Futhermore, `langle_token` can't be a Word, as it should have been
    // covered by |cw| special case. Thus, `langle_token` must be either a
    // Space or an Eol(_).

    if count_eq_1 {
        // First, if `count` == 1 and `langle_token` is an Eol(_), then the
        // last "word" moved over is an Eol, and we simply apply operator-colon
        // trick and let Vim to handle the complexity.
        //
        // Second, if `count` == 1 and `langle_token` is a Space, then the last
        // "word" moved over must be `langle_token`, the Space, regardless of
        // the next token after `langle_token` is (i.e. either a Word or and
        // Eol).
        let langle_token = ExtendedInlineTokensIter::new(&tokens)
            .skip_col(*col0)
            .expect("col0 too large")
            .next()
            .unwrap();
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
                    unreachable!("should have been covered by |cw|")
                }
            },
        };
        return Ok(s);
    }

    // Else, since `count` > 1, we focus on the last jump. The starting point
    // of the last jump must be the start of either Eol(1) or Word, since |w|
    // only lands on Eol(1) or Word, and that the starting position of the last
    // jump must be the destination of a previous jump.
    //
    // If the starting point of the last jump is an Eol(1), then we simply
    // apply operator-colon trick.
    //
    // If the starting point of the last jump is a Word, then we need to see if
    // we should use exclusive or inclusive. Denote word by 'w', space by 's',
    // eol by 'l'. We will brute force all possible jumps up to `rangle_token`
    // (exclusive) below:
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

    // Revert one jump using nmap |b| to `prev_token`.
    let mut prev_cursor = rangle_copy;
    let _ = UnitNmapB.unit_map(buffer, &mut prev_cursor)?;
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
