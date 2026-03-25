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

//! The primitive motions and position predicates.

#[cfg(test)]
macro_rules! assert_move {
    ($motion:ident, $buffer:ident: ($lnum_before:expr, $col_before:expr) => ($lnum_after:expr, $col_after:expr)) => {
        let mut cursor = crate::motion::core::position::Position::new(
            $lnum_before,
            $col_before,
        );
        assert_eq!(
            $motion.map(&mut $buffer, 1, &mut cursor)?,
            crate::motion::core::motion::MotionState::Success
        );
        assert_eq!(
            cursor,
            crate::motion::core::position::Position::new(
                $lnum_after,
                $col_after
            )
        );
    };
    ($motion:ident, $buffer:ident: ($lnum_before:expr, $col_before:expr) => Failure) => {
        let mut cursor = crate::motion::core::position::Position::new(
            $lnum_before,
            $col_before,
        );
        assert_eq!(
            $motion.map(&mut $buffer, 1, &mut cursor)?,
            crate::motion::core::motion::MotionState::Failure
        );
    };
    ($motion:ident, $buffer:ident: ($lnum_before:expr, $col_before:expr) => Failure ($lnum_after:expr, $col_after:expr)) => {
        let mut cursor = crate::motion::core::position::Position::new(
            $lnum_before,
            $col_before,
        );
        assert_eq!(
            $motion.map(&mut $buffer, 1, &mut cursor)?,
            crate::motion::core::motion::MotionState::Failure
        );
        assert_eq!(
            cursor,
            crate::motion::core::position::Position::new(
                $lnum_after,
                $col_after
            )
        );
    };
    ($motion:ident, $buffer:ident, $count:literal: ($lnum_before:expr, $col_before:expr) => ($lnum_after:expr, $col_after:expr)) => {
        let mut cursor = crate::motion::core::position::Position::new(
            $lnum_before,
            $col_before,
        );
        assert_eq!(
            $motion.map(&mut $buffer, $count, &mut cursor)?,
            crate::motion::core::motion::MotionState::Success
        );
        assert_eq!(
            cursor,
            crate::motion::core::position::Position::new(
                $lnum_after,
                $col_after
            )
        );
    };
    ($motion:ident, $buffer:ident, $count:literal: ($lnum_before:expr, $col_before:expr) => Failure) => {
        let mut cursor = crate::motion::core::position::Position::new(
            $lnum_before,
            $col_before,
        );
        assert_eq!(
            $motion.map(&mut $buffer, $count, &mut cursor)?,
            crate::motion::core::motion::MotionState::Failure
        );
    };
    ($motion:ident, $buffer:ident, $count:literal: ($lnum_before:expr, $col_before:expr) => Failure ($lnum_after:expr, $col_after:expr)) => {
        let mut cursor = crate::motion::core::position::Position::new(
            $lnum_before,
            $col_before,
        );
        assert_eq!(
            $motion.map(&mut $buffer, $count, &mut cursor)?,
            crate::motion::core::motion::MotionState::Failure
        );
        assert_eq!(
            cursor,
            crate::motion::core::position::Position::new(
                $lnum_after,
                $col_after
            )
        );
    };
}

mod misc;
pub mod predicate;
pub mod text_object;

use super::core;
