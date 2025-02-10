// Copyright 2025 Kaiwen Wu. All Rights Reserved.
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

//! This module defines abstraction over `Jieba`, and several implementation of
//! it.

/// Jieba-like types, defined so that this crate won't need to actually depend
/// on `jieba-rs`.
pub trait JiebaPlaceholder {
    /// Cut sentence with `hmm` enabled.
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str>;
}

#[cfg(test)]
use jieba_rs::Jieba;

#[cfg(test)]
impl JiebaPlaceholder for Jieba {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.cut(sentence, true)
    }
}

#[cfg(test)]
use trie_rs::Trie;

/// Cut words deterministically according to a predefined dictionary.
#[cfg(test)]
pub struct KeywordCutter {
    dict: Trie<u8>,
}

#[cfg(test)]
impl KeywordCutter {
    pub fn new(dict: impl IntoIterator<Item = String>) -> Self {
        Self {
            dict: dict.into_iter().collect(),
        }
    }
}

#[cfg(test)]
impl JiebaPlaceholder for KeywordCutter {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        // A `matched_len` basically means that at the i-th char position,
        // either sentence[i..i + matched_len[i]] is a keyword of `self`, or
        // `matched_len[i]` is zero.
        let matched_len: Vec<_> = sentence
            .char_indices()
            .map(|(start, ch)| {
                self.dict
                    .common_prefix_search(&sentence[start..])
                    .map(|n: String| (n.chars().count(), n.len()))
                    .max_by_key(|(n_chars, _)| *n_chars)
                    .unwrap_or((0, ch.len_utf8()))
            })
            .collect();

        /// Basically, what it does is to scan from left to right in
        /// `matched_len`. On nonzero value n, push n to the result, and skip
        /// the next n-1 element in `matched_len`; on zero value, find the next
        /// nonzero value and push the number of contiguous zero values before
        /// one is found. Finally, return the result Vec. It's guaranteed that
        /// `matched_len` is long enough.
        fn fold_to_bytes(matched_len: Vec<(usize, usize)>) -> Vec<usize> {
            let mut result = Vec::new();
            let mut i = 0;

            while i < matched_len.len() {
                let (n_chars, n_bytes) = matched_len[i];
                if n_chars > 0 {
                    result.push(n_bytes);
                    i += n_chars; // Skip the next n-1 elements
                } else {
                    let zero_byte_count = matched_len[i..]
                        .iter()
                        .take_while(|(n_chars, _)| n_chars == &0)
                        .map(|(_, n_bytes)| {
                            i += 1;
                            n_bytes
                        })
                        .sum();
                    result.push(zero_byte_count);
                }
            }

            result
        }

        let folded = fold_to_bytes(matched_len);
        let mut start = 0;
        folded
            .into_iter()
            .map(|n_bytes| {
                let segment = &sentence[start..start + n_bytes];
                start += n_bytes;
                segment
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{JiebaPlaceholder, KeywordCutter};

    #[test]
    fn test_keyword_cutter() {
        let kc =
            KeywordCutter::new(["fooo".into(), "bar".into(), "你好".into()]);
        assert!(kc.cut_hmm("").is_empty());
        assert_eq!(kc.cut_hmm("bazz"), vec!["bazz"]);
        assert_eq!(kc.cut_hmm("barr"), vec!["bar", "r"]);
        assert_eq!(kc.cut_hmm("rbar"), vec!["r", "bar"]);
        assert_eq!(kc.cut_hmm("foooquxbazbaz"), vec!["fooo", "quxbazbaz"]);
        assert_eq!(
            kc.cut_hmm("bazfooobarquxbazfoobar"),
            vec!["baz", "fooo", "bar", "quxbazfoo", "bar"]
        );
        assert_eq!(kc.cut_hmm("你好你"), vec!["你好", "你"]);
        assert_eq!(
            kc.cut_hmm("foo你bar好你好"),
            vec!["foo你", "bar", "好", "你好"]
        );

        let kc = KeywordCutter::new(["⼀".into()]);
        assert_eq!(kc.cut_hmm("\u{300}A⼀"), vec!["\u{300}A", "⼀"]);
    }
}
