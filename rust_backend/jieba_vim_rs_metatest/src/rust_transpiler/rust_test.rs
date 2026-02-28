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

//! Transpiler of metatest unit verification outputs, the unit test info jsonl
//! files, towards Rust tests.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use camino::Utf8PathBuf;
use clap::Parser;

use crate::vimscript_transpiler::unit_verification::ModelInputOutputSer;

use super::{Error, ToRust};

/// Json subset.
enum Value {
    Uint(u64),
    Str(String),
    Pos([usize; 4]),
    Curpos([usize; 5]),
}

fn wrap_i64_to_u64(x: i64) -> u64 {
    (x as u64).wrapping_add(i64::MIN as u64)
}

fn number_as_u64(x: serde_json::Number) -> Option<u64> {
    x.as_u64().or(x.as_i64().map(wrap_i64_to_u64))
}

#[cfg(target_pointer_width = "64")]
fn wrap_i64_to_usize(x: i64) -> usize {
    wrap_i64_to_u64(x) as usize
}

#[cfg(not(target_pointer_width = "64"))]
fn wrap_i64_to_usize(x: i64) -> usize {
    compile_error!("currently only support 64-bit architecture");
}

#[cfg(target_pointer_width = "64")]
fn u64_as_usize(x: u64) -> usize {
    x as usize
}

#[cfg(not(target_pointer_width = "64"))]
fn u64_as_usize(x: u64) -> usize {
    compile_error!("currently only support 64-bit architecture");
}

fn value_as_usize(x: &serde_json::Value) -> Option<usize> {
    x.as_u64()
        .map(u64_as_usize)
        .or(x.as_i64().map(wrap_i64_to_usize))
}

impl Value {
    fn from_json_opt(value: serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(s) => Some(Self::Str(s)),
            serde_json::Value::Number(x) => number_as_u64(x).map(Self::Uint),
            serde_json::Value::Array(arr) => match arr.len() {
                4 => Some(Self::Pos([
                    value_as_usize(&arr[0])?,
                    value_as_usize(&arr[1])?,
                    value_as_usize(&arr[2])?,
                    value_as_usize(&arr[3])?,
                ])),
                5 => Some(Self::Curpos([
                    value_as_usize(&arr[0])?,
                    value_as_usize(&arr[1])?,
                    value_as_usize(&arr[2])?,
                    value_as_usize(&arr[3])?,
                    value_as_usize(&arr[4])?,
                ])),
                _ => None,
            },
            serde_json::Value::Bool(_)
            | serde_json::Value::Null
            | serde_json::Value::Object(_) => None,
        }
    }
}

struct VecValue(Vec<Value>);

impl VecValue {
    fn from_json_opt(value: serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::Array(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for a in arr {
                    out.push(Value::from_json_opt(a)?);
                }
                Some(Self(out))
            }
            _ => None,
        }
    }
}

struct MapValue(Vec<(String, Value)>);

impl MapValue {
    fn from_json_opt(value: serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::Object(map) => {
                let mut out = Vec::with_capacity(map.len());
                for (k, v) in map {
                    out.push((k, Value::from_json_opt(v)?))
                }
                Some(Self(out))
            }
            _ => None,
        }
    }
}

impl ToRust for u8 {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        write!(stream, "{}u8", self)?;
        Ok(())
    }
}

// Write as &'static [u8].
impl ToRust for [u8] {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        stream.write_all(b"&[")?;
        for x in self {
            x.to_rust(stream)?;
            stream.write_all(b",")?;
        }
        stream.write_all(b"][..]")?;
        Ok(())
    }
}

impl ToRust for usize {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        write!(stream, "{}usize", self)?;
        Ok(())
    }
}

impl ToRust for u64 {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        write!(stream, "{}u64", self)?;
        Ok(())
    }
}

impl ToRust for [usize; 4] {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        stream.write_all(b"[")?;
        self[0].to_rust(stream)?;
        stream.write_all(b",")?;
        self[1].to_rust(stream)?;
        stream.write_all(b",")?;
        self[2].to_rust(stream)?;
        stream.write_all(b",")?;
        self[3].to_rust(stream)?;
        stream.write_all(b"]")?;
        Ok(())
    }
}

impl ToRust for [usize; 5] {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        stream.write_all(b"[")?;
        self[0].to_rust(stream)?;
        stream.write_all(b",")?;
        self[1].to_rust(stream)?;
        stream.write_all(b",")?;
        self[2].to_rust(stream)?;
        stream.write_all(b",")?;
        self[3].to_rust(stream)?;
        stream.write_all(b",")?;
        self[4].to_rust(stream)?;
        stream.write_all(b"]")?;
        Ok(())
    }
}

// Write as &'static str.
impl ToRust for String {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        stream.write_all(b"std::str::from_utf8(")?;
        self.as_bytes().to_rust(stream)?;
        stream.write_all(b").unwrap()")?;
        Ok(())
    }
}

// Write as &[&'static str].
impl ToRust for [String] {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        stream.write_all(b"&[")?;
        for s in self {
            s.to_rust(stream)?;
            stream.write_all(b",")?;
        }
        stream.write_all(b"][..]")?;
        Ok(())
    }
}

impl ToRust for Value {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        match self {
            Self::Uint(x) => x.to_rust(stream),
            Self::Str(s) => s.as_bytes().to_rust(stream),
            Self::Pos(p) => p.to_rust(stream),
            Self::Curpos(p) => p.to_rust(stream),
        }
    }
}

// Write as comma-separated list.
impl ToRust for VecValue {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        for v in self.0.iter() {
            v.to_rust(stream)?;
            stream.write_all(b",")?;
        }
        Ok(())
    }
}

// Write as #[test].
impl ToRust for ModelInputOutputSer {
    fn to_rust<W: Write>(&self, stream: &mut W) -> Result<(), Error> {
        stream.write_all(b"\n#[test]\nfn test_")?;
        stream.write_all(self.id.as_bytes())?;
        stream.write_all(b"() {\n")?;
        stream.write_all(b"#[allow(unused_mut)]\n")?;
        stream.write_all(
            b"let mut wm = WordMotion::new(\
            Tokenizer::try_new(\
            KeywordCutter::new([]), \"@,48-57,_,192-255\").unwrap());\n",
        )?;

        write!(stream, "let output = wm.{}(", self.fun_name)?;
        self.buffer.to_rust(stream)?;
        stream.write_all(b",")?;
        VecValue::from_json_opt(self.input.clone())
            .ok_or_else(|| {
                Error::Transpile(format!(
                    "cannot coerce input `{}` of id {} to rust type",
                    self.input, self.id
                ))
            })?
            .to_rust(stream)?;
        stream.write_all(b").unwrap();\n")?;

        let output =
            MapValue::from_json_opt(self.output.clone()).ok_or_else(|| {
                Error::Transpile(format!(
                    "cannot coerce output `{}` of id {} to rust type",
                    self.output, self.id
                ))
            })?;
        for (k, v) in output.0 {
            write!(stream, "assert_eq!(output.{}, ", k)?;
            v.to_rust(stream)?;
            stream.write_all(b");\n")?;
        }

        stream.write_all(b"}\n")?;

        Ok(())
    }
}

fn write_imports<W: Write>(writer: &mut W) -> io::Result<()> {
    writer.write_all(
        b"// This file is auto-generated by jieba_vim_rs_metatest binary.\n\n",
    )?;
    writer.write_all(b"mod keyword_cutter;\n\n")?;
    writer.write_all(b"use jieba_vim_rs_core::motion::WordMotion;\n")?;
    writer.write_all(b"use jieba_vim_rs_core::token::Tokenizer;\n\n")?;
    writer.write_all(b"use keyword_cutter::KeywordCutter;\n")
}

#[derive(Parser)]
pub struct Cli {
    /// The working directory under which unit test verifications were run.
    #[arg(short = 'd')]
    work_dir: Utf8PathBuf,
    /// An existing directory under which to write rust tests.
    #[arg(short = 't')]
    test_dir: Utf8PathBuf,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        for file in fs::read_dir(&self.work_dir)? {
            let file =
                Utf8PathBuf::from_path_buf(file?.path()).map_err(|p| {
                    anyhow::anyhow!(
                        "cannot encode path `{}` in utf-8",
                        p.display()
                    )
                })?;
            if let Some(stem) = file.file_stem()
                && stem.starts_with("unit-")
                && file.extension() == Some("jsonl")
            {
                let test_file =
                    self.test_dir.join(format!("{}.rs", &stem[5..]));
                let reader = BufReader::new(File::open(file)?);
                let mut writer = BufWriter::new(File::create(test_file)?);
                write_imports(&mut writer)?;
                for line in reader.lines() {
                    let unit: ModelInputOutputSer =
                        serde_json::from_str(&line?)?;
                    unit.to_rust(&mut writer)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::wrap_i64_to_u64;

    #[test]
    fn test_i64_to_u64() {
        assert_eq!(wrap_i64_to_u64(-9223372036854775808), 0);
        assert_eq!(wrap_i64_to_u64(0), 9223372036854775808);
        assert_eq!(wrap_i64_to_u64(i64::MAX), u64::MAX);
    }
}
