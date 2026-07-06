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

//! This file contains the Golden Master tests for [`jieba_vim_rs_core`]. For
//! details on how to use this harness, see the CI pipeline under .github/
//! directory.

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use bstr::io::BufReadExt;
use clap::Parser;
use flate2::bufread::GzDecoder;
use jieba_vim_rs_core::motion::WordMotion;
use jieba_vim_rs_core::token::Tokenizer;
use libtest_mimic::{Arguments, Failed, Trial};
use serde::Deserialize;
use serde_json::{Map, Value};

mod keyword_cutter;
use keyword_cutter::KeywordCutter;

/// The Golden Master test harness for `jieba_vim_rs_core`.
#[derive(Debug, Parser)]
struct Cli {
    /// Be quiet.
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
    /// Run test cases that contain this string only.
    #[arg(short, long)]
    case: Option<String>,
    /// The jsonl files containing model inputs/outputs origined from last unit
    /// verification. If the files are named ending with ".gz", they will be
    /// decompressed automatically. Will also read all jsonl or jsonl.gz files
    /// under directory pointed to by env variable GOLDEN_MASTER_DIR.
    test_info_jsonl: Vec<PathBuf>,
}

fn main() {
    let mut cli = Cli::parse();
    let mut trials = Vec::new();
    if let Ok(dir) = std::env::var("GOLDEN_MASTER_DIR") {
        let dir_path = PathBuf::from(dir);
        for entry in fs::read_dir(&dir_path).unwrap_or_else(|err| {
            panic!(
                "failed to read dir `{}` due to: {}",
                dir_path.display(),
                err
            )
        }) {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".jsonl")
                        || file_name.ends_with(".jsonl.gz")
                    {
                        cli.test_info_jsonl.push(dir_path.join(file_name));
                    }
                }
            }
        }
    }
    for path in cli.test_info_jsonl {
        match File::open(&path) {
            Err(err) => eprintln!(
                "can't open file `{}` due to: {}",
                path.display(),
                err
            ),
            Ok(file) => {
                let reader = BufReader::new(file);
                if path.extension().is_some_and(|ext| ext == "gz") {
                    let reader = BufReader::new(GzDecoder::new(reader));
                    push_trials_from_jsonlines(&mut trials, reader);
                } else {
                    push_trials_from_jsonlines(&mut trials, reader);
                }
            }
        }
    }
    let mut args = Arguments::default();
    args.quiet = cli.quiet;
    args.filter = cli.case;
    libtest_mimic::run(&args, trials).exit();
}

fn push_trials_from_jsonlines<R: BufRead>(
    trials: &mut Vec<Trial>,
    mut reader: R,
) {
    reader
        .for_byte_line(|line| {
            let record = serde_json::from_slice::<RecordDict>(line)
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to decode `{}` due to: {}",
                        String::from_utf8_lossy(line),
                        err
                    )
                });
            trials
                .push(Trial::test(record.id.to_string(), || run_test(record)));
            Ok(true)
        })
        .unwrap_or_else(|err| panic!("io error: {}", err));
}

/// Represent a line in the input jsonl data files.
#[derive(Debug, Deserialize)]
struct RecordDict {
    id: String,
    span: String,
    #[serde(rename = "f")]
    func_name: String,
    #[serde(rename = "b")]
    buffer: Vec<String>,
    #[serde(rename = "i")]
    inputs: Vec<Value>,
    #[serde(rename = "o")]
    outputs: Map<String, Value>,
}

/// Run a single test from json `value` and return the error, if failed.
fn run_test(dict: RecordDict) -> Result<(), Failed> {
    let mut wm = WordMotion::new(
        Tokenizer::try_new(KeywordCutter::new([]), "@,48-57,_,192-255")
            .unwrap(),
    );
    match dict.func_name.as_str() {
        "nmap" => {
            let motion = get_input(&dict.inputs, 0);
            let cursor = get_input(&dict.inputs, 1);
            let count = get_input(&dict.inputs, 2);
            match wm.nmap(&dict.buffer, motion, cursor, count) {
                Err(_) => {
                    Err(format!("{}: failed to access buffer", dict.span)
                        .into())
                }
                Ok(outputs) => {
                    assert_on_field(
                        &dict.outputs,
                        "cursor",
                        &outputs.cursor,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "prevent_change",
                        &outputs.prevent_change,
                        &dict.span,
                    )
                }
            }
        }
        "xmap" => {
            let visualmode = get_input(&dict.inputs, 0);
            let motion = get_input(&dict.inputs, 1);
            let visual_begin = get_input(&dict.inputs, 2);
            let visual_end = get_input(&dict.inputs, 3);
            let count = get_input(&dict.inputs, 4);
            match wm.xmap(
                &dict.buffer,
                visualmode,
                motion,
                visual_begin,
                visual_end,
                count,
            ) {
                Err(_) => {
                    Err(format!("{}: failed to access buffer", dict.span)
                        .into())
                }
                Ok(outputs) => {
                    assert_on_field(
                        &dict.outputs,
                        "langle",
                        &outputs.langle,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "rangle",
                        &outputs.rangle,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "visualmode",
                        &outputs.visualmode,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "prevent_change",
                        &outputs.prevent_change,
                        &dict.span,
                    )
                }
            }
        }
        "omap" => {
            let motion = get_input(&dict.inputs, 0);
            let cursor = get_input(&dict.inputs, 1);
            let count = get_input(&dict.inputs, 2);
            let operator = get_input(&dict.inputs, 3);
            match wm.omap(&dict.buffer, motion, cursor, count, operator) {
                Err(_) => {
                    Err(format!("{}: failed to access buffer", dict.span)
                        .into())
                }
                Ok(outputs) => {
                    assert_on_field(
                        &dict.outputs,
                        "cursor",
                        &outputs.cursor,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "langle",
                        &outputs.langle,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "rangle",
                        &outputs.rangle,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "visualmode",
                        &outputs.visualmode,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "selection",
                        &outputs.selection,
                        &dict.span,
                    )?;
                    assert_on_field(
                        &dict.outputs,
                        "prevent_change",
                        &outputs.prevent_change,
                        &dict.span,
                    )
                }
            }
        }
        "imap" => {
            let motion: &[u8] = get_input(&dict.inputs, 0);
            let cursor = get_input(&dict.inputs, 1);
            match motion {
                // \<C-w>
                b"\x17" | b"\\u0017" => match wm
                    .imap_ctrl_w(&dict.buffer, cursor)
                {
                    Err(_) => {
                        Err(format!("{}: failed to access buffer", dict.span)
                            .into())
                    }
                    Ok(outputs) => assert_on_field(
                        &dict.outputs,
                        "cursor",
                        &outputs.cursor,
                        &dict.span,
                    ),
                },
                _ => panic!("invalid motion: {:?}", motion),
            }
        }
        f => panic!("unexpected func_name: {}", f),
    }
}

trait GetFromInputs<'a>: Sized {
    fn get_from_inputs(value: &'a [Value], index: usize) -> Self;
}

impl<'a> GetFromInputs<'a> for [usize; 5] {
    fn get_from_inputs(value: &'a [Value], index: usize) -> Self {
        let value = value
            .get(index)
            .unwrap_or_else(|| panic!("expecting i[{}] to exist", index))
            .as_array()
            .unwrap_or_else(|| panic!("expecting i[{}] to be array", index))
            .into_iter()
            .map(|p| {
                p.as_u64().unwrap_or_else(|| {
                    panic!("expecting i[{}] to be array of uint", index)
                }) as usize
            })
            .collect::<Vec<_>>();

        [value[0], value[1], value[2], value[3], value[4]]
    }
}

impl<'a> GetFromInputs<'a> for [usize; 4] {
    fn get_from_inputs(value: &'a [Value], index: usize) -> Self {
        let value = value
            .get(index)
            .unwrap_or_else(|| panic!("expecting i[{}] to exist", index))
            .as_array()
            .unwrap_or_else(|| panic!("expecting i[{}] to be array", index))
            .into_iter()
            .map(|p| {
                p.as_u64().unwrap_or_else(|| {
                    panic!("expecting i[{}] to be array of uint", index)
                }) as usize
            })
            .collect::<Vec<_>>();

        [value[0], value[1], value[2], value[3]]
    }
}

impl<'a> GetFromInputs<'a> for &'a str {
    fn get_from_inputs(value: &'a [Value], index: usize) -> Self {
        value
            .get(index)
            .unwrap_or_else(|| panic!("expecting i[{}] to exist", index))
            .as_str()
            .unwrap_or_else(|| panic!("expecting i[{}] to be str", index))
    }
}

impl<'a> GetFromInputs<'a> for &'a [u8] {
    fn get_from_inputs(value: &'a [Value], index: usize) -> Self {
        value
            .get(index)
            .unwrap_or_else(|| panic!("expecting i[{}] to exist", index))
            .as_str()
            .unwrap_or_else(|| panic!("expecting i[{}] to be str", index))
            .as_bytes()
    }
}

impl<'a> GetFromInputs<'a> for u64 {
    fn get_from_inputs(value: &'a [Value], index: usize) -> Self {
        value
            .get(index)
            .unwrap_or_else(|| panic!("expecting i[{}] to exist", index))
            .as_u64()
            .unwrap_or_else(|| panic!("expecting i[{}] to be u64", index))
    }
}

fn get_input<'a, T: GetFromInputs<'a>>(value: &'a [Value], index: usize) -> T {
    T::get_from_inputs(value, index)
}

trait GetFromOutputs<'a>: Sized {
    fn get_from_outputs(
        value: &'a Map<String, Value>,
        key: &str,
    ) -> Option<Self>;
}

impl<'a> GetFromOutputs<'a> for [usize; 5] {
    fn get_from_outputs(
        value: &'a Map<String, Value>,
        key: &str,
    ) -> Option<Self> {
        let value = value
            .get(key)?
            .as_array()
            .unwrap_or_else(|| panic!("expecting o['{}'] to be array", key))
            .into_iter()
            .map(|x| {
                x.as_u64().unwrap_or_else(|| {
                    panic!("expecting o['{}'] to be array of uint", key)
                }) as usize
            })
            .collect::<Vec<_>>();
        Some([
            *value.get(0).unwrap_or_else(|| {
                panic!("expecting o['{}'][0] to exist", key)
            }),
            *value.get(1).unwrap_or_else(|| {
                panic!("expecting o['{}'][1] to exist", key)
            }),
            *value.get(2).unwrap_or_else(|| {
                panic!("expecting o['{}'][2] to exist", key)
            }),
            *value.get(3).unwrap_or_else(|| {
                panic!("expecting o['{}'][3] to exist", key)
            }),
            *value.get(4).unwrap_or_else(|| {
                panic!("expecting o['{}'][4] to exist", key)
            }),
        ])
    }
}

impl<'a> GetFromOutputs<'a> for [usize; 4] {
    fn get_from_outputs(
        value: &'a Map<String, Value>,
        key: &str,
    ) -> Option<Self> {
        let value = value
            .get(key)?
            .as_array()
            .unwrap_or_else(|| panic!("expecting o['{}'] to be array", key))
            .into_iter()
            .map(|x| {
                x.as_u64().unwrap_or_else(|| {
                    panic!("expecting o['{}'] to be array of uint", key)
                }) as usize
            })
            .collect::<Vec<_>>();
        Some([
            *value.get(0).unwrap_or_else(|| {
                panic!("expecting o['{}'][0] to exist", key)
            }),
            *value.get(1).unwrap_or_else(|| {
                panic!("expecting o['{}'][1] to exist", key)
            }),
            *value.get(2).unwrap_or_else(|| {
                panic!("expecting o['{}'][2] to exist", key)
            }),
            *value.get(3).unwrap_or_else(|| {
                panic!("expecting o['{}'][3] to exist", key)
            }),
        ])
    }
}

impl<'a> GetFromOutputs<'a> for &'a str {
    fn get_from_outputs(
        value: &'a Map<String, Value>,
        key: &str,
    ) -> Option<Self> {
        Some(
            value
                .get(key)?
                .as_str()
                .unwrap_or_else(|| panic!("expecting o['{}'] to be str", key)),
        )
    }
}

impl<'a> GetFromOutputs<'a> for &'a [u8] {
    fn get_from_outputs(
        value: &'a Map<String, Value>,
        key: &str,
    ) -> Option<Self> {
        Some(
            value
                .get(key)?
                .as_str()
                .unwrap_or_else(|| panic!("expecting o['{}'] to be str", key))
                .as_bytes(),
        )
    }
}

trait GetDebugInfo {
    fn get_debug_info(&self) -> String;
}

impl GetDebugInfo for &[u8] {
    fn get_debug_info(&self) -> String {
        format!(
            "`{:?}` (decoded as `{}`)",
            self,
            String::from_utf8_lossy(self)
        )
    }
}

impl<const N: usize> GetDebugInfo for [usize; N] {
    fn get_debug_info(&self) -> String {
        format!("{:?}", self)
    }
}

impl GetDebugInfo for u64 {
    fn get_debug_info(&self) -> String {
        self.to_string()
    }
}

impl GetDebugInfo for str {
    fn get_debug_info(&self) -> String {
        format!("{:?}", self)
    }
}

fn assert_on_field<'a, T: PartialEq + GetFromOutputs<'a> + GetDebugInfo>(
    outputs: &'a Map<String, Value>,
    key: &str,
    actual: &T,
    span: &str,
) -> Result<(), Failed> {
    match T::get_from_outputs(outputs, key) {
        None => Ok(()),
        Some(expected) => {
            if actual == &expected {
                Ok(())
            } else {
                Err(format!(
                    "{}: actual ({}) != expected ({}) on `{}`",
                    span,
                    actual.get_debug_info(),
                    expected.get_debug_info(),
                    key
                )
                .into())
            }
        }
    }
}
