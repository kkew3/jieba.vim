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

use std::fs::File;
use std::io::BufReader;
use std::sync::OnceLock;

use jieba_rs::Jieba;
use jieba_vim_rs_core::BufferLike;
use jieba_vim_rs_core::motion::{
    ImapOutput, NmapOutput, OmapOutput, WordMotion, XmapOutput,
};
use jieba_vim_rs_core::token::{JiebaPlaceholder, Tokenizer};
use mlua::{IntoLua, Lua, ObjectLike, Table, UserData, UserDataMethods, Value};

use crate::preview;

struct TableBufferWrapper(Table);

impl BufferLike for TableBufferWrapper {
    type Error = mlua::Error;

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        self.0.call_function("getline", lnum)
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        self.0.call_function("lines", ())
    }
}

struct JiebaWrapper(Jieba);

impl JiebaPlaceholder for JiebaWrapper {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.0.cut(sentence, true)
    }
}

struct LazyJiebaWrapper {
    path: Option<String>,
    jieba: OnceLock<Jieba>,
}

impl JiebaPlaceholder for LazyJiebaWrapper {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.jieba
            .get_or_init(|| match &self.path {
                None => Jieba::new(),
                Some(path) => {
                    let mut reader = BufReader::new(
                        File::open(path).unwrap_or_else(|err| {
                            panic!(
                                "failed to open file `{}` due to: {}",
                                path, err
                            )
                        }),
                    );
                    Jieba::with_dict(&mut reader)
                        .unwrap_or_else(|err| {
                            panic!("failed to initialize jieba from file `{}` due to: {}", path, err)
                        })
                }
            })
            .cut(sentence, true)
    }
}

fn to_utf8(s: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(s) }
}

pub struct NmapOutputWrapper(NmapOutput);

impl IntoLua for NmapOutputWrapper {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("cursor", self.0.cursor)?;
        table.set("prevent_change", to_utf8(self.0.prevent_change))?;
        Ok(Value::Table(table))
    }
}

pub struct XmapOutputWrapper(XmapOutput);

impl IntoLua for XmapOutputWrapper {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("langle", self.0.langle)?;
        table.set("rangle", self.0.rangle)?;
        table.set("visualmode", to_utf8(self.0.visualmode))?;
        table.set("prevent_change", to_utf8(self.0.prevent_change))?;
        Ok(Value::Table(table))
    }
}

pub struct OmapOutputWrapper(OmapOutput);

impl IntoLua for OmapOutputWrapper {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("cursor", self.0.cursor)?;
        table.set("langle", self.0.langle)?;
        table.set("rangle", self.0.rangle)?;
        table.set("prevent_change", to_utf8(self.0.prevent_change))?;
        table.set("selection", to_utf8(self.0.selection))?;
        table.set("visualmode", to_utf8(self.0.visualmode))?;
        Ok(Value::Table(table))
    }
}

pub struct ImapOutputWrapper(ImapOutput);

impl IntoLua for ImapOutputWrapper {
    fn into_lua(self, lua: &Lua) -> mlua::prelude::LuaResult<Value> {
        let table = lua.create_table()?;
        table.set("cursor", self.0.cursor)?;
        Ok(Value::Table(table))
    }
}

pub struct WordMotionWrapper {
    wm: WordMotion<JiebaWrapper>,
}

impl WordMotionWrapper {
    pub fn new(
        _lua: &Lua,
        (isk_option, path): (String, Option<String>),
    ) -> mlua::Result<Self> {
        let jieba = match path {
            None => Jieba::new(),
            Some(path) => {
                let mut reader =
                    BufReader::new(File::open(&path).map_err(|_| {
                        mlua::Error::runtime(format!(
                            "jieba_vim: failed to open file: {}",
                            path
                        ))
                    })?);
                Jieba::with_dict(&mut reader).map_err(|err| {
                    mlua::Error::runtime(format!(
                        "jieba_vim: jieba error: {}",
                        err
                    ))
                })?
            }
        };
        let tokenizer =
            Tokenizer::try_new(JiebaWrapper(jieba), isk_option.as_bytes())
                .map_err(|_| {
                    mlua::Error::runtime(format!(
                        "jieba_vim: failed to parse isk: {}",
                        isk_option
                    ))
                })?;
        Ok(Self {
            wm: WordMotion::new(tokenizer),
        })
    }

    fn set_isk(
        _lua: &Lua,
        this: &mut Self,
        isk_option: String,
    ) -> mlua::Result<()> {
        this.wm
            .get_tokenizer_mut()
            .try_set_word_predicate(isk_option.as_bytes())
            .map_err(|_| {
                mlua::Error::runtime(format!(
                    "jieba_vim: failed to parse isk: {}",
                    isk_option
                ))
            })
    }

    fn nmap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor, count): (Table, String, Vec<usize>, u64),
    ) -> mlua::Result<NmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(mlua::Error::runtime(
                "cursor must contain exactly 5 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(NmapOutputWrapper(this.wm.nmap(
            &buffer,
            motion.as_bytes(),
            cursor_arr,
            count,
        )?))
    }

    fn xmap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, visualmode, motion, visual_begin, visual_end, count): (
            Table,
            String,
            String,
            Vec<usize>,
            Vec<usize>,
            u64,
        ),
    ) -> mlua::Result<XmapOutputWrapper> {
        if visual_begin.len() != 4 {
            return Err(mlua::Error::runtime(
                "visual_begin must contain exactly 4 elements",
            ));
        }
        if visual_end.len() != 4 {
            return Err(mlua::Error::runtime(
                "visual_end must contain exactly 4 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut visual_begin_arr = [0usize; 4];
        visual_begin_arr.copy_from_slice(&visual_begin);
        let mut visual_end_arr = [0usize; 4];
        visual_end_arr.copy_from_slice(&visual_end);
        Ok(XmapOutputWrapper(this.wm.xmap(
            &buffer,
            visualmode.as_bytes(),
            motion.as_bytes(),
            visual_begin_arr,
            visual_end_arr,
            count,
        )?))
    }

    fn omap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor, count, operator): (
            Table,
            String,
            Vec<usize>,
            u64,
            String,
        ),
    ) -> mlua::Result<OmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(mlua::Error::runtime(
                "cursor must contain exactly 5 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(OmapOutputWrapper(this.wm.omap(
            &buffer,
            motion.as_bytes(),
            cursor_arr,
            count,
            operator.as_bytes(),
        )?))
    }

    fn imap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor): (Table, mlua::String, Vec<usize>),
    ) -> mlua::Result<ImapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(mlua::Error::runtime(
                "cursor must contain exactly 5 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(ImapOutputWrapper(this.wm.imap(
            &buffer,
            &motion.as_bytes(),
            cursor_arr,
        )?))
    }

    fn preview_nmap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor, preview_limit): (
            Table,
            String,
            Vec<usize>,
            usize,
        ),
    ) -> mlua::Result<Vec<[usize; 2]>> {
        if cursor.len() < 4 {
            return Err(mlua::Error::runtime(
                "cursor must contain at least 4 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 4];
        cursor_arr.copy_from_slice(&cursor[..4]);
        let [_, lnum, col, _] = cursor_arr;
        let preview_positions = preview::preview(
            |b, (lnum, col)| {
                let output = this.wm.nmap(
                    b,
                    motion.as_bytes(),
                    [0, lnum, col, 0, col],
                    1,
                )?;
                let [_, lnum, col, _] = output.cursor;
                Ok((lnum, col))
            },
            &buffer,
            (lnum, col),
            preview_limit,
        )?;
        Ok(preview_positions
            .into_iter()
            .map(|(lnum, col)| [lnum, col])
            .collect())
    }
}

impl UserData for WordMotionWrapper {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("set_isk", Self::set_isk);
        methods.add_method_mut("nmap", Self::nmap);
        methods.add_method_mut("xmap", Self::xmap);
        methods.add_method_mut("omap", Self::omap);
        methods.add_method_mut("imap", Self::imap);
        methods.add_method_mut("preview_nmap", Self::preview_nmap);
    }
}

pub struct LazyWordMotionWrapper {
    wm: WordMotion<LazyJiebaWrapper>,
}

impl LazyWordMotionWrapper {
    pub fn new(
        _lua: &Lua,
        (isk_option, path): (String, Option<String>),
    ) -> mlua::Result<Self> {
        // Check if `path` is readable beforehand.
        if let Some(path) = &path {
            File::open(path).map_err(|_| {
                mlua::Error::runtime(format!(
                    "jieba_vim: failed to open file: {}",
                    path
                ))
            })?;
        }
        let jieba = LazyJiebaWrapper {
            path,
            jieba: OnceLock::new(),
        };
        let tokenizer = Tokenizer::try_new(jieba, isk_option.as_bytes())
            .map_err(|_| {
                mlua::Error::runtime(format!(
                    "jieba_vim: failed to parse isk: {}",
                    isk_option
                ))
            })?;
        Ok(Self {
            wm: WordMotion::new(tokenizer),
        })
    }

    fn set_isk(
        _lua: &Lua,
        this: &mut Self,
        isk_option: String,
    ) -> mlua::Result<()> {
        this.wm
            .get_tokenizer_mut()
            .try_set_word_predicate(isk_option.as_bytes())
            .map_err(|_| {
                mlua::Error::runtime(format!(
                    "jieba_vim: failed to parse isk: {}",
                    isk_option
                ))
            })
    }

    fn nmap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor, count): (Table, String, Vec<usize>, u64),
    ) -> mlua::Result<NmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(mlua::Error::runtime(
                "cursor must contain exactly 5 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(NmapOutputWrapper(this.wm.nmap(
            &buffer,
            motion.as_bytes(),
            cursor_arr,
            count,
        )?))
    }

    fn xmap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, visualmode, motion, visual_begin, visual_end, count): (
            Table,
            String,
            String,
            Vec<usize>,
            Vec<usize>,
            u64,
        ),
    ) -> mlua::Result<XmapOutputWrapper> {
        if visual_begin.len() != 4 {
            return Err(mlua::Error::runtime(
                "visual_begin must contain exactly 4 elements",
            ));
        }
        if visual_end.len() != 4 {
            return Err(mlua::Error::runtime(
                "visual_end must contain exactly 4 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut visual_begin_arr = [0usize; 4];
        visual_begin_arr.copy_from_slice(&visual_begin);
        let mut visual_end_arr = [0usize; 4];
        visual_end_arr.copy_from_slice(&visual_end);
        Ok(XmapOutputWrapper(this.wm.xmap(
            &buffer,
            visualmode.as_bytes(),
            motion.as_bytes(),
            visual_begin_arr,
            visual_end_arr,
            count,
        )?))
    }

    fn omap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor, count, operator): (
            Table,
            String,
            Vec<usize>,
            u64,
            String,
        ),
    ) -> mlua::Result<OmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(mlua::Error::runtime(
                "cursor must contain exactly 5 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(OmapOutputWrapper(this.wm.omap(
            &buffer,
            motion.as_bytes(),
            cursor_arr,
            count,
            operator.as_bytes(),
        )?))
    }

    fn imap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor): (Table, mlua::String, Vec<usize>),
    ) -> mlua::Result<ImapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(mlua::Error::runtime(
                "cursor must contain exactly 5 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(ImapOutputWrapper(this.wm.imap(
            &buffer,
            &motion.as_bytes(),
            cursor_arr,
        )?))
    }

    fn preview_nmap(
        _lua: &Lua,
        this: &mut Self,
        (buffer, motion, cursor, preview_limit): (
            Table,
            String,
            Vec<usize>,
            usize,
        ),
    ) -> mlua::Result<Vec<[usize; 2]>> {
        if cursor.len() < 4 {
            return Err(mlua::Error::runtime(
                "cursor must contain at least 4 elements",
            ));
        }
        let buffer = TableBufferWrapper(buffer);
        let mut cursor_arr = [0usize; 4];
        cursor_arr.copy_from_slice(&cursor[..4]);
        let [_, lnum, col, _] = cursor_arr;
        let preview_positions = preview::preview(
            |b, (lnum, col)| {
                let output = this.wm.nmap(
                    b,
                    motion.as_bytes(),
                    [0, lnum, col, 0, col],
                    1,
                )?;
                let [_, lnum, col, _] = output.cursor;
                Ok((lnum, col))
            },
            &buffer,
            (lnum, col),
            preview_limit,
        )?;
        Ok(preview_positions
            .into_iter()
            .map(|(lnum, col)| [lnum, col])
            .collect())
    }
}

impl UserData for LazyWordMotionWrapper {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("set_isk", Self::set_isk);
        methods.add_method_mut("nmap", Self::nmap);
        methods.add_method_mut("xmap", Self::xmap);
        methods.add_method_mut("omap", Self::omap);
        methods.add_method_mut("imap", Self::imap);
        methods.add_method_mut("preview_nmap", Self::preview_nmap);
    }
}
