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
    /// Emojis are essentially non-word characters. However, Vim treats emojis
    /// differently from other non-word characters such as punctuation. For
    /// example, `🖖🖖🖖🖖,,,abc` contains three tokens (vulcan salutes,
    /// commas, and "abc"), instead of two tokens (vulcan salutes and commas,
    /// and "abc").
    Emoji,
}

/// Word character types.
#[derive(Debug)]
pub enum WordCharType {
    /// 汉字 characters. Note that 汉字 are words only when '@' is included in
    /// the `'iskeyword'` Vim option.
    Hanzi,
    /// Other word characters.
    Other,
}

/// Non-word character types.
#[derive(Debug)]
pub enum NonWordCharType {
    /// 汉字 characters, when '@' is not included in the `'iskeyword'` Vim
    /// option.
    Hanzi,
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
        // Ideographic number zero.
        '\u{3007}'
        // CJK unified ideographs.
        | '\u{4e00}'..='\u{9fff}'
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
                HANZI!() => {
                    if word_predicate.is_unicode_alphabet_word() {
                        CharType::Word(WordCharType::Hanzi)
                    } else {
                        CharType::NonWord(NonWordCharType::Hanzi)
                    }
                }
                RIGHT_PUNC!() => CharType::NonWord(NonWordCharType::RightPunc),
                // Although not `is_alphabetic`, apparently spacing modifier
                // letters are word characters in Vim (both compatible and
                // nocompatible).
                SPACING_MODIFIER_LETTER!() => {
                    CharType::Word(WordCharType::Other)
                }
                c => {
                    if unic_emoji_char::is_emoji(c) {
                        CharType::Emoji
                    } else if c.is_alphabetic()
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
        let wp = WordPredicate::from_isk_opt("").unwrap();
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
            CharType::NonWord(NonWordCharType::Hanzi)
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
        let wp = WordPredicate::from_isk_opt("a-z").unwrap();
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
            CharType::NonWord(NonWordCharType::Hanzi)
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
        let wp = WordPredicate::from_isk_opt("@,^A-Z,48-57").unwrap();
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
        let wp = WordPredicate::from_isk_opt("48-57,>").unwrap();
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
            CharType::NonWord(NonWordCharType::Hanzi)
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
