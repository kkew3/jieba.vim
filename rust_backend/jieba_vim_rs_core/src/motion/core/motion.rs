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

//! General motion traits and structs.

use std::marker::PhantomData;

use crate::token::TokenType;

use super::buffer::ParsedBufferLike;
use super::iter::GToken;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MotionState {
    Success,
    Failure,
}

impl MotionState {
    pub fn into_prevent_change(self) -> &'static [u8] {
        match self {
            MotionState::Failure => b"1",
            MotionState::Success => b"0",
        }
    }
}

/// A general motion.
pub trait Motion<P> {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut P,
    ) -> Result<MotionState, B::Error>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExtendedMotionState {
    Success,
    Failure,
    /// Occurs when the cursor stops in an empty line.
    SemiFailure,
    /// Occurs when the cursor stops in a space.
    Pending,
}

impl ExtendedMotionState {
    /// Return Self given the token we are arriving at, provided that it is
    /// either a Word or an empty line.
    pub fn from_dest_token(token: GToken) -> Option<Self> {
        match token {
            GToken::Eol(1) => Some(Self::SemiFailure),
            GToken::Eol(_) => None,
            GToken::T(t) => match t.ty {
                TokenType::Space => None,
                TokenType::Word => Some(Self::Success),
            },
        }
    }
}

/// A unit motion.
pub trait UnitMotion<P> {
    fn unit_map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        cursor: &mut P,
    ) -> Result<ExtendedMotionState, B::Error>;
}

/// Define how [`ExtendedMotionState`]s should be folded into [`MotionState`].
pub trait FoldState: Default {
    fn finalize(self) -> MotionState;
    /// Return the final state if it's an absorbing state.
    fn update(&mut self, state: ExtendedMotionState) -> Option<MotionState>;
}

/// The [`UnitMotion`] that makes up an underlying [`Markovian`] motion.
pub trait MarkovianUnit<P>: UnitMotion<P> {
    type FoldState: FoldState;
}

/// A Markovian motion.
pub struct Markovian<M, S> {
    unit_motion: M,
    phantom_data: PhantomData<S>,
}

impl<M, S> Markovian<M, S> {
    pub fn new(unit_motion: M) -> Self {
        Self {
            unit_motion,
            phantom_data: PhantomData,
        }
    }
}

impl<P, M> Motion<P> for Markovian<M, M::FoldState>
where
    M: MarkovianUnit<P>,
{
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        mut count: u64,
        cursor: &mut P,
    ) -> Result<MotionState, B::Error> {
        let mut state = M::FoldState::default();
        while count > 0 {
            if let Some(absorbing_state) =
                state.update(self.unit_motion.unit_map(buffer, cursor)?)
            {
                return Ok(absorbing_state);
            }
            count -= 1;
        }
        Ok(state.finalize())
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

pub trait OneOffUnit<P>: UnitMotion<P> {
    type FoldState: FoldState;
}

/// A non-Markovian motion that runs for at most once whatever the count is.
pub struct OneOffMotion<M, S> {
    unit_motion: M,
    phantom_data: PhantomData<S>,
}

impl<M, S> OneOffMotion<M, S> {
    pub fn new(unit_motion: M) -> Self {
        Self {
            unit_motion,
            phantom_data: PhantomData,
        }
    }
}

impl<P, M: OneOffUnit<P>> Motion<P> for OneOffMotion<M, M::FoldState> {
    fn map<B: ParsedBufferLike + ?Sized>(
        &mut self,
        buffer: &mut B,
        count: u64,
        cursor: &mut P,
    ) -> Result<MotionState, B::Error> {
        let mut state = M::FoldState::default();
        if count > 0 {
            state.update(self.unit_motion.unit_map(buffer, cursor)?);
        }
        Ok(state.finalize())
    }
}
