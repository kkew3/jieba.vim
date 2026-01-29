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

//! Figure out which letters are words, based on `'iskeyword'` Vim option.

use super::utils::Set256;

/// Predicate for whether an ASCII or unicode is a word.
#[derive(Debug)]
pub struct WordPredicate {
    /// Set of ASCII characters.
    ascii_set: Set256,
    /// True if '@' is included.
    include_alphabetic: bool,
}

trait Cursor<T> {
    fn next(&self, i: &mut usize) -> &T;
    fn prev(&self, i: &mut usize) -> &T;
}

impl<T> Cursor<T> for [T] {
    fn next(&self, i: &mut usize) -> &T {
        let c = &self[*i];
        *i += 1;
        c
    }

    fn prev(&self, i: &mut usize) -> &T {
        *i -= 1;
        &self[*i]
    }
}

impl WordPredicate {
    fn new() -> Self {
        Self {
            ascii_set: Set256::default(),
            include_alphabetic: false,
        }
    }

    pub fn from_isk_opt(value: &str) -> Option<Self> {
        let mut wp = Self::new();
        let chars = value.as_bytes();
        let mut i = 0;
        let n = chars.len();
        const ZERO: u8 = '0' as u8;
        const NINE: u8 = ZERO + 9;
        // The pre-computed segment values of '@' in `Set256`.
        const ALPHA: [u64; 4] =
            [0, 0x7FFFFFE07FFFFFE, 0x420040000000000, 0xFF7FFFFFFF7FFFFF];

        // Consume next arg, which is either an unsigned integer or an ascii
        // char (u8). Return None if it's an integer that overflows u8, or
        // there is no next arg.
        let expect_arg = |i: &mut usize| -> Option<u8> {
            if *i >= n {
                return None;
            }
            let c = chars.next(i);
            if (ZERO..=NINE).contains(c) {
                let mut num = (*c - ZERO) as u32;
                while *i < n {
                    let c = chars.next(i);
                    if (ZERO..=NINE).contains(c) {
                        num = num * 10 + (*c - ZERO) as u32;
                        if num > 255 {
                            return None;
                        }
                    } else {
                        chars.prev(i);
                        break;
                    }
                }
                Some(num as u8)
            } else {
                Some(*c)
            }
        };

        // Assert the next char is `c`. Consume and return `c` if it's indeed
        // the case. Return None if there's no next char, or it's not the same
        // as `c`.
        let expect_char = |c: u8, i: &mut usize| -> Option<u8> {
            if *i >= n {
                return None;
            }
            if chars.next(i) != &c {
                chars.prev(i);
                return None;
            }
            Some(c)
        };

        while i < n {
            let c1 = chars.next(&mut i);
            if c1 == &b'^' {
                // If a leading '^' is found, ...
                if i >= n {
                    wp.ascii_set.set(b'^');
                } else {
                    // Some arg follows '^'; we will clear it.
                    let lhs = expect_arg(&mut i)?;
                    if i >= n {
                        if lhs == b'@' {
                            // i.e. ^@
                            wp.ascii_set.clear_segments(ALPHA);
                            wp.include_alphabetic = false;
                        } else {
                            // e.g. ^a or ^48
                            wp.ascii_set.clear(lhs);
                        }
                    } else {
                        if expect_char(b',', &mut i).is_some() {
                            // If next char is ',', consume it.
                            if lhs == b'@' {
                                // i.e. ^@
                                wp.ascii_set.clear_segments(ALPHA);
                                wp.include_alphabetic = false;
                            } else {
                                // e.g. ^a or ^48
                                wp.ascii_set.clear(lhs);
                            }
                            if i >= n {
                                // Trailing ',' is forbidden.
                                return None;
                            }
                        } else {
                            // Otherwise, it must be '-'. Else, errors out.
                            expect_char(b'-', &mut i)?;
                            let rhs = expect_arg(&mut i)?;
                            if lhs > rhs {
                                return None;
                            }
                            // e.g. ^a-b or ^48-57 or ^97-z
                            wp.ascii_set.clear_range(lhs, rhs);
                            if i < n {
                                // Consume the next ',' if not at eos.
                                expect_char(b',', &mut i)?;
                                if i >= n {
                                    // Trailing ',' is forbidden.
                                    return None;
                                }
                            }
                        }
                    }
                }
            } else {
                chars.prev(&mut i);
                let lhs = expect_arg(&mut i)?;
                if i >= n {
                    if lhs == b'@' {
                        // i.e. @
                        wp.ascii_set.set_segments(ALPHA);
                        wp.include_alphabetic = true;
                    } else {
                        // e.g. a or 48
                        wp.ascii_set.set(lhs);
                    }
                } else {
                    if expect_char(b',', &mut i).is_some() {
                        // If next char is ',', consume it.
                        if lhs == b'@' {
                            // i.e. @
                            wp.ascii_set.set_segments(ALPHA);
                            wp.include_alphabetic = true;
                        } else {
                            // e.g. a or 48
                            wp.ascii_set.set(lhs);
                        }
                        if i >= n {
                            // Trailing ',' is forbidden.
                            return None;
                        }
                    } else {
                        // Otherwise, it must be '-'. Else, errors out.
                        expect_char(b'-', &mut i)?;
                        let rhs = expect_arg(&mut i)?;
                        if lhs > rhs {
                            return None;
                        }
                        // e.g. a-b or 48-57 or 97-z
                        wp.ascii_set.set_range(lhs, rhs);
                        if i < n {
                            // Consume the next ',' if not at eos.
                            expect_char(b',', &mut i)?;
                            if i >= n {
                                // Trailing ',' is forbidden.
                                return None;
                            }
                        }
                    }
                }
            }
        }

        Some(wp)
    }

    /// Check if `ascii` is a word.
    pub fn is_ascii_word(&self, ascii: u8) -> bool {
        self.ascii_set.contains(ascii)
    }

    /// Check if a unicode alphabet like 汉字 is a word.
    pub fn is_unicode_alphabet_word(&self) -> bool {
        self.include_alphabetic
    }
}

impl TryFrom<&str> for WordPredicate {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        WordPredicate::from_isk_opt(value).ok_or(())
    }
}

impl TryFrom<String> for WordPredicate {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        WordPredicate::from_isk_opt(&value).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use crate::token::utils::Set256;

    use super::WordPredicate;

    fn get_ascii_set(isk: &str) -> Result<Set256, ()> {
        WordPredicate::from_isk_opt(isk)
            .map(|wp| wp.ascii_set)
            .ok_or(())
    }

    macro_rules! assert_contains {
        ($set:expr, $($i:literal..=$j:literal),*) => {
            $(
                assert!($set.all_range($i, $j));
            )*
        };
    }

    macro_rules! assert_not_contains {
        ($set:expr, $($i:literal..=$j:literal),*) => {
            $(
                assert!(!$set.any_range($i, $j));
            )*
        };
    }

    #[test]
    fn test_parse_isk() -> Result<(), ()> {
        let s = get_ascii_set("")?;
        assert_not_contains!(s, 0..=255);

        let s = get_ascii_set("48")?;
        assert_contains!(s, 48..=48);
        assert_not_contains!(s, 0..=47, 49..=255);

        let s = get_ascii_set("-")?;
        assert_contains!(s, b'-'..=b'-');
        assert_not_contains!(s, 0..=0x2C, 0x2E..=255);

        let s = get_ascii_set("#-43")?;
        assert_contains!(s, b'#'..=43);
        assert_not_contains!(s, 0..=0x22, 44..=255);

        let s = get_ascii_set("128-140")?;
        assert_contains!(s, 128..=140);
        assert_not_contains!(s, 0..=127, 141..=255);

        let s = get_ascii_set("--57")?;
        assert_contains!(s, b'-'..=57);
        assert_not_contains!(s, 0..=44, 58..=255);

        let s = get_ascii_set("---")?;
        assert_contains!(s, b'-'..=b'-');
        assert_not_contains!(s, 0..=44, 46..=255);

        let s = get_ascii_set("^a-z")?;
        assert_not_contains!(s, 0..=255);

        let s = get_ascii_set("93-95,^^")?;
        assert_contains!(s, 93..=93, 95..=95);
        assert_not_contains!(s, 0..=92, 94..=94, 96..=255);

        let s = get_ascii_set("^93-^")?;
        assert_not_contains!(s, 0..=255);

        let s = get_ascii_set("^48-^,,,^")?;
        assert_contains!(s, b','..=b',', b'^'..=b'^');
        assert_not_contains!(s, 0..=43, 45..=93, 95..=255);

        let s = get_ascii_set("48-57,93-^")?;
        assert_contains!(s, 48..=57, 93..=b'^');
        assert_not_contains!(s, 0..=47, 58..=92, 95..=255);

        let s = get_ascii_set("^a-z,#,^")?;
        assert_contains!(s, b'#'..=b'#', b'^'..=b'^');
        assert_not_contains!(s, 0..=34, 36..=93, 95..=255);

        let s = get_ascii_set("^a-z,#,^^")?;
        assert_contains!(s, b'#'..=b'#');
        assert_not_contains!(s, 0..=34, 36..=255);

        assert!(get_ascii_set("^-^").is_err());
        assert!(get_ascii_set(r"\\").is_err());
        assert!(get_ascii_set(r"a-z,\\,.").is_err());

        let s = get_ascii_set("@")?;
        assert_contains!(
            s,
            65..=90,
            97..=122,
            170..=170,
            181..=181,
            186..=186,
            192..=214,
            216..=246,
            248..=255
        );
        assert_not_contains!(
            s,
            0..=64,
            91..=96,
            123..=169,
            171..=180,
            182..=185,
            187..=191,
            215..=215,
            247..=247
        );

        let s = get_ascii_set("@-@")?;
        assert_contains!(s, b'@'..=b'@');
        assert_not_contains!(s, 0..=63, 65..=255);

        let s = get_ascii_set("@-65")?;
        assert_contains!(s, b'@'..=65);
        assert_not_contains!(s, 0..=63, 66..=255);

        let s = get_ascii_set("@,^a-z,^A-Z")?;
        assert_contains!(
            s,
            170..=170,
            181..=181,
            186..=186,
            192..=214,
            216..=246,
            248..=255
        );
        assert_not_contains!(
            s,
            0..=169,
            171..=180,
            182..=185,
            187..=191,
            215..=215,
            247..=247
        );

        let s = get_ascii_set("a-z,A-Z,@-@")?;
        assert_contains!(s, b'a'..=b'z', b'A'..=b'Z', b'@'..=b'@');
        assert_not_contains!(
            s,
            b'0'..=b'9',
            171..=180,
            182..=185,
            187..=191,
            215..=215,
            247..=247
        );

        let s = get_ascii_set(",")?;
        assert_contains!(s, b','..=b',');
        assert_not_contains!(s, 0..=43, 45..=255);

        let s = get_ascii_set(",,,")?;
        assert_contains!(s, b','..=b',');
        assert_not_contains!(s, 0..=43, 45..=255);

        assert!(get_ascii_set(",,").is_err());
        assert!(get_ascii_set(",,,,").is_err());

        let s = get_ascii_set(r"48-57,,,_,\")?;
        assert_contains!(
            s,
            b'0'..=b'9',
            b','..=b',',
            b'_'..=b'_',
            b'\\'..=b'\\'
        );
        assert_not_contains!(s, 0..=43, 45..=47, 58..=91, 93..=94, 96..=255);

        let s = get_ascii_set("32-~,^,,9")?;
        assert_contains!(s, 32..=43, 45..=b'~', 9..=9);
        assert_not_contains!(s, 0..=8, 10..=31, 44..=44, 127..=255);

        let s = get_ascii_set(",,^,,^^,^")?;
        assert_contains!(s, b'^'..=b'^');
        assert_not_contains!(s, 0..=93, 95..=255);

        let s = get_ascii_set("^^,,,^,,^")?;
        assert_contains!(s, b'^'..=b'^');
        assert_not_contains!(s, 0..=93, 95..=255);

        let s = get_ascii_set("^^-^")?;
        assert_not_contains!(s, 0..=255);

        let s = get_ascii_set("0-255,^^-^")?;
        assert_contains!(s, 0..=93, 95..=255);
        assert_not_contains!(s, b'^'..=b'^');

        let s = get_ascii_set("0-255,^^")?;
        assert_contains!(s, 0..=93, 95..=255);
        assert_not_contains!(s, b'^'..=b'^');

        let s = get_ascii_set("^")?;
        assert_contains!(s, b'^'..=b'^');
        assert_not_contains!(s, 0..=93, 95..=255);

        let s = get_ascii_set("^^")?;
        assert_not_contains!(s, 0..=255);

        // 'iskeyword' value for vim help.
        let s = get_ascii_set(r#"!-~,^*,^|,^",192-255"#)?;
        assert_contains!(s, 33..=33, 35..=41, 43..=123, 125..=126, 192..=255);
        assert_not_contains!(s, 0..=32, 34..=34, 42..=42, 124..=124, 127..=191);

        let s = get_ascii_set("a-z,A-Z,48-57,_,.,-,>")?;
        assert_contains!(
            s,
            b'a'..=b'z',
            b'A'..=b'Z',
            48..=57,
            b'_'..=b'_',
            b'.'..=b'.',
            b'-'..=b'-',
            b'>'..=b'>'
        );

        let s = get_ascii_set("@,^A,48-57")?;
        assert!(!s.contains(b'A'));

        Ok(())
    }
}
