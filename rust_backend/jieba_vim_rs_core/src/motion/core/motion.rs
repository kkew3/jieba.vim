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

use crate::token::TokenType;

use super::buffer::ParsedBufferLike;
use super::iter::GToken;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MotionState {
    Success,
    Failure,
}

impl MotionState {
    pub fn into_prevent_change(self) -> bool {
        match self {
            MotionState::Failure => true,
            MotionState::Success => false,
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
pub struct Markovian<M> {
    unit_motion: M,
}

impl<M> Markovian<M> {
    pub fn new(unit_motion: M) -> Self {
        Self { unit_motion }
    }
}

impl<P, M> Motion<P> for Markovian<M>
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

/// `motion1.chain(motion2)` returns a new motion that, in its simplest form,
/// runs (motion1; motion2;) `count` times.
pub trait Chain<Rhs>: Sized {
    type Output;

    fn chain(self, rhs: Rhs) -> Self::Output;
}

/// Ergonomic extension trait: gives `a.chain(b)`.
pub trait Chained {
    fn chain<Rhs>(self, rhs: Rhs) -> <Self as Chain<Rhs>>::Output
    where
        Self: Chain<Rhs>,
    {
        <Self as Chain<Rhs>>::chain(self, rhs)
    }
}

impl<T> Chained for T {}
