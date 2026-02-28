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

//! Transpiler of metatest towards unit test verification. Unit test
//! verification aims to test the vimscript side functions and verify the
//! correctness of unit tests.

use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::{Command, Stdio};

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::parsing::{
    self, Ascii, BufferExpr, HeadConditional, ModelOutputItem, StateExpr,
    StateExprFunction, TestCaseBlock, TestHashId, UnitEditorMode,
    UnitTestCaseBlock,
};

use super::vimscript_transpiler::{
    Concat, EchoJson, EmbeddedLua, Flush, Func, Identifier, IdentifierString,
    Map, MapItem, MarkStr, Negate, NotEqTest, OptionVar, TranspilingError,
    TranspilingResult, VarAssign, VimCommand, VimLt, VimVariable,
};
use super::{Error, ToVimscript};

impl ToVimscript for HeadConditional {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let i = 0;
        match self {
            Self::Feature(f) => {
                VimCommand::new("if", (Negate(Func::new("has", (f,))),))
                    .flush(i, stream)?;
            }
            Self::NoFeature(f) => {
                VimCommand::new("if", (Func::new("has", (f,)),))
                    .flush(i, stream)?;
            }
            Self::VimVersionAtLeast(v) => {
                VimCommand::new(
                    "if",
                    (VimLt(
                        VimVariable {
                            scope: Ascii::new(b'v').unwrap(),
                            identifier: Identifier::new("version").unwrap(),
                        },
                        v,
                    ),),
                )
                .flush(i, stream)?;
            }
        }
        {
            let i = i + 1;
            let m = Map((MapItem(
                IdentifierString::new("cf").unwrap(),
                "continue",
            ),));
            let e = EchoJson(m);
            VimCommand::new("if", (Func::new("has", ("nvim",)),))
                .flush(i, stream)?;
            {
                let i = i + 1;
                EmbeddedLua(&e).flush(i, stream)?;
            }
            VimCommand::new("else", ()).flush(i, stream)?;
            {
                let i = i + 1;
                e.flush(i, stream)?;
            }
            VimCommand::new("endif", ()).flush(i, stream)?;
            VimCommand::new("quit", ()).flush(i, stream)?;
            VimCommand::new("finish", ()).flush(i, stream)?;
        }
        VimCommand::new("endif", ()).flush(i, stream)
    }
}

pub struct OracleModel {
    buffer_pending: Option<BufferExpr>,
    buffer_output: BufferExpr,
    model_output: Vec<ModelOutputItem>,
}

impl OracleModel {
    /// Return None if the position marks configuration is invalid.
    fn from_test_case_block_opt(block: UnitTestCaseBlock) -> Option<Self> {
        for i in block.model_output.iter() {
            match i {
                ModelOutputItem::Cursor | ModelOutputItem::CursorCurswant => {
                    if block.buffer_output.cursor.is_none() {
                        return None;
                    }
                }
                ModelOutputItem::Langle => match &block.buffer_pending {
                    None => {
                        if block.buffer_output.langle.is_none() {
                            return None;
                        }
                    }
                    Some(buffer_pending) => {
                        if (block.buffer_output.langle.is_none()
                            && buffer_pending.langle.is_none())
                            || (block.buffer_output.langle.is_some()
                                && buffer_pending.langle.is_some())
                        {
                            return None;
                        }
                    }
                },
                ModelOutputItem::Rangle => match &block.buffer_pending {
                    None => {
                        if block.buffer_output.rangle.is_none() {
                            return None;
                        }
                    }
                    Some(buffer_pending) => {
                        if (block.buffer_output.rangle.is_none()
                            && buffer_pending.rangle.is_none())
                            || (block.buffer_output.rangle.is_some()
                                && buffer_pending.rangle.is_some())
                        {
                            return None;
                        }
                    }
                },
                ModelOutputItem::KeyValue { .. } => (),
            }
        }
        Some(Self {
            buffer_pending: block.buffer_pending,
            buffer_output: block.buffer_output,
            model_output: block.model_output,
        })
    }

    fn get_cursor(&self) -> Option<&[u64]> {
        self.buffer_output.cursor.as_ref().map(|p| &p[..4])
    }

    fn get_cursor_curswant(&self) -> Option<&[u64]> {
        self.buffer_output.cursor.as_ref().map(|p| &p[..])
    }

    fn get_langle(&self) -> Option<&[u64]> {
        match self.buffer_output.langle.as_ref() {
            Some(p) => Some(&p[..]),
            None => match &self.buffer_pending {
                Some(buffer_pending) => {
                    buffer_pending.langle.as_ref().map(|p| &p[..])
                }
                None => None,
            },
        }
    }

    fn get_rangle(&self) -> Option<&[u64]> {
        match self.buffer_output.rangle.as_ref() {
            Some(p) => Some(&p[..]),
            None => match &self.buffer_pending {
                Some(buffer_pending) => {
                    buffer_pending.rangle.as_ref().map(|p| &p[..])
                }
                None => None,
            },
        }
    }
}

impl ToVimscript for OracleModel {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"function! JiebaOracleModel(...)\n");
        stream.extend(b"    let g:model_input = a:000\n");
        stream.extend(b"    let g:model_output = {");

        // Write the dict.
        let mut write_dict =
            |cont: bool, t: &ModelOutputItem| -> TranspilingResult {
                if cont {
                    stream.extend(b", ");
                }
                match t {
                    ModelOutputItem::Cursor => {
                        stream.extend(b"\"cursor\": ");
                        self.get_cursor()
                            .ok_or_else(|| {
                                TranspilingError(
                                    "no cursor `|` found in buffers".into(),
                                )
                            })?
                            .to_vimscript(stream)?;
                    }
                    ModelOutputItem::CursorCurswant => {
                        stream.extend(b"\"cursor\": ");
                        self.get_cursor_curswant()
                            .ok_or_else(|| {
                                TranspilingError(
                                    "no cursor `|` found in buffers".into(),
                                )
                            })?
                            .to_vimscript(stream)?;
                    }
                    ModelOutputItem::Langle => {
                        stream.extend(b"\"langle\": ");
                        self.get_langle()
                            .ok_or_else(|| {
                                TranspilingError(
                                    "no langle `<` found in buffers".into(),
                                )
                            })?
                            .to_vimscript(stream)?;
                    }
                    ModelOutputItem::Rangle => {
                        stream.extend(b"\"rangle\": ");
                        self.get_rangle()
                            .ok_or_else(|| {
                                TranspilingError(
                                    "no rangle `>` found in buffers".into(),
                                )
                            })?
                            .to_vimscript(stream)?;
                    }
                    ModelOutputItem::KeyValue { key, value } => {
                        key.to_vimscript(stream)?;
                        stream.extend(b": ");
                        value.to_vimscript(stream)?;
                    }
                }
                Ok(())
            };

        if !self.model_output.is_empty() {
            let t1 = &self.model_output[0];
            write_dict(false, t1)?;
            for t in self.model_output.iter().skip(1) {
                write_dict(true, t)?;
            }
        }

        stream.extend(b"}\n    return g:model_output\nendfunction\n");
        Ok(())
    }
}

pub(super) struct StateExprBefore(pub(super) StateExpr);

impl ToVimscript for StateExprBefore {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        match &self.0 {
            StateExpr::Option { name, value } => {
                let t = VarAssign {
                    lhs: OptionVar(name.into()),
                    rhs: value,
                };
                t.to_vimscript(stream)?;
            }
            StateExpr::Function(f) => match f {
                StateExprFunction::Visualmode(v) => {
                    // execute "normal! " . {v} . "\<Esc>"
                    stream.extend(b"execute ");
                    Concat(("normal! ", v, r"\<Esc>")).to_vimscript(stream)?;
                    stream.extend(b"\n");
                }
            },
            StateExpr::Mark { name, position } => {
                stream.extend(b"call ");
                let f = Func::new("setpos", (MarkStr(*name), &position[..]));
                f.to_vimscript(stream)?;
                stream.extend(b"\n");
            }
            StateExpr::Register { name, value } => {
                let t = Func::new("setreg", (*name, value));
                t.to_vimscript(stream)?;
                stream.extend(b"\n");
            }
        }
        Ok(())
    }
}

struct StateExprAfter(StateExpr);

impl ToVimscript for StateExprAfter {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        match &self.0 {
            StateExpr::Option { name, value } => {
                let t = NotEqTest {
                    a: OptionVar(name.into()),
                    b: value.to_owned(),
                    msg: format!("unexpected state_after in option '{}'", name),
                };
                t.to_vimscript(stream)
            }
            StateExpr::Function(f) => match f {
                StateExprFunction::Visualmode(v) => {
                    let t = NotEqTest {
                        a: Func::new("visualmode", ()),
                        b: v.to_owned(),
                        msg: "unexpected state_after in function visualmode()"
                            .to_owned(),
                    };
                    t.to_vimscript(stream)
                }
            },
            StateExpr::Mark { name, position } => {
                let t = NotEqTest {
                    a: Func::new("getpos", (MarkStr(*name),)),
                    b: &position[..],
                    msg: format!("unexpected state_after in mark '{}", name),
                };
                t.to_vimscript(stream)
            }
            StateExpr::Register { name, value } => {
                let t = NotEqTest {
                    a: Func::new("getreg", (*name,)),
                    b: value.to_owned(),
                    msg: format!(
                        "unexpected state_after in register \"{}",
                        name
                    ),
                };
                t.to_vimscript(stream)
            }
        }
    }
}

struct BufferAfter(BufferExpr);

impl ToVimscript for BufferAfter {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        if let Some(p) = self.0.cursor {
            let t = NotEqTest {
                a: Func::new("getcurpos", ()),
                b: p,
                msg: "unexpected cursor position in buffer_after".into(),
            };
            t.to_vimscript(stream)?;
        }
        if let Some(p) = self.0.langle {
            let t = NotEqTest {
                a: Func::new("getpos", (MarkStr(Ascii::new(b'<').unwrap()),)),
                b: p,
                msg: "unexpected langle position in buffer_after".into(),
            };
            t.to_vimscript(stream)?;
        }
        if let Some(p) = self.0.rangle {
            let t = NotEqTest {
                a: Func::new("getpos", (MarkStr(Ascii::new(b'>').unwrap()),)),
                b: p,
                msg: "unexpected rangle position in buffer_after".into(),
            };
            t.to_vimscript(stream)?;
        }
        if self.0.visual_begin.is_some() || self.0.visual_end.is_some() {
            stream.extend(b"normal! gvomaomb\n");
            if let Some(p) = self.0.visual_begin {
                let t = NotEqTest {
                    a: Func::new(
                        "getpos",
                        (MarkStr(Ascii::new(b'a').unwrap()),),
                    ),
                    b: p,
                    msg: "unexpected visual_begin position in buffer_after"
                        .into(),
                };
                t.to_vimscript(stream)?;
            }
            if let Some(p) = self.0.visual_end {
                let t = NotEqTest {
                    a: Func::new(
                        "getpos",
                        (MarkStr(Ascii::new(b'b').unwrap()),),
                    ),
                    b: p,
                    msg: "unexpected visual_end position in buffer_after"
                        .into(),
                };
                t.to_vimscript(stream)?;
            }
        }
        Ok(())
    }
}

fn write_shared_setup(
    block: &UnitTestCaseBlock,
    stream: &mut Vec<u8>,
) -> TranspilingResult {
    // Write head conditionals.
    stream.extend(b"\" head conditionals\n");
    for hc in block.head_conditionals.iter() {
        hc.to_vimscript(stream)?;
    }
    stream.extend(b"\n");

    // Write oracle model.
    let om = OracleModel::from_test_case_block_opt(block.clone()).ok_or_else(
        || {
            TranspilingError(
                "invalid position marks config in buffers \
                for normal mode unit verification"
                    .into(),
            )
        },
    )?;
    om.to_vimscript(stream)?;
    stream.extend(b"\n\n");

    // Write state_before setup.
    stream.extend(b"\" state_before setup\n");
    let mut visualmode_before = None;
    for sx in block.state_before.iter() {
        match sx {
            StateExpr::Function(StateExprFunction::Visualmode(v)) => {
                visualmode_before = Some(v);
            }
            sx => {
                let sx = StateExprBefore(sx.clone());
                sx.to_vimscript(stream)?;
            }
        }
    }
    stream.extend(b"\n\n");

    // Write buffer_before setup.
    stream.extend(b"\" buffer_before setup\n");

    let mut write_visual_setup =
        |v: &str, p1: [u64; 4], p2: [u64; 4]| -> TranspilingResult {
            // call setpos(".", {p1})
            let t = Func::new("setpos", (".", p1));
            stream.extend(b"call ");
            t.to_vimscript(stream)?;
            stream.extend(b"\n");

            // execute "normal! {v}\<Esc>"
            stream.extend(b"execute ");
            let t = Concat(("normal! ", v, r"\<Esc>"));
            t.to_vimscript(stream)?;
            stream.extend(b"\n");

            // call setpos("'>", {p2})
            let t =
                Func::new("setpos", (MarkStr(Ascii::new(b'>').unwrap()), p2));
            stream.extend(b"call ");
            t.to_vimscript(stream)?;
            stream.extend(b"\n");

            Ok(())
        };

    match block.editor_mode {
        UnitEditorMode::Normal | UnitEditorMode::OperatorPending => {
            // visual_begin / visual_end
            match (
                visualmode_before,
                block.buffer_before.visual_begin,
                block.buffer_before.visual_end,
            ) {
                (Some(v), Some(p1), Some(p2)) => {
                    write_visual_setup(v, p1, p2)?;
                }
                (Some(v), None, None) if v.is_empty() => (),
                (None, None, None) => (),
                _ => {
                    return Err(TranspilingError(
                        "`S0 visualmode()={v}`, `B0 [` and `B0 ]` must coexist"
                            .into(),
                    ));
                }
            }

            // cursor
            match block.buffer_before.cursor {
                Some(p) => {
                    // call setpos(".", {p})
                    let t = Func::new("setpos", (".", p));
                    stream.extend(b"call ");
                    t.to_vimscript(stream)?;
                    stream.extend(b"\n");
                }
                None => {
                    return Err(TranspilingError("missing `B0 |`".into()));
                }
            }
        }
        UnitEditorMode::VisualChar
        | UnitEditorMode::VisualLine
        | UnitEditorMode::VisualBlock => {
            // visual_begin / visual_end
            if visualmode_before.is_some() {
                return Err(TranspilingError("`S0 visualmode()={v}` must not coexist with `M` visual modes".into()));
            }
            let v = match block.editor_mode {
                UnitEditorMode::VisualChar => "v",
                UnitEditorMode::VisualLine => "V",
                UnitEditorMode::VisualBlock => r"\<C-v>",
                _ => unreachable!(),
            };
            match (
                block.buffer_before.visual_begin,
                block.buffer_before.visual_end,
            ) {
                (Some(p1), Some(p2)) => {
                    write_visual_setup(v, p1, p2)?;
                }
                _ => {
                    return Err(TranspilingError(
                        "missing `B0 [` and `B0 ]`".into(),
                    ));
                }
            }

            // cursor
            if let Some(p) = block.buffer_before.cursor {
                // call setpos(".", {p})
                let t = Func::new("setpos", (".", p));
                stream.extend(b"call ");
                t.to_vimscript(stream)?;
                stream.extend(b"\n");
            }
        }
    }
    stream.extend(b"\n");

    Ok(())
}

fn write_shared_teardown(
    block: &UnitTestCaseBlock,
    is_custom_run: bool,
    stream: &mut Vec<u8>,
) -> TranspilingResult {
    stream.extend(b"execute \"normal! \\<Esc>\"\n\n");

    // Write state_after checks.
    stream.extend(b"\" state_after checks\n");
    for sx in block.state_after.iter() {
        let sx = StateExprAfter(sx.clone());
        sx.to_vimscript(stream)?;
    }
    stream.extend(b"\n\n");

    // Write buffer_after checks.
    if let Some(buffer_after) = &block.buffer_after {
        stream.extend(b"\" buffer_after checks\n");
        let ba = BufferAfter(buffer_after.clone());
        ba.to_vimscript(stream)?;
        stream.extend(b"\n\n");
    }

    // Write run json response. See [`VimRunResponse`] below.
    let editor_mode = match block.editor_mode {
        UnitEditorMode::Normal => "n",
        UnitEditorMode::VisualChar
        | UnitEditorMode::VisualLine
        | UnitEditorMode::VisualBlock => "x",
        UnitEditorMode::OperatorPending => "o",
    };
    let key_sequence = block.key_sequence.as_ref();
    if is_custom_run {
        let i = 0;
        let g_model_input = VimVariable {
            scope: Ascii::new(b'g').unwrap(),
            identifier: Identifier::new("model_input").unwrap(),
        };
        let g_model_output = VimVariable {
            scope: Ascii::new(b'g').unwrap(),
            identifier: Identifier::new("model_output").unwrap(),
        };
        let m = Map((
            MapItem(IdentifierString::new("e").unwrap(), editor_mode),
            MapItem(IdentifierString::new("k").unwrap(), key_sequence),
            MapItem(IdentifierString::new("i").unwrap(), g_model_input),
            MapItem(IdentifierString::new("o").unwrap(), g_model_output),
        ));
        let payload = Map((
            MapItem(IdentifierString::new("cf").unwrap(), ""),
            MapItem(IdentifierString::new("m").unwrap(), m),
        ));
        let e = EchoJson(payload);

        VimCommand::new("if", (Func::new("has", ("nvim",)),))
            .flush(i, stream)?;
        {
            let i = i + 1;
            EmbeddedLua(&e).flush(i, stream)?;
        }
        VimCommand::new("else", ()).flush(i, stream)?;
        {
            let i = i + 1;
            e.flush(i, stream)?;
        }
        VimCommand::new("endif", ()).flush(i, stream)?;
    } else {
        let i = 0;
        let payload = Map((MapItem(IdentifierString::new("cf").unwrap(), ""),));
        let e = EchoJson(payload);

        VimCommand::new("if", (Func::new("has", ("nvim",)),))
            .flush(i, stream)?;
        {
            let i = i + 1;
            EmbeddedLua(&e).flush(i, stream)?;
        }
        VimCommand::new("else", ()).flush(i, stream)?;
        {
            let i = i + 1;
            e.flush(i, stream)?;
        }
        VimCommand::new("endif", ()).flush(i, stream)?;
    }
    stream.push(b'\n');

    // Exit.
    stream.extend(b"silent xit\n");

    Ok(())
}

struct StdRun(UnitTestCaseBlock);

impl ToVimscript for StdRun {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        write_shared_setup(&self.0, stream)?;

        stream.extend(b"\" cursor movement\n");
        let count = if self.0.count == 0 {
            "".into()
        } else {
            self.0.count.to_string()
        };
        match self.0.editor_mode {
            UnitEditorMode::Normal => {
                write!(
                    stream,
                    "normal! {}{}\n",
                    count,
                    self.0.key_sequence.as_ref()
                )
                .unwrap();
            }
            UnitEditorMode::VisualChar
            | UnitEditorMode::VisualLine
            | UnitEditorMode::VisualBlock => {
                write!(
                    stream,
                    "normal! gv{}{}\n",
                    count,
                    self.0.key_sequence.as_ref()
                )
                .unwrap();
            }
            UnitEditorMode::OperatorPending => {
                let register = self.0.register.ok_or_else(|| {
                    TranspilingError("missing register `R`".into())
                })?;
                let operator = self.0.operator.as_ref().ok_or_else(|| {
                    TranspilingError("missing operator `O`".into())
                })?;
                write!(
                    stream,
                    "normal! \"{}{}{}{}\n",
                    register,
                    operator,
                    count,
                    self.0.key_sequence.as_ref()
                )
                .unwrap();
            }
        }
        stream.extend(b"\n");

        write_shared_teardown(&self.0, false, stream)?;
        Ok(())
    }
}

struct CustomRun(UnitTestCaseBlock);

impl ToVimscript for CustomRun {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        write_shared_setup(&self.0, stream)?;

        stream.extend(b"\" cursor movement\n");
        stream.extend(b"call ");
        match self.0.editor_mode {
            UnitEditorMode::Normal => {
                let t = Func::new(
                    "JiebaNmap",
                    (
                        self.0.key_sequence.as_ref(),
                        self.0.count,
                        "JiebaOracleModel",
                    ),
                );
                t.to_vimscript(stream)?;
            }
            UnitEditorMode::VisualChar
            | UnitEditorMode::VisualLine
            | UnitEditorMode::VisualBlock => {
                let t = Func::new(
                    "JiebaXmap",
                    (
                        self.0.key_sequence.as_ref(),
                        self.0.count,
                        "JiebaOracleModel",
                    ),
                );
                t.to_vimscript(stream)?;
            }
            UnitEditorMode::OperatorPending => {
                let register = self.0.register.ok_or_else(|| {
                    TranspilingError("missing register `R`".into())
                })?;
                let operator = self.0.operator.as_ref().ok_or_else(|| {
                    TranspilingError("missing operator `O`".into())
                })?;
                let t = Func::new(
                    "JiebaOmap",
                    (
                        self.0.key_sequence.as_ref(),
                        0u32,
                        self.0.count,
                        operator,
                        register,
                        "JiebaOracleModel",
                    ),
                );
                t.to_vimscript(stream)?;
            }
        }
        stream.extend(b"\n");

        write_shared_teardown(&self.0, true, stream)?;
        Ok(())
    }
}

pub(super) fn write_clean_buffer<W: Write>(
    mut writer: W,
    clean_buffer: &[String],
) -> io::Result<()> {
    for line in clean_buffer {
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

pub(super) fn read_as_clean_buffer<R: BufRead>(
    reader: R,
) -> io::Result<Vec<String>> {
    let mut clean_buffer = Vec::new();
    for line in reader.lines() {
        clean_buffer.push(line?);
    }
    Ok(clean_buffer)
}

pub(super) fn write_run_file<W: Write, V: ToVimscript>(
    mut writer: W,
    obj: &V,
) -> Result<(), Error> {
    let mut buf = Vec::new();
    obj.to_vimscript(&mut buf)?;
    writer.write_all(&buf)?;
    Ok(())
}

enum VimBin {
    Path(Utf8PathBuf),
    DryRun,
}

pub(super) fn pretty_print_clean_buffer(clean_buffer: &[String]) -> String {
    let mut sbuf = String::new();
    if clean_buffer.is_empty() {
        sbuf.push('␀');
        sbuf.push('\n');
    }
    for line in clean_buffer {
        sbuf.push_str(&line.replace(' ', "·").replace('\t', "┤"));
        sbuf.push('␊');
        sbuf.push('\n');
    }
    sbuf
}

#[derive(Deserialize)]
struct VimRunResponse {
    #[serde(rename = "cf")]
    control_flow: String,
    #[serde(rename = "m", default)]
    model: Option<ModelInputOutputDes>,
}

#[derive(Deserialize)]
struct ModelInputOutputDes {
    #[serde(rename = "e")]
    editor_mode: String,
    #[serde(rename = "i")]
    input: serde_json::Value,
    #[serde(rename = "o")]
    output: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct ModelInputOutputSer {
    pub id: String,
    #[serde(rename = "f")]
    pub fun_name: String,
    #[serde(rename = "b")]
    pub buffer: Vec<String>,
    #[serde(rename = "i")]
    pub input: serde_json::Value,
    #[serde(rename = "o")]
    pub output: serde_json::Value,
}

enum RunType {
    Std,
    Custom,
}

impl AsRef<str> for RunType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Std => "std-run",
            Self::Custom => "custom-run",
        }
    }
}

impl fmt::Display for RunType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

enum VimType {
    Vim,
    Neovim,
}

fn verify_in_vim(
    vim_bin: &VimBin,
    vimrc: Option<&Utf8Path>,
    run_file: &Utf8Path,
    buffer_file: &Utf8Path,
    hash_id: &TestHashId,
    clean_buffer_before: Vec<String>,
    expected_buffer_after: &[String],
    run_type: RunType,
    vim_type: &VimType,
) -> anyhow::Result<Option<ModelInputOutputSer>> {
    match vim_bin {
        VimBin::DryRun => {
            let mut args = vec!["vim"];
            match vim_type {
                VimType::Vim => args.push("-es"),
                VimType::Neovim => args.push("--headless"),
            };
            if let Some(vimrc) = vimrc {
                args.extend(["-u", vimrc.as_str()]);
            }
            args.extend(["-S", run_file.as_str(), buffer_file.as_str()]);
            println!("> {:?}", args);
            Ok(None)
        }
        VimBin::Path(vim_bin) => {
            let mut cmd = Command::new(vim_bin);
            match vim_type {
                VimType::Vim => cmd.arg("-es"),
                VimType::Neovim => cmd.arg("--headless"),
            };
            if let Some(vimrc) = vimrc {
                cmd.arg("-u").arg(vimrc);
            }
            cmd.arg("-S").arg(run_file).arg(buffer_file);
            let st = cmd
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .output()?;
            if !st.status.success() {
                return Err(anyhow::anyhow!(
                    "unit verification failed ({}): {}",
                    run_type,
                    hash_id
                ));
            }
            let msg: VimRunResponse =
                serde_json::from_slice(st.stdout.trim_ascii_end())
                    .unwrap_or_else(|_| {
                        panic!(
                            "failed to decode json from `{}`",
                            String::from_utf8_lossy(&st.stdout)
                        )
                    });
            // If one of the head conditionals does not satisfy.
            if msg.control_flow == "continue" {
                return Ok(None);
            }

            let reader = BufReader::new(File::open(buffer_file)?);
            let actual_buffer_after = read_as_clean_buffer(reader)?;
            if actual_buffer_after != expected_buffer_after {
                eprintln!(
                    "expected buffer_after:\n\n{}\nactual buffer_after:\n\n{}",
                    pretty_print_clean_buffer(expected_buffer_after),
                    pretty_print_clean_buffer(&actual_buffer_after)
                );
                return Err(anyhow::anyhow!(
                    "unit verification failed ({}): {}",
                    run_type,
                    hash_id
                ));
            }

            match run_type {
                RunType::Std => Ok(None),
                RunType::Custom => {
                    let model_io = msg
                        .model
                        .expect("expecting ModelInputOutput but found None");
                    let fun_name = format!("{}map", model_io.editor_mode);
                    Ok(Some(ModelInputOutputSer {
                        id: hash_id.to_string(),
                        fun_name,
                        buffer: clean_buffer_before,
                        input: model_io.input,
                        output: model_io.output,
                    }))
                }
            }
        }
    }
}

impl UnitTestCaseBlock {
    fn run_unit_verification(
        self,
        vimrc: Option<&Utf8Path>,
        work_dir: &Utf8Path,
        vim_bin: &VimBin,
        vim_type: &VimType,
    ) -> anyhow::Result<Option<ModelInputOutputSer>> {
        fs::create_dir(work_dir).ok();

        let buffer_file = work_dir.join("buffer");
        let writer = BufWriter::new(File::create(&buffer_file)?);
        write_clean_buffer(writer, &self.buffer_before.clean_buffer)?;
        let std_run_file = work_dir.join("std_run.vim");
        let writer = BufWriter::new(File::create(&std_run_file)?);
        let std_run = StdRun(self);
        write_run_file(writer, &std_run)?;
        verify_in_vim(
            vim_bin,
            vimrc,
            &std_run_file,
            &buffer_file,
            &std_run.0.hash.id,
            std_run.0.buffer_before.clean_buffer.clone(),
            &std_run.0.buffer_output.clean_buffer,
            RunType::Std,
            vim_type,
        )?;
        let self_ = std_run.0;

        let buffer_file = work_dir.join("buffer");
        let writer = BufWriter::new(File::create(&buffer_file)?);
        write_clean_buffer(writer, &self_.buffer_before.clean_buffer)?;
        let custom_run_file = work_dir.join("custom_run.vim");
        let writer = BufWriter::new(File::create(&custom_run_file)?);
        let custom_run = CustomRun(self_);
        write_run_file(writer, &custom_run)?;
        let model_io_opt = verify_in_vim(
            vim_bin,
            vimrc,
            &custom_run_file,
            &buffer_file,
            &custom_run.0.hash.id,
            custom_run.0.buffer_before.clean_buffer.clone(),
            &custom_run.0.buffer_output.clean_buffer,
            RunType::Custom,
            vim_type,
        )?;

        Ok(model_io_opt)
    }
}

/// Show progress by printing dots to stdout.
pub struct DotsProgress {
    dots: u32,
    n_dots_in_a_row: u32,
}

impl Default for DotsProgress {
    fn default() -> Self {
        Self {
            dots: 0,
            n_dots_in_a_row: 80,
        }
    }
}

impl DotsProgress {
    pub fn step(&mut self) {
        print!(".");
        std::io::stdout().flush().ok();
        self.dots += 1;
        if self.dots >= self.n_dots_in_a_row {
            self.dots = 0;
            println!();
        }
    }

    pub fn reset(&mut self) {
        if self.dots > 1 {
            println!();
        }
        self.dots = 0;
    }
}

impl Drop for DotsProgress {
    fn drop(&mut self) {
        self.reset();
    }
}

#[derive(Parser)]
pub struct Cli {
    /// The vimrc path for unit test verification. If using vim instance from
    /// docker container where vimrc has been baked in, this option may not be
    /// necessary.
    #[arg(long = "rc")]
    vimrc: Option<Utf8PathBuf>,
    /// The full path or PATH-searchable name of vim/nvim binary. Leave this
    /// unspecified to enable dry-run mode, in which the run script etc. can
    /// be inspected.
    #[arg(short = 'v')]
    vim_bin: Option<Utf8PathBuf>,
    /// The vim/nvim distribution name. Default to the last component of `-v`,
    /// or "vim" if `-v` is not provided. The caller needs to ensure that the
    /// name contains only characters that are safe to be used in a file base
    /// name.
    #[arg(short = 'n')]
    vim_dist_name: Option<String>,
    /// Specify this if `-v` points to neovim.
    #[arg(long = "neovim", default_value_t = false)]
    is_neovim: bool,
    /// The working directory under which to run unit test verifications.
    #[arg(short = 'd')]
    work_dir: Utf8PathBuf,
    /// Verify this case id only.
    #[arg(short)]
    case_id: Option<String>,
    /// The *.jieba_test_case files.
    test_case_file: Vec<Utf8PathBuf>,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let mut ids = HashSet::new();
        fs::create_dir(&self.work_dir).ok();
        let vim_dist_name = self
            .vim_dist_name
            .or(self
                .vim_bin
                .as_ref()
                .map(|p| p.file_name())
                .flatten()
                .map(Into::into))
            .unwrap_or("vim".into());
        let vim_bin = self.vim_bin.map(VimBin::Path).unwrap_or(VimBin::DryRun);
        let vim_type = if self.is_neovim {
            VimType::Neovim
        } else {
            VimType::Vim
        };
        let mut progress = DotsProgress::default();
        let unit_info_file =
            self.work_dir.join(format!("unit-{}.jsonl", vim_dist_name));
        let written_to_unit_info = {
            let mut writer = BufWriter::new(File::create(&unit_info_file)?);
            let mut written_anything_to_unit_info = false;
            for path in self.test_case_file {
                progress.reset();
                let cases = parsing::parse_metatest_file(&path)?;
                eprintln!("I: {}: found {} test cases", path, cases.len());
                for mut c in cases {
                    let old_hash = c.fix_hash_id();
                    let fixed_hash = c.hash_id();
                    if fixed_hash != &old_hash {
                        return Err(anyhow::anyhow!(
                            "parsing failed: {}:{}: new hash = {}",
                            fixed_hash.file,
                            fixed_hash.lineno,
                            fixed_hash.id
                        ));
                    }
                    match &fixed_hash.id {
                        TestHashId::Sha2(bytes) => {
                            if !ids.insert(*bytes) {
                                eprintln!(
                                    "W: dup detected: ignored test case {}:{}-{}",
                                    c.file, c.lineno_begin, c.lineno_end
                                );
                                continue;
                            }
                        }
                        TestHashId::Unspecified => unreachable!(),
                    }
                    let id_hex = fixed_hash.id.to_string();
                    if self.case_id.as_ref().is_some_and(|id| id != &id_hex) {
                        continue;
                    }
                    if let TestCaseBlock::Unit(unit_case) = c.block {
                        let case_work_dir = self.work_dir.join(&id_hex);
                        let unit_io = unit_case.run_unit_verification(
                            self.vimrc.as_ref().map(|p| p.as_path()),
                            &case_work_dir,
                            &vim_bin,
                            &vim_type,
                        )?;
                        if let Some(unit_io) = unit_io {
                            serde_json::to_writer(&mut writer, &unit_io)?;
                            writer.write_all(b"\n")?;
                            written_anything_to_unit_info = true;
                        }
                        if let VimBin::Path(_) = &vim_bin {
                            progress.step();
                        }
                    }
                }
            }
            written_anything_to_unit_info
        };
        if !written_to_unit_info {
            fs::remove_file(unit_info_file)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::{
        Ascii, BufferExpr, HeadConditional, ModelOutputItem, StateExpr,
        StateExprFunction,
    };
    use crate::vimscript_transpiler::ToVimscript;

    use super::{OracleModel, StateExprBefore};

    #[test]
    fn test_head_conditional_to_vimscript() {
        let hc = HeadConditional::Feature("nvim".into());
        let mut sink = Vec::new();
        hc.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#"if !(has("nvim"))
    if has("nvim")
        lua <<EOF
io.write(vim.fn.json_encode({cf = "continue"}) .. "\n")
EOF

    else 
        execute "!echo " . shellescape(escape(json_encode({"cf": "continue"}), "\\"), 1) . ""
    endif 
    quit 
    finish 
endif 
"#
        );
    }

    #[test]
    fn test_oracle_model_to_vimscript() {
        let om = OracleModel {
            buffer_pending: None,
            buffer_output: BufferExpr::parse(r"<abc·|>def␊").unwrap(),
            model_output: vec![
                ModelOutputItem::CursorCurswant,
                ModelOutputItem::Langle,
                ModelOutputItem::Rangle,
                ModelOutputItem::KeyValue {
                    key: "foo".into(),
                    value: "bar".into(),
                },
            ],
        };
        let mut sink = Vec::new();
        om.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#"function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = {"cursor": [0, 1, 5, 0, 5], "langle": [0, 1, 1, 0], "rangle": [0, 1, 5, 0], "foo": "bar"}
    return g:model_output
endfunction
"#
        );
    }

    #[test]
    fn test_state_expr_before_to_vimscript() {
        let s = StateExprBefore(StateExpr::Function(
            StateExprFunction::Visualmode(r"\<C-v>".into()),
        ));
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#"execute "normal! " . "\<C-v>" . "\<Esc>"
"#
        );

        let s = StateExprBefore(StateExpr::Mark {
            name: Ascii::new(b'a').unwrap(),
            position: [0, 1, 5, 0],
        });
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#"call setpos("'a", [0, 1, 5, 0])
"#
        );

        let s = StateExprBefore(StateExpr::Register {
            name: Ascii::new(b'a').unwrap(),
            value: r"foo\<Space>bar".into(),
        });
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#"setreg("a", "foo\<Space>bar")
"#
        );

        let s = StateExprBefore(StateExpr::Option {
            name: "isf".into(),
            value: r"a,\<Space>b".into(),
        });
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#"let &isf = "a,\<Space>b"
"#
        );
    }
}
