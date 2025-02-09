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

/// Jieba-like types, defined so that this crate won't need to actually depend
/// on `jieba-rs`.
pub trait JiebaPlaceholder {
    /// Cut sentence with `hmm` enabled.
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str>;
}

#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
pub struct KeywordCutter {
    dict: HashSet<String>,
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
        let mut i = 0;
        let boundaries: Vec<_> = sentence
            .char_indices()
            .map(|(i, _)| i)
            .chain([sentence.len()])
            .collect();
        let mut result = Vec::new();
        let n_chars = boundaries.len();
        while i < n_chars {
            let mut found = false;
            for j in (i + 1..=n_chars).rev() {
                let segment = &sentence[boundaries[i]..boundaries[j]];
                if self.dict.contains(segment) {
                    result.push(segment);
                    i = j;
                    found = true;
                    break;
                }
            }
            if !found {
                let mut j = i + 1;
                let mut segment = &sentence[boundaries[i]..boundaries[j]];
                while j < n_chars && !self.dict.contains(segment) {
                    j += 1;
                    segment = &sentence[boundaries[i]..boundaries[j]];
                }
                result.push(segment);
                i = j;
            }
        }

        result
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
    }
}
