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

//! Quoted from vimhelp.org:
//!
//! > 1. If the motion is exclusive and the end of the motion is in column 1,
//! >    the end of the motion is moved to the end of the previous line and the
//! >    motion becomes inclusive.  Example: "}" moves to the first line after
//! >    a paragraph, but "d}" will not include that line.
//! >
//! > 2. If the motion is exclusive, the end of the motion is in column 1
//! >    and the start of the motion was at or before the first non-blank
//! >    in the line, the motion becomes linewise.  Example: If a paragraph
//! >    begins with some blanks and you do "d}" while standing on the first
//! >    non-blank, all the lines of the paragraph are deleted, including the
//! >    blanks.  If you do a put now, the deleted lines will be inserted below
//! >    the cursor position.
//!
//! Check <https://vimhelp.org/motion.txt.html#exclusive> for details.
