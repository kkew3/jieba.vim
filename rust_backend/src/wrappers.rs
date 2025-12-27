// Copyright 2024-2025 Kaiwen Wu. All Rights Reserved.
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
use jieba_vim_rs_core::motion::{MotionOutput, WordMotion};
use jieba_vim_rs_core::token::{JiebaPlaceholder, Tokenizer};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

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
        Ok(self.0.get_item(lnum - 1)?.extract::<String>()?)
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        Ok(self.0.len()?)
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
                    let mut reader = BufReader::new(File::open(path).unwrap());
                    Jieba::with_dict(&mut reader).unwrap()
                }
            })
            .cut(sentence, true)
    }
}

#[pyclass]
#[pyo3(name = "MotionOutput")]
pub struct MotionOutputWrapper(MotionOutput);

#[pymethods]
impl MotionOutputWrapper {
    #[getter]
    pub fn cursor(&self) -> (usize, usize) {
        self.0.new_cursor_pos
    }

    #[getter]
    pub fn d_special(&self) -> bool {
        self.0.d_special
    }

    #[getter]
    pub fn prevent_change(&self) -> bool {
        self.0.prevent_change
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
    pub fn new(isk_option: &str, path: Option<&str>) -> PyResult<Self> {
        let jieba = match path {
            None => Jieba::new(),
            Some(path) => {
                let mut reader = BufReader::new(
                    File::open(path).map_err(|err| PyIOError::new_err(err))?,
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
                    isk_option
                ))
            })?;
        Ok(Self {
            wm: WordMotion::new(tokenizer),
        })
    }

    pub fn set_isk(&mut self, isk_option: &str) -> PyResult<()> {
        self.wm
            .get_tokenizer_mut()
            .try_set_word_predicate(isk_option)
            .map_err(|_| {
                PyValueError::new_err(format!(
                    "failed to parse isk: {}",
                    isk_option
                ))
            })
    }

    pub fn nmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "c" {
            Ok(MotionOutputWrapper(self.wm.omap_c_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        }
    }

    #[allow(non_snake_case)]
    pub fn omap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "c" {
            Ok(MotionOutputWrapper(self.wm.omap_c_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        }
    }

    pub fn nmap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        }
    }

    #[allow(non_snake_case)]
    pub fn omap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        }
    }

    pub fn nmap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.omap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn omap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.omap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn nmap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        }
    }

    #[allow(non_snake_case)]
    pub fn omap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        }
    }

    pub fn preview_nmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_w(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_w(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    pub fn preview_nmap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_e(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_e(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    pub fn preview_nmap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_b(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_b(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    pub fn preview_nmap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_ge(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_ge(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
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
    pub fn new(isk_option: &str, path: Option<String>) -> PyResult<Self> {
        // Check if `path` is readable beforehand.
        if let Some(path) = &path {
            File::open(path).map_err(|err| PyIOError::new_err(err))?;
        }
        let jieba = LazyJiebaWrapper {
            path,
            jieba: OnceLock::new(),
        };
        let tokenizer =
            Tokenizer::try_new(jieba, isk_option).map_err(|_| {
                PyValueError::new_err(format!(
                    "failed to parse isk: {}",
                    isk_option
                ))
            })?;
        Ok(Self {
            wm: WordMotion::new(tokenizer),
        })
    }

    pub fn set_isk(&mut self, isk_option: &str) -> PyResult<()> {
        self.wm
            .get_tokenizer_mut()
            .try_set_word_predicate(isk_option)
            .map_err(|_| {
                PyValueError::new_err(format!(
                    "failed to parse isk: {}",
                    isk_option
                ))
            })
    }

    pub fn nmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_w(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "c" {
            Ok(MotionOutputWrapper(self.wm.omap_c_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        }
    }

    #[allow(non_snake_case)]
    pub fn omap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "c" {
            Ok(MotionOutputWrapper(self.wm.omap_c_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_w(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        }
    }

    pub fn nmap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_e(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        }
    }

    #[allow(non_snake_case)]
    pub fn omap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_e(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        }
    }

    pub fn nmap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.omap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn omap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.omap_b(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn nmap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn nmap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.nmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn xmap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            true,
        )?))
    }

    #[allow(non_snake_case)]
    pub fn xmap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        Ok(MotionOutputWrapper(self.wm.xmap_ge(
            &BoundWrapper(buffer),
            cursor_pos,
            count,
            false,
        )?))
    }

    pub fn omap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                true,
            )?))
        }
    }

    #[allow(non_snake_case)]
    pub fn omap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        operator: &str,
        count: u64,
    ) -> PyResult<MotionOutputWrapper> {
        if operator == "d" {
            Ok(MotionOutputWrapper(self.wm.omap_d_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        } else {
            Ok(MotionOutputWrapper(self.wm.omap_ge(
                &BoundWrapper(buffer),
                cursor_pos,
                count,
                false,
            )?))
        }
    }

    pub fn preview_nmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_w(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_w(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    pub fn preview_nmap_e(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_e(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_E(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_e(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    pub fn preview_nmap_b(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_b(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_B(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_b(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    pub fn preview_nmap_ge(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_ge(b, c, 1, true)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }

    #[allow(non_snake_case)]
    pub fn preview_nmap_gE(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        preview_limit: usize,
    ) -> PyResult<Vec<(usize, usize)>> {
        preview::preview(
            |b, c| Ok(self.wm.nmap_ge(b, c, 1, false)?.new_cursor_pos),
            &BoundWrapper(buffer),
            cursor_pos,
            preview_limit,
        )
    }
}
