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

use super::tokenize::{Token, TokenLike};

/// Get the index of the token in `tokens` that covers `col`. Return `None` if
/// `col` is to the right of the last token.
pub fn index_tokens(tokens: &[Token], col: usize) -> Option<usize> {
    use std::cmp::Ordering;
    tokens
        .binary_search_by(|tok| {
            if col < tok.first_char() {
                Ordering::Greater
            } else if col >= tok.last_char1() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
}

/// Try to convert `c` to an ASCII. If failed, give back `c`.
pub fn ascii_or(c: char) -> Option<u8> {
    if c as u32 <= u8::MAX as u32 {
        Some(c as u8)
    } else {
        None
    }
}

mod set256 {
    use std::fmt;

    const fn max(x: u8, y: u8) -> u8 {
        if x > y { x } else { y }
    }

    const fn min(x: u8, y: u8) -> u8 {
        if x < y { x } else { y }
    }

    const fn get_seg_range_op([i, j]: [u8; 2]) -> u64 {
        if i > j {
            return 0;
        }
        assert!(j < 64);
        let n = j - i + 1;
        (u64::MAX >> (64 - n)) << i
    }

    const fn split_range(i: u8, j: u8) -> [[u8; 2]; 4] {
        let l = [0, 64, 128, 192];
        let r = [63, 127, 191, 255];
        let il = [max(i, l[0]), max(i, l[1]), max(i, l[2]), max(i, l[3])];
        let jr = [min(j, r[0]), min(j, r[1]), min(j, r[2]), min(j, r[3])];

        const fn which(il: u8, jr: u8, l: u8) -> [u8; 2] {
            if il <= jr { [il - l, jr - l] } else { [1, 0] }
        }

        [
            which(il[0], jr[0], l[0]),
            which(il[1], jr[1], l[1]),
            which(il[2], jr[2], l[2]),
            which(il[3], jr[3], l[3]),
        ]
    }

    const fn get_range_op(i: u8, j: u8) -> [u64; 4] {
        let a = split_range(i, j);
        [
            get_seg_range_op(a[0]),
            get_seg_range_op(a[1]),
            get_seg_range_op(a[2]),
            get_seg_range_op(a[3]),
        ]
    }

    /// A bitset with 256 slots.
    #[derive(Default)]
    pub struct Set256 {
        segments: [u64; 4],
    }

    impl fmt::Debug for Set256 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{:0>64b}{:0>64b}{:0>64b}{:0>64b}",
                self.segments[3],
                self.segments[2],
                self.segments[1],
                self.segments[0]
            )
        }
    }

    impl Set256 {
        #[inline]
        const fn set_segment(&mut self, s: usize, op: u64) {
            self.segments[s] |= op;
        }

        #[inline]
        const fn clear_segment(&mut self, s: usize, op: u64) {
            self.segments[s] &= !op;
        }

        #[cfg(test)]
        #[inline]
        const fn all_segment(&self, s: usize, op: u64) -> bool {
            (self.segments[s] & op) == op
        }

        #[inline]
        const fn any_segment(&self, s: usize, op: u64) -> bool {
            (self.segments[s] & op) > 0
        }

        pub const fn set_segments(&mut self, op: [u64; 4]) {
            self.set_segment(0, op[0]);
            self.set_segment(1, op[1]);
            self.set_segment(2, op[2]);
            self.set_segment(3, op[3]);
        }

        pub const fn clear_segments(&mut self, op: [u64; 4]) {
            self.clear_segment(0, op[0]);
            self.clear_segment(1, op[1]);
            self.clear_segment(2, op[2]);
            self.clear_segment(3, op[3]);
        }

        #[cfg(test)]
        pub const fn all_segments(&self, op: [u64; 4]) -> bool {
            self.all_segment(0, op[0])
                && self.all_segment(1, op[1])
                && self.all_segment(2, op[2])
                && self.all_segment(3, op[3])
        }

        pub const fn any_segments(&self, op: [u64; 4]) -> bool {
            self.any_segment(0, op[0])
                || self.any_segment(1, op[1])
                || self.any_segment(2, op[2])
                || self.any_segment(3, op[3])
        }

        /// Set range inclusive. Panics if i > j.
        pub const fn set_range(&mut self, i: u8, j: u8) {
            assert!(i <= j);
            self.set_segments(get_range_op(i, j));
        }

        pub const fn clear_range(&mut self, i: u8, j: u8) {
            assert!(i <= j);
            self.clear_segments(get_range_op(i, j));
        }

        #[cfg(test)]
        pub const fn all_range(&self, i: u8, j: u8) -> bool {
            assert!(i <= j);
            self.all_segments(get_range_op(i, j))
        }

        pub const fn any_range(&self, i: u8, j: u8) -> bool {
            assert!(i <= j);
            self.any_segments(get_range_op(i, j))
        }

        pub const fn set(&mut self, i: u8) {
            self.set_range(i, i);
        }

        pub const fn clear(&mut self, i: u8) {
            self.clear_range(i, i);
        }

        pub const fn contains(&self, i: u8) -> bool {
            self.any_range(i, i)
        }
    }
}

pub use set256::Set256;

#[cfg(test)]
mod tests {
    use super::{Set256, index_tokens};

    #[test]
    fn test_index_tokens() {
        assert_eq!(index_tokens(&[], 0), None);
    }

    #[test]
    fn test_set256() {
        for i in 0..=255 {
            let mut s = Set256::default();
            s.set(i);
            assert!(s.all_range(i, i));
            assert!(s.any_range(i, i));
            if i == 0 {
                assert!(!s.any_range(1, 255));
            } else if i == 255 {
                assert!(!s.any_range(0, 254));
            } else {
                assert!(!s.any_range(0, i - 1));
                assert!(!s.any_range(i + 1, 255));
            }
        }

        let mut s = Set256::default();

        s.set_range(20, 64);
        assert!((0..=19).all(|i| !s.contains(i)));
        assert!((20..=64).all(|i| s.contains(i)));
        assert!((65..=255).all(|i| !s.contains(i)));
        assert!(s.all_range(20, 64));
        assert!(!s.any_range(0, 19));
        assert!(!s.any_range(65, 255));

        s.set_range(78, 127);
        assert!((0..=19).all(|i| !s.contains(i)));
        assert!((20..=64).all(|i| s.contains(i)));
        assert!((65..=77).all(|i| !s.contains(i)));
        assert!((78..=127).all(|i| s.contains(i)));
        assert!((128..=255).all(|i| !s.contains(i)));
        assert!(!s.any_range(0, 19));
        assert!(s.all_range(20, 64));
        assert!(!s.any_range(65, 77));
        assert!(s.all_range(78, 127));
        assert!(!s.any_range(128, 255));

        s.set_range(190, 191);
        assert!((0..=19).all(|i| !s.contains(i)));
        assert!((20..=64).all(|i| s.contains(i)));
        assert!((65..=77).all(|i| !s.contains(i)));
        assert!((78..=127).all(|i| s.contains(i)));
        assert!((128..=189).all(|i| !s.contains(i)));
        assert!((190..=191).all(|i| s.contains(i)));
        assert!((192..=255).all(|i| !s.contains(i)));
        assert!(!s.any_range(0, 19));
        assert!(s.all_range(20, 64));
        assert!(!s.any_range(65, 77));
        assert!(s.all_range(78, 127));
        assert!(!s.any_range(128, 189));
        assert!(s.all_range(190, 191));
        assert!(!s.any_range(192, 255));

        s.set_range(201, 201);
        assert!((0..=19).all(|i| !s.contains(i)));
        assert!((20..=64).all(|i| s.contains(i)));
        assert!((65..=77).all(|i| !s.contains(i)));
        assert!((78..=127).all(|i| s.contains(i)));
        assert!((128..=189).all(|i| !s.contains(i)));
        assert!((190..=191).all(|i| s.contains(i)));
        assert!((192..=200).all(|i| !s.contains(i)));
        assert!(s.contains(201));
        assert!((202..=255).all(|i| !s.contains(i)));
        assert!(!s.any_range(0, 19));
        assert!(s.all_range(20, 64));
        assert!(!s.any_range(65, 77));
        assert!(s.all_range(78, 127));
        assert!(!s.any_range(128, 189));
        assert!(s.all_range(190, 191));
        assert!(!s.any_range(192, 200));
        assert!(s.all_range(201, 201));
        assert!(!s.any_range(202, 255));

        s.clear_range(30, 63);
        assert!((0..=19).all(|i| !s.contains(i)));
        assert!((20..30).all(|i| s.contains(i)));
        assert!((30..=63).all(|i| !s.contains(i)));
        assert!(s.contains(64));
        assert!((65..=77).all(|i| !s.contains(i)));
        assert!((78..=127).all(|i| s.contains(i)));
        assert!((128..=189).all(|i| !s.contains(i)));
        assert!((190..=191).all(|i| s.contains(i)));
        assert!((192..=200).all(|i| !s.contains(i)));
        assert!(s.contains(201));
        assert!((202..=255).all(|i| !s.contains(i)));
        assert!(!s.any_range(0, 19));
        assert!(s.all_range(20, 29));
        assert!(!s.any_range(30, 63));
        assert!(s.all_range(64, 64));
        assert!(!s.any_range(65, 77));
        assert!(s.all_range(78, 127));
        assert!(!s.any_range(128, 189));
        assert!(s.all_range(190, 191));
        assert!(!s.any_range(192, 200));
        assert!(s.all_range(201, 201));
        assert!(!s.any_range(202, 255));

        s.clear_range(127, 130);
        assert!((0..=19).all(|i| !s.contains(i)));
        assert!((20..30).all(|i| s.contains(i)));
        assert!((30..=63).all(|i| !s.contains(i)));
        assert!(s.contains(64));
        assert!((65..=77).all(|i| !s.contains(i)));
        assert!((78..127).all(|i| s.contains(i)));
        assert!((127..=189).all(|i| !s.contains(i)));
        assert!((190..=191).all(|i| s.contains(i)));
        assert!((192..=200).all(|i| !s.contains(i)));
        assert!(s.contains(201));
        assert!((202..=255).all(|i| !s.contains(i)));
        assert!(!s.any_range(0, 19));
        assert!(s.all_range(20, 29));
        assert!(!s.any_range(30, 63));
        assert!(s.all_range(64, 64));
        assert!(!s.any_range(65, 77));
        assert!(s.all_range(78, 126));
        assert!(!s.any_range(127, 189));
        assert!(s.all_range(190, 191));
        assert!(!s.any_range(192, 200));
        assert!(s.all_range(201, 201));
        assert!(!s.any_range(202, 255));

        s.clear_range(195, 230);
        assert!((0..=19).all(|i| !s.contains(i)));
        assert!((20..30).all(|i| s.contains(i)));
        assert!((30..=63).all(|i| !s.contains(i)));
        assert!(s.contains(64));
        assert!((65..=77).all(|i| !s.contains(i)));
        assert!((78..127).all(|i| s.contains(i)));
        assert!((127..=189).all(|i| !s.contains(i)));
        assert!((190..=191).all(|i| s.contains(i)));
        assert!((192..=255).all(|i| !s.contains(i)));
        assert!(!s.any_range(0, 19));
        assert!(s.all_range(20, 29));
        assert!(!s.any_range(30, 63));
        assert!(s.all_range(64, 64));
        assert!(!s.any_range(65, 77));
        assert!(s.all_range(78, 126));
        assert!(!s.any_range(127, 189));
        assert!(s.all_range(190, 191));
        assert!(!s.any_range(192, 255));
    }
}
