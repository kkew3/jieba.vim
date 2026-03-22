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

//! If the motion spans multiple lines, motion start and end are in column 1,
//! and the motion is exclusive, yank becomes linewise.

use crate::motion::api::MotionType;

use super::core::position::OperatorRange;

/// Check if current motion satisfies the condition that makes yank linewise,
/// and make the motion linewise if true.
pub trait YankLinewise {
    fn yank_linewise(&mut self);
}

impl<'o> YankLinewise for OperatorRange<'o> {
    fn yank_linewise(&mut self) {
        if self.operator == b"y"
            && self.langle.lnum != self.rangle.lnum
            && self.langle.col == 1
            && self.rangle.col == 1
            && self.mtype == MotionType::CharExclusive
        {
            self.mtype = MotionType::LineInclusive;
        }
    }
}
