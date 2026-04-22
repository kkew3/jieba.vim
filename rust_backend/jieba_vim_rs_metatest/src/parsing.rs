// Copyright 2026 Kaiwen Wu. All Rights Reserved.
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

//! Metatest parser.

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, Write};

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use serde::Serialize;
use sha2::{Digest, Sha224};
use thiserror::Error;

const ID_LEN: usize = 28;

/// The test hash (`H`). The hash name is ignored.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestHashId {
    /// If given by "?".
    Unspecified,
    /// If an sha224 hash.
    Sha2([u8; ID_LEN]),
}

/// The test hash (`H`) and its span (file, line No.).
#[derive(Clone, PartialEq, Eq)]
pub struct TestHash {
    pub file: Utf8PathBuf,
    pub lineno: usize,
    pub id: TestHashId,
}

fn is_hex_char(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        _ => None,
    }
}

fn parse_id_hex(s: &str) -> Option<[u8; ID_LEN]> {
    let mut bytes = [0u8; ID_LEN];
    let mut chars = s.chars();
    let mut i = 0;
    while let Some(c1) = chars.next() {
        let x1 = is_hex_char(c1)?;
        let x2 = is_hex_char(chars.next()?)?;
        let x = (x1 << 4) | x2;
        if i >= ID_LEN {
            return None;
        }
        bytes[i] = x;
        i += 1;
    }
    if i != ID_LEN {
        return None;
    }
    Some(bytes)
}

impl TestHashId {
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if dr == "H" {
            let id = remainder.next().ok_or_else(|| {
                span.as_col_span(2)
                    .to_parse_error("expecting test id but found none")
            })?;
            if id == "?" {
                Ok(Self::Unspecified)
            } else if let Some(bytes) = parse_id_hex(id) {
                Ok(Self::Sha2(bytes))
            } else {
                Err(span
                    .as_col_span(2)
                    .to_parse_error("invalid test id hex string"))
            }
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

impl fmt::Display for TestHashId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unspecified => f.write_str("?"),
            Self::Sha2(bytes) => {
                for b in bytes {
                    let x1 = b >> 4;
                    if x1 < 10 {
                        write!(f, "{}", (b'0' + x1) as char)?;
                    } else {
                        write!(f, "{}", (b'a' + (x1 - 10)) as char)?;
                    }
                    let x2 = b & 0x0F;
                    if x2 < 10 {
                        write!(f, "{}", (b'0' + x2) as char)?;
                    } else {
                        write!(f, "{}", (b'a' + (x2 - 10)) as char)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl TestHash {
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let id = TestHashId::parse(span, line)?;
        Ok(Self {
            file: span.file.clone(),
            lineno: span.lineno,
            id,
        })
    }
}

/// How to export the test case block (`X`).
#[derive(Clone, Copy, Serialize)]
pub enum ExportType {
    /// If `X` is "b".
    Bootstrap,
    /// If `X` is "u".
    Unit,
    /// If `X` is "i".
    Integration,
    /// If `X` is "ui" or "iu".
    UnitIntegration,
}

impl ExportType {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if (!as_default && dr == "X") || (as_default && dr == "#X") {
            let export_type = remainder.next().ok_or_else(|| {
                span.as_col_span(dr.len() + 1)
                    .to_parse_error("expecting export type but found none")
            })?;
            match export_type {
                "u" => Ok(Self::Unit),
                "i" => Ok(Self::Integration),
                "ui" | "iu" => Ok(Self::UnitIntegration),
                "b" => Ok(Self::Bootstrap),
                s => Err(span
                    .as_col_span(dr.len() + 1)
                    .to_parse_error(format!("unexpected export type `{}`", s))),
            }
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// The editor mode hint (`M`).
#[derive(Clone, Copy, Serialize)]
pub enum EditorMode {
    Normal,
    VisualChar,
    VisualLine,
    VisualBlock,
    OperatorPending,
    MixedNormal,
    MixedVisualChar,
    MixedVisualLine,
    MixedVisualBlock,
}

impl EditorMode {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if (!as_default && dr == "M") || (as_default && dr == "#M") {
            let mode = remainder.next().ok_or_else(|| {
                span.as_col_span(dr.len() + 1)
                    .to_parse_error("expecting editor mode but found none")
            })?;
            match mode {
                "n" => Ok(Self::Normal),
                "v" => Ok(Self::VisualChar),
                "V" => Ok(Self::VisualLine),
                r"\<C-v>" => Ok(Self::VisualBlock),
                "o" => Ok(Self::OperatorPending),
                "mn" => Ok(Self::MixedNormal),
                "mv" => Ok(Self::MixedVisualChar),
                "mV" => Ok(Self::MixedVisualLine),
                r"m\<C-v>" => Ok(Self::MixedVisualBlock),
                s => Err(span
                    .as_col_span(dr.len() + 1)
                    .to_parse_error(format!("unexpected editor mode `{}`", s))),
            }
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// Motion keys.
#[derive(Clone, Copy, Serialize)]
pub enum MotionKey {
    /// w
    Ws,
    /// W
    Wl,
    /// b
    Bs,
    /// B
    Bl,
    /// e
    Es,
    /// E
    El,
    /// ge
    Ges,
    /// gE
    Gel,
    /// iw
    Iws,
    /// iW
    Iwl,
    /// aw
    Aws,
    /// aW
    Awl,
}

impl AsRef<str> for MotionKey {
    fn as_ref(&self) -> &str {
        match self {
            Self::Ws => "w",
            Self::Wl => "W",
            Self::Bs => "b",
            Self::Bl => "B",
            Self::Es => "e",
            Self::El => "E",
            Self::Ges => "ge",
            Self::Gel => "gE",
            Self::Iws => "iw",
            Self::Iwl => "iW",
            Self::Aws => "aw",
            Self::Awl => "aW",
        }
    }
}

/// The key sequence (`K`).
#[derive(Clone)]
pub enum KeySequence {
    Motion(MotionKey),
    AnyNormal(String),
}

impl KeySequence {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if (!as_default && dr == "K") || (as_default && dr == "#K") {
            let ks = remainder.next().ok_or_else(|| {
                span.as_col_span(dr.len() + 1)
                    .to_parse_error("expecting key sequence but found none")
            })?;
            match ks {
                "w" => Ok(Self::Motion(MotionKey::Ws)),
                "W" => Ok(Self::Motion(MotionKey::Wl)),
                "b" => Ok(Self::Motion(MotionKey::Bs)),
                "B" => Ok(Self::Motion(MotionKey::Bl)),
                "e" => Ok(Self::Motion(MotionKey::Es)),
                "E" => Ok(Self::Motion(MotionKey::El)),
                "ge" => Ok(Self::Motion(MotionKey::Ges)),
                "gE" => Ok(Self::Motion(MotionKey::Gel)),
                "iw" => Ok(Self::Motion(MotionKey::Iws)),
                "iW" => Ok(Self::Motion(MotionKey::Iwl)),
                "aw" => Ok(Self::Motion(MotionKey::Aws)),
                "aW" => Ok(Self::Motion(MotionKey::Awl)),
                ks => Ok(Self::AnyNormal(ks.into())),
            }
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }

    fn into_motion_key(self) -> Option<MotionKey> {
        match self {
            Self::Motion(mk) => Some(mk),
            _ => None,
        }
    }

    fn into_normal_sequence(self) -> String {
        match self {
            Self::AnyNormal(ks) => ks,
            Self::Motion(MotionKey::Ws) => "w".into(),
            Self::Motion(MotionKey::Wl) => "W".into(),
            Self::Motion(MotionKey::Bs) => "b".into(),
            Self::Motion(MotionKey::Bl) => "B".into(),
            Self::Motion(MotionKey::Es) => "e".into(),
            Self::Motion(MotionKey::El) => "E".into(),
            Self::Motion(MotionKey::Ges) => "ge".into(),
            Self::Motion(MotionKey::Gel) => "gE".into(),
            Self::Motion(MotionKey::Iws) => "iw".into(),
            Self::Motion(MotionKey::Iwl) => "iW".into(),
            Self::Motion(MotionKey::Aws) => "aw".into(),
            Self::Motion(MotionKey::Awl) => "aW".into(),
        }
    }
}

/// The operator string (`O`).
#[derive(Clone)]
struct Operator(String);

impl Operator {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if (!as_default && dr == "O") || (as_default && dr == "#O") {
            let op = remainder.next().ok_or_else(|| {
                span.as_col_span(dr.len() + 1)
                    .to_parse_error("expecting operator but found none")
            })?;
            Ok(Self(op.into()))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// One printable 7-bit ASCII, treated as a single-char string.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Ascii(char);

impl Ascii {
    pub fn new(c: u8) -> Option<Self> {
        match c {
            0x20..=0x7E => Some(Self(c as char)),
            _ => None,
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c as u32 {
            0x20..=0x7E => Some(Self(c)),
            _ => None,
        }
    }
}

impl From<Ascii> for u8 {
    fn from(value: Ascii) -> Self {
        value.0 as u8
    }
}

impl From<Ascii> for char {
    fn from(value: Ascii) -> Self {
        value.0
    }
}

impl From<&Ascii> for u8 {
    fn from(value: &Ascii) -> Self {
        value.0 as u8
    }
}

impl From<&Ascii> for char {
    fn from(value: &Ascii) -> Self {
        value.0
    }
}

impl fmt::Display for Ascii {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The register (`R`).
#[derive(Clone, Copy)]
struct Register(Ascii);

impl Default for Register {
    fn default() -> Self {
        Self(Ascii('"'))
    }
}

impl Register {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if (!as_default && dr == "R") || (as_default && dr == "#R") {
            let reg = remainder.next().ok_or_else(|| {
                span.as_col_span(dr.len() + 1);
                span.to_parse_error("expecting register char but found none")
            })?;
            if reg.len() != 1 {
                return Err(span.to_parse_error(
                    "expecting register char but found string",
                ));
            }
            let reg_char = Ascii::from_char(reg.chars().next().unwrap())
                .ok_or_else(|| {
                    span.to_parse_error(
                        "register char not printable 7-bit ascii",
                    )
                })?;
            Ok(Self(reg_char))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// The count (`C`).
#[derive(Clone, Copy, Default)]
struct Count(u64);

impl Count {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if (!as_default && dr == "C") || (as_default && dr == "#C") {
            let count_str = remainder.next().ok_or_else(|| {
                span.as_col_span(dr.len() + 1)
                    .to_parse_error("expecting count but found none")
            })?;
            match count_str.parse() {
                Ok(count) => Ok(Self(count)),
                Err(err) => Err(span.as_col_span(dr.len() + 1).to_parse_error(
                    format!("error parsing count unsigned integer: {}", err),
                )),
            }
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// Supported functions in [`StateExpr::Function`].
#[derive(Clone, Serialize)]
pub enum StateExprFunction {
    /// visualmode() function.
    Visualmode(String),
}

/// A buffer position.
pub type Position = [u64; 4];

/// A buffer position with curswant.
pub type PositionCurswant = [u64; 5];

fn parse_position(s: &str) -> Option<Vec<u64>> {
    if s.starts_with('[') && s.ends_with(']') {
        let mut pos = Vec::new();
        for s in s[1..s.len() - 1].split(',') {
            pos.push(s.parse().ok()?);
        }
        Some(pos)
    } else {
        None
    }
}

/// Max curswant value, used for curswant at end-of-line (eol).
pub const CURSWANT_MAX: u64 = 2147483647;

/// The state expression of `S0` or `S1`.
#[derive(Clone, Serialize)]
pub enum StateExpr {
    /// A Vim option.
    Option {
        /// The option name.
        name: String,
        /// The option value.
        value: String,
    },
    /// A Vim function.
    Function(StateExprFunction),
    Mark {
        /// The mark name.
        name: Ascii,
        /// The mark position.
        position: Position,
    },
    Register {
        /// The register name.
        name: Ascii,
        /// The register value.
        value: String,
    },
}

impl StateExpr {
    fn parse(s: &str) -> Option<Self> {
        if let Some((func_name, value)) = s.split_once("()=") {
            if func_name == "visualmode" {
                Some(Self::Function(StateExprFunction::Visualmode(
                    value.into(),
                )))
            } else {
                None
            }
        } else if s.starts_with('\'')
            && let Some((mark, position_str)) = s[1..].split_once("=")
        {
            if mark.len() != 1 {
                return None;
            }
            let mark_char = Ascii::from_char(mark.chars().next().unwrap())?;
            let pos = parse_position(position_str)?;
            if pos.len() == 4 {
                Some(Self::Mark {
                    name: mark_char,
                    position: [pos[0], pos[1], pos[2], pos[3]],
                })
            } else {
                None
            }
        } else if s.starts_with('"')
            && let Some((register, value)) = s[1..].split_once("=")
        {
            if register.len() != 1 {
                return None;
            }
            let register_char =
                Ascii::from_char(register.chars().next().unwrap())?;
            Some(Self::Register {
                name: register_char,
                value: value.into(),
            })
        } else if let Some((option_name, value)) = s.split_once("=") {
            Some(Self::Option {
                name: option_name.into(),
                value: value.into(),
            })
        } else {
            None
        }
    }
}

/// The state before (`S0`).
#[derive(Clone)]
struct StateBefore(Vec<StateExpr>);

impl StateBefore {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, remainder) = split_directive(span, line)?;
        if (!as_default && dr == "S0") || (as_default && dr == "#S0") {
            let mut states = Vec::new();
            for s in remainder {
                states.push(StateExpr::parse(s).ok_or_else(|| {
                    span.as_col_span(dr.len() + 1)
                        .to_parse_error("invalid state expression")
                })?);
            }
            Ok(Self(states))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// The state after (`S1`).
struct StateAfter(Vec<StateExpr>);

impl StateAfter {
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let (dr, remainder) = split_directive(span, line)?;
        if dr == "S1" {
            let mut states = Vec::new();
            for s in remainder {
                states.push(StateExpr::parse(s).ok_or_else(|| {
                    span.as_col_span(dr.len() + 1)
                        .to_parse_error("invalid state expression")
                })?);
            }
            Ok(Self(states))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// The buffer expression.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct BufferExpr {
    pub clean_buffer: Vec<String>,
    pub langle: Option<Position>,
    pub rangle: Option<Position>,
    pub visual_begin: Option<Position>,
    pub visual_end: Option<Position>,
    pub cursor: Option<PositionCurswant>,
}

impl BufferExpr {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        let position_marks = ['<', '>', '[', ']', '|', '\\'];
        let mut markup_buffer = Vec::new();
        let mut line = String::new();
        let mut chars = s.chars().peekable();
        let mut is_empty_buffer = false;
        let eol_marks = ['␊', '␀'];
        while let Some(c) = chars.next() {
            line.push(c);
            if eol_marks.contains(&c) {
                if c == '␀' {
                    is_empty_buffer = true;
                }
                let mut chars_after_lf = String::new();
                let mut any_tilde = false;
                while let Some(c1) = chars.peek().copied()
                    && (c1 == '~' || position_marks.contains(&c1))
                {
                    chars_after_lf.push(chars.next().unwrap());
                    if c1 == '~' {
                        any_tilde = true;
                    }
                }
                if any_tilde {
                    line.push_str(&chars_after_lf);
                    markup_buffer.push(line);
                    line = String::new();
                } else {
                    markup_buffer.push(line);
                    line = chars_after_lf;
                }
            }
        }
        if !line.is_empty() {
            return None;
        }
        if is_empty_buffer && markup_buffer.len() > 1 {
            return None;
        }

        let mut langle = None;
        let mut rangle = None;
        let mut visual_begin = None;
        let mut visual_end = None;
        let mut cursor = None;
        let mut clean_buffer = Vec::new();
        for (lineno, mut markup_line) in markup_buffer.into_iter().enumerate() {
            let mut langle_col = None;
            let mut rangle_col = None;
            let mut visual_begin_col = None;
            let mut visual_end_col = None;
            let mut cursor_col = None;
            let mut curswant_col = None;

            let lineno = lineno as u64 + 1;
            let mut clean_chars = Vec::new();
            let mut has_virtual_cols = false;
            for (i, c) in markup_line.drain(..).rev().enumerate() {
                if i == 0 && c == '~' {
                    has_virtual_cols = true;
                }
                if !position_marks.contains(&c) {
                    clean_chars.push(c);
                } else {
                    if clean_chars.is_empty() {
                        return None;
                    }
                    let counter = clean_chars.len();
                    if c == '<' {
                        if langle.is_some() || langle_col.is_some() {
                            return None;
                        }
                        langle_col = Some(counter);
                    } else if c == '>' {
                        if rangle.is_some() || rangle_col.is_some() {
                            return None;
                        }
                        rangle_col = Some(counter);
                    } else if c == '[' {
                        if visual_begin.is_some() || visual_begin_col.is_some()
                        {
                            return None;
                        }
                        visual_begin_col = Some(counter);
                    } else if c == ']' {
                        if visual_end.is_some() || visual_end_col.is_some() {
                            return None;
                        }
                        visual_end_col = Some(counter);
                    } else if c == '|' {
                        if cursor.is_some() || cursor_col.is_some() {
                            return None;
                        }
                        cursor_col = Some(counter);
                    } else if c == '\\' {
                        if cursor.is_some() || curswant_col.is_some() {
                            return None;
                        }
                        curswant_col = Some(counter);
                    } else {
                        unreachable!();
                    }
                }
            }

            if cursor_col.is_none() && curswant_col.is_some() {
                return None;
            }

            let mut implicit_curswant = false;
            if cursor_col.is_some() && curswant_col.is_none() {
                curswant_col = cursor_col;
                implicit_curswant = true;
            }

            // Take complementation.
            let vars = [
                &mut langle_col,
                &mut rangle_col,
                &mut visual_begin_col,
                &mut visual_end_col,
                &mut cursor_col,
                &mut curswant_col,
            ];

            let n_clean = clean_chars.len();
            for c in vars.into_iter().flatten() {
                *c = n_clean - *c;
            }

            // Compute (col, off) and prune auxiliary chars.
            let mut col = 0;
            let mut off = 0;
            let mut curswant = 0;
            let mut cursor_curswant = None;
            let mut final_clean_chars = Vec::new();
            let mut cursor_assigned = false;
            for (i, c) in clean_chars.into_iter().rev().enumerate() {
                if col == 0 && c == '~' {
                    return None;
                }
                if c == '~' {
                    off += 1;
                } else {
                    col += 1;
                    off = 0;
                }
                curswant += 1;

                if let Some(j) = langle_col
                    && i == j
                {
                    langle = Some([0, lineno, col, off]);
                }
                if let Some(j) = rangle_col
                    && i == j
                {
                    rangle = Some([0, lineno, col, off]);
                }
                if let Some(j) = visual_begin_col
                    && i == j
                {
                    visual_begin = Some([0, lineno, col, off]);
                }
                if let Some(j) = visual_end_col
                    && i == j
                {
                    visual_end = Some([0, lineno, col, off]);
                }
                if let Some(j) = cursor_col
                    && i == j
                {
                    cursor = Some([0, lineno, col, off, 0]);
                    cursor_assigned = true;
                }
                if let Some(j) = curswant_col
                    && i == j
                {
                    if eol_marks.contains(&c)
                        && !implicit_curswant
                        && !has_virtual_cols
                    {
                        cursor_curswant = Some(CURSWANT_MAX);
                    } else {
                        cursor_curswant = Some(curswant);
                    }
                }

                if c == '·' {
                    final_clean_chars.push(' ');
                } else if c == '┤' {
                    final_clean_chars.push('\t');
                } else if !eol_marks.contains(&c) && c != '@' && c != '~' {
                    final_clean_chars.push(c);
                }
            }

            if cursor_assigned && let Some(p) = &mut cursor {
                p[4] = cursor_curswant.unwrap();
            }

            clean_buffer
                .push(final_clean_chars.into_iter().collect::<String>());
        }

        if is_empty_buffer {
            if clean_buffer.iter().any(|s| !s.is_empty()) {
                return None;
            }
            clean_buffer.clear();
        }

        Some(Self {
            clean_buffer,
            langle,
            rangle,
            visual_begin,
            visual_end,
            cursor,
        })
    }
}

/// The buffer before (`B0`).
#[derive(Clone)]
struct BufferBefore(BufferExpr);

impl BufferBefore {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if (!as_default && dr == "B0") || (as_default && dr == "#B0") {
            span.as_col_span(dr.len() + 1);
            let buffer_expr =
                BufferExpr::parse(remainder.next().ok_or_else(|| {
                    span.to_parse_error(
                        "expecting buffer expression but found none",
                    )
                })?)
                .ok_or_else(|| {
                    span.to_parse_error("invalid buffer expression")
                })?;
            if buffer_expr.langle.is_some() || buffer_expr.rangle.is_some() {
                return Err(span.to_parse_error(
                    "invalid position marks [, ] in buffer expression `B0`",
                ));
            }
            Ok(Self(buffer_expr))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// The buffer pending (`Bp`).
struct BufferPending(BufferExpr);

impl BufferPending {
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if dr == "Bp" {
            span.as_col_span(dr.len() + 1);
            let buffer_expr =
                BufferExpr::parse(remainder.next().ok_or_else(|| {
                    span.to_parse_error(
                        "expecting buffer expression but found none",
                    )
                })?)
                .ok_or_else(|| {
                    span.to_parse_error("invalid buffer expression")
                })?;
            if buffer_expr.cursor.is_some()
                || buffer_expr.visual_begin.is_some()
                || buffer_expr.visual_end.is_some()
                || buffer_expr.langle.is_none()
                || buffer_expr.rangle.is_none()
            {
                return Err(
                    span.to_parse_error("invalid buffer expression `Bp`")
                );
            }
            Ok(Self(buffer_expr))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

struct BufferOutput(BufferExpr);

impl BufferOutput {
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if dr == "Bo" {
            let buffer_expr =
                BufferExpr::parse(remainder.next().ok_or_else(|| {
                    span.as_col_span(dr.len() + 1).to_parse_error(
                        "expecting buffer expression but found none",
                    )
                })?)
                .ok_or_else(|| {
                    span.as_col_span(dr.len() + 1)
                        .to_parse_error("invalid buffer expression")
                })?;
            Ok(Self(buffer_expr))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// The buffer after (`B1`).
struct BufferAfter(BufferExpr);

impl BufferAfter {
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if dr == "B1" {
            let buffer_expr =
                BufferExpr::parse(remainder.next().ok_or_else(|| {
                    span.as_col_span(dr.len() + 1).to_parse_error(
                        "expecting buffer expression but found none",
                    )
                })?)
                .ok_or_else(|| {
                    span.as_col_span(dr.len() + 1)
                        .to_parse_error("invalid buffer expression")
                })?;
            Ok(Self(buffer_expr))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// Model output (`Q`) items.
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Serialize)]
pub enum ModelOutputItem {
    /// |
    Cursor,
    /// | \
    CursorCurswant,
    /// <
    Langle,
    /// >
    Rangle,
    /// Other key-value pairs.
    KeyValue { key: String, value: String },
}

/// Model output (`Q`).
#[derive(Clone)]
struct ModelOutput(Vec<ModelOutputItem>);

impl ModelOutput {
    fn parse(
        as_default: bool,
        span: &mut ErrorSpan,
        line: &str,
    ) -> Result<Self, Error> {
        let (dr, remainder) = split_directive(span, line)?;
        if (!as_default && dr == "Q") || (as_default && dr == "#Q") {
            span.as_col_span(dr.len() + 1);
            let mut curswant_exists = false;
            let mut output_items = Vec::new();
            for token in remainder {
                if token == "|" {
                    output_items.push(ModelOutputItem::Cursor);
                } else if token == "\\" {
                    curswant_exists = true;
                } else if token == "<" {
                    output_items.push(ModelOutputItem::Langle);
                } else if token == ">" {
                    output_items.push(ModelOutputItem::Rangle);
                } else if let Some((key, value)) = token.split_once('=') {
                    output_items.push(ModelOutputItem::KeyValue {
                        key: key.into(),
                        value: value.into(),
                    });
                } else {
                    return Err(span.to_parse_error(format!(
                        "unexpected model output item: {}",
                        token
                    )));
                }
            }
            if curswant_exists {
                for i in output_items.iter_mut() {
                    if *i == ModelOutputItem::Cursor {
                        *i = ModelOutputItem::CursorCurswant;
                    }
                }
            }
            output_items.sort();
            Ok(Self(output_items))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

#[derive(Clone, Serialize)]
pub struct AutocmdEventCount {
    pub event_name: String,
    pub count: Option<u64>,
}

#[derive(Default)]
pub struct AutocmdEventsCount(Vec<AutocmdEventCount>);

impl AutocmdEventsCount {
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let (dr, remainder) = split_directive(span, line)?;
        if dr == "E" {
            span.as_col_span(dr.len() + 1);
            let mut event_counts = Vec::new();
            for token in remainder {
                match token.split_once('=') {
                    Some((event_name, count_str)) => {
                        let ec = if count_str.is_empty() {
                            AutocmdEventCount {
                                event_name: event_name.to_string(),
                                count: None,
                            }
                        } else {
                            let count = count_str.parse().map_err(|_| {
                                span.to_parse_error(format!(
                                    "unexpected autocmd event count: {}",
                                    token
                                ))
                            })?;
                            AutocmdEventCount {
                                event_name: event_name.to_string(),
                                count: Some(count),
                            }
                        };
                        event_counts.push(ec);
                    }
                    None => Err(span.to_parse_error(format!(
                        "unexpected autocmd event count: {}",
                        token
                    )))?,
                }
            }
            Ok(Self(event_counts))
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

/// The defaults (`#(M|K|O|R|C|S0|B0|P|Q)`).
#[derive(Default)]
struct Defaults {
    export_type: Option<ExportType>,
    editor_mode: Option<EditorMode>,
    key_sequence: Option<KeySequence>,
    operator: Option<Operator>,
    register: Register,
    count: Count,
    state_before: Option<StateBefore>,
    buffer_before: Option<BufferBefore>,
    model_output: Option<ModelOutput>,
}

/// The expected metatest file version.
struct ExpectedVersion(&'static str);

impl Default for ExpectedVersion {
    fn default() -> Self {
        Self(include_str!("version").trim())
    }
}

/// The metatest file version.
struct Version(String);

impl Version {
    /// `line` should already be trimmed.
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        let (dr, mut remainder) = split_directive(span, line)?;
        if dr == "#V" {
            let file_version = remainder.next().ok_or_else(|| {
                span.as_col_span(3).to_parse_error("expecting file version")
            })?;
            Ok(Self(file_version.into()))
        } else {
            Err(span.to_invalid_token_error())
        }
    }
}

fn split_directive<'a>(
    span: &mut ErrorSpan,
    line: &'a str,
) -> Result<(&'a str, std::str::SplitAsciiWhitespace<'a>), Error> {
    let mut splits = line.split_ascii_whitespace();
    let dr = splits
        .next()
        .ok_or_else(|| span.as_line_span().to_invalid_token_error())?;
    Ok((dr, splits))
}

/// A test case head conditional.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum HeadConditional {
    /// Assert has feature.
    Feature(String),
    /// Assert has no feature.
    NoFeature(String),
    /// Assert vim version at least.
    VimVersionAtLeast(u32),
}

impl HeadConditional {
    /// `line` should already be trimmed.
    fn parse(span: &mut ErrorSpan, line: &str) -> Result<Self, Error> {
        if line.starts_with("?") {
            if line.starts_with("?has:") {
                // "?has:".len(): 5
                let feature = &line[5..];
                if feature.is_empty() {
                    return Err(span
                        .as_col_span(5)
                        .to_parse_error("expecting feature after `?has:`"));
                }
                Ok(Self::Feature(feature.into()))
            } else if line.starts_with("?!has:") {
                // "?!has:".len(): 6
                let non_feature = &line[6..];
                if non_feature.is_empty() {
                    return Err(span
                        .as_col_span(6)
                        .to_parse_error("expecting feature after `?has:`"));
                }
                Ok(Self::NoFeature(non_feature.into()))
            } else if line.starts_with("?version:") {
                // "?version:".len(): 9
                let vim_version_str = &line[9..];
                if vim_version_str.is_empty() {
                    return Err(span.as_col_span(9).to_parse_error(
                        "expecting vim version after `?version:`",
                    ));
                }
                let vim_version = vim_version_str.parse().map_err(|err| {
                    span.as_col_span(9 + 1).to_parse_error(format!(
                        "error parsing vim version: {}",
                        err
                    ))
                })?;
                Ok(Self::VimVersionAtLeast(vim_version))
            } else {
                Err(span.as_col_span(1).to_parse_error(format!(
                    "unsupported conditional encountered: `{}`",
                    &line[1..]
                )))
            }
        } else {
            Err(span.as_line_span().to_invalid_token_error())
        }
    }
}

#[derive(Clone)]
struct HeadConditionals(Vec<HeadConditional>);

impl HeadConditionals {
    /// `line` should already be trimmed.
    fn parse<T>(
        &mut self,
        span: &mut ErrorSpan,
        test_cases: &[T],
        line: &str,
    ) -> Result<(), Error> {
        let hc = HeadConditional::parse(span, line)?;
        if !test_cases.is_empty() {
            Err(span.to_parse_error(
                "head conditionals must be presented before test cases",
            ))
        } else {
            self.0.push(hc);
            Ok(())
        }
    }
}

/// A raw text case block before validation.
pub struct RawTestCaseBlock {
    head_conditionals: HeadConditionals,
    hash: TestHash,
    export_type: Option<ExportType>,
    editor_mode: Option<EditorMode>,
    key_sequence: Option<KeySequence>,
    operator: Option<Operator>,
    register: Option<Register>,
    count: Option<Count>,
    state_before: Option<StateBefore>,
    state_after: Option<StateAfter>,
    buffer_before: Option<BufferBefore>,
    buffer_pending: Option<BufferPending>,
    buffer_output: Option<BufferOutput>,
    buffer_after: Option<BufferAfter>,
    model_output: Option<ModelOutput>,
    autocmd_events_count: Option<AutocmdEventsCount>,
}

impl RawTestCaseBlock {
    fn new(head_conditionals: HeadConditionals, hash: TestHash) -> Self {
        Self {
            head_conditionals,
            hash,
            export_type: None,
            editor_mode: None,
            key_sequence: None,
            operator: None,
            register: None,
            count: None,
            state_before: None,
            state_after: None,
            buffer_before: None,
            buffer_pending: None,
            buffer_output: None,
            buffer_after: None,
            model_output: None,
            autocmd_events_count: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
pub enum UnitEditorMode {
    Normal,
    VisualChar,
    VisualLine,
    VisualBlock,
    OperatorPending,
}

impl UnitEditorMode {
    fn from_editor_mode_opt(editor_mode: EditorMode) -> Option<Self> {
        match editor_mode {
            EditorMode::Normal => Some(Self::Normal),
            EditorMode::VisualChar => Some(Self::VisualChar),
            EditorMode::VisualLine => Some(Self::VisualLine),
            EditorMode::VisualBlock => Some(Self::VisualBlock),
            EditorMode::OperatorPending => Some(Self::OperatorPending),
            _ => None,
        }
    }
}

impl From<UnitEditorMode> for EditorMode {
    fn from(value: UnitEditorMode) -> Self {
        match value {
            UnitEditorMode::Normal => Self::Normal,
            UnitEditorMode::VisualChar => Self::VisualChar,
            UnitEditorMode::VisualLine => Self::VisualLine,
            UnitEditorMode::VisualBlock => Self::VisualBlock,
            UnitEditorMode::OperatorPending => Self::OperatorPending,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
pub enum UnitExportType {
    /// Export to unit test only.
    Unit,
    /// Export to both unit test and integration test.
    UnitIntegration,
}

impl UnitExportType {
    fn from_export_type_opt(export_type: ExportType) -> Option<Self> {
        match export_type {
            ExportType::Unit => Some(Self::Unit),
            ExportType::UnitIntegration => Some(Self::UnitIntegration),
            ExportType::Integration | ExportType::Bootstrap => None,
        }
    }
}

impl From<UnitExportType> for ExportType {
    fn from(value: UnitExportType) -> Self {
        match value {
            UnitExportType::Unit => Self::Unit,
            UnitExportType::UnitIntegration => Self::UnitIntegration,
        }
    }
}

#[derive(Clone)]
pub struct UnitTestCaseBlock {
    pub head_conditionals: Vec<HeadConditional>,
    pub hash: TestHash,
    pub export_type: UnitExportType,
    pub editor_mode: UnitEditorMode,
    pub key_sequence: MotionKey,
    pub operator: Option<String>,
    pub register: Option<Ascii>,
    pub count: u64,
    pub state_before: Vec<StateExpr>,
    pub state_after: Vec<StateExpr>,
    pub buffer_before: BufferExpr,
    pub buffer_pending: Option<BufferExpr>,
    pub buffer_output: BufferExpr,
    pub buffer_after: Option<BufferExpr>,
    pub model_output: Vec<ModelOutputItem>,
    pub autocmd_events_count: Vec<AutocmdEventCount>,
}

impl UnitTestCaseBlock {
    /// Fix outdated hash and return the original.
    pub fn fix_hash(&mut self) -> TestHash {
        let old = self.hash.clone();
        let mut sha = Sha224::new();
        let msg = "failed to fix hash";
        sha.update(serde_json::to_vec(&self.head_conditionals).expect(msg));
        sha.update(serde_json::to_vec(&self.export_type).expect(msg));
        sha.update(serde_json::to_vec(&self.editor_mode).expect(msg));
        sha.update(serde_json::to_vec(&self.key_sequence).expect(msg));
        sha.update(serde_json::to_vec(&self.operator).expect(msg));
        sha.update(serde_json::to_vec(&self.register).expect(msg));
        sha.update(serde_json::to_vec(&self.count).expect(msg));
        sha.update(serde_json::to_vec(&self.state_before).expect(msg));
        sha.update(serde_json::to_vec(&self.state_after).expect(msg));
        sha.update(serde_json::to_vec(&self.buffer_before).expect(msg));
        sha.update(serde_json::to_vec(&self.buffer_output).expect(msg));
        sha.update(serde_json::to_vec(&self.buffer_pending).expect(msg));
        sha.update(serde_json::to_vec(&self.buffer_after).expect(msg));
        sha.update(serde_json::to_vec(&self.model_output).expect(msg));
        sha.update(serde_json::to_vec(&self.autocmd_events_count).expect(msg));
        let new_hash = TestHash {
            file: old.file.clone(),
            lineno: old.lineno,
            id: TestHashId::Sha2(sha.finalize().into()),
        };
        self.hash = new_hash;
        old
    }
}

impl From<UnitTestCaseBlock> for RawTestCaseBlock {
    fn from(value: UnitTestCaseBlock) -> Self {
        Self {
            head_conditionals: HeadConditionals(value.head_conditionals),
            hash: value.hash,
            export_type: Some(value.export_type.into()),
            editor_mode: Some(value.editor_mode.into()),
            key_sequence: Some(KeySequence::Motion(value.key_sequence)),
            operator: value.operator.map(Operator),
            register: value.register.map(Register),
            count: Some(Count(value.count)),
            state_before: Some(StateBefore(value.state_before)),
            state_after: Some(StateAfter(value.state_after)),
            buffer_before: Some(BufferBefore(value.buffer_before)),
            buffer_pending: value.buffer_pending.map(BufferPending),
            buffer_output: Some(BufferOutput(value.buffer_output)),
            buffer_after: value.buffer_after.map(BufferAfter),
            model_output: Some(ModelOutput(value.model_output)),
            autocmd_events_count: if value.autocmd_events_count.is_empty() {
                None
            } else {
                Some(AutocmdEventsCount(value.autocmd_events_count))
            },
        }
    }
}

pub struct CompositeTestCaseBlock {
    pub head_conditionals: Vec<HeadConditional>,
    pub hash: TestHash,
    pub editor_mode: EditorMode,
    pub key_sequence: String,
    pub state_before: Vec<StateExpr>,
    pub state_after: Vec<StateExpr>,
    pub buffer_before: BufferExpr,
    pub buffer_after: Option<BufferExpr>,
}

impl CompositeTestCaseBlock {
    /// Fix outdated hash and return the original.
    pub fn fix_hash(&mut self) -> TestHash {
        let old = self.hash.clone();
        let mut sha = Sha224::new();
        let msg = "failed to fix hash";
        sha.update(serde_json::to_vec(&self.head_conditionals).expect(msg));
        sha.update(serde_json::to_vec(&self.editor_mode).expect(msg));
        sha.update(serde_json::to_vec(&self.key_sequence).expect(msg));
        sha.update(serde_json::to_vec(&self.state_before).expect(msg));
        sha.update(serde_json::to_vec(&self.state_after).expect(msg));
        sha.update(serde_json::to_vec(&self.buffer_before).expect(msg));
        sha.update(serde_json::to_vec(&self.buffer_after).expect(msg));
        let new_hash = TestHash {
            file: old.file.clone(),
            lineno: old.lineno,
            id: TestHashId::Sha2(sha.finalize().into()),
        };
        self.hash = new_hash;
        old
    }
}

pub struct BootstrapTestCaseBlock {
    pub hash: TestHash,
    pub editor_mode: UnitEditorMode,
    pub key_sequence: MotionKey,
    pub operator: Option<String>,
    pub register: Option<Ascii>,
    pub count: u64,
    pub state_before: Vec<StateExpr>,
    pub state_after: Vec<StateExpr>,
    pub buffer_before: BufferExpr,
    pub autocmd_events_count: Vec<AutocmdEventCount>,
}

impl BootstrapTestCaseBlock {
    /// Fix outdated hash and return the original.
    pub fn fix_hash(&mut self) -> TestHash {
        let old = self.hash.clone();
        let mut sha = Sha224::new();
        let msg = "failed to fix hash";
        sha.update(serde_json::to_vec(&self.editor_mode).expect(msg));
        sha.update(serde_json::to_vec(&self.key_sequence).expect(msg));
        sha.update(serde_json::to_vec(&self.operator).expect(msg));
        sha.update(serde_json::to_vec(&self.register).expect(msg));
        sha.update(serde_json::to_vec(&self.count).expect(msg));
        sha.update(serde_json::to_vec(&self.state_before).expect(msg));
        sha.update(serde_json::to_vec(&self.state_after).expect(msg));
        sha.update(serde_json::to_vec(&self.buffer_before).expect(msg));
        sha.update(serde_json::to_vec(&self.autocmd_events_count).expect(msg));
        let new_hash = TestHash {
            file: old.file.clone(),
            lineno: old.lineno,
            id: TestHashId::Sha2(sha.finalize().into()),
        };
        self.hash = new_hash;
        old
    }
}

pub enum TestCaseBlock {
    Unit(UnitTestCaseBlock),
    Composite(CompositeTestCaseBlock),
    Bootstrap(BootstrapTestCaseBlock),
}

impl TestCaseBlock {
    /// Fix outdated hash and return the original.
    fn fix_hash_id(&mut self) -> TestHash {
        match self {
            Self::Unit(b) => b.fix_hash(),
            Self::Composite(b) => b.fix_hash(),
            Self::Bootstrap(b) => b.fix_hash(),
        }
    }

    fn hash_id(&self) -> &TestHash {
        match self {
            Self::Unit(b) => &b.hash,
            Self::Composite(b) => &b.hash,
            Self::Bootstrap(b) => &b.hash,
        }
    }

    fn into_hash_id(self) -> TestHash {
        match self {
            Self::Unit(b) => b.hash,
            Self::Composite(b) => b.hash,
            Self::Bootstrap(b) => b.hash,
        }
    }
}

pub struct TestCase {
    pub block: TestCaseBlock,
    pub file: Utf8PathBuf,
    pub lineno_begin: usize,
    pub lineno_end: usize,
}

impl TestCase {
    pub fn fix_hash_id(&mut self) -> TestHash {
        self.block.fix_hash_id()
    }

    pub fn hash_id(&self) -> &TestHash {
        self.block.hash_id()
    }

    fn into_hash_id(self) -> TestHash {
        self.block.into_hash_id()
    }
}

/// The origin of an error.
struct ErrorSpan {
    /// The file where the error occurs.
    file: Utf8PathBuf,
    /// The line No. in `file`. If `lineno` == 0, it means the error is global
    /// to `file`.
    lineno: usize,
    /// The column No. in `lineno`. If `col` == 0, it means the error is global
    /// to `file` and `lineno`.
    col: usize,
    /// The ending line No. in `file`. If `lineno_end` == 0, it means it equals
    /// `lineno`.
    lineno_end: usize,
}

impl ErrorSpan {
    fn new(file: Utf8PathBuf) -> Self {
        Self {
            file,
            lineno: 0,
            col: 0,
            lineno_end: 0,
        }
    }

    /// Convert to `self` to [`Error::Parse`].
    fn to_parse_error<S: Into<String>>(&self, reason: S) -> Error {
        if self.col == 0 && self.lineno_end == 0 {
            Error::Parse {
                span: format!("{}:{}", self.file, self.lineno),
                reason: reason.into(),
            }
        } else if self.col == 0 {
            Error::Parse {
                span: format!(
                    "{}:{}-{}",
                    self.file, self.lineno, self.lineno_end
                ),
                reason: reason.into(),
            }
        } else if self.lineno_end == 0 {
            Error::Parse {
                span: format!("{}:{}:{}", self.file, self.lineno, self.col),
                reason: reason.into(),
            }
        } else {
            Error::Parse {
                span: format!(
                    "{}:{}-{}:{}",
                    self.file, self.lineno, self.lineno_end, self.col
                ),
                reason: reason.into(),
            }
        }
    }

    /// Convert to [`Error::InvalidToken`].
    fn to_invalid_token_error(&self) -> Error {
        Error::InvalidToken {
            file: self.file.clone(),
            lineno: self.lineno,
        }
    }

    fn as_line_span(&mut self) -> &mut Self {
        self.lineno_end = 0;
        self.col = 0;
        self
    }

    fn as_line_span_full(&mut self, lineno: usize) -> &mut Self {
        self.lineno = lineno;
        self.lineno_end = 0;
        self.col = 0;
        self
    }

    fn as_col_span(&mut self, col: usize) -> &mut Self {
        self.lineno_end = 0;
        self.col = col;
        self
    }

    fn as_line_range_span(
        &mut self,
        lineno_begin: usize,
        lineno_end: usize,
    ) -> &mut Self {
        self.lineno = lineno_begin;
        self.lineno_end = lineno_end;
        self.col = 0;
        self
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("parsing error: {span}: {reason}")]
    Parse { span: String, reason: String },
    #[error("parsing error: {file}:{lineno}: invalid token")]
    InvalidToken { file: Utf8PathBuf, lineno: usize },
}

trait HandleInvalidToken: Sized {
    /// Return None if `self` is [`Error::InvalidToken`].
    fn handle_invalid_token<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self;
}

impl<T> HandleInvalidToken for Result<T, Error> {
    fn handle_invalid_token<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        match self {
            Err(Error::InvalidToken { .. }) => f(),
            t => t,
        }
    }
}

fn parse_test_case_block(
    span: &mut ErrorSpan,
    lineno: usize,
    lines: &mut std::io::Lines<BufReader<File>>,
    mut case: RawTestCaseBlock,
) -> Result<(TestCaseBlock, usize), Error> {
    let mut lineno_offset = 0;
    for line in lines.by_ref() {
        lineno_offset += 1;
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if line.starts_with("//") {
            continue;
        }

        Err(span
            .as_line_range_span(lineno, lineno + lineno_offset)
            .to_invalid_token_error())
        .handle_invalid_token(|| {
            case.export_type = Some(ExportType::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.editor_mode = Some(EditorMode::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.key_sequence = Some(KeySequence::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.operator = Some(Operator::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.register = Some(Register::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.count = Some(Count::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.state_before = Some(StateBefore::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.state_after = Some(StateAfter::parse(
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.buffer_before = Some(BufferBefore::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.buffer_pending = Some(BufferPending::parse(
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.buffer_output = Some(BufferOutput::parse(
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.buffer_after = Some(BufferAfter::parse(
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.model_output = Some(ModelOutput::parse(
                false,
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })
        .handle_invalid_token(|| {
            case.autocmd_events_count = Some(AutocmdEventsCount::parse(
                span.as_line_span_full(lineno + lineno_offset),
                line,
            )?);
            Ok(())
        })?;
    }
    span.as_line_range_span(lineno, lineno + lineno_offset);
    if case.export_type.is_none() {
        Err(span.to_parse_error("export type directive (X) not found"))?;
    }
    let case = match case.export_type.unwrap() {
        ExportType::Unit | ExportType::UnitIntegration => {
            let block = UnitTestCaseBlock {
                head_conditionals: case.head_conditionals.0,
                hash: case.hash,
                export_type: UnitExportType::from_export_type_opt(
                    case.export_type.unwrap(),
                )
                .unwrap(),
                editor_mode: UnitEditorMode::from_editor_mode_opt(
                    case.editor_mode.ok_or_else(|| {
                        span.to_parse_error(
                            "editor mode directive (M) not found",
                        )
                    })?,
                )
                .ok_or_else(|| {
                    span.to_parse_error(
                        "invalid editor mode (M) for unit test case",
                    )
                })?,
                key_sequence: case
                    .key_sequence
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "key sequence directive (K) not found",
                        )
                    })?
                    .into_motion_key()
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "key sequence must be motion in unit case block",
                        )
                    })?,
                operator: case.operator.map(|o| o.0),
                register: case.register.map(|r| r.0),
                count: case
                    .count
                    .ok_or_else(|| {
                        span.to_parse_error("count directive (C) not found")
                    })?
                    .0,
                state_before: case
                    .state_before
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "state before directive (S0) not found",
                        )
                    })?
                    .0,
                state_after: case
                    .state_after
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "state after directive (S1) not found",
                        )
                    })?
                    .0,
                buffer_before: case
                    .buffer_before
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "buffer before directive (B0) not found",
                        )
                    })?
                    .0,
                buffer_pending: case.buffer_pending.map(|b| b.0),
                buffer_output: case
                    .buffer_output
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "buffer output directive (Bo) not found",
                        )
                    })?
                    .0,
                buffer_after: case.buffer_after.map(|b| b.0),
                model_output: case
                    .model_output
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "model output directive (Q) not found",
                        )
                    })?
                    .0,
                autocmd_events_count: case
                    .autocmd_events_count
                    .unwrap_or_default()
                    .0,
            };
            // Some validation on `block`.
            match block.editor_mode {
                UnitEditorMode::Normal
                | UnitEditorMode::VisualChar
                | UnitEditorMode::VisualLine
                | UnitEditorMode::VisualBlock => {
                    if block.buffer_before.clean_buffer
                        != block.buffer_output.clean_buffer
                    {
                        return Err(span.to_parse_error(
                            "buffer before (B0) content \
                        does not equal to buffer output (Bo) content",
                        ));
                    }
                }
                UnitEditorMode::OperatorPending => {
                    if let Some(buffer_pending) = &block.buffer_pending
                        && block.buffer_before.clean_buffer
                            != buffer_pending.clean_buffer
                    {
                        return Err(span.to_parse_error(
                            "buffer pending (Bp) content \
                        does not equal to buffer before (B0) content",
                        ));
                    }
                }
            }
            if let Some(buffer_after) = &block.buffer_after
                && block.buffer_output.clean_buffer != buffer_after.clean_buffer
            {
                return Err(span.to_parse_error(
                    "buffer output (Bo) content \
                does not equal to buffer after (B1) content",
                ));
            }
            TestCaseBlock::Unit(block)
        }
        ExportType::Integration => {
            let block = CompositeTestCaseBlock {
                head_conditionals: case.head_conditionals.0,
                hash: case.hash,
                editor_mode: case.editor_mode.ok_or_else(|| {
                    span.to_parse_error("editor mode directive (M) not found")
                })?,
                key_sequence: case
                    .key_sequence
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "key sequence directive (K) not found",
                        )
                    })?
                    .into_normal_sequence(),
                state_before: case
                    .state_before
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "state before directive (S0) not found",
                        )
                    })?
                    .0,
                state_after: case
                    .state_after
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "state after directive (S1) not found",
                        )
                    })?
                    .0,
                buffer_before: case
                    .buffer_before
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "buffer before directive (B0) not found",
                        )
                    })?
                    .0,
                buffer_after: case.buffer_after.map(|b| b.0),
            };
            TestCaseBlock::Composite(block)
        }
        ExportType::Bootstrap => {
            let block = BootstrapTestCaseBlock {
                hash: case.hash,
                editor_mode: UnitEditorMode::from_editor_mode_opt(
                    case.editor_mode.ok_or_else(|| {
                        span.to_parse_error(
                            "editor mode directive (M) not found",
                        )
                    })?,
                )
                .ok_or_else(|| {
                    span.to_parse_error(
                        "invalid editor mode (M) for unit test case",
                    )
                })?,
                key_sequence: case
                    .key_sequence
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "key sequence directive (K) not found",
                        )
                    })?
                    .into_motion_key()
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "key sequence must be motion in unit case block",
                        )
                    })?,
                operator: case.operator.map(|o| o.0),
                register: case.register.map(|r| r.0),
                count: case
                    .count
                    .ok_or_else(|| {
                        span.to_parse_error("count directive (C) not found")
                    })?
                    .0,
                state_before: case
                    .state_before
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "state before directive (S0) not found",
                        )
                    })?
                    .0,
                state_after: case
                    .state_after
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "state after directive (S1) not found",
                        )
                    })?
                    .0,
                buffer_before: case
                    .buffer_before
                    .ok_or_else(|| {
                        span.to_parse_error(
                            "buffer before directive (B0) not found",
                        )
                    })?
                    .0,
                autocmd_events_count: case
                    .autocmd_events_count
                    .unwrap_or_default()
                    .0,
            };
            TestCaseBlock::Bootstrap(block)
        }
    };
    Ok((case, lineno_offset))
}

pub fn parse_metatest_file(file: &Utf8Path) -> Result<Vec<TestCase>, Error> {
    let mut span = ErrorSpan::new(file.to_path_buf());
    let mut conditionals = HeadConditionals(Vec::new());
    let mut defaults = Defaults::default();
    let mut test_cases = Vec::new();

    let reader = BufReader::new(File::open(file)?);
    let mut lines = reader.lines();
    let mut lineno = 0;
    while let Some(line) = lines.next() {
        lineno += 1;
        let line = line?;
        let line = line.trim();
        span.as_line_span_full(lineno);

        // Comments.
        if line.starts_with("//") {
            continue;
        }

        // Reset the defaults.
        if line.starts_with("##") {
            defaults = Defaults::default();
            continue;
        }

        Err(span.as_line_span().to_invalid_token_error())
            // Head conditionals.
            .handle_invalid_token(|| {
                conditionals.parse(&mut span, &test_cases, line)
            })
            // File version #V.
            .handle_invalid_token(|| {
                let Version(file_version) = Version::parse(&mut span, line)?;
                let ExpectedVersion(expected_version) =
                    ExpectedVersion::default();
                if file_version != expected_version {
                    Err(span.to_parse_error(format!(
                        "unexpected metatest file version: {}",
                        file_version
                    )))
                } else {
                    Ok(())
                }
            })
            // Defaults.
            .handle_invalid_token(|| {
                defaults.export_type =
                    Some(ExportType::parse(true, &mut span, line)?);
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.editor_mode =
                    Some(EditorMode::parse(true, &mut span, line)?);
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.key_sequence =
                    Some(KeySequence::parse(true, &mut span, line)?);
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.operator =
                    Some(Operator::parse(true, &mut span, line)?);
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.register = Register::parse(true, &mut span, line)?;
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.count = Count::parse(true, &mut span, line)?;
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.state_before =
                    Some(StateBefore::parse(true, &mut span, line)?);
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.buffer_before =
                    Some(BufferBefore::parse(true, &mut span, line)?);
                Ok(())
            })
            .handle_invalid_token(|| {
                defaults.model_output =
                    Some(ModelOutput::parse(true, &mut span, line)?);
                Ok(())
            })
            // Blank line.
            .handle_invalid_token(|| {
                if line.is_empty() {
                    Ok(())
                } else {
                    Err(span.as_line_span().to_invalid_token_error())
                }
            })
            // Test case block.
            .handle_invalid_token(|| {
                let hash = TestHash::parse(&mut span, line)?;
                let mut case =
                    RawTestCaseBlock::new(conditionals.clone(), hash);
                if let Some(export_type) = defaults.export_type {
                    case.export_type = Some(export_type);
                }
                if let Some(editor_mode) = defaults.editor_mode {
                    case.editor_mode = Some(editor_mode);
                }
                if let Some(key_sequence) = &defaults.key_sequence {
                    case.key_sequence = Some(key_sequence.clone());
                }
                if let Some(operator) = &defaults.operator {
                    case.operator = Some(operator.clone());
                }
                case.register = Some(defaults.register);
                case.count = Some(defaults.count);
                if let Some(state_before) = &defaults.state_before {
                    case.state_before = Some(state_before.clone());
                }
                if let Some(buffer_before) = &defaults.buffer_before {
                    case.buffer_before = Some(buffer_before.clone());
                }
                if let Some(model_output) = &defaults.model_output {
                    case.model_output = Some(model_output.clone());
                }

                if case.state_before.is_none() {
                    case.state_before = Some(StateBefore(Vec::new()));
                }
                if case.state_after.is_none() {
                    case.state_after = Some(StateAfter(Vec::new()));
                }
                let (case, lineno_offset) =
                    parse_test_case_block(&mut span, lineno, &mut lines, case)?;

                test_cases.push(TestCase {
                    block: case,
                    file: file.to_path_buf(),
                    lineno_begin: lineno,
                    lineno_end: lineno + lineno_offset,
                });
                lineno += lineno_offset;
                Ok(())
            })?;
    }

    Ok(test_cases)
}

/// Serialize the test case block to file.
pub mod unparsing {
    use std::io::{self, Write};

    use thiserror::Error;

    use super::{
        Ascii, AutocmdEventCount, AutocmdEventsCount, BufferAfter,
        BufferBefore, BufferExpr, BufferOutput, BufferPending, CURSWANT_MAX,
        Count, EditorMode, ExportType, HeadConditional, HeadConditionals,
        KeySequence, ModelOutput, ModelOutputItem, MotionKey, Operator,
        Position, PositionCurswant, RawTestCaseBlock, Register, StateAfter,
        StateBefore, StateExpr, StateExprFunction, TestHash,
    };

    #[derive(Debug, Error)]
    enum Error {
        #[error("io error: {0}")]
        Io(#[from] io::Error),
        #[error("unparse error: {0}")]
        Unparsing(String),
    }

    type UnparsingResult<T> = Result<T, Error>;

    trait ToJiebaTestCase {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()>;

        fn write<W: Write>(&self, writer: &mut W) -> UnparsingResult<()> {
            let mut sink = Vec::new();
            self.to_jieba_test_case(&mut sink)?;
            writer.write_all(&sink)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for str {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            let s = self
                .replace(r"\u0016", r"\<C-v>")
                .replace(" ", r"\<Space>")
                .replace("\t", r"\<Tab>")
                .replace("\n", r"\<NL>")
                .replace("\r", r"\<CR>")
                .replace("\u{0016}", r"\<C-v>");
            stream.extend(s.as_bytes());
            Ok(())
        }
    }

    impl ToJiebaTestCase for StateExpr {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            match self {
                StateExpr::Option { name, value } => {
                    write!(stream, " {}=", name)?;
                    value.to_jieba_test_case(stream)?;
                }
                StateExpr::Function(f) => match f {
                    StateExprFunction::Visualmode(value) => {
                        stream.extend(b" visualmode()=");
                        value.to_jieba_test_case(stream)?;
                    }
                },
                StateExpr::Mark { name, position: p } => {
                    write!(
                        stream,
                        " '{}=[{},{},{},{}]",
                        name, p[0], p[1], p[2], p[3]
                    )?;
                }
                StateExpr::Register { name, value } => {
                    write!(stream, " \"{}=", name)?;
                    value.to_jieba_test_case(stream)?;
                }
            }
            Ok(())
        }
    }

    struct LineCol {
        // A cursor position.
        lnum_col: (u64, u64),
        // The non-mark buffer expression character at this position.
        c: char,

        // Whether these position marks exist in this position.
        langle: bool,
        rangle: bool,
        visual_begin: bool,
        visual_end: bool,
        cursor: bool,
        curswant: bool,
    }

    impl LineCol {
        fn new(lnum: u64, col: u64, c: char) -> Self {
            Self {
                lnum_col: (lnum, col),
                c,
                langle: false,
                rangle: false,
                visual_begin: false,
                visual_end: false,
                cursor: false,
                curswant: false,
            }
        }
    }

    /// Return the index i at which `lc[i]`'s position equals `p`.
    fn find_position(lc: &[LineCol], p: Position) -> Option<usize> {
        let [_, lnum, col, off] = p;
        if off == 0 {
            // Plain old linear search is enough, as we don't need super speed,
            // and `lc`'s length is supposed to be small.
            for (i, e) in lc.iter().enumerate() {
                if e.lnum_col == (lnum, col) {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Return the indices (i, j) at which `lc[i]`'s position equals `p`, and
    /// `lc[j]`'s position equals `p`'s curswant. If `p`'s curswant equals col,
    /// return j as None.
    fn find_curposition(
        lc: &[LineCol],
        p: PositionCurswant,
    ) -> Option<(usize, Option<usize>)> {
        let [bufnum, lnum, col, off, curswant] = p;
        let i = find_position(lc, [bufnum, lnum, col, off])?;
        if curswant == col {
            return Some((i, None));
        }
        for (j, e) in lc.iter().enumerate() {
            if e.lnum_col == (lnum, curswant) {
                return Some((i, Some(j)));
            }
        }
        if curswant == CURSWANT_MAX {
            let eol = ['␊', '␀'];
            for (j, e) in lc.iter().enumerate() {
                if e.lnum_col.0 == lnum && eol.contains(&e.c) {
                    return Some((i, Some(j)));
                }
            }
        }

        None
    }

    impl ToJiebaTestCase for BufferExpr {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            if self.langle.is_some_and(|a| a[3] > 0)
                || self.rangle.is_some_and(|a| a[3] > 0)
                || self.visual_begin.is_some_and(|a| a[3] > 0)
                || self.visual_end.is_some_and(|a| a[3] > 0)
                || self.cursor.is_some_and(|a| a[3] > 0)
            {
                return Err(Error::Unparsing("off > 0 is unsupported".into()));
            }
            let mut lc = Vec::new();
            if self.clean_buffer.is_empty() {
                lc.push(LineCol::new(1, 1, '␀'));
            }
            let mut i = 1;
            for line in self.clean_buffer.iter() {
                let mut j = 1;
                for c in line.chars() {
                    let buffer_expr_c = if c == ' ' {
                        '·'
                    } else if c == '\t' {
                        '┤'
                    } else if let Some(Ascii(c)) = Ascii::from_char(c) {
                        c
                    } else {
                        return Err(Error::Unparsing(format!(
                            "unsupported buffer char: `{}`",
                            c
                        )));
                    };
                    lc.push(LineCol::new(i, j, buffer_expr_c));
                    j += 1;
                }
                lc.push(LineCol::new(i, j, '␊'));
                i += 1;
            }
            if let Some(p) = self.langle {
                let i = find_position(&lc, p).ok_or_else(|| {
                    Error::Unparsing(format!(
                        "unable to locate langle `{:?}`",
                        p
                    ))
                })?;
                lc[i].langle = true;
            }
            if let Some(p) = self.rangle {
                let i = find_position(&lc, p).ok_or_else(|| {
                    Error::Unparsing(format!(
                        "unable to locate rangle `{:?}`",
                        p
                    ))
                })?;
                lc[i].rangle = true;
            }
            if let Some(p) = self.visual_begin {
                let i = find_position(&lc, p).ok_or_else(|| {
                    Error::Unparsing(format!(
                        "unable to locate visual_begin `{:?}`",
                        p
                    ))
                })?;
                lc[i].visual_begin = true;
            }
            if let Some(p) = self.visual_end {
                let i = find_position(&lc, p).ok_or_else(|| {
                    Error::Unparsing(format!(
                        "unable to locate visual_end `{:?}`",
                        p
                    ))
                })?;
                lc[i].visual_end = true;
            }
            if let Some(p) = self.cursor {
                let (i, j) = find_curposition(&lc, p).ok_or_else(|| {
                    Error::Unparsing(format!(
                        "unable to locate cursor_curswant `{:?}`",
                        p
                    ))
                })?;
                lc[i].cursor = true;
                if let Some(j) = j {
                    lc[j].curswant = true;
                }
            }

            for e in lc {
                if e.langle {
                    stream.push(b'<');
                }
                if e.rangle {
                    stream.push(b'>');
                }
                if e.visual_begin {
                    stream.push(b'[');
                }
                if e.visual_end {
                    stream.push(b']');
                }
                if e.cursor {
                    stream.push(b'|');
                }
                if e.curswant {
                    stream.push(b'\\');
                }
                write!(stream, "{}", e.c)?;
            }

            Ok(())
        }
    }

    impl ToJiebaTestCase for HeadConditionals {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            for hc in self.0.iter() {
                match hc {
                    HeadConditional::Feature(f) => {
                        writeln!(stream, "?has:{}", f)?;
                    }
                    HeadConditional::NoFeature(f) => {
                        writeln!(stream, "?!has:{}", f)?;
                    }
                    HeadConditional::VimVersionAtLeast(v) => {
                        writeln!(stream, "?version:{}", v)?;
                    }
                }
            }
            Ok(())
        }
    }

    impl ToJiebaTestCase for TestHash {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            writeln!(stream, "H {}", self.id)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for ExportType {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            let export_type = match self {
                ExportType::Bootstrap => "b",
                ExportType::Unit => "u",
                ExportType::Integration => "i",
                ExportType::UnitIntegration => "ui",
            };
            writeln!(stream, "X {}", export_type)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for EditorMode {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            let editor_mode = match self {
                EditorMode::Normal => "n",
                EditorMode::VisualChar => "v",
                EditorMode::VisualLine => "V",
                EditorMode::VisualBlock => r"\<C-v>",
                EditorMode::OperatorPending => "o",
                EditorMode::MixedNormal => "mn",
                EditorMode::MixedVisualChar => "mv",
                EditorMode::MixedVisualLine => "mV",
                EditorMode::MixedVisualBlock => r"m\<C-v>",
            };
            writeln!(stream, "M {}", editor_mode)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for KeySequence {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            let key_sequence = match self {
                KeySequence::AnyNormal(ks) => ks.as_str(),
                KeySequence::Motion(mk) => match mk {
                    MotionKey::Ws => "w",
                    MotionKey::Wl => "W",
                    MotionKey::Bs => "b",
                    MotionKey::Bl => "B",
                    MotionKey::Es => "e",
                    MotionKey::El => "E",
                    MotionKey::Ges => "ge",
                    MotionKey::Gel => "gE",
                    MotionKey::Iws => "iw",
                    MotionKey::Iwl => "iW",
                    MotionKey::Aws => "aw",
                    MotionKey::Awl => "aW",
                },
            };
            writeln!(stream, "K {}", key_sequence)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for Operator {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            writeln!(stream, "O {}", self.0)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for Register {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            writeln!(stream, "R {}", self.0)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for Count {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            writeln!(stream, "C {}", self.0)?;
            Ok(())
        }
    }

    impl ToJiebaTestCase for StateBefore {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.extend(b"S0");
            for s in self.0.iter() {
                s.to_jieba_test_case(stream)?;
            }
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for StateAfter {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.extend(b"S1");
            for s in self.0.iter() {
                s.to_jieba_test_case(stream)?;
            }
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for BufferBefore {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.extend(b"B0 ");
            self.0.to_jieba_test_case(stream)?;
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for BufferAfter {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.extend(b"B1 ");
            self.0.to_jieba_test_case(stream)?;
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for BufferPending {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.extend(b"Bp ");
            self.0.to_jieba_test_case(stream)?;
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for BufferOutput {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.extend(b"Bo ");
            self.0.to_jieba_test_case(stream)?;
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for ModelOutputItem {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            match self {
                ModelOutputItem::Cursor => stream.extend(br" |"),
                ModelOutputItem::CursorCurswant => {
                    stream.extend(br" | \");
                }
                ModelOutputItem::Langle => stream.extend(b" <"),
                ModelOutputItem::Rangle => stream.extend(b" >"),
                ModelOutputItem::KeyValue { key, value } => {
                    write!(stream, " {}=", key).unwrap();
                    value.to_jieba_test_case(stream)?;
                }
            }
            Ok(())
        }
    }

    impl ToJiebaTestCase for ModelOutput {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.push(b'Q');
            for mo in self.0.iter() {
                mo.to_jieba_test_case(stream)?;
            }
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for AutocmdEventCount {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.push(b' ');
            stream.extend(self.event_name.as_bytes());
            stream.push(b'=');
            if let Some(count) = &self.count {
                write!(stream, "{}", count).unwrap();
            }
            Ok(())
        }
    }

    impl ToJiebaTestCase for AutocmdEventsCount {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.push(b'E');
            for ec in self.0.iter() {
                ec.to_jieba_test_case(stream)?;
            }
            stream.push(b'\n');
            Ok(())
        }
    }

    impl ToJiebaTestCase for RawTestCaseBlock {
        fn to_jieba_test_case(
            &self,
            stream: &mut Vec<u8>,
        ) -> UnparsingResult<()> {
            stream.push(b'\n');
            self.hash.to_jieba_test_case(stream)?;
            if let Some(export_type) = self.export_type {
                export_type.to_jieba_test_case(stream)?;
            }
            if let Some(editor_mode) = self.editor_mode {
                editor_mode.to_jieba_test_case(stream)?;
            }
            if let Some(key_sequence) = &self.key_sequence {
                key_sequence.to_jieba_test_case(stream)?;
            }
            if let Some(operator) = &self.operator {
                operator.to_jieba_test_case(stream)?;
            }
            if let Some(register) = &self.register {
                register.to_jieba_test_case(stream)?;
            }
            if let Some(count) = &self.count {
                count.to_jieba_test_case(stream)?;
            }
            if let Some(model_output) = &self.model_output {
                model_output.to_jieba_test_case(stream)?;
            }
            if let Some(buffer_before) = &self.buffer_before {
                buffer_before.to_jieba_test_case(stream)?;
            }
            if let Some(buffer_pending) = &self.buffer_pending {
                buffer_pending.to_jieba_test_case(stream)?;
            }
            if let Some(buffer_output) = &self.buffer_output {
                buffer_output.to_jieba_test_case(stream)?;
            }
            if let Some(buffer_after) = &self.buffer_after {
                buffer_after.to_jieba_test_case(stream)?;
            }
            if let Some(autocmd_events_count) = &self.autocmd_events_count {
                autocmd_events_count.to_jieba_test_case(stream)?;
            }
            Ok(())
        }
    }

    pub struct Serializer<W> {
        head_conditionals: Vec<HeadConditional>,
        writer: W,
    }

    impl<W: Write> Serializer<W> {
        /// Do some setup work.
        pub fn setup(
            mut writer: W,
            head_conditionals: Vec<HeadConditional>,
        ) -> anyhow::Result<Self> {
            writeln!(writer, "#V {}", include_str!("version"))?;
            HeadConditionals(head_conditionals.clone()).write(&mut writer)?;
            Ok(Self {
                head_conditionals,
                writer,
            })
        }

        pub fn write(
            &mut self,
            block: &RawTestCaseBlock,
        ) -> anyhow::Result<()> {
            if block.head_conditionals.0 != self.head_conditionals {
                return Err(anyhow::anyhow!(
                    "current head conditionals `{:?}` not matching the expected one: `{:?}`",
                    block.head_conditionals.0,
                    self.head_conditionals
                ));
            }
            block.write(&mut self.writer)?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::parsing::CURSWANT_MAX;

        use super::{BufferExpr, ToJiebaTestCase};

        fn s(b: Vec<u8>) -> String {
            String::from_utf8(b).unwrap()
        }

        #[test]
        fn test_buffer_expr_to_jieba_test_case() {
            let mut sink = Vec::new();
            BufferExpr {
                clean_buffer: vec![],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, CURSWANT_MAX]),
            }
            .to_jieba_test_case(&mut sink)
            .unwrap();
            assert_eq!(s(sink), r"|\␀");

            let mut sink = Vec::new();
            BufferExpr {
                clean_buffer: vec!["".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, CURSWANT_MAX]),
            }
            .to_jieba_test_case(&mut sink)
            .unwrap();
            assert_eq!(s(sink), r"|\␊");

            let mut sink = Vec::new();
            BufferExpr {
                clean_buffer: vec!["abc".into()],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 3, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 3, 0, 3]),
            }
            .to_jieba_test_case(&mut sink)
            .unwrap();
            assert_eq!(s(sink), "<ab>|c␊");

            let mut sink = Vec::new();
            BufferExpr {
                clean_buffer: vec!["abc".into()],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 3, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 3, 0, CURSWANT_MAX]),
            }
            .to_jieba_test_case(&mut sink)
            .unwrap();
            assert_eq!(s(sink), r"<ab>|c\␊");

            let mut sink = Vec::new();
            BufferExpr {
                clean_buffer: vec!["abc".into(), "de".into()],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 3, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 2, 1, 0, 2]),
            }
            .to_jieba_test_case(&mut sink)
            .unwrap();
            assert_eq!(s(sink), r"<ab>c␊|d\e␊");

            let mut sink = Vec::new();
            BufferExpr {
                clean_buffer: vec!["abc".into(), "de".into()],
                langle: None,
                rangle: None,
                visual_begin: Some([0, 1, 3, 0]),
                visual_end: Some([0, 1, 1, 0]),
                cursor: Some([0, 2, 1, 0, CURSWANT_MAX]),
            }
            .to_jieba_test_case(&mut sink)
            .unwrap();
            assert_eq!(s(sink), r"]ab[c␊|de\␊");
        }
    }
}

#[derive(Parser)]
pub struct Cli {
    /// The *.jieba_test_case files to parse.
    file: Vec<Utf8PathBuf>,
    /// Automatically fix errors. Be sure to backup or commit before using this
    /// option.
    #[arg(long, default_value_t = false)]
    fix: bool,
}

fn fix_new_hash<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    lineno_to_hash: &HashMap<usize, String>,
) -> io::Result<()> {
    for (lineno, line) in reader.lines().enumerate() {
        let lineno = lineno + 1;
        let mut line = line?;
        match lineno_to_hash.get(&lineno) {
            None => {
                writeln!(writer, "{}", line)?;
            }
            Some(new_hash) => {
                let hash_index = line[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .expect("hash id not found")
                    + 1;
                let hash_index_end = if line[hash_index..].starts_with('?') {
                    hash_index + 1
                } else {
                    hash_index + ID_LEN * 2
                };
                line.replace_range(hash_index..hash_index_end, new_hash);
                writeln!(writer, "{}", line)?;
            }
        }
    }
    Ok(())
}

fn copy_back<R: Read, W: Write>(
    reader: &mut R,
    mut writer: W,
) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    loop {
        let n_read = reader.read(&mut buf)?;
        if n_read == 0 {
            break;
        }
        writer.write_all(&buf[..n_read])?;
    }
    Ok(())
}

fn fix_dup<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    dup_lines: &mut Vec<usize>,
) -> io::Result<()> {
    dup_lines.sort();
    dup_lines.reverse();
    for (lineno, line) in reader.lines().enumerate() {
        let lineno = lineno + 1;
        let line = line?;
        match dup_lines.last() {
            None => writeln!(writer, "{}", line)?,
            Some(dup_lineno) => {
                if dup_lineno == &lineno {
                    dup_lines.pop();
                } else {
                    writeln!(writer, "{}", line)?;
                }
            }
        }
    }
    Ok(())
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let mut ids = HashMap::new();
        for path in self.file {
            let cases = parse_metatest_file(&path)?;
            println!("{}: found {} test cases", path, cases.len());
            let mut lineno_to_new_hash = HashMap::new();
            let mut dup_lines = Vec::new();
            for mut c in cases {
                let c_file = c.file.clone();
                let c_lineno_begin = c.lineno_begin;
                let c_lineno_end = c.lineno_end;
                let old_hash = c.fix_hash_id();
                let fixed_hash = c.into_hash_id();
                if old_hash != fixed_hash {
                    if !self.fix {
                        println!(
                            "{}:{}: new hash = {}",
                            fixed_hash.file, fixed_hash.lineno, fixed_hash.id
                        );
                    }
                    lineno_to_new_hash
                        .insert(fixed_hash.lineno, fixed_hash.id.to_string());
                }
                if let TestHashId::Sha2(bytes) = fixed_hash.id {
                    match ids.get(&bytes) {
                        None => {
                            ids.insert(
                                bytes,
                                (fixed_hash.file, fixed_hash.lineno),
                            );
                        }
                        Some((origin_file, origin_lineno)) => {
                            if !self.fix {
                                println!(
                                    "{}:{}-{}: duplicate with {}:{}",
                                    c_file,
                                    c_lineno_begin,
                                    c_lineno_end,
                                    origin_file,
                                    origin_lineno
                                );
                            }
                            dup_lines.extend(c_lineno_begin..=c_lineno_end);
                        }
                    }
                }
            }
            if self.fix {
                let tmpf = tempfile::tempfile()?;
                let mut writer = BufWriter::new(tmpf);
                let reader = BufReader::new(File::open(&path)?);
                fix_new_hash(reader, &mut writer, &lineno_to_new_hash)?;
                let mut tmpf = writer.into_inner()?;
                tmpf.rewind()?;

                let mut reader = BufReader::new(tmpf);
                let writer = BufWriter::new(File::create(&path)?);
                copy_back(&mut reader, writer)?;
                let mut tmpf = reader.into_inner();
                tmpf.rewind()?;
                tmpf.set_len(0)?;

                let mut writer = BufWriter::new(tmpf);
                let reader = BufReader::new(File::open(&path)?);
                fix_dup(reader, &mut writer, &mut dup_lines)?;
                let mut tmpf = writer.into_inner()?;
                tmpf.rewind()?;

                let mut reader = BufReader::new(tmpf);
                let writer = BufWriter::new(File::create(&path)?);
                copy_back(&mut reader, writer)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BufferExpr;

    #[test]
    fn test_parse_buffer_expr() {
        assert_eq!(
            BufferExpr::parse("␀"),
            Some(BufferExpr {
                clean_buffer: vec![],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: None
            })
        );
        assert_eq!(
            BufferExpr::parse("<|>␀"),
            Some(BufferExpr {
                clean_buffer: vec![],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 1, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, 1]),
            })
        );
        assert_eq!(
            BufferExpr::parse(r"<|>\␀"),
            Some(BufferExpr {
                clean_buffer: vec![],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 1, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, super::CURSWANT_MAX]),
            })
        );
        assert_eq!(
            BufferExpr::parse(r"<|>\␀~"),
            Some(BufferExpr {
                clean_buffer: vec![],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 1, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, 1]),
            })
        );
        assert_eq!(
            BufferExpr::parse(r"<|>␀\~"),
            Some(BufferExpr {
                clean_buffer: vec![],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 1, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, 2]),
            })
        );
        assert_eq!(
            BufferExpr::parse(r"<>␀|\~"),
            Some(BufferExpr {
                clean_buffer: vec![],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 1, 0]),
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 1, 2]),
            })
        );
        assert_eq!(
            BufferExpr::parse("␊"),
            Some(BufferExpr {
                clean_buffer: vec!["".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: None
            })
        );
        assert_eq!(
            BufferExpr::parse("|␊"),
            Some(BufferExpr {
                clean_buffer: vec!["".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, 1])
            })
        );
        assert_eq!(
            BufferExpr::parse(r"|\␊"),
            Some(BufferExpr {
                clean_buffer: vec!["".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, super::CURSWANT_MAX])
            })
        );
        assert_eq!(
            BufferExpr::parse(r"|\␊~"),
            Some(BufferExpr {
                clean_buffer: vec!["".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 1, 0, 1])
            })
        );
        assert_eq!(
            BufferExpr::parse("abc·|def␊"),
            Some(BufferExpr {
                clean_buffer: vec!["abc def".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 5, 0, 5])
            })
        );
        assert_eq!(
            BufferExpr::parse(r"<[ab]c·|def\>␊"),
            Some(BufferExpr {
                clean_buffer: vec!["abc def".into()],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 8, 0]),
                visual_begin: Some([0, 1, 1, 0]),
                visual_end: Some([0, 1, 3, 0]),
                cursor: Some([0, 1, 5, 0, super::CURSWANT_MAX])
            })
        );
        assert_eq!(
            BufferExpr::parse(r"<[ab]c·|de\f>␊"),
            Some(BufferExpr {
                clean_buffer: vec!["abc def".into()],
                langle: Some([0, 1, 1, 0]),
                rangle: Some([0, 1, 8, 0]),
                visual_begin: Some([0, 1, 1, 0]),
                visual_end: Some([0, 1, 3, 0]),
                cursor: Some([0, 1, 5, 0, 7])
            })
        );
        assert_eq!(
            BufferExpr::parse("aa␊|e␊cc␊"),
            Some(BufferExpr {
                clean_buffer: vec!["aa".into(), "e".into(), "cc".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 2, 1, 0, 1])
            })
        );
        assert_eq!(
            BufferExpr::parse(r"abc┤~~|~~def␊~~\~~~gh␊"),
            Some(BufferExpr {
                clean_buffer: vec!["abc\tdef".into(), "gh".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 4, 3, 15])
            })
        );
        assert_eq!(
            BufferExpr::parse(r"abc┤~~|~~def\␊~~~~~gh␊"),
            Some(BufferExpr {
                clean_buffer: vec!["abc\tdef".into(), "gh".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 4, 3, 12])
            })
        );
        assert_eq!(
            BufferExpr::parse(r"abc@@|d␊efgh␊"),
            Some(BufferExpr {
                clean_buffer: vec!["abcd".into(), "efgh".into()],
                langle: None,
                rangle: None,
                visual_begin: None,
                visual_end: None,
                cursor: Some([0, 1, 6, 0, 6])
            })
        );
        assert_eq!(BufferExpr::parse("␊␀"), None);
        assert_eq!(BufferExpr::parse("␀␊"), None);
        assert_eq!(BufferExpr::parse("abc␀"), None);
        assert_eq!(BufferExpr::parse("␀abc␊"), None);
        assert_eq!(BufferExpr::parse(r"~abc·def␊"), None);
        assert_eq!(BufferExpr::parse(r"|abc␊\def␊"), None);
        assert_eq!(BufferExpr::parse(r"|abc·def"), None);
        assert_eq!(BufferExpr::parse(r"\abc·def␊"), None);
        assert_eq!(BufferExpr::parse(r"|abc·|def␊"), None);
        assert_eq!(BufferExpr::parse(r"abc·\d\ef␊"), None);
        assert_eq!(BufferExpr::parse(r"<<abc·def␊"), None);
    }
}
