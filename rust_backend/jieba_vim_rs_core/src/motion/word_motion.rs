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

use std::marker::PhantomData;

use crate::token::{JiebaPlaceholder, TokenType, Tokenizer};
use crate::{BufferLike, CursorPositionCurswant, Position};

use super::parsed_buffer::ParsedBufferLike;
use super::token_iter::GToken;

pub struct WordMotion<C> {
    pub(super) tokenizer: Tokenizer<C>,
}

pub struct NmapOutput {
    pub cursor: Position,
    pub prevent_change: &'static [u8],
}

pub struct XmapOutput<'a> {
    pub langle: Position,
    pub rangle: Position,
    pub visualmode: &'a [u8],
    pub prevent_change: &'static [u8],
}

pub struct OmapOutput {
    pub cursor: Position,
    pub langle: Position,
    pub rangle: Position,
    pub visualmode: &'static [u8],
    pub selection: &'static [u8],
    pub prevent_change: &'static [u8],
}

impl<C> WordMotion<C> {
    pub fn new(tokenizer: Tokenizer<C>) -> Self {
        Self { tokenizer }
    }

    pub fn get_tokenizer_mut(&mut self) -> &mut Tokenizer<C> {
        &mut self.tokenizer
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    pub fn nmap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &[u8],
        cursor: CursorPositionCurswant,
        mut count: u64,
    ) -> Result<NmapOutput, B::Error> {
        if count == 0 {
            count = 1;
        }
        match motion {
            b"w" | b"W" => {
                self.nmap_w(buffer, cursor, count, motion[0] == b'w')
            }
            b"b" | b"B" => {
                self.nmap_b(buffer, cursor, count, motion[0] == b'b')
            }
            b"e" | b"E" => {
                self.nmap_e(buffer, cursor, count, motion[0] == b'e')
            }
            b"ge" | b"gE" => {
                self.nmap_ge(buffer, cursor, count, motion[1] == b'e')
            }
            _ => unreachable!("invalid motion key sequence: {:?}", motion),
        }
    }

    pub fn xmap<'a, B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        visualmode: &'a [u8],
        motion: &[u8],
        visual_begin: Position,
        visual_end: Position,
        mut count: u64,
    ) -> Result<XmapOutput<'a>, B::Error> {
        if count == 0 {
            count = 1;
        }
        match motion {
            b"w" | b"W" => self.xmap_w(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[0] == b'w',
            ),
            b"b" | b"B" => self.xmap_b(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[0] == b'b',
            ),
            b"e" | b"E" => self.xmap_e(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[0] == b'e',
            ),
            b"ge" | b"gE" => self.xmap_ge(
                buffer,
                visualmode,
                visual_begin,
                visual_end,
                count,
                motion[1] == b'e',
            ),
            _ => unreachable!("invalid motion key sequence: {:?}", motion),
        }
    }

    pub fn omap<B: BufferLike + ?Sized>(
        &mut self,
        buffer: &B,
        motion: &[u8],
        cursor: CursorPositionCurswant,
        mut count: u64,
        operator: &[u8],
    ) -> Result<OmapOutput, B::Error> {
        if count == 0 {
            count = 1;
        }
        match motion {
            b"w" | b"W" => {
                self.omap_w(buffer, cursor, count, motion[0] == b'w', operator)
            }
            b"b" | b"B" => {
                self.omap_b(buffer, cursor, count, motion[0] == b'b', operator)
            }
            b"e" | b"E" => {
                self.omap_e(buffer, cursor, count, motion[0] == b'e', operator)
            }
            b"ge" | b"gE" => {
                self.omap_ge(buffer, cursor, count, motion[1] == b'e', operator)
            }
            _ => unreachable!("invalid motion key sequence: {:?}", motion),
        }
    }
}

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
    Pending,
}

impl FoldState for SemiTolerable {
    fn finalize(self) -> MotionState {
        match self {
            Self::Success => MotionState::Success,
            Self::Failure => MotionState::Failure,
            Self::Pending => MotionState::Success,
        }
    }

    fn update(&mut self, state: ExtendedMotionState) -> Option<MotionState> {
        use SemiTolerable::*;

        *self = match (*self, state) {
            (Failure, _) => Failure,
            (Success, ExtendedMotionState::Failure) => Failure,
            (Success, ExtendedMotionState::Pending) => Pending,
            (Success, _) => Success,
            (Pending, _) => Pending,
        };
        match *self {
            Failure => Some(MotionState::Failure),
            Pending => Some(MotionState::Success),
            Success => None,
        }
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
