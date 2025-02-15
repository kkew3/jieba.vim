// Copyright 2024 Kaiwen Wu. All Rights Reserved.
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

mod base;
mod nmap_b;
mod nmap_e;
mod nmap_ge;
mod nmap_w;
mod omap_c_b;
mod omap_c_e;
mod omap_c_ge;
mod omap_c_w;
mod omap_d_b;
mod omap_d_e;
mod omap_d_ge;
mod omap_d_w;
mod omap_y_b;
mod omap_y_e;
mod omap_y_ge;
mod omap_y_w;
mod utils;
mod xmap_b;
mod xmap_e;
mod xmap_ge;
mod xmap_w;

pub use base::MotionOutput;
pub use base::VerifiableCase;
pub use nmap_b::NmapBCase;
pub use nmap_e::NmapECase;
pub use nmap_ge::NmapGeCase;
pub use nmap_w::NmapWCase;
pub use omap_c_b::OmapCBCase;
pub use omap_c_e::OmapCECase;
pub use omap_c_ge::OmapCGeCase;
pub use omap_c_w::OmapCWCase;
pub use omap_d_b::OmapDBCase;
pub use omap_d_e::OmapDECase;
pub use omap_d_ge::OmapDGeCase;
pub use omap_d_w::OmapDWCase;
pub use omap_y_b::OmapYBCase;
pub use omap_y_e::OmapYECase;
pub use omap_y_ge::OmapYGeCase;
pub use omap_y_w::OmapYWCase;
pub use xmap_b::XmapBCase;
pub use xmap_e::XmapECase;
pub use xmap_ge::XmapGeCase;
pub use xmap_w::XmapWCase;

use minijinja::Environment;
use once_cell::sync::Lazy;

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template("execute_nmap", include_str!("templates/execute_nmap.j2"))
        .unwrap();
    env.add_template(
        "execute_omap_c",
        include_str!("templates/execute_omap_c.j2"),
    )
    .unwrap();
    env.add_template(
        "execute_omap_d",
        include_str!("templates/execute_omap_d.j2"),
    )
    .unwrap();
    env.add_template(
        "execute_omap_y",
        include_str!("templates/execute_omap_y.j2"),
    )
    .unwrap();
    env.add_template("execute_xmap", include_str!("templates/execute_xmap.j2"))
        .unwrap();
    env.add_template("setup_omap", include_str!("templates/setup_omap.j2"))
        .unwrap();
    env.add_template("setup", include_str!("templates/setup.j2"))
        .unwrap();

    env
});
