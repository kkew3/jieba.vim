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

//! Positions in a buffer.

/// The 4-element list of numbers \[0, lnum, col, off] as returned by Vim's
/// `getpos(...)` where ... equals `.` or `'{local_mark}``. `lnum` and `col` are
/// indexed from 1. `off` is indexed from 0.
pub type Position = [usize; 4];

/// The 5-element list of numbers \[0, lnum, col, off, curswant] as returned by
/// Vim's `getcurpos()`. `lnum`, `col` and `curswant` are indexed from 1. `off`
/// is indexed from 0.
pub type CursorPositionCurswant = [usize; 5];
