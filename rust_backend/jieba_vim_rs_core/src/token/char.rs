// Copyright 2025-2026 Kaiwen Wu. All Rights Reserved.
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

//! This module defines the taxonomy of characters.

use super::isk::WordPredicate;

/// Character types.
#[derive(Debug)]
pub enum CharType {
    /// Whitespace characters.
    Space,
    /// Word characters.
    Word(WordCharType),
    /// Non-word characters.
    NonWord(NonWordCharType),
    /// Unicode combining characters. See
    /// https://en.wikipedia.org/wiki/Combining_character. Note that this
    /// intentionally does not include combining diacritical marks extended,
    /// which might be included in the future in case of frequent need in
    /// practice.
    CombiningDiacriticalMark,
    // We should have listed emoji as a subtype of [`Word`](Self::Word).
    // Nevertheless, this will cause trouble, because in current tokenization
    // procedure, all char groups can be bipartited in terms of whether it
    // contains 汉字. Since we also want emojis to stand on their own without
    // being merged into [`WordCharType::Other`], we have to list emoji as a
    // separate category. This is indeed awkward. But as current policy isn't
    // doing any harm either, we will postpone fixing this to future work.
    /// Emojis appears to be non-word characters at first glance. Current Vim,
    /// however, treats emojis as words. For example, `🖖🖖🖖🖖,,,abc` contains
    /// three tokens (vulcan salutes, commas, and "abc"), instead of two tokens
    /// (vulcan salutes and commas, and "abc").
    Emoji,
}

/// Word character types.
#[derive(Debug)]
pub enum WordCharType {
    /// 汉字 characters.
    Hanzi,
    /// Other word characters.
    Other,
}

/// Non-word character types.
#[derive(Debug)]
pub enum NonWordCharType {
    /// Right-associated CJK punctuations. When a word character follows a
    /// [`NonWordCharType::RightPunc`], an implicit space is added in between.
    RightPunc,
    /// Other non-word characters. This includes the zero-width joiner (ZWJ).
    Other,
}

/// Combining diacritical mark characters. See
/// [`CharType::CombiningDiacriticalMark`].
macro_rules! COMBINING_DIACRITICAL_MARK {
    () => {
        '\u{0300}'..='\u{036f}'
    };
}

// This will be used in `tokenize` module.
pub(super) fn is_combining_diacritical_mark(c: char) -> bool {
    matches!(c, COMBINING_DIACRITICAL_MARK!())
}

/// Spacing modifier letters
/// (https://en.wikipedia.org/wiki/Spacing_Modifier_Letters).
macro_rules! SPACING_MODIFIER_LETTER {
    () => {
        '\u{02b0}'..='\u{02ff}'
    };
}

/// Whitespace characters. See [`CharType::Space`].
macro_rules! SPACE {
    () => {
        // Vim ASCII whitespace.
        ' ' | '\t'
        // CJK ideographic space, suggested by GPT. See also
        // https://www.compart.com/en/unicode/U+3000.
        | '\u{3000}'
        // CJK ideographic half fill space. See also
        // https://www.compart.com/en/unicode/block/U+3000.
        | '\u{303f}'
    };
}

// === BEGIN OF QUOTES FROM Thomas Roten ===

// The unicodes of CJK characters and punctuations are quoted from Github
// repository: https://github.com/tsroten/zhon.
// File: https://github.com/tsroten/zhon/blob/main/src/zhon/hanzi.py.
// License: MIT (https://github.com/tsroten/zhon/blob/main/LICENSE.txt),
// attached below:
//
// ---
// Copyright (c) 2013-2014 Thomas Roten
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

/// 汉字 characters. See [`WordCharType::Hanzi`].
macro_rules! HANZI {
    () => {
        // CJK unified ideographs. Placed at the top because rust tends to
        // check patterns top-down, and this range covers almost all 汉字
        // characters.
        '\u{4e00}'..='\u{9fff}'
        // Ideographic number zero.
        | '\u{3007}'
        // CJK unified ideographs extension A.
        | '\u{3400}'..='\u{4dbf}'
        // CJK compatibility ideographs.
        | '\u{f900}'..='\u{faff}'
        // CJK unified ideographs extension B.
        | '\u{20000}'..='\u{2a6df}'
        // CJK unified ideographs extension C.
        | '\u{2a700}'..='\u{2b73f}'
        // CJK unified ideographs extension D.
        | '\u{2b740}'..='\u{2b81f}'
        // CJK compatibility ideographs supplement.
        | '\u{2f800}'..='\u{2fa1f}'
        // Character code ranges for the Kangxi radicals and CJK radicals
        // supplement.
        | '\u{2f00}'..='\u{2fd5}'
        | '\u{2e80}'..='\u{2ef3}'
    };
}

/// Right-associated 汉字 punctuations. See [`NonWordCharType::RightPunc`].
macro_rules! RIGHT_PUNC {
    () => {
        // Fullwidth ASCII variants.
        '\u{ff0c}' | '\u{ff1a}' | '\u{ff1b}'
        // Halfwidth CJK punctuation.
        | '\u{ff64}'
        // CJK symbols and punctuation.
        | '\u{3001}'
        // Small form variants.
        | '\u{fe51}' | '\u{fe54}'
        // Fullwidth full stop.
        | '\u{ff0e}'
        // Fullwidth exclamation mark.
        | '\u{ff01}'
        // Fullwidth question mark.
        | '\u{ff1f}'
        // Halfwidth ideographic full stop.
        | '\u{ff61}'
        // Ideographic full stop.
        | '\u{3002}'
    };
}

// === END OF QUOTES FROM Thomas Roten ===

// === BEGIN OF QUOTES FROM Vim C source code by Bram Moolenaar ===
// The unicodes of emojis are quoted from https://github.com/vim/vim.git
// (src/mbyte.c), which was organized from
// http://unicode.org/emoji/charts/emoji-list.html.
//
// See Vim license under project root.

macro_rules! EMOJI {
    () => {
        '\u{203c}'..='\u{203c}'
        | '\u{2049}'..='\u{2049}'
        | '\u{2122}'..='\u{2122}'
        | '\u{2139}'..='\u{2139}'
        | '\u{2194}'..='\u{2199}'
        | '\u{21a9}'..='\u{21aa}'
        | '\u{231a}'..='\u{231b}'
        | '\u{2328}'..='\u{2328}'
        | '\u{23cf}'..='\u{23cf}'
        | '\u{23e9}'..='\u{23f3}'
        | '\u{23f8}'..='\u{23fa}'
        | '\u{24c2}'..='\u{24c2}'
        | '\u{25aa}'..='\u{25ab}'
        | '\u{25b6}'..='\u{25b6}'
        | '\u{25c0}'..='\u{25c0}'
        | '\u{25fb}'..='\u{25fe}'
        | '\u{2600}'..='\u{2604}'
        | '\u{260e}'..='\u{260e}'
        | '\u{2611}'..='\u{2611}'
        | '\u{2614}'..='\u{2615}'
        | '\u{2618}'..='\u{2618}'
        | '\u{261d}'..='\u{261d}'
        | '\u{2620}'..='\u{2620}'
        | '\u{2622}'..='\u{2623}'
        | '\u{2626}'..='\u{2626}'
        | '\u{262a}'..='\u{262a}'
        | '\u{262e}'..='\u{262f}'
        | '\u{2638}'..='\u{263a}'
        | '\u{2640}'..='\u{2640}'
        | '\u{2642}'..='\u{2642}'
        | '\u{2648}'..='\u{2653}'
        | '\u{265f}'..='\u{2660}'
        | '\u{2663}'..='\u{2663}'
        | '\u{2665}'..='\u{2666}'
        | '\u{2668}'..='\u{2668}'
        | '\u{267b}'..='\u{267b}'
        | '\u{267e}'..='\u{267f}'
        | '\u{2692}'..='\u{2697}'
        | '\u{2699}'..='\u{2699}'
        | '\u{269b}'..='\u{269c}'
        | '\u{26a0}'..='\u{26a1}'
        | '\u{26a7}'..='\u{26a7}'
        | '\u{26aa}'..='\u{26ab}'
        | '\u{26b0}'..='\u{26b1}'
        | '\u{26bd}'..='\u{26be}'
        | '\u{26c4}'..='\u{26c5}'
        | '\u{26c8}'..='\u{26c8}'
        | '\u{26ce}'..='\u{26cf}'
        | '\u{26d1}'..='\u{26d1}'
        | '\u{26d3}'..='\u{26d4}'
        | '\u{26e9}'..='\u{26ea}'
        | '\u{26f0}'..='\u{26f5}'
        | '\u{26f7}'..='\u{26fa}'
        | '\u{26fd}'..='\u{26fd}'
        | '\u{2702}'..='\u{2702}'
        | '\u{2705}'..='\u{2705}'
        | '\u{2708}'..='\u{270d}'
        | '\u{270f}'..='\u{270f}'
        | '\u{2712}'..='\u{2712}'
        | '\u{2714}'..='\u{2714}'
        | '\u{2716}'..='\u{2716}'
        | '\u{271d}'..='\u{271d}'
        | '\u{2721}'..='\u{2721}'
        | '\u{2728}'..='\u{2728}'
        | '\u{2733}'..='\u{2734}'
        | '\u{2744}'..='\u{2744}'
        | '\u{2747}'..='\u{2747}'
        | '\u{274c}'..='\u{274c}'
        | '\u{274e}'..='\u{274e}'
        | '\u{2753}'..='\u{2755}'
        | '\u{2757}'..='\u{2757}'
        | '\u{2763}'..='\u{2764}'
        | '\u{2795}'..='\u{2797}'
        | '\u{27a1}'..='\u{27a1}'
        | '\u{27b0}'..='\u{27b0}'
        | '\u{27bf}'..='\u{27bf}'
        | '\u{2934}'..='\u{2935}'
        | '\u{2b05}'..='\u{2b07}'
        | '\u{2b1b}'..='\u{2b1c}'
        | '\u{2b50}'..='\u{2b50}'
        | '\u{2b55}'..='\u{2b55}'
        | '\u{3030}'..='\u{3030}'
        | '\u{303d}'..='\u{303d}'
        | '\u{3297}'..='\u{3297}'
        | '\u{3299}'..='\u{3299}'
        | '\u{1f004}'..='\u{1f004}'
        | '\u{1f0cf}'..='\u{1f0cf}'
        | '\u{1f170}'..='\u{1f171}'
        | '\u{1f17e}'..='\u{1f17f}'
        | '\u{1f18e}'..='\u{1f18e}'
        | '\u{1f191}'..='\u{1f19a}'
        | '\u{1f1e6}'..='\u{1f1ff}'
        | '\u{1f201}'..='\u{1f202}'
        | '\u{1f21a}'..='\u{1f21a}'
        | '\u{1f22f}'..='\u{1f22f}'
        | '\u{1f232}'..='\u{1f23a}'
        | '\u{1f250}'..='\u{1f251}'
        | '\u{1f300}'..='\u{1f321}'
        | '\u{1f324}'..='\u{1f393}'
        | '\u{1f396}'..='\u{1f397}'
        | '\u{1f399}'..='\u{1f39b}'
        | '\u{1f39e}'..='\u{1f3f0}'
        | '\u{1f3f3}'..='\u{1f3f5}'
        | '\u{1f3f7}'..='\u{1f4fd}'
        | '\u{1f4ff}'..='\u{1f53d}'
        | '\u{1f549}'..='\u{1f54e}'
        | '\u{1f550}'..='\u{1f567}'
        | '\u{1f56f}'..='\u{1f570}'
        | '\u{1f573}'..='\u{1f57a}'
        | '\u{1f587}'..='\u{1f587}'
        | '\u{1f58a}'..='\u{1f58d}'
        | '\u{1f590}'..='\u{1f590}'
        | '\u{1f595}'..='\u{1f596}'
        | '\u{1f5a4}'..='\u{1f5a5}'
        | '\u{1f5a8}'..='\u{1f5a8}'
        | '\u{1f5b1}'..='\u{1f5b2}'
        | '\u{1f5bc}'..='\u{1f5bc}'
        | '\u{1f5c2}'..='\u{1f5c4}'
        | '\u{1f5d1}'..='\u{1f5d3}'
        | '\u{1f5dc}'..='\u{1f5de}'
        | '\u{1f5e1}'..='\u{1f5e1}'
        | '\u{1f5e3}'..='\u{1f5e3}'
        | '\u{1f5e8}'..='\u{1f5e8}'
        | '\u{1f5ef}'..='\u{1f5ef}'
        | '\u{1f5f3}'..='\u{1f5f3}'
        | '\u{1f5fa}'..='\u{1f64f}'
        | '\u{1f680}'..='\u{1f6c5}'
        | '\u{1f6cb}'..='\u{1f6d2}'
        | '\u{1f6d5}'..='\u{1f6d7}'
        | '\u{1f6dc}'..='\u{1f6e5}'
        | '\u{1f6e9}'..='\u{1f6e9}'
        | '\u{1f6eb}'..='\u{1f6ec}'
        | '\u{1f6f0}'..='\u{1f6f0}'
        | '\u{1f6f3}'..='\u{1f6fc}'
        | '\u{1f7e0}'..='\u{1f7eb}'
        | '\u{1f7f0}'..='\u{1f7f0}'
        | '\u{1f90c}'..='\u{1f93a}'
        | '\u{1f93c}'..='\u{1f945}'
        | '\u{1f947}'..='\u{1f9ff}'
        | '\u{1fa70}'..='\u{1fa7c}'
        | '\u{1fa80}'..='\u{1fa88}'
        | '\u{1fa90}'..='\u{1fabd}'
        | '\u{1fabf}'..='\u{1fac5}'
        | '\u{1face}'..='\u{1fadb}'
        | '\u{1fae0}'..='\u{1fae8}'
        | '\u{1faf0}'..='\u{1faf8}'
    };
}

// == END OF QUOTES FROM Vim C source code by Bram Moolenaar

/// Categorize a char into [`CharType`], according to [`WordPredicate`].
pub fn categorize_char(c: char, word_predicate: &WordPredicate) -> CharType {
    match c {
        SPACE!() => CharType::Space,
        COMBINING_DIACRITICAL_MARK!() => CharType::CombiningDiacriticalMark,
        c => match super::ascii_or(c) {
            Some(ascii) => {
                if word_predicate.is_ascii_word(ascii) {
                    CharType::Word(WordCharType::Other)
                } else {
                    CharType::NonWord(NonWordCharType::Other)
                }
            }
            None => match c {
                // 汉字 are always words regardless if `'iskeyword'`. See
                // charset.c and mbyte.c of Vim C source codebase.
                HANZI!() => CharType::Word(WordCharType::Hanzi),
                RIGHT_PUNC!() => CharType::NonWord(NonWordCharType::RightPunc),
                // Although not `is_alphabetic`, apparently spacing modifier
                // letters are word characters in Vim (both compatible and
                // nocompatible).
                SPACING_MODIFIER_LETTER!() => {
                    CharType::Word(WordCharType::Other)
                }
                EMOJI!() => CharType::Emoji,
                c => {
                    if c.is_alphabetic()
                        && word_predicate.is_unicode_alphabet_word()
                    {
                        CharType::Word(WordCharType::Other)
                    } else {
                        CharType::NonWord(NonWordCharType::Other)
                    }
                }
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::token::isk::WordPredicate;

    use super::{CharType, NonWordCharType, WordCharType, categorize_char};

    #[test]
    fn test_categorize_char() {
        // Empty iskeyword.
        let wp = WordPredicate::from_isk_opt(b"").unwrap();
        assert!(matches!(
            categorize_char('a', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('A', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('\u{c0}', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('3', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('_', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(categorize_char(' ', &wp), CharType::Space));
        assert!(matches!(categorize_char('\u{3000}', &wp), CharType::Space));
        assert!(matches!(
            categorize_char('我', &wp),
            CharType::Word(WordCharType::Hanzi)
        ));
        assert!(matches!(
            categorize_char('，', &wp),
            CharType::NonWord(NonWordCharType::RightPunc)
        ));
        assert!(matches!(
            categorize_char('＃', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('>', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(categorize_char('😀', &wp), CharType::Emoji));
        assert!(matches!(categorize_char('\u{1f3ff}', &wp), CharType::Emoji));
        assert!(matches!(
            categorize_char('\u{0302}', &wp), // combining diacritical mark
            CharType::CombiningDiacriticalMark
        ));
        assert!(matches!(
            categorize_char('\u{02b1}', &wp), // spacing modifier letter
            CharType::Word(WordCharType::Other)
        ));

        // Lowercase ASCII iskeyword.
        let wp = WordPredicate::from_isk_opt(b"a-z").unwrap();
        assert!(matches!(
            categorize_char('a', &wp),
            CharType::Word(WordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('A', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('\u{c0}', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('3', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('_', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(categorize_char(' ', &wp), CharType::Space));
        assert!(matches!(categorize_char('\u{3000}', &wp), CharType::Space));
        assert!(matches!(
            categorize_char('我', &wp),
            CharType::Word(WordCharType::Hanzi)
        ));
        assert!(matches!(
            categorize_char('，', &wp),
            CharType::NonWord(NonWordCharType::RightPunc)
        ));
        assert!(matches!(
            categorize_char('＃', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('>', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(categorize_char('😀', &wp), CharType::Emoji));
        assert!(matches!(categorize_char('\u{1f3ff}', &wp), CharType::Emoji));
        assert!(matches!(
            categorize_char('\u{0302}', &wp), // combining diacritical mark
            CharType::CombiningDiacriticalMark
        ));
        assert!(matches!(
            categorize_char('\u{02b1}', &wp), // spacing modifier letter
            CharType::Word(WordCharType::Other)
        ));

        // Lowercase ASCII, digits and 汉字 iskeyword.
        let wp = WordPredicate::from_isk_opt(b"@,^A-Z,48-57").unwrap();
        assert!(matches!(
            categorize_char('a', &wp),
            CharType::Word(WordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('A', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('\u{c0}', &wp),
            CharType::Word(WordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('3', &wp),
            CharType::Word(WordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('_', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(categorize_char(' ', &wp), CharType::Space));
        assert!(matches!(categorize_char('\u{3000}', &wp), CharType::Space));
        assert!(matches!(
            categorize_char('我', &wp),
            CharType::Word(WordCharType::Hanzi)
        ));
        assert!(matches!(
            categorize_char('，', &wp),
            CharType::NonWord(NonWordCharType::RightPunc)
        ));
        assert!(matches!(
            categorize_char('＃', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('>', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(categorize_char('😀', &wp), CharType::Emoji));
        assert!(matches!(categorize_char('\u{1f3ff}', &wp), CharType::Emoji));
        assert!(matches!(
            categorize_char('\u{0302}', &wp), // combining diacritical mark
            CharType::CombiningDiacriticalMark
        ));
        assert!(matches!(
            categorize_char('\u{02b1}', &wp), // spacing modifier letter
            CharType::Word(WordCharType::Other)
        ));

        // Digits and '>' iskeyword.
        let wp = WordPredicate::from_isk_opt(b"48-57,>").unwrap();
        assert!(matches!(
            categorize_char('a', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('A', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('\u{c0}', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('3', &wp),
            CharType::Word(WordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('_', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(categorize_char(' ', &wp), CharType::Space));
        assert!(matches!(categorize_char('\u{3000}', &wp), CharType::Space));
        assert!(matches!(
            categorize_char('我', &wp),
            CharType::Word(WordCharType::Hanzi)
        ));
        assert!(matches!(
            categorize_char('，', &wp),
            CharType::NonWord(NonWordCharType::RightPunc)
        ));
        assert!(matches!(
            categorize_char('＃', &wp),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('>', &wp),
            CharType::Word(WordCharType::Other)
        ));
        assert!(matches!(categorize_char('😀', &wp), CharType::Emoji));
        assert!(matches!(categorize_char('\u{1f3ff}', &wp), CharType::Emoji));
        assert!(matches!(
            categorize_char('\u{0302}', &wp), // combining diacritical mark
            CharType::CombiningDiacriticalMark
        ));
        assert!(matches!(
            categorize_char('\u{02b1}', &wp), // spacing modifier letter
            CharType::Word(WordCharType::Other)
        ));
    }
}
