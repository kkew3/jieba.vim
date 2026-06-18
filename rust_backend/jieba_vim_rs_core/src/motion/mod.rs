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

mod api;
pub(crate) mod core;
mod imap_c_w;
mod nmap_b;
mod nmap_e;
mod nmap_ge;
mod nmap_w;
mod omap_aw;
mod omap_b;
mod omap_e;
mod omap_ge;
mod omap_iw;
mod omap_w;
pub(crate) mod policy;
pub(crate) mod primitives;
mod xmap_aw;
mod xmap_b;
mod xmap_e;
mod xmap_ge;
mod xmap_iw;
mod xmap_w;

pub use api::WordMotion;
pub use api::ffi::{ImapCtrlWOutput, NmapOutput, OmapOutput, XmapOutput};
