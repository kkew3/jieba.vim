// Copyright 2024-2025 Kaiwen Wu. All Rights Reserved.
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

use jieba_vim_rs_core::BufferLike;

/// Construct highlight positions. `motion1` should be a one-step motion
/// function. `cursor_pos` is the current cursor position. To preview current
/// line only, `preview_limit` should be zero; otherwise positive.
pub fn preview<'b, B: BufferLike, M>(
    mut motion1: M,
    buffer: &'b B,
    mut cursor_pos: (usize, usize),
    preview_limit: usize,
) -> Result<Vec<(usize, usize)>, B::Error>
where
    M: FnMut(&'b B, (usize, usize)) -> Result<(usize, usize), B::Error>,
{
    let mut positions = vec![];
    if preview_limit == 0 {
        loop {
            let next_cursor_pos = motion1(buffer, cursor_pos)?;
            // Reaches either beginning of file or end of file.
            if next_cursor_pos == cursor_pos {
                break;
            }
            // Reaches either previous line or next line.
            if next_cursor_pos.0 != cursor_pos.0 {
                break;
            }
            positions.push(next_cursor_pos);
            cursor_pos = next_cursor_pos;
        }
    } else {
        while positions.len() < preview_limit {
            let next_cursor_pos = motion1(buffer, cursor_pos)?;
            // Reaches either beginning of file or end of file.
            if next_cursor_pos == cursor_pos {
                break;
            }
            positions.push(next_cursor_pos);
            cursor_pos = next_cursor_pos;
        }
    }

    Ok(positions)
}
