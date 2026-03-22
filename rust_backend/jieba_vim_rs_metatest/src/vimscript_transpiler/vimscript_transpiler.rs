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

use thiserror::Error;

use crate::parsing::Ascii;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Transpile(#[from] TranspilingError),
}

#[derive(Debug, Error)]
#[error("vimscript transpiling error: {0}")]
pub struct TranspilingError(pub String);

pub type TranspilingResult = Result<(), TranspilingError>;

pub trait ToVimscript {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult;
}

/// A marker trait for vimscript expression (rather than a statement).
pub trait ToVimExpr: ToVimscript {}

/// Transpiling to lua embedded in vimscript.
pub trait ToLua {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult;
}

/// A marker trait for lua expression (rather than a statement).
pub trait ToLuaExpr: ToLua {}

struct Vimscript;
struct VimExpr;
struct Lua;
struct LuaExpr;

trait Render<Ctx> {
    fn render(&self, stream: &mut Vec<u8>) -> TranspilingResult;
}

impl<T: ToVimscript> Render<Vimscript> for T {
    fn render(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.to_vimscript(stream)
    }
}

impl<T: ToVimExpr> Render<VimExpr> for T {
    fn render(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.to_vimscript(stream)
    }
}

impl<T: ToLua> Render<Lua> for T {
    fn render(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.to_lua(stream)
    }
}

impl<T: ToLuaExpr> Render<LuaExpr> for T {
    fn render(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.to_lua(stream)
    }
}

/// Identifier string, of regex `[a-zA-Z_][a-zA-Z0-9_]*`.
#[derive(Debug, PartialEq, Eq)]
pub struct IdentifierString(String);

impl IdentifierString {
    pub fn new<S: Into<String>>(s: S) -> Option<Self> {
        let s = s.into();
        if s.starts_with(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_'))
            && s.chars()
                .skip(1)
                .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
        {
            Some(Self(s))
        } else {
            None
        }
    }
}

impl ToVimscript for IdentifierString {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.push(b'"');
        stream.extend(self.0.as_bytes());
        stream.push(b'"');
        Ok(())
    }
}

impl ToVimExpr for IdentifierString {}

/// Raw identifier, of regex `[a-zA-Z_][a-zA-Z0-9_]*`.
#[derive(Debug, PartialEq, Eq)]
pub struct Identifier(String);

impl Identifier {
    pub fn new<S: Into<String>>(s: S) -> Option<Self> {
        IdentifierString::new(s).map(|ids| Self(ids.0))
    }
}

impl ToVimscript for Identifier {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(self.0.as_bytes());
        Ok(())
    }
}

impl ToLua for Identifier {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(self.0.as_bytes());
        Ok(())
    }
}

pub struct VimVariable {
    pub scope: Ascii,
    pub identifier: Identifier,
}

impl ToVimscript for VimVariable {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.push(self.scope.into());
        stream.push(b':');
        self.identifier.to_vimscript(stream)
    }
}

impl ToVimExpr for VimVariable {}

impl ToLua for VimVariable {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"vim.");
        stream.push(self.scope.into());
        stream.push(b'.');
        self.identifier.to_lua(stream)
    }
}

impl ToLuaExpr for VimVariable {}

impl ToVimscript for &str {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let mut parts = self.split('"');
        if let Some(first) = parts.next() {
            stream.push(b'"');
            stream.extend(first.as_bytes());
            stream.push(b'"');
            for part in parts {
                if part.is_empty() {
                    stream.extend(b" . '\"'");
                } else {
                    stream.extend(b" . '\"' . \"");
                    stream.extend(part.as_bytes());
                    stream.push(b'"');
                }
            }
        }
        Ok(())
    }
}

impl ToVimExpr for &str {}

impl ToLua for &str {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let mut parts = self.split('"');
        if let Some(first) = parts.next() {
            stream.push(b'"');
            stream.extend(first.as_bytes());
            stream.push(b'"');
            for part in parts {
                if part.is_empty() {
                    stream.extend(b" .. '\"'");
                } else {
                    stream.extend(b" .. '\"' .. \"");
                    stream.extend(part.as_bytes());
                    stream.push(b'"');
                }
            }
        }
        Ok(())
    }
}

impl ToLuaExpr for &str {}

impl ToVimscript for String {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.as_str().to_vimscript(stream)
    }
}

impl ToVimExpr for String {}

impl ToLua for String {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.as_str().to_lua(stream)
    }
}

impl ToLuaExpr for String {}

macro_rules! impl_to_vimscript_lua_for_number {
    ($num:ty) => {
        impl ToVimscript for $num {
            fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
                stream.extend(self.to_string().as_bytes());
                Ok(())
            }
        }

        impl ToVimExpr for $num {}

        impl ToLua for $num {
            fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
                stream.extend(self.to_string().as_bytes());
                Ok(())
            }
        }

        impl ToLuaExpr for $num {}
    };
}

impl_to_vimscript_lua_for_number!(u32);
impl_to_vimscript_lua_for_number!(u64);
impl_to_vimscript_lua_for_number!(usize);

impl<T: ToVimscript> ToVimscript for &T {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        (*self).to_vimscript(stream)
    }
}

impl<T: ToVimExpr> ToVimExpr for &T {}

impl<T: ToLua> ToLua for &T {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        (*self).to_lua(stream)
    }
}

impl<T: ToLuaExpr> ToLuaExpr for &T {}

impl<T: ToVimscript, const N: usize> ToVimscript for [T; N] {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self[..].to_vimscript(stream)
    }
}

impl<T: ToVimExpr, const N: usize> ToVimExpr for [T; N] {}

impl<T: ToLua, const N: usize> ToLua for [T; N] {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self[..].to_lua(stream)
    }
}

impl<T: ToLuaExpr, const N: usize> ToLuaExpr for [T; N] {}

impl<T: ToVimscript> ToVimscript for [T] {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.push(b'[');
        if !self.is_empty() {
            self[0].to_vimscript(stream)?;
            for t in &self[1..] {
                stream.extend(b", ");
                t.to_vimscript(stream)?;
            }
        }
        stream.push(b']');
        Ok(())
    }
}

impl<T: ToVimExpr> ToVimExpr for [T] {}

impl<T: ToLua> ToLua for [T] {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.push(b'{');
        if !self.is_empty() {
            self[0].to_lua(stream)?;
            for t in &self[1..] {
                stream.extend(b", ");
                t.to_lua(stream)?;
            }
        }
        stream.push(b'}');
        Ok(())
    }
}

impl<T: ToLuaExpr> ToLuaExpr for [T] {}

impl<T: ToVimscript> ToVimscript for &[T] {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        (*self).to_vimscript(stream)
    }
}

impl<T: ToVimExpr> ToVimExpr for &[T] {}

impl<T: ToLua> ToLua for &[T] {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        (*self).to_lua(stream)
    }
}

impl<T: ToLuaExpr> ToLuaExpr for &[T] {}

impl ToVimscript for Ascii {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let b = self.into();
        if b == b'"' {
            stream.extend(b"'\"'");
        } else {
            stream.push(b'"');
            stream.push(b);
            stream.push(b'"');
        }
        Ok(())
    }
}

impl ToVimExpr for Ascii {}

impl ToLua for Ascii {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let b = self.into();
        if b == b'"' {
            stream.extend(b"'\"'");
        } else {
            stream.push(b'"');
            stream.push(b);
            stream.push(b'"');
        }
        Ok(())
    }
}

impl ToLuaExpr for Ascii {}

trait JoinDelimitedList<Ctx> {
    fn join_with(
        &self,
        delimiter: &[u8],
        stream: &mut Vec<u8>,
    ) -> TranspilingResult;
}

macro_rules! impl_join_delimited_list_for_tuple {
    () => {
        impl<Ctx> JoinDelimitedList<Ctx> for () {
            fn join_with(&self, _delimiter: &[u8], _stream: &mut Vec<u8>) -> TranspilingResult {
                Ok(())
            }
        }
    };
    ($($i:tt: $a:ident),+) => {
        impl<Ctx, $($a: Render<Ctx>),+> JoinDelimitedList<Ctx> for ($($a,)+) {
            #[allow(unused_assignments)]
            fn join_with(&self, delimiter: &[u8], stream: &mut Vec<u8>) -> TranspilingResult {
                let mut first = true;
                $(
                    if !first {
                        stream.extend(delimiter);
                    }
                    self.$i.render(stream)?;
                    first = false;
                )+
                Ok(())
            }
        }
    };
}

impl_join_delimited_list_for_tuple!();
impl_join_delimited_list_for_tuple!(0: A);
impl_join_delimited_list_for_tuple!(0: A, 1: B);
impl_join_delimited_list_for_tuple!(0: A, 1: B, 2: C);
impl_join_delimited_list_for_tuple!(0: A, 1: B, 2: C, 3: D);
impl_join_delimited_list_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_join_delimited_list_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_join_delimited_list_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);

/// Vim function.
pub struct Func<A> {
    name: Identifier,
    args: A,
}

impl<A> Func<A> {
    pub fn new<S: Into<String>>(name: S, args: A) -> Self {
        Self {
            name: Identifier::new(name.into())
                .expect("func name not an identifier"),
            args,
        }
    }
}

impl<A: JoinDelimitedList<VimExpr>> ToVimscript for Func<A> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.name.to_vimscript(stream)?;
        stream.push(b'(');
        self.args.join_with(b", ", stream)?;
        stream.push(b')');
        Ok(())
    }
}

impl<A: JoinDelimitedList<VimExpr>> ToVimExpr for Func<A> {}

impl<A: JoinDelimitedList<LuaExpr>> ToLua for Func<A> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"vim.fn.");
        self.name.to_lua(stream)?;
        stream.push(b'(');
        self.args.join_with(b", ", stream)?;
        stream.push(b')');
        Ok(())
    }
}

impl<A: JoinDelimitedList<LuaExpr>> ToLuaExpr for Func<A> {}

pub struct LuaFunc<A> {
    name: String,
    args: A,
}

impl<A> LuaFunc<A> {
    pub fn new<S: Into<String>>(name: S, args: A) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }
}

impl<A: JoinDelimitedList<LuaExpr>> ToLua for LuaFunc<A> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(self.name.as_bytes());
        stream.push(b'(');
        self.args.join_with(b", ", stream)?;
        stream.push(b')');
        Ok(())
    }
}

impl<A: JoinDelimitedList<LuaExpr>> ToLuaExpr for LuaFunc<A> {}

pub struct Concat<A>(pub A);

impl<A: JoinDelimitedList<VimExpr>> ToVimscript for Concat<A> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.0.join_with(b" . ", stream)
    }
}

impl<A: JoinDelimitedList<VimExpr>> ToVimExpr for Concat<A> {}

impl<A: JoinDelimitedList<LuaExpr>> ToLua for Concat<A> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.0.join_with(b" .. ", stream)
    }
}

impl<A: JoinDelimitedList<LuaExpr>> ToLuaExpr for Concat<A> {}

pub struct Negate<T>(pub T);

impl<T: ToVimExpr> ToVimscript for Negate<T> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"!(");
        self.0.to_vimscript(stream)?;
        stream.push(b')');
        Ok(())
    }
}

impl<T: ToVimExpr> ToVimExpr for Negate<T> {}

impl<T: ToLuaExpr> ToLua for Negate<T> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"not (");
        self.0.to_lua(stream)?;
        stream.push(b')');
        Ok(())
    }
}

impl<T: ToLuaExpr> ToLuaExpr for Negate<T> {}

pub struct OptionVar(pub String);

impl ToVimscript for OptionVar {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.push(b'&');
        stream.extend(self.0.as_bytes());
        Ok(())
    }
}

impl ToVimExpr for OptionVar {}

impl ToLua for OptionVar {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"vim.o.");
        stream.extend(self.0.as_bytes());
        Ok(())
    }
}

impl ToLuaExpr for OptionVar {}

pub struct MarkStr(pub Ascii);

impl ToVimscript for MarkStr {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"\"'");
        stream.push(self.0.into());
        stream.push(b'"');
        Ok(())
    }
}

impl ToVimExpr for MarkStr {}

impl ToLua for MarkStr {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"\"'");
        stream.push(self.0.into());
        stream.push(b'"');
        Ok(())
    }
}

impl ToLuaExpr for MarkStr {}

pub struct VarAssign<T, U> {
    pub lhs: T,
    pub rhs: U,
}

impl<T: ToVimscript, U: ToVimExpr> ToVimscript for VarAssign<T, U> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"let ");
        self.lhs.to_vimscript(stream)?;
        stream.extend(b" = ");
        self.rhs.to_vimscript(stream)?;
        stream.push(b'\n');
        Ok(())
    }
}

impl<T: ToLua, U: ToLuaExpr> ToLua for VarAssign<T, U> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"local ");
        self.lhs.to_lua(stream)?;
        stream.extend(b" = ");
        self.rhs.to_lua(stream)?;
        stream.push(b'\n');
        Ok(())
    }
}

pub struct MapItem<V>(pub IdentifierString, pub V);

impl<V: ToVimExpr> ToVimscript for MapItem<V> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.0.to_vimscript(stream)?;
        stream.extend(b": ");
        self.1.to_vimscript(stream)
    }
}

impl<V: ToLuaExpr> ToLua for MapItem<V> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(self.0.0.as_bytes());
        stream.extend(b" = ");
        self.1.to_lua(stream)
    }
}

struct VimscriptMapItem;

impl<V: ToVimExpr> Render<VimscriptMapItem> for MapItem<V> {
    fn render(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.to_vimscript(stream)
    }
}

struct LuaMapItem;

impl<V: ToLuaExpr> Render<LuaMapItem> for MapItem<V> {
    fn render(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.to_lua(stream)
    }
}

pub struct Map<A>(pub A);

impl<A: JoinDelimitedList<VimscriptMapItem>> ToVimscript for Map<A> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.push(b'{');
        self.0.join_with(b", ", stream)?;
        stream.push(b'}');
        Ok(())
    }
}

impl<A: JoinDelimitedList<VimscriptMapItem>> ToVimExpr for Map<A> {}

impl<A: JoinDelimitedList<LuaMapItem>> ToLua for Map<A> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.push(b'{');
        self.0.join_with(b", ", stream)?;
        stream.push(b'}');
        Ok(())
    }
}

impl<A: JoinDelimitedList<LuaMapItem>> ToLuaExpr for Map<A> {}

pub struct VimCommand<A> {
    cmd: String,
    args: A,
}

impl<A: JoinDelimitedList<VimExpr>> ToVimscript for VimCommand<A> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(self.cmd.as_bytes());
        stream.push(b' ');
        self.args.join_with(b" ", stream)
    }
}

impl<A> VimCommand<A> {
    pub fn new<S: Into<String>>(name: S, args: A) -> Self {
        Self {
            cmd: name.into(),
            args,
        }
    }
}

pub struct VimNotEq<U, V>(pub U, pub V);

impl<U: ToVimExpr, V: ToVimExpr> ToVimscript for VimNotEq<U, V> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.0.to_vimscript(stream)?;
        stream.extend(b" !=# ");
        self.1.to_vimscript(stream)
    }
}

impl<U: ToVimExpr, V: ToVimExpr> ToVimExpr for VimNotEq<U, V> {}

pub struct VimLt<U, V>(pub U, pub V);

impl<U: ToVimExpr, V: ToVimExpr> ToVimscript for VimLt<U, V> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        self.0.to_vimscript(stream)?;
        stream.extend(b" < ");
        self.1.to_vimscript(stream)
    }
}

impl<U: ToVimExpr, V: ToVimExpr> ToVimExpr for VimLt<U, V> {}

/// Echo to stdout/stderr.
pub struct Echo<T> {
    obj: T,
    err: bool,
}

impl<T> Echo<T> {
    /// Set `err` to true to echo to stderr.
    pub fn new(err: bool, obj: T) -> Self {
        Self { obj, err }
    }
}

impl<T: ToVimExpr> ToVimscript for Echo<T> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let c = VimCommand::new(
            "execute",
            (Concat((
                "!echo ",
                Func::new("shellescape", (&self.obj, 1u32)),
                if self.err { " >&2" } else { "" },
            )),),
        );
        c.to_vimscript(stream)
    }
}

impl<T: ToLuaExpr> ToLua for Echo<T> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let c = LuaFunc::new(
            if self.err {
                "io.stderr:write"
            } else {
                "io.write"
            },
            (Concat((&self.obj, r"\n")),),
        );
        c.to_lua(stream)
    }
}

/// Echo json to stdout.
pub struct EchoJson<T>(pub T);

fn vim_json_repr<T: ToVimExpr>(obj: &T) -> Func<(Func<(&T,)>, &'static str)> {
    Func::new("escape", (Func::new("json_encode", (obj,)), r"\\"))
}

fn lua_json_repr<T: ToLuaExpr>(obj: &T) -> Func<(&T,)> {
    Func::new("json_encode", (obj,))
}

impl<T: ToVimExpr> ToVimscript for EchoJson<T> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let c = Echo::new(false, vim_json_repr(&self.0));
        c.to_vimscript(stream)
    }
}

impl<T: ToLuaExpr> ToLua for EchoJson<T> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let c = Echo::new(false, lua_json_repr(&self.0));
        c.to_lua(stream)
    }
}

/// Pretty print an object.
pub struct Pretty<T>(pub T);

impl<T: ToVimExpr> ToVimscript for Pretty<T> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        vim_json_repr(&self.0).to_vimscript(stream)
    }
}

impl<T: ToVimExpr> ToVimExpr for Pretty<T> {}

impl<T: ToLuaExpr> ToLua for Pretty<T> {
    fn to_lua(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        lua_json_repr(&self.0).to_lua(stream)
    }
}

impl<T: ToLuaExpr> ToLuaExpr for Pretty<T> {}

pub struct EmbeddedLua<T>(pub T);

impl<T: ToLua> ToVimscript for EmbeddedLua<T> {
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        stream.extend(b"lua <<EOF\n");
        self.0.to_lua(stream)?;
        stream.extend(b"\nEOF\n");
        Ok(())
    }
}

/// Add newline.
pub fn nl(stream: &mut Vec<u8>) {
    stream.push(b'\n');
}

/// Indent and add newline to some vimscript segment (often a [`VimCommand`]).
pub trait Flush {
    fn flush(&self, indent: u32, stream: &mut Vec<u8>) -> TranspilingResult;
}

impl<T: ToVimscript> Flush for T {
    fn flush(&self, indent: u32, stream: &mut Vec<u8>) -> TranspilingResult {
        for _ in 0..indent {
            stream.extend(b"    ");
        }
        self.to_vimscript(stream)?;
        nl(stream);
        Ok(())
    }
}

pub struct NotEqTest<U, V> {
    pub a: U,
    pub b: V,
    pub msg: String,
}

impl<U: ToVimExpr + ToLuaExpr, V: ToVimExpr + ToLuaExpr> ToVimscript
    for NotEqTest<U, V>
{
    fn to_vimscript(&self, stream: &mut Vec<u8>) -> TranspilingResult {
        let i = 0;
        VimCommand::new("if", (VimNotEq(&self.a, &self.b),))
            .flush(i, stream)?;
        {
            let i = i + 1;

            let pretty_a = Pretty(&self.a);
            let pretty_b = Pretty(&self.b);
            let e = Echo::new(
                true,
                Concat((&self.msg, " :: ", pretty_a, " :: ", pretty_b)),
            );

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

            VimCommand::new("cquit", ()).flush(i, stream)?;
            VimCommand::new("finish", ()).flush(i, stream)?;
        }
        VimCommand::new("endif", ()).flush(i, stream)
    }
}

#[cfg(test)]
mod tests {
    use super::{Concat, NotEqTest, OptionVar, ToVimscript};

    #[test]
    fn test_string_to_vimscript() {
        let s = "";
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(String::from_utf8(sink).unwrap(), "\"\"");

        let s = "abc";
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(String::from_utf8(sink).unwrap(), "\"abc\"");

        let s = "\"abc\"";
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#""" . '"' . "abc" . '"'"#
        );

        let s = "a\"c";
        let mut sink = Vec::new();
        s.to_vimscript(&mut sink).unwrap();
        assert_eq!(String::from_utf8(sink).unwrap(), r#""a" . '"' . "c""#);
    }

    #[test]
    fn test_concat_to_vimscript() {
        let cs = Concat(("foo", "bar"));
        let mut sink = Vec::new();
        cs.to_vimscript(&mut sink).unwrap();
        assert_eq!(String::from_utf8(sink).unwrap(), r#""foo" . "bar""#);
    }

    #[test]
    fn test_not_eq_test_to_vimscript() {
        let net = NotEqTest {
            a: OptionVar("filetype".into()),
            b: "python",
            msg: "mismatched file type".into(),
        };
        let mut sink = Vec::new();
        net.to_vimscript(&mut sink).unwrap();
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            r#"if &filetype !=# "python"
    if has("nvim")
        lua <<EOF
io.stderr:write("mismatched file type" .. " :: " .. vim.fn.json_encode(vim.o.filetype) .. " :: " .. vim.fn.json_encode("python") .. "\n")
EOF

    else 
        execute "!echo " . shellescape("mismatched file type" . " :: " . escape(json_encode(&filetype), "\\") . " :: " . escape(json_encode("python"), "\\"), 1) . " >&2"
    endif 
    cquit 
    finish 
endif 
"#
        );
    }
}
