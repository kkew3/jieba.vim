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

//! It's very difficult to know the correct output of the jieba model given an
//! input a priori, and must be worked out manually, thus the heavy workload
//! to write tests with high coverage. However, finding whether the jieba
//! model produces a wrong output given any input is very easy. Hence, we may
//! bootstrap the model's accuracy by repeatedly running the model and make
//! correct its wrong behavior.

use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::process::{Command, Stdio};

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use serde::Deserialize;

use crate::parsing::unparsing::Serializer;
use crate::parsing::{
    self, Ascii, BootstrapTestCaseBlock, HeadConditional, Position,
    PositionCurswant, RawTestCaseBlock, StateExpr, StateExprFunction,
    TestCaseBlock, TestHashId, UnitEditorMode, UnitTestCaseBlock,
};
use crate::vimscript_transpiler::unit_verification::{
    self, DotsProgress, StateExprBefore,
};
use crate::vimscript_transpiler::vimscript_transpiler::Flush;

use super::ToVimscript;
use super::vimscript_transpiler::{
    Concat, EchoJson, EmbeddedLua, Func, Identifier, IdentifierString, Map,
    MapItem, MarkStr, NotEqTest, OptionVar, TranspilingError,
    TranspilingResult, VimCommand, VimVariable,
};

struct OracleModel(UnitEditorMode);

impl ToVimscript for OracleModel {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        write!(
            stream,
            r#"
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("{}"), a:000)
    return g:model_output
endfunction

"#,
            match self.0 {
                UnitEditorMode::Normal => "JiebaModelNmap",
                UnitEditorMode::VisualChar
                | UnitEditorMode::VisualLine
                | UnitEditorMode::VisualBlock => "JiebaModelXmap",
                UnitEditorMode::OperatorPending => "JiebaModelOmap",
            }
        )
        .unwrap();
        Ok(())
    }
}

fn write_shared_setup(
    block: &BootstrapTestCaseBlock,
    stream: &mut Vec<u8>,
) -> TranspilingResult {
    // Write oracle model.
    let om = OracleModel(block.editor_mode);
    om.to_vimscript(stream)?;
    stream.extend(b"\n\n");

    // Below are copied from unit_verification's `write_shared_setup`.

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

struct StdRun(BootstrapTestCaseBlock);

impl ToVimscript for StdRun {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        write_shared_setup(&self.0, stream)?;

        // Cursor movement.
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
        stream.extend(b"execute \"normal! \\<Esc>\"\n\n");

        // State after quering.
        stream.extend(b"\" State after quering\n");
        for s in self.0.state_after.iter() {
            match s {
                StateExpr::Option { name, .. } => {
                    writeln!(
                        stream,
                        "let g:JiebaTestGroundtruthOption_{0} = &{0}",
                        name
                    )
                    .unwrap();
                }
                StateExpr::Function(f) => match f {
                    StateExprFunction::Visualmode(_) => {
                        stream.extend(b"let g:JiebaTestGroundtruthFunc_visualmode = visualmode()\n");
                    }
                },
                StateExpr::Mark { name, .. } => {
                    let v = match name.into() {
                        b'<' => "JiebaTestGroundtruthMark_langle".into(),
                        b'>' => "JiebaTestGroundtruthMark_rangle".into(),
                        b'a'..=b'z' => {
                            format!("JiebaTestGroundtruthMark_{}", name)
                        }
                        _ => {
                            return Err(TranspilingError(format!(
                                "failed to serialize mark `{}`",
                                name
                            )));
                        }
                    };
                    writeln!(
                        stream,
                        r#"let g:{} = json_encode(getpos("'{}"))"#,
                        v, name
                    )
                    .unwrap();
                }
                StateExpr::Register { name, .. } => {
                    let v = match name.into() {
                        b'"' => "JiebaTestGroundtruthReg_default".into(),
                        b'a'..=b'z' => {
                            format!("JiebaTestGroundtruthReg_{}", name)
                        }
                        _ => {
                            return Err(TranspilingError(format!(
                                "failed to serialize register `{}`",
                                name
                            )));
                        }
                    };
                    writeln!(stream, r#"let g:{} = getreg('{}')"#, v, name)
                        .unwrap();
                }
            }
        }
        stream.push(b'\n');

        // Buffer after querying and echoing.
        stream.extend(b"\" Buffer after quering\n");
        match self.0.editor_mode {
            UnitEditorMode::Normal | UnitEditorMode::OperatorPending => {
                stream.extend(b"let g:JiebaTestGroundtruthCursor = json_encode(getcurpos())\n");
                let i = 0;
                let e = EchoJson(Map((MapItem(
                    IdentifierString::new("cursor").unwrap(),
                    Func::new("getcurpos", ()),
                ),)));
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
            UnitEditorMode::VisualChar
            | UnitEditorMode::VisualLine
            | UnitEditorMode::VisualBlock => {
                stream.extend(
                    br#"normal! gvomaomb
let g:JiebaTestGroundtruthVisualBegin = json_encode(getpos("'a"))
let g:JiebaTestGroundtruthVisualEnd = json_encode(getpos("'b"))
"#,
                );
                let i = 0;
                let e = EchoJson(Map((
                    MapItem(
                        IdentifierString::new("visual_begin").unwrap(),
                        Func::new(
                            "getpos",
                            (MarkStr(Ascii::new(b'a').unwrap()),),
                        ),
                    ),
                    MapItem(
                        IdentifierString::new("visual_end").unwrap(),
                        Func::new(
                            "getpos",
                            (MarkStr(Ascii::new(b'b').unwrap()),),
                        ),
                    ),
                )));
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
        }

        // Make session and exit.
        stream.extend(
            br#"
execute "mksession! " . expand("%:p:h") . "/Session.vim"
silent xit
"#,
        );

        Ok(())
    }
}

struct CustomRun(BootstrapTestCaseBlock);

impl ToVimscript for CustomRun {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        // Load session.
        stream.extend(
            br#"silent execute "source " . expand("%:p:h") . "/Session.vim"

"#,
        );

        // Setup.
        write_shared_setup(&self.0, stream)?;

        // Cursor movement (copied primarily from unit_verification's
        // `CustomRun::to_vimscript`).
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
        stream.extend(b"execute \"normal! \\<Esc>\"\n\n");

        let g = Ascii::new(b'g').unwrap();

        // State after checking.
        stream.extend(b"\" State after checking\n");
        for s in self.0.state_after.iter() {
            match s {
                StateExpr::Option { name, .. } => {
                    NotEqTest {
                        a: OptionVar(name.into()),
                        b: VimVariable {
                            scope: g,
                            identifier: Identifier::new(format!(
                                "JiebaTestGroundtruthOption_{}",
                                name
                            ))
                            .expect("not an identifier"),
                        },
                        msg: format!(
                            "unexpected state_after in option '{}'",
                            name
                        ),
                    }
                    .to_vimscript(stream)?;
                }
                StateExpr::Function(f) => match f {
                    StateExprFunction::Visualmode(_) => {
                        let v = "JiebaTestGroundtruthFunc_visualmode";
                        let msg =
                            "unexpected state_after in function visualmode()";
                        NotEqTest {
                            a: Func::new("visualmode", ()),
                            b: VimVariable {
                                scope: g,
                                identifier: Identifier::new(v).unwrap(),
                            },
                            msg: msg.into(),
                        }
                        .to_vimscript(stream)?;
                    }
                },
                StateExpr::Mark { name, .. } => {
                    let v = match name.into() {
                        b'<' => "JiebaTestGroundtruthMark_langle".into(),
                        b'>' => "JiebaTestGroundtruthMark_rangle".into(),
                        b'a'..=b'z' => {
                            format!("JiebaTestGroundtruthMark_{}", name)
                        }
                        _ => {
                            return Err(TranspilingError(format!(
                                "failed to serialize mark `{}`",
                                name
                            )));
                        }
                    };
                    NotEqTest {
                        a: Func::new("getpos", (MarkStr(*name),)),
                        b: Func::new(
                            "json_decode",
                            (VimVariable {
                                scope: g,
                                identifier: Identifier::new(v)
                                    .expect("not an identifier"),
                            },),
                        ),
                        msg: format!(
                            "unexpected state_after in mark '{}",
                            name
                        ),
                    }
                    .to_vimscript(stream)?;
                }
                StateExpr::Register { name, .. } => {
                    let v = match name.into() {
                        b'"' => "JiebaTestGroundtruthReg_default".into(),
                        b'a'..=b'z' => {
                            format!("JiebaTestGroundtruthReg_{}", name)
                        }
                        _ => {
                            return Err(TranspilingError(format!(
                                "failed to serialize register `{}`",
                                name
                            )));
                        }
                    };
                    NotEqTest {
                        a: Func::new("getreg", (name,)),
                        b: VimVariable {
                            scope: g,
                            identifier: Identifier::new(v)
                                .expect("not an identifier"),
                        },
                        msg: format!(
                            "unexpected state_after in register \"{}",
                            name
                        ),
                    }
                    .to_vimscript(stream)?;
                }
            }
        }
        stream.push(b'\n');

        // Buffer after checking.
        stream.extend(b"\" Buffer after checking\n");
        match self.0.editor_mode {
            UnitEditorMode::Normal | UnitEditorMode::OperatorPending => {
                NotEqTest {
                    a: Func::new("getcurpos", ()),
                    b: Func::new(
                        "json_decode",
                        (VimVariable {
                            scope: g,
                            identifier: Identifier::new(
                                "JiebaTestGroundtruthCursor",
                            )
                            .unwrap(),
                        },),
                    ),
                    msg: "unexpected cursor position in buffer_after".into(),
                }
                .to_vimscript(stream)?;
            }
            UnitEditorMode::VisualChar
            | UnitEditorMode::VisualLine
            | UnitEditorMode::VisualBlock => {
                stream.extend(b"normal! gvomaomb\n");
                let mark_begin = Ascii::new(b'a').unwrap();
                let mark_end = Ascii::new(b'b').unwrap();
                let v_begin = "JiebaTestGroundtruthVisualBegin";
                let v_end = "JiebaTestGroundtruthVisualEnd";
                NotEqTest {
                    a: Func::new("getpos", (MarkStr(mark_begin),)),
                    b: Func::new(
                        "json_decode",
                        (VimVariable {
                            scope: g,
                            identifier: Identifier::new(v_begin).unwrap(),
                        },),
                    ),
                    msg: "unexpected visual_begin position in buffer_after"
                        .into(),
                }
                .to_vimscript(stream)?;
                NotEqTest {
                    a: Func::new("getpos", (MarkStr(mark_end),)),
                    b: Func::new(
                        "json_decode",
                        (VimVariable {
                            scope: g,
                            identifier: Identifier::new(v_end).unwrap(),
                        },),
                    ),
                    msg: "unexpected visual_end position in buffer_after"
                        .into(),
                }
                .to_vimscript(stream)?;
            }
        }

        // Model output echoing.
        {
            let i = 0;
            let e = EchoJson(VimVariable {
                scope: Ascii::new(b'g').unwrap(),
                identifier: Identifier::new("model_output").unwrap(),
            });
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
        stream.extend(b"\nsilent xit\n");

        Ok(())
    }
}

#[derive(Deserialize)]
struct StdState {
    #[serde(default)]
    visual_begin: Option<Position>,
    #[serde(default)]
    visual_end: Option<Position>,
    #[serde(default)]
    cursor: Option<PositionCurswant>,
}

#[derive(Deserialize)]
struct ModelOutput {
    #[serde(default)]
    langle: Option<Position>,
    #[serde(default)]
    rangle: Option<Position>,
    #[serde(default)]
    cursor: Option<PositionCurswant>,
    #[serde(default)]
    visualmode: Option<String>,
    #[serde(default)]
    prevent_change: Option<String>,
}

enum VimRunResponse {
    Std(StdState),
    Model(ModelOutput),
}

enum VimBin {
    Path(Utf8PathBuf),
    DryRun,
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
    expected_buffer_after: Option<&[String]>,
    run_type: RunType,
    vim_type: &VimType,
) -> anyhow::Result<Option<VimRunResponse>> {
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
                    "bootstrap verification failed ({}): {}",
                    run_type,
                    hash_id
                ));
            }

            let reader = BufReader::new(File::open(buffer_file)?);
            let actual_buffer_after =
                unit_verification::read_as_clean_buffer(reader)?;
            if let Some(expected_buffer_after) = expected_buffer_after
                && actual_buffer_after != expected_buffer_after
            {
                eprintln!(
                    "expected buffer_after:\n\n{}\nactual buffer_after:\n\n{}",
                    unit_verification::pretty_print_clean_buffer(
                        expected_buffer_after
                    ),
                    unit_verification::pretty_print_clean_buffer(
                        &actual_buffer_after
                    )
                );
                return Err(anyhow::anyhow!(
                    "bootstrap verification failed ({}): {}",
                    run_type,
                    hash_id
                ));
            }

            let resp = match run_type {
                RunType::Std => {
                    let msg: StdState =
                        serde_json::from_slice(st.stdout.trim_ascii_end())
                            .unwrap_or_else(|_| {
                                panic!(
                                    "failed to decode json from: `{}`",
                                    String::from_utf8_lossy(&st.stdout)
                                )
                            });
                    VimRunResponse::Std(msg)
                }
                RunType::Custom => {
                    let msg: ModelOutput =
                        serde_json::from_slice(st.stdout.trim_ascii_end())
                            .unwrap_or_else(|_| {
                                panic!(
                                    "failed to decode json from `{}`",
                                    String::from_utf8_lossy(&st.stdout)
                                )
                            });
                    VimRunResponse::Model(msg)
                }
            };
            Ok(Some(resp))
        }
    }
}

fn head_conditionals_from_vim_type(vim_type: &VimType) -> Vec<HeadConditional> {
    vec![match vim_type {
        VimType::Neovim => HeadConditional::Feature("nvim".into()),
        VimType::Vim => HeadConditional::NoFeature("nvim".into()),
    }]
}

impl BootstrapTestCaseBlock {
    fn run_bootstrap_verification(
        self,
        vimrc: Option<&Utf8Path>,
        work_dir: &Utf8Path,
        vim_bin: &VimBin,
        vim_type: &VimType,
    ) -> anyhow::Result<Option<UnitTestCaseBlock>> {
        fs::create_dir(work_dir).ok();

        let buffer_file = work_dir.join("buffer");
        let writer = BufWriter::new(File::create(&buffer_file)?);
        unit_verification::write_clean_buffer(
            writer,
            &self.buffer_before.clean_buffer,
        )?;
        let std_run_file = work_dir.join("std_run.vim");
        let writer = BufWriter::new(File::create(&std_run_file)?);
        let std_run = StdRun(self);
        unit_verification::write_run_file(writer, &std_run)?;
        let std_resp = verify_in_vim(
            vim_bin,
            vimrc,
            &std_run_file,
            &buffer_file,
            &std_run.0.hash.id,
            None,
            RunType::Std,
            vim_type,
        )?;
        let reader = BufReader::new(File::open(&buffer_file)?);
        let expected_buffer_after =
            unit_verification::read_as_clean_buffer(reader)?;
        let self_ = std_run.0;

        let buffer_file = work_dir.join("buffer");
        let writer = BufWriter::new(File::create(&buffer_file)?);
        unit_verification::write_clean_buffer(
            writer,
            &self_.buffer_before.clean_buffer,
        )?;
        let custom_run_file = work_dir.join("custom_run.vim");
        let writer = BufWriter::new(File::create(&custom_run_file)?);
        let custom_run = CustomRun(self_);
        unit_verification::write_run_file(writer, &custom_run)?;
        let custom_resp = verify_in_vim(
            vim_bin,
            vimrc,
            &custom_run_file,
            &buffer_file,
            &custom_run.0.hash.id,
            Some(&expected_buffer_after),
            RunType::Custom,
            vim_type,
        )?;
        let self_ = custom_run.0;

        match (std_resp, custom_resp) {
            (None, None) => Ok(None),
            (
                Some(VimRunResponse::Std(std_state)),
                Some(VimRunResponse::Model(model_output)),
            ) => {
                use crate::parsing::{
                    BufferExpr, ModelOutputItem, TestHash, TestHashId,
                    UnitExportType, UnitTestCaseBlock,
                };

                let buffer_after = BufferExpr {
                    clean_buffer: expected_buffer_after.clone(),
                    langle: None,
                    rangle: None,
                    visual_begin: std_state.visual_begin,
                    visual_end: std_state.visual_end,
                    cursor: std_state.cursor,
                };
                let buffer_output = match &self_.editor_mode {
                    UnitEditorMode::Normal
                    | UnitEditorMode::OperatorPending => BufferExpr {
                        clean_buffer: expected_buffer_after.clone(),
                        langle: None,
                        rangle: None,
                        visual_begin: None,
                        visual_end: None,
                        cursor: model_output.cursor,
                    },
                    UnitEditorMode::VisualChar
                    | UnitEditorMode::VisualLine
                    | UnitEditorMode::VisualBlock => BufferExpr {
                        clean_buffer: expected_buffer_after.clone(),
                        langle: model_output.langle,
                        rangle: model_output.rangle,
                        visual_begin: None,
                        visual_end: None,
                        cursor: None,
                    },
                };
                let buffer_pending = match &self_.editor_mode {
                    UnitEditorMode::OperatorPending => Some(BufferExpr {
                        clean_buffer: self_.buffer_before.clean_buffer.clone(),
                        langle: model_output.langle,
                        rangle: model_output.rangle,
                        visual_begin: None,
                        visual_end: None,
                        cursor: None,
                    }),
                    _ => None,
                };
                let mut model_output_items = Vec::new();
                if model_output.cursor.is_some() {
                    // We assume cursor to be a 4-tuple of numbers (w/out
                    // curswant) for simplicity, which also matches current
                    // implementation.
                    model_output_items.push(ModelOutputItem::Cursor);
                }
                if model_output.langle.is_some() {
                    model_output_items.push(ModelOutputItem::Langle);
                }
                if model_output.rangle.is_some() {
                    model_output_items.push(ModelOutputItem::Rangle);
                }
                if let Some(value) = model_output.prevent_change {
                    model_output_items.push(ModelOutputItem::KeyValue {
                        key: "prevent_change".into(),
                        value,
                    });
                }
                if let Some(value) = model_output.visualmode {
                    model_output_items.push(ModelOutputItem::KeyValue {
                        key: "visualmode".into(),
                        value,
                    });
                }
                let mut block = UnitTestCaseBlock {
                    head_conditionals: head_conditionals_from_vim_type(
                        vim_type,
                    ),
                    hash: TestHash {
                        file: "".into(),
                        lineno: 0,
                        id: TestHashId::Unspecified,
                    },
                    export_type: UnitExportType::Unit,
                    editor_mode: self_.editor_mode.clone(),
                    key_sequence: self_.key_sequence.clone(),
                    operator: self_.operator.clone(),
                    register: self_.register.clone(),
                    count: self_.count,
                    state_before: self_.state_before.clone(),
                    state_after: vec![],
                    buffer_before: self_.buffer_before.clone(),
                    buffer_pending,
                    buffer_output,
                    buffer_after: Some(buffer_after),
                    model_output: model_output_items,
                };
                block.fix_hash();
                Ok(Some(block))
            }
            _ => unreachable!(),
        }
    }
}

// Copied primarily from unit_verification's `Cli`.
#[derive(Parser)]
pub struct Cli {
    /// The vimrc path for bootstrap test verification. If using vim instance
    /// from docker container where vimrc has been baked in, this option may
    /// not be necessary.
    #[arg(long = "rc")]
    vimrc: Option<Utf8PathBuf>,
    /// The full path or PATH-searchable name of vim/nvim binary. Leave this
    /// unspecified to enable dry-run mode, in which the run script etc. can
    /// be inspected.
    #[arg(short = 'v')]
    vim_bin: Option<Utf8PathBuf>,
    /// Specify this if `-v` points to neovim.
    #[arg(long = "neovim", default_value_t = false)]
    is_neovim: bool,
    /// The working directory under which to run bootstrap test verifications.
    #[arg(short = 'd')]
    work_dir: Utf8PathBuf,
    /// Verify this case id only.
    #[arg(short)]
    case_id: Option<String>,
    /// Materialize jieba_test_cases to this file.
    #[arg(short)]
    output_jieba_test_case: Option<Utf8PathBuf>,
    /// The *.jieba_test_case files.
    test_case_file: Vec<Utf8PathBuf>,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let mut ids = HashSet::new();
        fs::create_dir(&self.work_dir).ok();
        let vim_bin = self.vim_bin.map(VimBin::Path).unwrap_or(VimBin::DryRun);
        let vim_type = if self.is_neovim {
            VimType::Neovim
        } else {
            VimType::Vim
        };
        let mut jieba_test_writer = match self.output_jieba_test_case {
            None => None,
            Some(path) => {
                let writer = BufWriter::new(File::create(path)?);
                let serializer = Serializer::setup(
                    writer,
                    head_conditionals_from_vim_type(&vim_type),
                )?;
                Some(serializer)
            }
        };
        let mut progress = DotsProgress::default();

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
                if let TestCaseBlock::Bootstrap(bootstrap_case) = c.block {
                    let case_work_dir = self.work_dir.join(&id_hex);
                    let block = bootstrap_case.run_bootstrap_verification(
                        self.vimrc.as_ref().map(|p| p.as_path()),
                        &case_work_dir,
                        &vim_bin,
                        &vim_type,
                    )?;
                    if let Some(writer) = jieba_test_writer.as_mut()
                        && let Some(block) = block
                    {
                        let block: RawTestCaseBlock = block.into();
                        writer.write(&block)?;
                    }
                    if let VimBin::Path(_) = &vim_bin {
                        progress.step();
                    }
                }
            }
        }

        Ok(())
    }
}
