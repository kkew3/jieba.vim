// Copyright 2024-2026 Kaiwen Wu. All Rights Reserved.
// Portions Copyright (c) by Bram Moolenaar and others.
//
// This module contains code adapted from Vim's textobject.c. The Vim License
// applies to the adapted portions. See the vim-LICENSE.txt file in the project
// root for the full license text.
//
// In accordance with the Vim License (Section II):
// - Contact: Kaiwen Wu <kps6326@hotmail.com>
// - Changes are available to the Vim maintainer upon request.
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

//! Text objects.
//!
//! Note that the term "text objects" is quoted from Bram's C code. It refers
//! to all atomic word motions underneath |w|, |b|, |e|, |ge|, |iw|, |aw|.

use crate::token::{Token, TokenLike, TokenType};

use super::core::buffer::ParsedBufferLike;
use super::core::failure::{Intolerable, SemiTolerable, Tolerable};
use super::core::iter::{ExtendedInlineTokensIter, GToken, TokenLikeExt};
use super::core::motion::{
    ExtendedMotionState, FoldState, Markovian, MarkovianUnit, Motion,
    MotionState, UnitMotion,
};
use super::core::position::Position;

mod bck_word;
mod bckend_word;
mod end_word;
mod fwd_word;

pub use bck_word::BackwardWord;
pub use bckend_word::BackwardEndWord;
pub use end_word::EndWord;
pub use fwd_word::ForwardWord;
