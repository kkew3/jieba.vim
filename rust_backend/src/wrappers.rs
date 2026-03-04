// Copyright 2024-2026 Kaiwen Wu. All Rights Reserved.
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
    NmapOutput, OmapOutput, WordMotion, XmapOutput,
};
use jieba_vim_rs_core::token::{JiebaPlaceholder, Tokenizer};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::preview;

struct BoundWrapper<'b, 'py, T>(&'b Bound<'py, T>);

impl<'b, 'py, T> From<&'b Bound<'py, T>> for BoundWrapper<'b, 'py, T> {
    fn from(value: &'b Bound<'py, T>) -> Self {
        Self(value)
    }
}

impl<'b, 'py> BufferLike for BoundWrapper<'b, 'py, PyAny> {
    type Error = PyErr;

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        self.0.get_item(lnum - 1)?.extract::<String>()
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        self.0.len()
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

pub struct NmapOutputWrapper(NmapOutput);

impl<'py> IntoPyObject<'py> for NmapOutputWrapper {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(
        self,
        py: Python<'py>,
    ) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        let [a, b, c, d] = self.0.cursor;
        dict.set_item("cursor", vec![a, b, c, d])?;
        dict.set_item("prevent_change", self.0.prevent_change)?;
        Ok(dict)
    }
}

pub struct XmapOutputWrapper<'a>(XmapOutput<'a>);

impl<'a, 'py> IntoPyObject<'py> for XmapOutputWrapper<'a> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(
        self,
        py: Python<'py>,
    ) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        let [la, lb, lc, ld] = self.0.langle;
        let [ra, rb, rc, rd] = self.0.rangle;
        dict.set_item("langle", vec![la, lb, lc, ld])?;
        dict.set_item("rangle", vec![ra, rb, rc, rd])?;
        dict.set_item("visualmode", self.0.visualmode)?;
        dict.set_item("prevent_change", self.0.prevent_change)?;
        Ok(dict)
    }
}

pub struct OmapOutputWrapper(OmapOutput);

impl<'py> IntoPyObject<'py> for OmapOutputWrapper {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(
        self,
        py: Python<'py>,
    ) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        let [a, b, c, d] = self.0.cursor;
        let [la, lb, lc, ld] = self.0.langle;
        let [ra, rb, rc, rd] = self.0.rangle;
        dict.set_item("cursor", vec![a, b, c, d])?;
        dict.set_item("langle", vec![la, lb, lc, ld])?;
        dict.set_item("rangle", vec![ra, rb, rc, rd])?;
        dict.set_item("prevent_change", self.0.prevent_change)?;
        dict.set_item("selection", self.0.selection)?;
        dict.set_item("visualmode", self.0.visualmode)?;
        Ok(dict)
    }
}

#[pyclass]
#[pyo3(name = "WordMotion")]
pub struct WordMotionWrapper {
    wm: WordMotion<JiebaWrapper>,
}

#[pymethods]
impl WordMotionWrapper {
    /// Load jieba with the default dictionary, or with custom dictionary given
    /// dictionary path.
    #[new]
    #[pyo3(signature = (isk_option, path=None))]
    pub fn new(isk_option: &[u8], path: Option<&str>) -> PyResult<Self> {
        let jieba = match path {
            None => Jieba::new(),
            Some(path) => {
                let mut reader = BufReader::new(
                    File::open(path).map_err(PyIOError::new_err)?,
                );
                Jieba::with_dict(&mut reader).map_err(|err| {
                    PyValueError::new_err(format!("jieba error: {}", err))
                })?
            }
        };
        let tokenizer = Tokenizer::try_new(JiebaWrapper(jieba), isk_option)
            .map_err(|_| {
                PyValueError::new_err(format!(
                    "failed to parse isk: {}",
                    unsafe { std::str::from_utf8_unchecked(isk_option) }
                ))
            })?;
        Ok(Self {
            wm: WordMotion::new(tokenizer),
        })
    }

    pub fn set_isk(&mut self, isk_option: &[u8]) -> PyResult<()> {
        self.wm
            .get_tokenizer_mut()
            .try_set_word_predicate(isk_option)
            .map_err(|_| {
                PyValueError::new_err(format!(
                    "failed to parse isk: {}",
                    unsafe { std::str::from_utf8_unchecked(isk_option) }
                ))
            })
    }

    pub fn nmap(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        motion: &[u8],
        cursor: Vec<usize>,
        count: u64,
    ) -> PyResult<NmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(PyValueError::new_err(
                "cursor must contain exactly 5 elements",
            ));
        }
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(NmapOutputWrapper(self.wm.nmap(
            &BoundWrapper(buffer),
            motion,
            cursor_arr,
            count,
        )?))
    }

    pub fn xmap<'a>(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        visualmode: &'a [u8],
        motion: &[u8],
        visual_begin: Vec<usize>,
        visual_end: Vec<usize>,
        count: u64,
    ) -> PyResult<XmapOutputWrapper<'a>> {
        if visual_begin.len() != 4 {
            return Err(PyValueError::new_err(
                "visual_begin must contain exactly 4 elements",
            ));
        }
        if visual_end.len() != 4 {
            return Err(PyValueError::new_err(
                "visual_end must contain exactly 4 elements",
            ));
        }
        let mut visual_begin_arr = [0usize; 4];
        visual_begin_arr.copy_from_slice(&visual_begin);
        let mut visual_end_arr = [0usize; 4];
        visual_end_arr.copy_from_slice(&visual_end);
        Ok(XmapOutputWrapper(self.wm.xmap(
            &BoundWrapper(buffer),
            visualmode,
            motion,
            visual_begin_arr,
            visual_end_arr,
            count,
        )?))
    }

    pub fn omap(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        motion: &[u8],
        cursor: Vec<usize>,
        count: u64,
        operator: &[u8],
    ) -> PyResult<OmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(PyValueError::new_err(
                "cursor must contain exactly 5 elements",
            ));
        }
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(OmapOutputWrapper(self.wm.omap(
            &BoundWrapper(buffer),
            motion,
            cursor_arr,
            count,
            operator,
        )?))
    }

    pub fn preview_nmap(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        motion: &[u8],
        cursor: Vec<usize>,
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        if cursor.len() < 4 {
            return Err(PyValueError::new_err(
                "cursor must contain at least 4 elements",
            ));
        }
        let mut cursor_arr = [0usize; 4];
        cursor_arr.copy_from_slice(&cursor[..4]);
        let [_, lnum, col, _] = cursor_arr;
        preview::preview(
            |b, (lnum, col)| {
                let output =
                    self.wm.nmap(b, motion, [0, lnum, col, 0, col], 1)?;
                let [_, lnum, col, _] = output.cursor;
                Ok((lnum, col))
            },
            &BoundWrapper(buffer),
            (lnum, col),
            preview_limit,
        )
    }
}

#[pyclass]
#[pyo3(name = "LazyWordMotion")]
pub struct LazyWordMotionWrapper {
    wm: WordMotion<LazyJiebaWrapper>,
}

#[pymethods]
impl LazyWordMotionWrapper {
    #[new]
    #[pyo3(signature = (isk_option, path=None))]
    pub fn new(isk_option: &[u8], path: Option<String>) -> PyResult<Self> {
        // Check if `path` is readable beforehand.
        if let Some(path) = &path {
            File::open(path).map_err(PyIOError::new_err)?;
        }
        let jieba = LazyJiebaWrapper {
            path,
            jieba: OnceLock::new(),
        };
        let tokenizer =
            Tokenizer::try_new(jieba, isk_option).map_err(|_| {
                PyValueError::new_err(format!(
                    "failed to parse isk: {}",
                    unsafe { std::str::from_utf8_unchecked(isk_option) }
                ))
            })?;
        Ok(Self {
            wm: WordMotion::new(tokenizer),
        })
    }

    pub fn set_isk(&mut self, isk_option: &[u8]) -> PyResult<()> {
        self.wm
            .get_tokenizer_mut()
            .try_set_word_predicate(isk_option)
            .map_err(|_| {
                PyValueError::new_err(format!(
                    "failed to parse isk: {}",
                    unsafe { std::str::from_utf8_unchecked(isk_option) }
                ))
            })
    }

    pub fn nmap(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        motion: &[u8],
        cursor: Vec<usize>,
        count: u64,
    ) -> PyResult<NmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(PyValueError::new_err(
                "cursor must contain exactly 5 elements",
            ));
        }
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(NmapOutputWrapper(self.wm.nmap(
            &BoundWrapper(buffer),
            motion,
            cursor_arr,
            count,
        )?))
    }

    pub fn xmap<'a>(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        visualmode: &'a [u8],
        motion: &[u8],
        visual_begin: Vec<usize>,
        visual_end: Vec<usize>,
        count: u64,
    ) -> PyResult<XmapOutputWrapper<'a>> {
        if visual_begin.len() != 4 {
            return Err(PyValueError::new_err(
                "visual_begin must contain exactly 4 elements",
            ));
        }
        if visual_end.len() != 4 {
            return Err(PyValueError::new_err(
                "visual_end must contain exactly 4 elements",
            ));
        }
        let mut visual_begin_arr = [0usize; 4];
        visual_begin_arr.copy_from_slice(&visual_begin);
        let mut visual_end_arr = [0usize; 4];
        visual_end_arr.copy_from_slice(&visual_end);
        Ok(XmapOutputWrapper(self.wm.xmap(
            &BoundWrapper(buffer),
            visualmode,
            motion,
            visual_begin_arr,
            visual_end_arr,
            count,
        )?))
    }

    pub fn omap(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        motion: &[u8],
        cursor: Vec<usize>,
        count: u64,
        operator: &[u8],
    ) -> PyResult<OmapOutputWrapper> {
        if cursor.len() != 5 {
            return Err(PyValueError::new_err(
                "cursor must contain exactly 5 elements",
            ));
        }
        let mut cursor_arr = [0usize; 5];
        cursor_arr.copy_from_slice(&cursor);
        Ok(OmapOutputWrapper(self.wm.omap(
            &BoundWrapper(buffer),
            motion,
            cursor_arr,
            count,
            operator,
        )?))
    }

    pub fn preview_nmap(
        &mut self,
        buffer: &Bound<'_, PyAny>,
        motion: &[u8],
        cursor: Vec<usize>,
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        if cursor.len() < 4 {
            return Err(PyValueError::new_err(
                "cursor must contain at least 4 elements",
            ));
        }
        let mut cursor_arr = [0usize; 4];
        cursor_arr.copy_from_slice(&cursor[..4]);
        let [_, lnum, col, _] = cursor_arr;
        preview::preview(
            |b, (lnum, col)| {
                let output =
                    self.wm.nmap(b, motion, [0, lnum, col, 0, col], 1)?;
                let [_, lnum, col, _] = output.cursor;
                Ok((lnum, col))
            },
            &BoundWrapper(buffer),
            (lnum, col),
            preview_limit,
        )
    }
}
