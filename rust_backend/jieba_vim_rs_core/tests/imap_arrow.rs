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

use jieba_vim_rs_core::motion::WordMotion;
use jieba_vim_rs_core::token::Tokenizer;

mod keyword_cutter;
use keyword_cutter::KeywordCutter;

const CTRL_RIGHT: &[u8] = b"\x80\xfdV";
const CTRL_LEFT: &[u8] = b"\x80\xfdU";
const SHIFT_RIGHT: &[u8] = b"\x80%i";
const SHIFT_LEFT: &[u8] = b"\x80#4";

#[test]
fn test_imap_ctrl_right() {
    let mut wm = WordMotion::new(
        Tokenizer::try_new(KeywordCutter::new([]), "@,48-57,_,192-255")
            .unwrap(),
    );
    let buffer = vec!["f,o".into()];

    let output = wm.imap(&buffer, CTRL_RIGHT, [0, 1, 1, 0, 1]).unwrap();
    assert_eq!(output.cursor, [0, 1, 4, 0]);

    let output = wm.imap(&buffer, CTRL_RIGHT, [0, 1, 4, 0, 4]).unwrap();
    assert_eq!(output.cursor, [0, 1, 4, 0]);

    let buffer = vec!["foo".into(), "bar".into()];

    let output = wm.imap(&buffer, CTRL_RIGHT, [0, 1, 1, 0, 1]).unwrap();
    assert_eq!(output.cursor, [0, 2, 1, 0]);

    let output = wm.imap(&buffer, CTRL_RIGHT, [0, 1, 3, 0, 3]).unwrap();
    assert_eq!(output.cursor, [0, 2, 1, 0]);

    let output = wm.imap(&buffer, CTRL_RIGHT, [0, 1, 4, 0, 4]).unwrap();
    assert_eq!(output.cursor, [0, 2, 1, 0]);

    let output = wm.imap(&buffer, CTRL_RIGHT, [0, 2, 1, 0, 1]).unwrap();
    assert_eq!(output.cursor, [0, 2, 4, 0]);
}

#[test]
fn test_imap_shift_right() {
    let mut wm = WordMotion::new(
        Tokenizer::try_new(KeywordCutter::new([]), "@,48-57,_,192-255")
            .unwrap(),
    );
    let buffer = vec!["f,o".into()];

    let output = wm.imap(&buffer, SHIFT_RIGHT, [0, 1, 1, 0, 1]).unwrap();
    assert_eq!(output.cursor, [0, 1, 2, 0]);

    let buffer = vec!["foo".into()];

    let output = wm.imap(&buffer, SHIFT_RIGHT, [0, 1, 1, 0, 1]).unwrap();
    assert_eq!(output.cursor, [0, 1, 4, 0]);

    let output = wm.imap(&buffer, SHIFT_RIGHT, [0, 1, 4, 0, 4]).unwrap();
    assert_eq!(output.cursor, [0, 1, 4, 0]);

    let buffer = vec!["foo".into(), "bar".into()];

    let output = wm.imap(&buffer, SHIFT_RIGHT, [0, 1, 1, 0, 1]).unwrap();
    assert_eq!(output.cursor, [0, 2, 1, 0]);

    let output = wm.imap(&buffer, SHIFT_RIGHT, [0, 1, 3, 0, 3]).unwrap();
    assert_eq!(output.cursor, [0, 2, 1, 0]);

    let output = wm.imap(&buffer, SHIFT_RIGHT, [0, 1, 4, 0, 4]).unwrap();
    assert_eq!(output.cursor, [0, 2, 1, 0]);

    let output = wm.imap(&buffer, SHIFT_RIGHT, [0, 2, 1, 0, 1]).unwrap();
    assert_eq!(output.cursor, [0, 2, 4, 0]);
}

#[test]
fn test_imap_ctrl_left() {
    let mut wm = WordMotion::new(
        Tokenizer::try_new(KeywordCutter::new([]), "@,48-57,_,192-255")
            .unwrap(),
    );
    let buffer = vec!["f,o".into()];

    let output = wm.imap(&buffer, CTRL_LEFT, [0, 1, 4, 0, 4]).unwrap();
    assert_eq!(output.cursor, [0, 1, 1, 0]);

    let output = wm.imap(&buffer, CTRL_LEFT, [0, 1, 3, 0, 3]).unwrap();
    assert_eq!(output.cursor, [0, 1, 1, 0]);
}

#[test]
fn test_imap_shift_left() {
    let mut wm = WordMotion::new(
        Tokenizer::try_new(KeywordCutter::new([]), "@,48-57,_,192-255")
            .unwrap(),
    );
    let buffer = vec!["f,o".into()];

    let output = wm.imap(&buffer, SHIFT_LEFT, [0, 1, 4, 0, 4]).unwrap();
    assert_eq!(output.cursor, [0, 1, 3, 0]);

    let output = wm.imap(&buffer, SHIFT_LEFT, [0, 1, 3, 0, 3]).unwrap();
    assert_eq!(output.cursor, [0, 1, 2, 0]);

    let buffer = vec!["foo".into()];

    let output = wm.imap(&buffer, SHIFT_LEFT, [0, 1, 4, 0, 4]).unwrap();
    assert_eq!(output.cursor, [0, 1, 1, 0]);

    let output = wm.imap(&buffer, SHIFT_LEFT, [0, 1, 3, 0, 3]).unwrap();
    assert_eq!(output.cursor, [0, 1, 1, 0]);
}
