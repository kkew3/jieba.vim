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

//! Motion state transitions.

use super::motion::{ExtendedMotionState, FoldState, MotionState};

/// A tolerable motion state transition.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Tolerable {
    Success,
    Failure,
    #[default]
    SemiFailure,
}

impl FoldState for Tolerable {
    fn finalize(self) -> MotionState {
        match self {
            Self::SemiFailure | Self::Success => MotionState::Success,
            Self::Failure => MotionState::Failure,
        }
    }

    fn update(&mut self, state: ExtendedMotionState) -> Option<MotionState> {
        use Tolerable::*;

        *self = match (*self, state) {
            (Failure, _) => Failure,
            (SemiFailure, ExtendedMotionState::Failure) => Failure,
            (SemiFailure, ExtendedMotionState::Success) => Success,
            (SemiFailure, ExtendedMotionState::Pending) => Success,
            (SemiFailure, ExtendedMotionState::SemiFailure) => SemiFailure,
            (Success, ExtendedMotionState::SemiFailure) => SemiFailure,
            (Success, ExtendedMotionState::Success) => Success,
            (Success, ExtendedMotionState::Failure) => Success,
            (Success, ExtendedMotionState::Pending) => Success,
        };
        if state == ExtendedMotionState::Failure {
            Some(self.finalize())
        } else {
            None
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum SemiTolerable {
    #[default]
    Success,
    Failure,
}

impl FoldState for SemiTolerable {
    fn finalize(self) -> MotionState {
        match self {
            Self::Success => MotionState::Success,
            Self::Failure => MotionState::Failure,
        }
    }

    fn update(&mut self, state: ExtendedMotionState) -> Option<MotionState> {
        use SemiTolerable::*;

        *self = match (*self, state) {
            (Failure, _) => Failure,
            (Success, ExtendedMotionState::Failure) => Failure,
            (Success, _) => Success,
        };
        if *self == Failure {
            return Some(MotionState::Failure);
        }
        if state == ExtendedMotionState::Pending {
            return Some(MotionState::Success);
        }
        None
    }
}

/// An intolerable motion state transition.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Intolerable {
    #[default]
    Success,
    Failure,
}

impl FoldState for Intolerable {
    fn finalize(self) -> MotionState {
        match self {
            Self::Success => MotionState::Success,
            Self::Failure => MotionState::Failure,
        }
    }

    fn update(&mut self, state: ExtendedMotionState) -> Option<MotionState> {
        use Intolerable::*;

        *self = match (*self, state) {
            (Failure, _) => Failure,
            (Success, ExtendedMotionState::Failure) => Failure,
            (Success, _) => Success,
        };
        if *self == Failure {
            Some(MotionState::Failure)
        } else {
            None
        }
    }
}

/// An absolutely intolerable motion state transition.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum AbsolutelyIntolerable {
    #[default]
    Success,
    Failure,
}

impl FoldState for AbsolutelyIntolerable {
    fn finalize(self) -> MotionState {
        match self {
            Self::Success => MotionState::Success,
            Self::Failure => MotionState::Failure,
        }
    }

    fn update(&mut self, state: ExtendedMotionState) -> Option<MotionState> {
        use AbsolutelyIntolerable::*;

        *self = match (*self, state) {
            (Failure, _) => Failure,
            (Success, ExtendedMotionState::Success) => Success,
            (Success, _) => Failure,
        };
        if *self == Failure {
            Some(MotionState::Failure)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct SuppressFailure<S>(S);

impl<S: FoldState> FoldState for SuppressFailure<S> {
    fn finalize(self) -> MotionState {
        MotionState::Success
    }

    fn update(&mut self, state: ExtendedMotionState) -> Option<MotionState> {
        self.0.update(state).map(|_| MotionState::Success)
    }
}
