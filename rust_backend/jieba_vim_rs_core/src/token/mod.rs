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

mod char;
mod isk;
pub(crate) mod jieba;
pub mod token_iter;
mod tokenize;
mod utils;

pub use jieba::JiebaPlaceholder;
pub use tokenize::{Token, TokenLike, TokenType, Tokenizer};
use utils::ascii_or;
pub(crate) use utils::index_tokens;
