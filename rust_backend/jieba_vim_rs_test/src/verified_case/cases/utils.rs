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

/// Replace space with '·', and append '␊' as newline.
pub fn display_buffer(buffer: &[String]) -> String {
    let mut out = String::new();
    for line in buffer {
        out.push_str(&line.replace(' ', "·"));
        out.push('␊');
        out.push('\n');
    }
    out
}

pub fn to_vim_col(col: usize) -> usize {
    col + 1
}
