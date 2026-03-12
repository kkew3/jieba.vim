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

mod d_special;
mod nmap_b;
mod nmap_e;
mod nmap_ge;
mod nmap_w;
mod omap_b;
mod omap_e;
mod omap_ge;
mod omap_w;
mod parsed_buffer;
mod token_iter;
mod word_motion;
mod xmap_b;
mod xmap_e;
mod xmap_ge;
mod xmap_w;

pub use word_motion::{NmapOutput, OmapOutput, WordMotion, XmapOutput};

/// The motion return type.
#[derive(Debug)]
pub struct MotionOutput {
    /// The new cursor position after the motion.
    pub new_cursor_pos: (usize, usize),
    /// Whether the motion induces d-special. Should be false when not in
    /// operator-pending mode
    pub d_special: bool,
    /// Whether the motion should prevent changes, where the operation is
    /// silently aborted. Should be false when not in operator-pending mode
    pub prevent_change: bool,
}
