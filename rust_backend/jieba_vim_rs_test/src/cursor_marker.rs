// Copyright 2024 Kaiwen Wu. All Rights Reserved.
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

use std::fmt;

use serde::{Deserialize, Serialize};

/// `{` represents the cursor before a motion. `}` represents the cursor after
/// a motion.
pub struct CursorMarker;

/// The error that may be raised by [`CursorMarker`].
#[derive(PartialEq, Eq)]
pub enum Error {
    /// If more than one the cursor marker enclosed is found.
    MoreThanOne(char),
    /// If the cursor marker enclosed is not found.
    Missing(char),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MoreThanOne(marker) => {
                write!(f, "More than one marker `{}` is found", marker)
            }
            Self::Missing(marker) => write!(f, "Missing marker `{}`", marker),
        }
    }
}

/// The position (lnum, col) of a cursor. `lnum` is 1-indexed while `col` is
/// 0-indexed.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub struct CursorPosition {
    pub lnum: usize,
    pub col: usize,
}

/// The output of [`CursorMarker::strip_markers`].
#[derive(Debug)]
pub struct StripMarkerOutput {
    pub before_cursor_position: CursorPosition,
    pub after_cursor_position: CursorPosition,
    pub stripped_buffer: Vec<String>,
}

// We assume that each cursor marker is ASCII, and consumes exactly one byte.
const CURSOR_BEFORE_CHAR: char = '{';
const CURSOR_AFTER_CHAR: char = '}';

impl CursorMarker {
    fn marker_predicate(&self, c: char) -> bool {
        match c {
            CURSOR_BEFORE_CHAR | CURSOR_AFTER_CHAR => true,
            _ => false,
        }
    }

    fn strip_marker_str(
        &self,
        s: &mut String,
    ) -> Result<(Option<usize>, Option<usize>), Error> {
        let mut before_cursor_col = None;
        let mut after_cursor_col = None;
        for _ in 0..2 {
            if let Some(i) = s.find(|c| self.marker_predicate(c)) {
                let c = s.drain(i..i + 1).next().unwrap();
                if c == CURSOR_BEFORE_CHAR {
                    if before_cursor_col.is_some() {
                        return Err(Error::MoreThanOne(CURSOR_BEFORE_CHAR));
                    }
                    before_cursor_col.get_or_insert(i);
                } else {
                    if after_cursor_col.is_some() {
                        return Err(Error::MoreThanOne(CURSOR_AFTER_CHAR));
                    }
                    after_cursor_col.get_or_insert(i);
                }
            }
        }
        if let Some(i) = s.find(|c| self.marker_predicate(c)) {
            let c = s.drain(i..i + 1).next().unwrap();
            if c == CURSOR_BEFORE_CHAR {
                return Err(Error::MoreThanOne(CURSOR_BEFORE_CHAR));
            } else {
                return Err(Error::MoreThanOne(CURSOR_AFTER_CHAR));
            }
        }
        Ok((before_cursor_col, after_cursor_col))
    }

    /// Strip the markers off `lines`, and return the cursor positions
    /// `(lnum, col)` before and after the underlying motion. Panics if the
    /// markers are not found or duplicate markers are detected.
    pub fn strip_markers<L: IntoIterator<Item = String>>(
        &self,
        lines: L,
    ) -> Result<StripMarkerOutput, Error> {
        let mut lines: Vec<_> = lines.into_iter().collect();
        let mut before_position = None;
        let mut after_position = None;
        for (lnum, line) in lines.iter_mut().enumerate() {
            let lnum = lnum + 1;
            let (before_col, after_col) = self.strip_marker_str(line)?;
            if let Some(i) = before_col {
                if before_position.is_some() {
                    return Err(Error::MoreThanOne(CURSOR_BEFORE_CHAR));
                }
                before_position.get_or_insert(CursorPosition { lnum, col: i });
            }
            if let Some(j) = after_col {
                if after_position.is_some() {
                    return Err(Error::MoreThanOne(CURSOR_AFTER_CHAR));
                }
                after_position.get_or_insert(CursorPosition { lnum, col: j });
            }
        }
        if before_position.is_none() {
            return Err(Error::Missing(CURSOR_BEFORE_CHAR));
        }
        if after_position.is_none() {
            return Err(Error::Missing(CURSOR_AFTER_CHAR));
        }
        Ok(StripMarkerOutput {
            before_cursor_position: before_position.unwrap(),
            after_cursor_position: after_position.unwrap(),
            stripped_buffer: lines,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CursorMarker, CursorPosition, Error, CURSOR_AFTER_CHAR,
        CURSOR_BEFORE_CHAR,
    };

    fn into_vec_string<I: IntoIterator<Item = &'static str>>(
        v: I,
    ) -> Vec<String> {
        v.into_iter().map(|s| s.to_string()).collect()
    }

    impl PartialEq<(usize, usize)> for CursorPosition {
        fn eq(&self, other: &(usize, usize)) -> bool {
            self.lnum == other.0 && self.col == other.1
        }
    }

    #[test]
    fn test_cursor_marker_strip_markers() {
        let cm = CursorMarker;

        let lines = into_vec_string(["foo {bar", "hel}lo"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (1, 4));
        assert_eq!(o.after_cursor_position, (2, 3));
        assert_eq!(o.stripped_buffer, vec!["foo bar", "hello"]);

        let lines = into_vec_string(["foo{ b}ar", "hello"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (1, 3));
        assert_eq!(o.after_cursor_position, (1, 5));
        assert_eq!(o.stripped_buffer, vec!["foo bar", "hello"]);

        let lines = into_vec_string(["foo} b{ar", "hello"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (1, 5));
        assert_eq!(o.after_cursor_position, (1, 3));
        assert_eq!(o.stripped_buffer, vec!["foo bar", "hello"]);

        let lines = into_vec_string(["fo{}o bar", "hello"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (1, 2));
        assert_eq!(o.after_cursor_position, (1, 2));
        assert_eq!(o.stripped_buffer, vec!["foo bar", "hello"]);

        let lines = into_vec_string(["fo}{o bar", "hello"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (1, 2));
        assert_eq!(o.after_cursor_position, (1, 2));
        assert_eq!(o.stripped_buffer, vec!["foo bar", "hello"]);

        let lines = into_vec_string(["hello", "foo{ b}ar"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (2, 3));
        assert_eq!(o.after_cursor_position, (2, 5));
        assert_eq!(o.stripped_buffer, vec!["hello", "foo bar"]);

        let lines = into_vec_string(["hello", "foo} b{ar"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (2, 5));
        assert_eq!(o.after_cursor_position, (2, 3));
        assert_eq!(o.stripped_buffer, vec!["hello", "foo bar"]);

        let lines = into_vec_string(["hello", "fo{}o bar"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (2, 2));
        assert_eq!(o.after_cursor_position, (2, 2));
        assert_eq!(o.stripped_buffer, vec!["hello", "foo bar"]);

        let lines = into_vec_string(["hello", "fo}{o bar"]);
        let o = cm.strip_markers(lines).unwrap();
        assert_eq!(o.before_cursor_position, (2, 2));
        assert_eq!(o.after_cursor_position, (2, 2));
        assert_eq!(o.stripped_buffer, vec!["hello", "foo bar"]);

        let lines = into_vec_string(["hello"]);
        let err = cm.strip_markers(lines).unwrap_err();
        assert_eq!(err, Error::Missing(CURSOR_BEFORE_CHAR));

        let lines = into_vec_string(["ab{{c"]);
        let err = cm.strip_markers(lines).unwrap_err();
        assert_eq!(err, Error::MoreThanOne(CURSOR_BEFORE_CHAR));

        let lines = into_vec_string(["ab}}c"]);
        let err = cm.strip_markers(lines).unwrap_err();
        assert_eq!(err, Error::MoreThanOne(CURSOR_AFTER_CHAR));

        let lines = into_vec_string(["a{{b}c"]);
        let err = cm.strip_markers(lines).unwrap_err();
        assert_eq!(err, Error::MoreThanOne(CURSOR_BEFORE_CHAR));

        let lines = into_vec_string(["a}}b{c"]);
        let err = cm.strip_markers(lines).unwrap_err();
        assert_eq!(err, Error::MoreThanOne(CURSOR_AFTER_CHAR));

        let lines = into_vec_string(["a{{b}}c"]);
        let err = cm.strip_markers(lines).unwrap_err();
        assert_eq!(err, Error::MoreThanOne(CURSOR_BEFORE_CHAR));
    }
}
